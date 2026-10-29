use serde::Serialize;

/// Stable machine-readable codes for every boundary of the app (Master Plan §12.6).
/// The UI decides behavior from `code`; `message` is for people and may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationError,
    NotFound,
    VersionConflict,
    TemporaryFailure,
    InternalError,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NightingaleError {
    pub code: ErrorCode,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, NightingaleError>;

impl NightingaleError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn version_conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::VersionConflict, message)
    }

    pub fn temporary(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::TemporaryFailure, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_to_stable_camel_case_body() {
        let error = NightingaleError::version_conflict("expected version 2");

        let value = serde_json::to_value(&error).unwrap();

        assert_eq!(
            value,
            json!({
                "code": "VERSION_CONFLICT",
                "message": "expected version 2",
            })
        );
    }

    #[test]
    fn constructors_set_distinct_codes() {
        assert_eq!(
            NightingaleError::validation("x").code,
            ErrorCode::ValidationError
        );
        assert_eq!(NightingaleError::not_found("x").code, ErrorCode::NotFound);
        assert_eq!(
            NightingaleError::temporary("x").code,
            ErrorCode::TemporaryFailure
        );
        assert_eq!(
            NightingaleError::internal("x").code,
            ErrorCode::InternalError
        );
    }
}
