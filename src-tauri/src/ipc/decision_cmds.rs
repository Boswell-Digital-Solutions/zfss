//! Decision IPC Commands
//!
//! Commands for recording and viewing decisions.
//! Only Stewards can make decisions (enforced here).

use crate::constraints::DEFAULT_STEWARD_DEADLINE_DAYS;
use crate::models::{Decision, DecisionCreate, DecisionHistoryEntry, DecisionType};
use crate::repository;
use crate::service::error::repository_error;
use crate::service::input::{require_id, require_text};
use crate::service::{AuthorityAction, require_authority};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Record a new decision on an issue (Steward only)
#[tauri::command]
pub async fn record_decision(
    issue_id: String,
    decision_type: String,
    rationale: String,
    steward_deadline_days: Option<i32>,
    state: State<'_, Arc<AppState>>,
) -> Result<Decision, String> {
    // Enforce Steward-only authority
    require_authority(state.current_user_role()?, AuthorityAction::MakeDecision)?;

    require_id(&issue_id, "issue_id", "iss")?;
    require_text(&rationale, "rationale", 10, None)?;
    if !matches!(steward_deadline_days, None | Some(1..=3650)) {
        return Err("steward_deadline_days must be between 1 and 3650".to_string());
    }

    // Parse decision type
    let decision_type_enum = DecisionType::from_str(&decision_type).ok_or_else(|| {
        format!(
            "Invalid decision_type: '{}'. Valid: FixNow, FixLater, DocumentClarify, WontFix, DeEscalate",
            decision_type
        )
    })?;

    // Verify issue exists
    let issue = repository::get_issue(&state.pool, &issue_id)
        .await
        .map_err(|error| repository_error("get issue for decision", error))?
        .ok_or_else(|| format!("Issue not found: {}", issue_id))?;

    let decided_by = state.current_user_id();

    repository::append_decision(
        &state.pool,
        DecisionCreate {
            issue_id,
            decision_type: decision_type_enum,
            rationale,
            steward_deadline_days: steward_deadline_days.unwrap_or(DEFAULT_STEWARD_DEADLINE_DAYS),
        },
        &decided_by,
    )
    .await
    .map_err(|error| repository_error("record decision", error))
}

/// Get a single decision by ID
#[tauri::command]
pub async fn get_decision(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Decision>, String> {
    require_id(&id, "id", "dec")?;
    repository::get_decision(&state.pool, &id)
        .await
        .map_err(|error| repository_error("get decision", error))
}

/// List all decisions for an issue (history)
#[tauri::command]
pub async fn list_decisions_for_issue(
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<DecisionHistoryEntry>, String> {
    require_id(&issue_id, "issue_id", "iss")?;
    repository::list_decisions_for_issue(&state.pool, &issue_id)
        .await
        .map_err(|error| repository_error("list decisions", error))
}

/// Get the current (latest) decision for an issue
#[tauri::command]
pub async fn get_current_decision(
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Decision>, String> {
    require_id(&issue_id, "issue_id", "iss")?;
    repository::get_current_decision_for_issue(&state.pool, &issue_id)
        .await
        .map_err(|error| repository_error("get current decision", error))
}
