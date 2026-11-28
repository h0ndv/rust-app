use contracts::ErrorKind;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Network failure: {0}")]
    Network(String),
    #[error("Session expired or not authorized")]
    Unauthorized,
    #[error("Link not allowed")]
    Forbidden,
    #[error("Unexpected response: {0}")]
    Parse(String),
    #[error("Not found")]
    NotFound,
    #[error("The request limit for this run was reached")]
    RateLimited,
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn kind(&self) -> ErrorKind {
        match self {
            AppError::Network(_) => ErrorKind::Network,
            AppError::Unauthorized => ErrorKind::Unauthorized,
            AppError::Forbidden => ErrorKind::Forbidden,
            AppError::Parse(_) => ErrorKind::Parse,
            AppError::NotFound => ErrorKind::NotFound,
            AppError::RateLimited => ErrorKind::RateLimited,
            AppError::Internal(_) => ErrorKind::Internal,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", &self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_includes_kind_and_message() {
        let json = serde_json::to_value(AppError::Parse("empty".into())).unwrap();
        assert_eq!(json["kind"], "parse");
        assert_eq!(json["message"], "Unexpected response: empty");
    }

    #[test]
    fn every_variant_has_its_kind() {
        let casos = [
            (AppError::Network("dns".into()), "network"),
            (AppError::Unauthorized, "unauthorized"),
            (AppError::Forbidden, "forbidden"),
            (AppError::Parse("html".into()), "parse"),
            (AppError::NotFound, "not_found"),
            (AppError::RateLimited, "rate_limited"),
            (AppError::Internal("db".into()), "internal"),
        ];
        for (error, kind) in casos {
            let json = serde_json::to_value(&error).unwrap();
            assert_eq!(json["kind"], kind, "wrong kind for {error:?}");
        }
    }
}
