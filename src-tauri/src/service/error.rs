//! Stable, redacted errors for the Tauri IPC boundary.

use anyhow::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcErrorCode {
    Validation,
    Forbidden,
    IdentityUnavailable,
    NotFound,
    Conflict,
    RepositoryUnavailable,
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

pub fn public_error(code: IpcErrorCode, message: impl AsRef<str>) -> String {
    format!("{code}: {}", message.as_ref())
}

pub fn validation_error(message: impl AsRef<str>) -> String {
    public_error(IpcErrorCode::Validation, message)
}

pub fn forbidden_error(message: impl AsRef<str>) -> String {
    public_error(IpcErrorCode::Forbidden, message)
}

pub fn identity_error(message: impl AsRef<str>) -> String {
    public_error(IpcErrorCode::IdentityUnavailable, message)
}

pub fn not_found_error(entity: &str) -> String {
    public_error(IpcErrorCode::NotFound, format!("{entity} not found"))
}

pub fn conflict_error(message: impl AsRef<str>) -> String {
    public_error(IpcErrorCode::Conflict, message)
}

pub fn repository_error(operation: &str, error: Error) -> String {
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
            "ZFSS_NOT_FOUND: load issue failed"
        );
        assert_eq!(
            repository_error(
                "transition issue",
                anyhow!("Invalid status transition: closed -> pending_decision")
            ),
            "ZFSS_CONFLICT: transition issue failed"
        );
    }

    #[test]
    fn expected_boundary_failures_receive_stable_codes() {
        assert_eq!(
            validation_error("title is required"),
            "ZFSS_VALIDATION: title is required"
        );
        assert_eq!(
            forbidden_error("role cannot close issues"),
            "ZFSS_FORBIDDEN: role cannot close issues"
        );
        assert_eq!(
            identity_error("current user state is unavailable"),
            "ZFSS_IDENTITY_UNAVAILABLE: current user state is unavailable"
        );
        assert_eq!(not_found_error("issue"), "ZFSS_NOT_FOUND: issue not found");
        assert_eq!(
            conflict_error("verified artifact required"),
            "ZFSS_CONFLICT: verified artifact required"
        );
    }

    #[test]
    fn database_failures_are_redacted() {
        assert_eq!(
            repository_error("list signals", sqlx::Error::PoolTimedOut.into()),
            "ZFSS_REPOSITORY_UNAVAILABLE: list signals failed"
        );

        let rendered = repository_error(
            "create issue",
            anyhow!("connection failed for postgres://admin:secret@private-host/zfss"),
        );
        assert_eq!(rendered, "ZFSS_INTERNAL: create issue failed");
        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("private-host"));
    }
}
