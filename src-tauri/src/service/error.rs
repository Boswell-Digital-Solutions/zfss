//! Stable, redacted errors for the Tauri IPC boundary.

use anyhow::Error;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IpcErrorCode {
    #[serde(rename = "ZFSS_VALIDATION")]
    Validation,
    #[serde(rename = "ZFSS_FORBIDDEN")]
    Forbidden,
    #[serde(rename = "ZFSS_IDENTITY_UNAVAILABLE")]
    IdentityUnavailable,
    #[serde(rename = "ZFSS_NOT_FOUND")]
    NotFound,
    #[serde(rename = "ZFSS_CONFLICT")]
    Conflict,
    #[serde(rename = "ZFSS_REPOSITORY_UNAVAILABLE")]
    RepositoryUnavailable,
    #[serde(rename = "ZFSS_INTERNAL")]
    Internal,
}

impl fmt::Display for IpcErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::Validation => "ZFSS_VALIDATION",
            Self::Forbidden => "ZFSS_FORBIDDEN",
            Self::IdentityUnavailable => "ZFSS_IDENTITY_UNAVAILABLE",
            Self::NotFound => "ZFSS_NOT_FOUND",
            Self::Conflict => "ZFSS_CONFLICT",
            Self::RepositoryUnavailable => "ZFSS_REPOSITORY_UNAVAILABLE",
            Self::Internal => "ZFSS_INTERNAL",
        };
        formatter.write_str(code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IpcError {
    pub code: IpcErrorCode,
    pub message: String,
}

impl IpcError {
    pub fn new(code: IpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for IpcError {}

pub fn public_error(code: IpcErrorCode, message: impl Into<String>) -> IpcError {
    IpcError::new(code, message)
}

pub fn validation_error(message: impl Into<String>) -> IpcError {
    public_error(IpcErrorCode::Validation, message)
}

pub fn forbidden_error(message: impl Into<String>) -> IpcError {
    public_error(IpcErrorCode::Forbidden, message)
}

pub fn identity_error(message: impl Into<String>) -> IpcError {
    public_error(IpcErrorCode::IdentityUnavailable, message)
}

pub fn not_found_error(entity: &str) -> IpcError {
    public_error(IpcErrorCode::NotFound, format!("{entity} not found"))
}

pub fn conflict_error(message: impl Into<String>) -> IpcError {
    public_error(IpcErrorCode::Conflict, message)
}

pub fn repository_error(operation: &str, error: Error) -> IpcError {
    let chain = format!("{error:#}").to_ascii_lowercase();
    let code = if chain.contains("not found") || chain.contains("disappeared") {
        IpcErrorCode::NotFound
    } else if chain.contains("invalid status transition")
        || chain.contains("invalid approval state transition")
        || chain.contains("already exists")
        || chain.contains("duplicate key")
    {
        IpcErrorCode::Conflict
    } else if let Some(sqlx_error) = error.downcast_ref::<sqlx::Error>() {
        match sqlx_error {
            sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::Io(_)
            | sqlx::Error::Tls(_) => IpcErrorCode::RepositoryUnavailable,
            sqlx::Error::RowNotFound => IpcErrorCode::NotFound,
            _ => IpcErrorCode::Internal,
        }
    } else {
        IpcErrorCode::Internal
    };

    public_error(code, format!("{operation} failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn domain_failures_receive_stable_codes() {
        assert_eq!(
            repository_error("load issue", anyhow!("Issue not found: iss_missing")),
            IpcError::new(IpcErrorCode::NotFound, "load issue failed")
        );
        assert_eq!(
            repository_error(
                "transition issue",
                anyhow!("Invalid status transition: closed -> pending_decision")
            ),
            IpcError::new(IpcErrorCode::Conflict, "transition issue failed")
        );
    }

    #[test]
    fn expected_boundary_failures_receive_stable_codes() {
        assert_eq!(
            validation_error("title is required"),
            IpcError::new(IpcErrorCode::Validation, "title is required")
        );
        assert_eq!(
            forbidden_error("role cannot close issues"),
            IpcError::new(IpcErrorCode::Forbidden, "role cannot close issues")
        );
        assert_eq!(
            identity_error("current user state is unavailable"),
            IpcError::new(
                IpcErrorCode::IdentityUnavailable,
                "current user state is unavailable"
            )
        );
        assert_eq!(
            not_found_error("issue"),
            IpcError::new(IpcErrorCode::NotFound, "issue not found")
        );
        assert_eq!(
            conflict_error("verified artifact required"),
            IpcError::new(IpcErrorCode::Conflict, "verified artifact required")
        );
    }

    #[test]
    fn error_envelope_serializes_for_tauri() {
        assert_eq!(
            serde_json::to_value(validation_error("title is required")).unwrap(),
            serde_json::json!({
                "code": "ZFSS_VALIDATION",
                "message": "title is required"
            })
        );
    }

    #[test]
    fn every_error_code_has_an_exact_wire_value() {
        let cases = [
            (IpcErrorCode::Validation, "ZFSS_VALIDATION"),
            (IpcErrorCode::Forbidden, "ZFSS_FORBIDDEN"),
            (
                IpcErrorCode::IdentityUnavailable,
                "ZFSS_IDENTITY_UNAVAILABLE",
            ),
            (IpcErrorCode::NotFound, "ZFSS_NOT_FOUND"),
            (IpcErrorCode::Conflict, "ZFSS_CONFLICT"),
            (
                IpcErrorCode::RepositoryUnavailable,
                "ZFSS_REPOSITORY_UNAVAILABLE",
            ),
            (IpcErrorCode::Internal, "ZFSS_INTERNAL"),
        ];

        for (code, expected) in cases {
            assert_eq!(serde_json::to_value(code).unwrap(), expected);
        }
    }

    #[test]
    fn database_failures_are_redacted() {
        assert_eq!(
            repository_error("list signals", sqlx::Error::PoolTimedOut.into()),
            IpcError::new(IpcErrorCode::RepositoryUnavailable, "list signals failed")
        );

        let rendered = repository_error(
            "create issue",
            anyhow!("connection failed for postgres://admin:secret@private-host/zfss"),
        );
        assert_eq!(
            rendered,
            IpcError::new(IpcErrorCode::Internal, "create issue failed")
        );
        assert!(!rendered.message.contains("secret"));
        assert!(!rendered.message.contains("private-host"));
    }
}
