use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

/// App error. The interface receives `kind` and `message`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Fallo de red: {0}")]
    Network(String),
    #[error("Sesión vencida o sin permiso")]
    Unauthorized,
    #[error("Respuesta inesperada: {0}")]
    Parse(String),
    #[error("No encontrado")]
    NotFound,
    #[error("Se agotó el límite de peticiones de esta corrida")]
    RateLimited,
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::Network(_) => "network",
            AppError::Unauthorized => "unauthorized",
            AppError::Parse(_) => "parse",
            AppError::NotFound => "not_found",
            AppError::RateLimited => "rate_limited",
            AppError::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_includes_kind_and_message() {
        let json = serde_json::to_value(AppError::Parse("vacío".into())).unwrap();
        assert_eq!(json["kind"], "parse");
        assert_eq!(json["message"], "Respuesta inesperada: vacío");
    }

    #[test]
    fn cada_variante_tiene_su_kind() {
        let casos = [
            (AppError::Network("dns".into()), "network"),
            (AppError::Unauthorized, "unauthorized"),
            (AppError::Parse("html".into()), "parse"),
            (AppError::NotFound, "not_found"),
            (AppError::RateLimited, "rate_limited"),
            (AppError::Internal("db".into()), "internal"),
        ];
        for (error, kind) in casos {
            let json = serde_json::to_value(&error).unwrap();
            assert_eq!(json["kind"], kind, "kind incorrecto para {error:?}");
        }
    }
}
