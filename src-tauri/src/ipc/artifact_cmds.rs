//! Artifact IPC Commands
//!
//! Commands for creating, listing, and verifying artifacts.
//! Engineers create artifacts, Stewards verify them.

use crate::models::{Artifact, ArtifactCreate, ArtifactSummary, ArtifactType};
use crate::repository;
use crate::service::input::{require_id, require_text};
use crate::service::{AuthorityAction, require_authority};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Create a new artifact (Engineer or Steward only)
#[tauri::command]
pub async fn create_artifact(
    issue_id: String,
    artifact_type: String,
    title: String,
    description: Option<String>,
    ref_url: Option<String>,
    note: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<Artifact, String> {
    // Enforce Engineer or Steward authority
    require_authority(state.current_user_role()?, AuthorityAction::CreateArtifact)?;

    require_id(&issue_id, "issue_id", "iss")?;
    require_text(&title, "title", 1, Some(500))?;
    if let Some(value) = ref_url.as_deref() {
        require_text(value, "ref_url", 1, Some(1000))?;
    }

    // Parse artifact type
    let artifact_type_enum = ArtifactType::from_str(&artifact_type).ok_or_else(|| {
        format!(
            "Invalid artifact_type: '{}'. Valid: Code, Logic, Knowledge, Test, Law",
            artifact_type
        )
    })?;

    // Verify issue exists
    repository::get_issue(&state.pool, &issue_id)
        .await
        .map_err(|e| format!("Failed to get issue: {}", e))?
        .ok_or_else(|| format!("Issue not found: {}", issue_id))?;

    let created_by = state.current_user_id();

    repository::append_artifact(
        &state.pool,
        ArtifactCreate {
            issue_id,
            artifact_type: artifact_type_enum,
            title,
            description,
            ref_url,
            note,
        },
        &created_by,
    )
    .await
    .map_err(|e| format!("Failed to create artifact: {}", e))
}

/// Get a single artifact by ID
#[tauri::command]
pub async fn get_artifact(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<Artifact>, String> {
    require_id(&id, "id", "art")?;
    repository::get_artifact(&state.pool, &id)
        .await
        .map_err(|e| format!("Failed to get artifact: {}", e))
}

/// List all artifacts for an issue
#[tauri::command]
pub async fn list_artifacts_for_issue(
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ArtifactSummary>, String> {
    require_id(&issue_id, "issue_id", "iss")?;
    repository::list_artifacts_for_issue(&state.pool, &issue_id)
        .await
        .map_err(|e| format!("Failed to list artifacts: {}", e))
}

/// Verify an artifact (Steward only)
#[tauri::command]
pub async fn verify_artifact(
    artifact_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Artifact, String> {
    // Enforce Steward-only authority
    require_authority(state.current_user_role()?, AuthorityAction::VerifyArtifact)?;
    require_id(&artifact_id, "artifact_id", "art")?;

    let verified_by = state.current_user_id();

    repository::verify_artifact(&state.pool, &artifact_id, &verified_by)
        .await
        .map_err(|e| format!("Failed to verify artifact: {}", e))
}

/// Check if an issue has any verified artifacts
#[tauri::command]
pub async fn has_verified_artifact(
    issue_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    require_id(&issue_id, "issue_id", "iss")?;
    repository::has_verified_artifact(&state.pool, &issue_id)
        .await
        .map_err(|e| format!("Failed to check verified artifacts: {}", e))
}
