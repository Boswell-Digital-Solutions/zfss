//! Issue IPC Commands
//!
//! Commands for creating, listing, and managing issues.

use crate::models::{Classification, Issue, IssueCreate, IssueStatus, IssueSummary, Severity};
use crate::repository;
use crate::service::error::{
    IpcError, conflict_error, not_found_error, repository_error, validation_error,
};
use crate::service::input::{list_limit, require_id, require_text};
use crate::service::{AuthorityAction, require_authority};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Create a new issue
#[tauri::command]
pub async fn create_issue(
    title: String,
    description: Option<String>,
    classification: String,
    severity: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Issue, IpcError> {
    require_authority(state.current_user_role()?, AuthorityAction::CreateIssue)?;

    // Validate title
    require_text(&title, "title", 1, Some(500))?;

    // Parse classification
    let classification_enum = Classification::from_str(&classification).ok_or_else(|| {
        validation_error(format!(
            "Invalid classification: '{}'. Valid: Bug, UX, Feature, Limitation",
            classification
        ))
    })?;

    // Parse severity
    let severity_enum = Severity::from_str(&severity).ok_or_else(|| {
        validation_error(format!(
            "Invalid severity: '{}'. Valid: blocker, major, minor, idea",
            severity
        ))
    })?;

    let created_by = state.current_user_id();

    repository::append_issue(
        &state.pool,
        IssueCreate {
            title,
            description,
            classification: classification_enum,
            severity: severity_enum,
        },
        &created_by,
    )
    .await
    .map_err(|error| repository_error("create issue", error))
}

/// List issues with optional status filter
#[tauri::command]
pub async fn list_issues(
    status: Option<String>,
    limit: Option<i32>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<IssueSummary>, IpcError> {
    let limit = list_limit(limit)?;

    let status_filter = if let Some(status_value) = status {
        Some(
            IssueStatus::from_str(&status_value)
                .ok_or_else(|| validation_error(format!("Invalid status: '{}'", status_value)))?,
        )
    } else {
        None
    };

    repository::list_issues(&state.pool, status_filter, limit)
        .await
        .map_err(|error| repository_error("list issues", error))
}

/// Get a single issue by ID
#[tauri::command]
pub async fn get_issue(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Issue>, IpcError> {
    require_id(&id, "id", "iss")?;
    repository::get_issue(&state.pool, &id)
        .await
        .map_err(|error| repository_error("get issue", error))
}

/// Transition an issue's status
#[tauri::command]
pub async fn transition_issue(
    issue_id: String,
    new_status: String,
    reason: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<Issue, IpcError> {
    require_id(&issue_id, "issue_id", "iss")?;
    if let Some(value) = reason.as_deref() {
        require_text(value, "reason", 1, None)?;
    }
    let new_status_enum = IssueStatus::from_str(&new_status).ok_or_else(|| {
        validation_error(format!(
            "Invalid status: '{}'. Valid: pending_decision, decided, in_progress, ready_for_verification, closed",
            new_status
        ))
    })?;

    // Special handling for closing: check if artifact is required
    if new_status_enum == IssueStatus::Closed {
        require_authority(state.current_user_role()?, AuthorityAction::CloseIssue)?;

        let issue = repository::get_issue(&state.pool, &issue_id)
            .await
            .map_err(|error| repository_error("get issue for closure", error))?
            .ok_or_else(|| not_found_error("issue"))?;

        if issue.close_requires_artifact {
            let has_verified = repository::has_verified_artifact(&state.pool, &issue_id)
                .await
                .map_err(|error| repository_error("check closure artifacts", error))?;

            if !has_verified {
                return Err(conflict_error(
                    "cannot close issue: a verified artifact is required",
                ));
            }
        }
    }

    let actor = state.current_user_id();

    repository::transition_issue_status(
        &state.pool,
        &issue_id,
        new_status_enum,
        &actor,
        reason.as_deref(),
    )
    .await
    .map_err(|error| repository_error("transition issue", error))
}
