//! ZFSS IPC Commands
//!
//! Tauri commands for frontend-backend communication.

pub mod artifact_cmds;
pub mod decision_cmds;
pub mod issue_cmds;
pub mod response_cmds;
pub mod signal_cmds;

// Re-export all commands for registration
pub use artifact_cmds::{
    create_artifact, get_artifact, has_verified_artifact, list_artifacts_for_issue, verify_artifact,
};
pub use decision_cmds::{
    get_current_decision, get_decision, list_decisions_for_issue, record_decision,
};
pub use issue_cmds::{create_issue, get_issue, list_issues, transition_issue};
pub use response_cmds::{
    approve_response, block_response, draft_response, get_response, list_responses_for_signal,
    mark_response_sent, submit_response,
};
pub use signal_cmds::{capture_signal, get_signal, link_signal_to_issue, list_signals};

#[cfg(test)]
mod command_error_contract_tests {
    use super::{capture_signal, create_issue};
    use crate::config::Settings;
    use crate::state::AppState;
    use crate::{
        __cmd__capture_signal, __cmd__create_issue, __tauri_command_name_capture_signal,
        __tauri_command_name_create_issue,
    };
    use serde_json::{Value, json};
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{App, WebviewWindow, WebviewWindowBuilder};
    use uuid::Uuid;

    fn test_app(role: &str) -> App<MockRuntime> {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgresql://localhost/zfss_ipc_contract")
            .expect("test database URL must parse");
        let settings = Settings {
            current_user_role: role.to_string(),
            ..Settings::default()
        };
        let state = Arc::new(AppState::new(pool, settings, Uuid::nil()));

        mock_builder()
            .manage(state)
            .invoke_handler(tauri::generate_handler![capture_signal, create_issue])
            .build(mock_context(noop_assets()))
            .expect("mock Tauri application must build")
    }

    fn invoke_error(webview: &WebviewWindow<MockRuntime>, command: &str, body: Value) -> Value {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().expect("test URL must parse"),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect_err("command must reject the test request")
    }

    #[tokio::test]
    async fn validation_failure_crosses_the_tauri_command_boundary_as_an_envelope() {
        let app = test_app("Steward");
        let webview = WebviewWindowBuilder::new(&app, "validation", Default::default())
            .build()
            .expect("mock webview must build");

        assert_eq!(
            invoke_error(
                &webview,
                "capture_signal",
                json!({ "source": "in_app", "rawText": "", "appKey": null })
            ),
            json!({
                "code": "ZFSS_VALIDATION",
                "message": "raw_text must contain at least 1 character(s)"
            })
        );
    }

    #[tokio::test]
    async fn authority_failure_crosses_the_tauri_command_boundary_as_an_envelope() {
        let app = test_app("Engineer");
        let webview = WebviewWindowBuilder::new(&app, "authority", Default::default())
            .build()
            .expect("mock webview must build");

        assert_eq!(
            invoke_error(
                &webview,
                "create_issue",
                json!({
                    "title": "Test issue",
                    "description": null,
                    "classification": "Bug",
                    "severity": "minor"
                })
            ),
            json!({
                "code": "ZFSS_FORBIDDEN",
                "message": "Permission denied: role Engineer cannot create issues"
            })
        );
    }
}
