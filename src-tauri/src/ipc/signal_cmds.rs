//! Signal IPC Commands
//!
//! Fast signal capture is the primary use case (<60s target).

use crate::constraints::MAX_RAW_TEXT_BYTES;
use crate::models::{Signal, SignalCreate, SignalSource, SignalStatus};
use crate::repository;
use crate::service::error::repository_error;
use crate::service::input::{list_limit, require_id, require_text, truncate_utf8};
use crate::service::{AuthorityAction, require_authority};
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

/// Result of signal capture
#[derive(Debug, Serialize)]
pub struct CaptureResult {
    pub signal_id: String,
    pub status: String,
    pub created_at: String,
}

/// Capture a new signal - fast path for quick feedback capture
#[tauri::command]
pub async fn capture_signal(
    source: String,
    raw_text: String,
    app_key: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<CaptureResult, String> {
    // Validate source
    let source_enum = SignalSource::from_str(&source).ok_or_else(|| {
        format!(
            "Invalid source: '{}'. Valid sources: in_app, email, dm, call, internal, partner, monitoring",
            source
        )
    })?;

    // Validate raw_text is not empty
    require_text(&raw_text, "raw_text", 1, None)?;
    if let Some(value) = app_key.as_deref() {
        require_text(value, "app_key", 1, Some(100))?;
    }

    // Truncate if too long
    let raw_text = truncate_utf8(raw_text, MAX_RAW_TEXT_BYTES, "... [truncated]");

    let created_by = state.current_user_id();
    let signal = repository::append_signal(
        &state.pool,
        SignalCreate {
            source: source_enum,
            raw_text,
            app_key,
            app_version: None,
            environment: None,
            reporter: None,
        },
        &created_by,
    )
    .await
    .map_err(|error| repository_error("capture signal", error))?;

    Ok(CaptureResult {
        signal_id: signal.id,
        status: signal.status,
        created_at: signal.created_at.to_rfc3339(),
    })
}

/// List signals with optional status filter
#[tauri::command]
pub async fn list_signals(
    status: Option<String>,
    limit: Option<i32>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Signal>, String> {
    let limit = list_limit(limit)?;

    let status_filter = if let Some(status_value) = status {
        Some(
            SignalStatus::from_str(&status_value)
                .ok_or_else(|| format!("Invalid status: '{}'", status_value))?,
        )
    } else {
        None
    };

    repository::list_signals(&state.pool, status_filter, limit)
        .await
        .map_err(|error| repository_error("list signals", error))
}

/// Get a single signal by ID
#[tauri::command]
pub async fn get_signal(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Signal>, String> {
    require_id(&id, "id", "sig")?;
    repository::get_signal(&state.pool, &id)
        .await
        .map_err(|error| repository_error("get signal", error))
}

/// Link a signal to an issue
#[tauri::command]
pub async fn link_signal_to_issue(
    signal_id: String,
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Signal, String> {
    require_authority(state.current_user_role()?, AuthorityAction::LinkSignal)?;
    require_id(&signal_id, "signal_id", "sig")?;
    require_id(&issue_id, "issue_id", "iss")?;

    let user_id = state.current_user_id();

    repository::link_signal_to_issue(&state.pool, &signal_id, &issue_id, &user_id)
        .await
        .map_err(|error| repository_error("link signal", error))
}
