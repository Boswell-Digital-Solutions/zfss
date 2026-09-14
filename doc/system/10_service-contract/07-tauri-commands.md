## 7. Tauri Command Interface

### Command Registration

23 IPC commands registered in `main.rs` via `tauri::Builder::invoke_handler()`.

### Commands by Domain

#### Signal Commands (ipc/signal_cmds.rs)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `capture_signal` | raw_text, source, user_id, device_id | SignalId | Create new Signal (append-only) |
| `list_signals` | — | Vec\<Signal\> | List all Signals |
| `get_signal` | signal_id | Signal | Get Signal by ID |
| `link_signal_to_issue` | signal_id, issue_id | () | Link Signal to Issue |

#### Issue Commands (ipc/issue_cmds.rs)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `create_issue` | title, description, classification, severity, frequency | IssueId | Create new Issue |
| `list_issues` | — | Vec\<Issue\> | List all Issues |
| `get_issue` | issue_id | Issue | Get Issue by ID |
| `transition_issue` | issue_id, new_status | () | Transition Issue state |

#### Decision Commands (ipc/decision_cmds.rs)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `record_decision` | issue_id, decision_type, body | DecisionId | Record Decision (append-only, can supersede) |
| `get_decision` | decision_id | Decision | Get Decision by ID |
| `list_decisions_for_issue` | issue_id | Vec\<Decision\> | List all Decisions for Issue |
| `get_current_decision` | issue_id | Decision? | Get latest active Decision |

#### Artifact Commands (ipc/artifact_cmds.rs)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `create_artifact` | issue_id, artifact_type, title, url | ArtifactId | Create Artifact |
| `get_artifact` | artifact_id | Artifact | Get Artifact by ID |
| `list_artifacts_for_issue` | issue_id | Vec\<Artifact\> | List Artifacts for Issue |
| `verify_artifact` | artifact_id, verified_by | () | Mark Artifact as verified (Steward only) |
| `has_verified_artifact` | issue_id | bool | Check if Issue has verified Artifact |

#### Response Commands (ipc/response_cmds.rs)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `draft_response` | signal_id, channel, body | ResponseId | Create draft Response |
| `get_response` | response_id | Response | Get Response by ID |
| `list_responses_for_signal` | signal_id | Vec\<Response\> | List Responses for Signal |
| `submit_response` | response_id | () | Submit for approval |
| `approve_response` | response_id, approved_by | () | Approve Response (Steward only) |
| `block_response` | response_id, reason | () | Block Response with reason |
| `mark_response_sent` | response_id | () | Mark as sent |

### State Access

All commands access `AppState` via Tauri's managed state:

```rust
pub struct AppState {
    pub pool: PgPool,
    pub settings: Settings,
    pub device_id: String,
    pub current_user: Option<User>,
}
```

### Error Handling

Commands return `Result<T, IpcError>`. Tauri serializes failures as an object with a stable machine-readable code and a redacted human-readable message:

```json
{
  "code": "ZFSS_VALIDATION",
  "message": "title must contain at least 1 character(s)"
}
```

Validation failures use `ZFSS_VALIDATION`, denied role actions use `ZFSS_FORBIDDEN`, unavailable or invalid user-role state uses `ZFSS_IDENTITY_UNAVAILABLE`, missing entities use `ZFSS_NOT_FOUND`, invalid lifecycle conditions use `ZFSS_CONFLICT`, and repository failures use `ZFSS_REPOSITORY_UNAVAILABLE` or `ZFSS_INTERNAL` as appropriate. Repository messages are centrally redacted, so raw SQLx messages, connection strings, hosts, credentials, and internal context chains are never returned to the frontend.

The frontend admits only this documented envelope and code set. Malformed rejection values fail closed to `An unexpected application error occurred. (ZFSS_INTERNAL)`; their original contents are not displayed.

Rust command-boundary tests send real `InvokeRequest` values through Tauri's generated handlers and assert the serialized rejection object for both validation and role-authority failures. These tests do not stop at helper return values.

### Global Hotkey

**Ctrl+Alt+Z** — toggles signal capture window visibility. Implemented via `tauri-plugin-global-shortcut` with 100ms debounce to prevent rapid re-triggering.
