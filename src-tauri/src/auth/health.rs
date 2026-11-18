use serde_json::json;

use crate::auth::Source;
use crate::error::AppError;
use crate::http::HttpClient;

const MOODLE_MY: &str = "https://aulasvirtuales.santotomas.cl/my/";
const ELLUCIAN_GRAPHQL: &str = "https://experience.elluciancloud.com/ustc/api/ethos-query";
const BANNER_TERM: &str =
    "https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades/term";

/// `true` if the source answers authenticated. An expired session is `false`,
/// not an error: errors are left for the network and for unexpected responses.
pub async fn check(http: &HttpClient, source: Source, cookies: &str) -> Result<bool, AppError> {
    match source {
        Source::Moodle => moodle(http, cookies).await,
        Source::Ellucian => ellucian(http, cookies).await,
        Source::Banner => banner(http, cookies).await,
    }
}

async fn moodle(http: &HttpClient, cookies: &str) -> Result<bool, AppError> {
    match http.get(MOODLE_MY, cookies).await {
        Ok(response) => Ok(!response.url().path().contains("/login/index.php")),
        Err(AppError::Unauthorized) => Ok(false),
        Err(error) => Err(error),
    }
}

async fn ellucian(http: &HttpClient, cookies: &str) -> Result<bool, AppError> {
    let body = json!({ "query": "{__typename}" });
    match http.post_json(ELLUCIAN_GRAPHQL, cookies, &body).await {
        Ok(response) => Ok(matches!(response.status().as_u16(), 200 | 400)),
        Err(AppError::Unauthorized) => Ok(false),
        Err(error) => Err(error),
    }
}

async fn banner(http: &HttpClient, cookies: &str) -> Result<bool, AppError> {
    match http.get(BANNER_TERM, cookies).await {
        Ok(response) => Ok(response.json::<serde_json::Value>().await.is_ok()),
        Err(AppError::Unauthorized) => Ok(false),
        Err(error) => Err(error),
    }
}
