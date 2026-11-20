use std::time::Duration;

use rust_app_lib::error::AppError;
use rust_app_lib::http::HttpClient;
use rust_app_lib::http::rate_limit::RateLimiter;

#[tokio::test(start_paused = true)]
async fn limits_to_six_per_second() {
    let limiter = RateLimiter::new(6, 600);
    for _ in 0..6 {
        limiter.acquire().await.unwrap();
    }
    let start = tokio::time::Instant::now();
    limiter.acquire().await.unwrap();
    assert!(start.elapsed() >= Duration::from_millis(150));
}

#[tokio::test(start_paused = true)]
async fn stops_when_run_budget_is_exhausted() {
    let limiter = RateLimiter::new(6, 3);
    for _ in 0..3 {
        limiter.acquire().await.unwrap();
    }
    assert!(matches!(
        limiter.acquire().await,
        Err(AppError::RateLimited)
    ));
    assert!(matches!(
        limiter.acquire().await,
        Err(AppError::RateLimited)
    ));
}

#[tokio::test(start_paused = true)]
async fn reset_run_enables_another_run() {
    let limiter = RateLimiter::new(6, 2);
    limiter.acquire().await.unwrap();
    limiter.acquire().await.unwrap();
    assert!(matches!(
        limiter.acquire().await,
        Err(AppError::RateLimited)
    ));

    limiter.reset_run();
    limiter.acquire().await.unwrap();
}

#[tokio::test]
async fn invalid_url_is_network_error() {
    let client = HttpClient::new("rust-app-test");
    let result = client.get("no-es-una-url", "").await;
    assert!(matches!(result, Err(AppError::Network(_))));
}
