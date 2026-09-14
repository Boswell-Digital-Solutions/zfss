//! Signal IPC Commands
//!
//! Fast signal capture is the primary use case (<60s target).

use crate::constraints::MAX_RAW_TEXT_BYTES;
use crate::models::{Signal, SignalCreate, SignalSource, SignalStatus};
use crate::repository;
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
    if raw_text.trim().is_empty() {
        return Err("raw_text cannot be empty".to_string());
    }

    // Truncate if too long
    let raw_text = if raw_text.len() > MAX_RAW_TEXT_BYTES {
        format!("{}... [truncated]", &raw_text[..MAX_RAW_TEXT_BYTES - 15])
    } else {
        raw_text
    };

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
    .map_err(|e| format!("Failed to capture signal: {}", e))?;

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
    let limit = (limit.unwrap_or(50).min(100)) as i64;

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
        .map_err(|e| format!("Failed to list signals: {}", e))
}

/// Get a single signal by ID
#[tauri::command]
pub async fn get_signal(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Signal>, String> {
    repository::get_signal(&state.pool, &id)
        .await
        .map_err(|e| format!("Failed to get signal: {}", e))
}

/// Link a signal to an issue
#[tauri::command]
pub async fn link_signal_to_issue(
    signal_id: String,
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Signal, String> {
    require_authority(state.current_user_role()?, AuthorityAction::LinkSignal)?;

    let user_id = state.current_user_id();

    repository::link_signal_to_issue(&state.pool, &signal_id, &issue_id, &user_id)
        .await
        .map_err(|e| format!("Failed to link signal: {}", e))
}
