//! Stable, redacted errors for the Tauri IPC boundary.

use anyhow::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcErrorCode {
    NotFound,
    Conflict,
    RepositoryUnavailable,
    Internal,
}

impl fmt::Display for IpcErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::NotFound => "ZFSS_NOT_FOUND",
            Self::Conflict => "ZFSS_CONFLICT",
            Self::RepositoryUnavailable => "ZFSS_REPOSITORY_UNAVAILABLE",
            Self::Internal => "ZFSS_INTERNAL",
        };
        formatter.write_str(code)
    }
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

    format!("{code}: {operation} failed")
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
