//! Response IPC Commands
//!
//! Commands for drafting, approving, and sending responses.
//! Responses follow a workflow: draft -> pending -> approved -> sent (or blocked)

use crate::models::{ApprovalState, Response, ResponseChannel, ResponseCreate, ResponseSummary};
use crate::repository;
use crate::service::error::repository_error;
use crate::service::input::{require_id, require_text};
use crate::service::{AuthorityAction, require_authority};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Draft a new response
#[tauri::command]
pub async fn draft_response(
    signal_id: String,
    issue_id: Option<String>,
    response_class: String,
    channel: String,
    body: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Response, String> {
    require_authority(state.current_user_role()?, AuthorityAction::DraftResponse)?;

    require_id(&signal_id, "signal_id", "sig")?;
    if let Some(value) = issue_id.as_deref() {
        require_id(value, "issue_id", "iss")?;
    }
    require_text(&response_class, "response_class", 1, Some(100))?;
    require_text(&body, "body", 1, None)?;

    // Parse channel
    let channel_enum = ResponseChannel::from_str(&channel).ok_or_else(|| {
        format!(
            "Invalid channel: '{}'. Valid: email, in_app, dm, phone, other",
            channel
        )
    })?;

    // Verify signal exists
    repository::get_signal(&state.pool, &signal_id)
        .await
        .map_err(|error| repository_error("get signal for response", error))?
        .ok_or_else(|| format!("Signal not found: {}", signal_id))?;

    // Verify issue exists if provided
    if let Some(ref iss_id) = issue_id {
        repository::get_issue(&state.pool, iss_id)
            .await
            .map_err(|error| repository_error("get issue for response", error))?
            .ok_or_else(|| format!("Issue not found: {}", iss_id))?;
    }

    let drafted_by = state.current_user_id();

    repository::append_response(
        &state.pool,
        ResponseCreate {
            signal_id,
            issue_id,
            response_class,
            channel: channel_enum,
            body,
        },
        &drafted_by,
    )
    .await
    .map_err(|error| repository_error("draft response", error))
}

/// Get a single response by ID
#[tauri::command]
pub async fn get_response(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Response>, String> {
    require_id(&id, "id", "rsp")?;
    repository::get_response(&state.pool, &id)
        .await
        .map_err(|error| repository_error("get response", error))
}

/// List all responses for a signal
#[tauri::command]
pub async fn list_responses_for_signal(
    signal_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ResponseSummary>, String> {
    require_id(&signal_id, "signal_id", "sig")?;
    repository::list_responses_for_signal(&state.pool, &signal_id)
        .await
        .map_err(|error| repository_error("list responses", error))
}

/// Submit response for approval (draft -> pending)
#[tauri::command]
pub async fn submit_response(
    response_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Response, String> {
    require_id(&response_id, "response_id", "rsp")?;
    let actor = state.current_user_id();

    repository::transition_response_state(
        &state.pool,
        &response_id,
        ApprovalState::Pending,
        &actor,
        Some("Submitted for approval"),
        None,
    )
    .await
    .map_err(|error| repository_error("submit response", error))
}

/// Approve a response (Steward only)
#[tauri::command]
pub async fn approve_response(
    response_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Response, String> {
    // Enforce Steward-only authority
    require_authority(state.current_user_role()?, AuthorityAction::ApproveResponse)?;
    require_id(&response_id, "response_id", "rsp")?;

    let actor = state.current_user_id();

    repository::transition_response_state(
        &state.pool,
        &response_id,
        ApprovalState::Approved,
        &actor,
        Some("Approved by Steward"),
        None,
    )
    .await
    .map_err(|error| repository_error("approve response", error))
}

/// Block a response (Steward only)
#[tauri::command]
pub async fn block_response(
    response_id: String,
    reason: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Response, String> {
    // Enforce Steward-only authority
    require_authority(state.current_user_role()?, AuthorityAction::ApproveResponse)?;

    require_id(&response_id, "response_id", "rsp")?;
    require_text(&reason, "reason", 1, None)?;

    let actor = state.current_user_id();

    repository::transition_response_state(
        &state.pool,
        &response_id,
        ApprovalState::Blocked,
        &actor,
        Some("Blocked by Steward"),
        Some(&reason),
    )
    .await
    .map_err(|error| repository_error("block response", error))
}

/// Mark a response as sent (after approval)
#[tauri::command]
pub async fn mark_response_sent(
    response_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Response, String> {
    require_id(&response_id, "response_id", "rsp")?;
    let actor = state.current_user_id();

    repository::transition_response_state(
        &state.pool,
        &response_id,
        ApprovalState::Sent,
        &actor,
        Some("Response sent"),
        None,
    )
    .await
    .map_err(|error| repository_error("mark response sent", error))
}
