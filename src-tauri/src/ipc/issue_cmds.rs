//! Issue IPC Commands
//!
//! Commands for creating, listing, and managing issues.

use crate::models::{Classification, Issue, IssueCreate, IssueStatus, IssueSummary, Severity};
use crate::repository;
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
) -> Result<Issue, String> {
    // Validate title
    if title.trim().is_empty() {
        return Err("title cannot be empty".to_string());
    }

    if title.len() > 500 {
        return Err("title cannot exceed 500 characters".to_string());
    }

    // Parse classification
    let classification_enum = Classification::from_str(&classification).ok_or_else(|| {
        format!(
            "Invalid classification: '{}'. Valid: Bug, UX, Feature, Limitation",
            classification
        )
    })?;

    // Parse severity
    let severity_enum = Severity::from_str(&severity).ok_or_else(|| {
        format!(
            "Invalid severity: '{}'. Valid: blocker, major, minor, idea",
            severity
        )
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
    .map_err(|e| format!("Failed to create issue: {}", e))
}

/// List issues with optional status filter
#[tauri::command]
pub async fn list_issues(
    status: Option<String>,
    limit: Option<i32>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<IssueSummary>, String> {
    let limit = (limit.unwrap_or(50).min(100)) as i64;

    let status_filter = if let Some(status_value) = status {
        Some(
            IssueStatus::from_str(&status_value)
                .ok_or_else(|| format!("Invalid status: '{}'", status_value))?,
        )
    } else {
        None
    };

    repository::list_issues(&state.pool, status_filter, limit)
        .await
        .map_err(|e| format!("Failed to list issues: {}", e))
}

/// Get a single issue by ID
#[tauri::command]
pub async fn get_issue(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Issue>, String> {
    repository::get_issue(&state.pool, &id)
        .await
        .map_err(|e| format!("Failed to get issue: {}", e))
}

/// Transition an issue's status
#[tauri::command]
pub async fn transition_issue(
    issue_id: String,
    new_status: String,
    reason: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<Issue, String> {
    let new_status_enum = IssueStatus::from_str(&new_status).ok_or_else(|| {
        format!(
            "Invalid status: '{}'. Valid: pending_decision, decided, in_progress, ready_for_verification, closed",
            new_status
        )
    })?;

    // Special handling for closing: check if artifact is required
    if new_status_enum == IssueStatus::Closed {
        let issue = repository::get_issue(&state.pool, &issue_id)
            .await
            .map_err(|e| format!("Failed to get issue: {}", e))?
            .ok_or_else(|| format!("Issue not found: {}", issue_id))?;

        if issue.close_requires_artifact {
            let has_verified = repository::has_verified_artifact(&state.pool, &issue_id)
                .await
                .map_err(|e| format!("Failed to check artifacts: {}", e))?;

            if !has_verified {
                return Err(
                    "Cannot close issue: no verified artifact exists (close_requires_artifact = true)"
                        .to_string(),
                );
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
    .map_err(|e| format!("Failed to transition issue: {}", e))
}
