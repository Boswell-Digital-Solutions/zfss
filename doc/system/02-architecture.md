## 2. Architecture

### System Diagram

```
┌─────────────────────────────────────────┐
│        Frontend (TypeScript)            │
│  - Signal capture UI (main.ts)          │
│  - Views (issues, signals, dashboard)   │
│  - Router + API layer (invoke Tauri)    │
└──────────────┬──────────────────────────┘
               │ Tauri IPC (invoke)
               ▼
┌─────────────────────────────────────────┐
│      Tauri v2 Backend (Rust)            │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │ IPC Commands (23 handlers)      │    │
│  │ signal_cmds, issue_cmds,        │    │
│  │ decision_cmds, artifact_cmds,   │    │
│  │ response_cmds                   │    │
│  └──────────┬──────────────────────┘    │
│             ▼                           │
│  ┌─────────────────────────────────┐    │
│  │ Service Layer                   │    │
│  │ (business logic, validation)    │    │
│  └──────────┬──────────────────────┘    │
│             ▼                           │
│  ┌─────────────────────────────────┐    │
│  │ Repository (append-only)        │    │
│  │ append_signal(), append_issue() │    │
│  │ NO UPDATE / NO DELETE           │    │
│  └──────────┬──────────────────────┘    │
│             ▼                           │
│  ┌─────────────────────────────────┐    │
│  │ sqlx Pool → PostgreSQL          │    │
│  │ (ZFSS operational source)       │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘

Global Hotkey: Ctrl+Alt+Z (debounced toggle)
```

### Data Flow

1. **Capture:** User presses Ctrl+Alt+Z → signal capture window appears → user types feedback → `capture_signal` IPC command → `append_signal()` → PostgreSQL INSERT
2. **Triage:** Steward/Operator links Signal to Issue → `link_signal_to_issue` → status transition logged
3. **Decision:** Steward records Decision for Issue → `record_decision` → append-only insert (can supersede)
4. **Proof:** Engineer creates Artifact → Steward verifies → `verify_artifact`
5. **Response:** Draft → submit → Steward approves → mark sent

### Layer Responsibilities

| Layer | Location | Responsibility |
|-------|----------|----------------|
| Frontend | `src/` | UI rendering, user input, IPC invocation |
| IPC | `src-tauri/src/ipc/` | Parameter validation, state access, command dispatch |
| Service | `src-tauri/src/service/` | Business logic, lifecycle enforcement |
| Repository | `src-tauri/src/repository/` | Append-only data access, SQL queries |
| Database | PostgreSQL | Storage, triggers, views, constraints |

### Key Invariants

- No data crosses the IPC boundary unless serializable via Serde
- All mutations go through the repository layer (no raw SQL in IPC handlers)
- AppState is Arc-wrapped for safe sharing across async commands
- Device ID persisted to `~/.local/share/zfss/device_id.txt`
