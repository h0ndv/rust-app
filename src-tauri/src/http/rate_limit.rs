use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::error::AppError;

pub struct RateLimiter {
    interval: Duration,
    max_per_run: u32,
    used: AtomicU32,
    next_slot: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(per_second: u32, max_per_run: u32) -> Self {
        let per_second = per_second.max(1);
        Self {
            interval: Duration::from_secs_f64(1.0 / f64::from(per_second)),
            max_per_run,
            used: AtomicU32::new(0),
            next_slot: Mutex::new(Instant::now()),
        }
    }

    pub async fn acquire(&self) -> Result<(), AppError> {
        if self.used.fetch_add(1, Ordering::SeqCst) >= self.max_per_run {
            return Err(AppError::RateLimited);
        }

        let mut next_slot = self.next_slot.lock().await;
        let now = Instant::now();
        let slot = if *next_slot > now { *next_slot } else { now };
        *next_slot = slot + self.interval;
        drop(next_slot);

        if slot > now {
            tokio::time::sleep_until(slot).await;
        }
        Ok(())
    }

    pub fn reset_run(&self) {
        self.used.store(0, Ordering::SeqCst);
    }

    pub fn count(&self) -> u32 {
        self.used.load(Ordering::SeqCst)
    }
}
