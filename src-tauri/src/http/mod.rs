pub mod rate_limit;

pub use rate_limit::RateLimiter;

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::Serialize;
use tokio::sync::Semaphore;

use crate::error::AppError;

const TIMEOUT: Duration = Duration::from_secs(30);
const BODY_TIMEOUT: Duration = Duration::from_secs(30);
const BINARY_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_REDIRECTS: usize = 10;
const MAX_IN_FLIGHT: usize = 4;

/// `reqwest` cuts the wait for headers, not the body read: a huge PDF
/// or a server that trickles would leave the run hanging forever.
pub async fn read_text(response: Response) -> Result<String, AppError> {
    match tokio::time::timeout(BODY_TIMEOUT, response.text()).await {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(error)) => Err(AppError::Network(error.to_string())),
        Err(_) => Err(AppError::Network("la respuesta tardó demasiado".into())),
    }
}

pub async fn read_bytes(response: Response) -> Result<Vec<u8>, AppError> {
    match tokio::time::timeout(BINARY_TIMEOUT, response.bytes()).await {
        Ok(Ok(bytes)) => Ok(bytes.to_vec()),
        Ok(Err(error)) => Err(AppError::Network(error.to_string())),
        Err(_) => Err(AppError::Network("la descarga tardó demasiado".into())),
    }
}

/// HTTP client shared by the three sources. It is `Clone`: copies
/// share pooled connections, rate limit and budget.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    limiter: Arc<RateLimiter>,
    in_flight: Arc<Semaphore>,
}

impl HttpClient {
    pub fn new(user_agent: &str) -> Self {
        let client = Client::builder()
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .user_agent(user_agent)
            .build()
            .expect("no se pudo crear el cliente HTTP");

        Self {
            client,
            limiter: Arc::new(RateLimiter::new(6, 600)),
            in_flight: Arc::new(Semaphore::new(MAX_IN_FLIGHT)),
        }
    }

    /// Rate limiter handle so the orchestrator can reset the budget
    /// at the start of each run.
    pub fn limiter(&self) -> Arc<RateLimiter> {
        Arc::clone(&self.limiter)
    }

    pub async fn get(&self, url: &str, cookies: &str) -> Result<Response, AppError> {
        self.send(with_cookies(self.client.get(url), cookies)).await
    }

    pub async fn post_json<T: Serialize>(
        &self,
        url: &str,
        cookies: &str,
        body: &T,
    ) -> Result<Response, AppError> {
        let payload = serde_json::to_vec(body)
            .map_err(|e| AppError::Parse(format!("cuerpo JSON inválido: {e}")))?;

        let request = self
            .client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload);

        self.send(with_cookies(request, cookies)).await
    }

    async fn send(&self, request: RequestBuilder) -> Result<Response, AppError> {
        self.limiter.acquire().await?;

        let _permit = self
            .in_flight
            .acquire()
            .await
            .map_err(|e| AppError::Internal(format!("semáforo cerrado: {e}")))?;

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        match status_error(response.status()) {
            Some(error) => Err(error),
            None => Ok(response),
        }
    }
}

fn with_cookies(request: RequestBuilder, cookies: &str) -> RequestBuilder {
    if cookies.is_empty() {
        request
    } else {
        request.header(reqwest::header::COOKIE, cookies)
    }
}

/// 401 and 403 mean expired session; 404 and 429 are told apart above so
/// each adapter decides. Other codes return to the caller with their body.
fn status_error(status: StatusCode) -> Option<AppError> {
    match status.as_u16() {
        401 | 403 => Some(AppError::Unauthorized),
        404 => Some(AppError::NotFound),
        429 => Some(AppError::RateLimited),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapea_estados_a_errores() {
        assert!(matches!(
            status_error(StatusCode::UNAUTHORIZED),
            Some(AppError::Unauthorized)
        ));
        assert!(matches!(
            status_error(StatusCode::FORBIDDEN),
            Some(AppError::Unauthorized)
        ));
        assert!(matches!(
            status_error(StatusCode::NOT_FOUND),
            Some(AppError::NotFound)
        ));
        assert!(matches!(
            status_error(StatusCode::TOO_MANY_REQUESTS),
            Some(AppError::RateLimited)
        ));
    }

    #[test]
    fn deja_pasar_los_estados_que_el_adaptador_interpreta() {
        assert!(status_error(StatusCode::OK).is_none());
        assert!(status_error(StatusCode::BAD_REQUEST).is_none());
        assert!(status_error(StatusCode::INTERNAL_SERVER_ERROR).is_none());
    }
}
