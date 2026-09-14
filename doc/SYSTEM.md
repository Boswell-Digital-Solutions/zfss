# ZFSS — System Documentation

> **Superseded snapshot:** The canonical compiled reference is `doc/ZFSSYSTEM.md`, generated from `doc/system/`. This legacy snapshot is retained for history and must not be used for authority decisions.

**Document version:** 1.1 (2026-03-06) — Normalized to Forge Documentation Protocol v1
**Protocol:** Forge Documentation Protocol v1

> Zen Feedback & Service System — append-only feedback metabolism for the Forge ecosystem.
> "Capture. Triage. Decide. Prove. Respond."

This `doc/system/` tree uses explicit truth classes:
- Canonical facts define ZFSS's append-only doctrine, local authority model, and ecosystem boundary.
- Snapshot facts define audit-derived counts such as commands, tables, migrations, tests, or implementation inventory.

Assembly contract:
- Command: `bash doc/system/BUILD.sh`
- Output: `doc/zsSYSTEM.md`

| Part | File | Contents |
|------|------|----------|
| §1 | `01-overview-philosophy.md` | Service identity, append-only doctrine, ecosystem role |
| §2 | `02-architecture.md` | Tauri v2 IPC bridge, repository pattern, data flow |
| §3 | `03-tech-stack.md` | Rust 2024, TypeScript 5, Tauri 2, sqlx, dependencies |
| §4 | `04-project-structure.md` | Dual source trees (src-tauri/ + src/), module map |
| §5 | `05-config-env.md` | Environment variables, settings, device ID persistence |
| §6 | `06-frontend.md` | Signal capture UI, views, router, API layer |
| §7 | `07-tauri-commands.md` | Tauri command surface across canonical object domains |
| §8 | `08-backend-internals.md` | Models, lifecycle state machines, repository, roles |
| §9 | `09-database-schema.md` | Database schema, append-only triggers, and migration guidance |
| §10 | `10-ecosystem-integration.md` | DataForge authority, Forge design patterns, future hooks |
| §11 | `11-handover.md` | Implementation status, known issues, next priorities |

## Quick Assembly

```bash
bash doc/system/BUILD.sh   # Assembles all parts into doc/zsSYSTEM.md
```

*Last updated: 2026-03-06*

---

## 1. Overview & Philosophy

### Identity

**ZFSS** (Zen Feedback & Service System) is a Tauri v2 desktop application that captures, triages, and responds to user feedback with append-only PostgreSQL as the authoritative data store.

- **Purpose:** Feedback metabolism — turning raw user signals into verified learning artifacts
- **Paradigm:** Append-only, lifecycle-governed, role-based authority
- **Deployment:** Local desktop app (Tauri), local PostgreSQL
- **Status:** Phase 1 complete, Phase 2 in progress

### Design Commitments

1. **DataForgeDB (local PostgreSQL) is authoritative** — single source of truth, no cloud authority
2. **Append-only semantics** — no UPDATE/DELETE on canonical records, enforced at database level via triggers
3. **Cloud services are stateless consumers** — can only read or submit new Signals
4. **SQLite is optional** — only as write-behind buffer for offline Signal capture
5. **Lifecycle enforced in code** — no Issue may close without verified Artifact
6. **Role-based authority** — Steward decides, Operator executes, Engineer builds, AI suggests

### Canonical Objects

| Object | ID Pattern | Purpose |
|--------|------------|---------|
| Signal | `sig_*` | Raw immutable user expression |
| Issue | `iss_*` | System's understanding (many Signals → one Issue) |
| Decision | `dec_*` | Declared intent (append-only, can supersede) |
| Artifact | `art_*` | Proof of learning (verified by Steward) |
| Response | `rsp_*` | Controlled outward communication |

### Ecosystem Role

ZFSS sits at the feedback boundary of the Forge ecosystem. User-facing signals flow in, get triaged into Issues, receive Decisions, produce Artifacts (proof of learning), and generate controlled Responses back to users. The append-only model ensures a complete audit trail of every feedback interaction.

---

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
│  │ (local, authoritative)          │    │
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

---

## 3. Tech Stack

### Runtime

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 2024 edition | Backend language |
| TypeScript | 5.x | Frontend language |
| Node.js | 18+ | Build tooling |
| PostgreSQL | 14+ | Authoritative data store |

### Framework

| Component | Version | Purpose |
|-----------|---------|---------|
| Tauri | 2.0 | Desktop app framework |
| Vite | 5.x | Frontend build system |

### Rust Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 2.0 | Desktop framework core |
| tauri-plugin-global-shortcut | 2.0 | Ctrl+Alt+Z hotkey |
| sqlx | 0.7 | Async PostgreSQL driver |
| tokio | 1.x | Async runtime (multi-threaded) |
| serde / serde_json | 1.x | Serialization |
| uuid | 1.x | v4 UUID generation |
| chrono | 0.4 | Timezone-aware timestamps |
| directories | 5.x | OS-specific app data paths |
| anyhow | 1.x | Error handling |
| thiserror | 1.x | Typed errors |
| regex | 1.x | Input validation |
| rand | 0.8 | Random generation |
| dotenvy | 0.15 | .env file loading |

### Frontend Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| @tauri-apps/api | 2.0 | Tauri IPC bridge |

### Build Tooling

| Tool | Purpose |
|------|---------|
| @tauri-apps/cli | Desktop app build/dev |
| vite | Dev server + bundling |
| typescript | Type checking |

### Build Targets

- Linux: `.deb`, `.appimage`
- Dev server: `http://localhost:5173`
- Frontend output: `dist/`
- Browser targets: ES2021, Chrome 100+, Safari 13+

---

## 4. Project Structure

```
zfss/
├── src/                             # TypeScript frontend
│   ├── index.html                   # HTML entry point
│   ├── main.ts                      # Signal capture UI + initialization
│   ├── styles.css                   # Global styles
│   └── lib/
│       ├── api.ts                   # Tauri IPC wrappers (25+ functions)
│       ├── router.ts                # Hash-based SPA router
│       └── types.ts                 # Type definitions for all models
│
├── src-tauri/                       # Rust backend
│   ├── Cargo.toml                   # Rust dependencies
│   ├── tauri.conf.json              # Tauri app configuration
│   └── src/
│       ├── main.rs                  # Entry: Tauri setup, hotkey, 23 commands
│       ├── state.rs                 # AppState (pool, settings, device_id)
│       ├── constraints.rs           # Constants (MAX_RAW_TEXT_BYTES, etc.)
│       ├── config/
│       │   └── settings.rs          # Env-driven configuration
│       ├── db/
│       │   └── pool.rs              # PgPool creation + health check
│       ├── models/
│       │   ├── ids.rs               # Typed IDs (SignalId, IssueId, etc.)
│       │   ├── signal.rs            # Signal + SignalSource + SignalStatus
│       │   ├── issue.rs             # Issue + Classification + Severity
│       │   ├── decision.rs          # Decision + DecisionType
│       │   ├── artifact.rs          # Artifact + ArtifactType
│       │   ├── response.rs          # Response + ResponseChannel
│       │   └── user.rs              # User + UserRole enum
│       ├── ipc/
│       │   ├── signal_cmds.rs       # capture, list, get, link
│       │   ├── issue_cmds.rs        # create, list, get, transition
│       │   ├── decision_cmds.rs     # record, get, list, current
│       │   ├── artifact_cmds.rs     # create, get, list, verify, has_verified
│       │   └── response_cmds.rs     # draft, get, list, submit, approve, block, sent
│       ├── lifecycle/
│       │   └── mod.rs               # State machine enforcement
│       ├── repository/
│       │   └── mod.rs               # Append-only data access
│       ├── service/
│       │   └── mod.rs               # Business logic
│       ├── offline/
│       │   └── mod.rs               # Optional SQLite buffer
│       └── util/
│           └── paths.rs             # App data directory helpers
│
├── migrations/                      # PostgreSQL DDL
│   ├── 001_initial_schema.sql       # 11 tables + 7 views
│   ├── 002_append_only_enforcement.sql  # Mutation triggers
│   └── 003_signal_link_events.sql   # Signal linking history
│
├── scripts/                         # Operational tooling
│   ├── apply_schema.sh              # Apply migrations
│   ├── db_status.sh                 # Check DB connectivity
│   ├── verify_local_postgres.sh     # Verify local setup
│   ├── export_render_snapshot.sh    # Export from Render
│   ├── import_snapshot_to_local.sh  # Import to local
│   ├── verify_migration.py          # Row count verification
│   └── check_verify_prereqs.sh     # Prereq checks
│
├── docs/                            # Operational documentation
│   ├── ops/                         # Runbooks
│   ├── final_doctrine_review.md     # Append-only doctrine
│   ├── local_postgres_authority.md  # Cutover documentation
│   └── ...                          # Migration/authority docs
│
├── doc/                             # Forge Documentation Protocol docs
│   ├── system/                      # Modular sections
│   │   ├── _index.md
│   │   ├── BUILD.sh
│   │   └── 01-*.md through 11-*.md
│   └── zsSYSTEM.md                  # Assembled output
│
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

### Key File Counts

| Category | Count |
|----------|-------|
| TypeScript source files | 6 |
| Rust source files | 25 |
| IPC command modules | 5 |
| Database tables | 11 |
| Database views | 7 |
| Migrations | 3 |
| Operational scripts | 7 |

---

## 5. Configuration & Environment

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ZFSS_DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `DATABASE_URL` | Fallback | — | Used if `ZFSS_DATABASE_URL` not set |
| `ZFSS_USER_ID` | No | auto-generated | Current user UUID |
| `ZFSS_USER_ROLE` | No | `steward` | User role (steward/operator/engineer/ai) |
| `ZFSS_ALWAYS_ON_TOP` | No | `false` | Keep capture window above all windows |

### Settings (config/settings.rs)

```rust
pub struct Settings {
    pub database_url: String,      // PostgreSQL connection URL
    pub user_id: String,           // Current user identifier
    pub user_role: UserRole,       // Role for authority checks
    pub always_on_top: bool,       // Window z-order preference
}
```

Settings are loaded from environment on startup. The database URL is validated to ensure it points to PostgreSQL (must start with `postgresql://` or `postgres://`).

### Device ID

A persistent device UUID is stored at:
```
~/.local/share/zfss/device_id.txt
```

Generated on first launch via `uuid::Uuid::new_v4()`. Used to identify the device across sessions.

### .env File

```bash
ZFSS_DATABASE_URL=postgresql://localhost/zfss
```

Loaded via `dotenvy` on application startup.

### Database Connection

The `db/pool.rs` module creates a `PgPool` with:
- Connection string from Settings
- Health check on startup (`SELECT 1`)
- Pool shared via `AppState` (Arc-wrapped)

### Tauri Configuration (tauri.conf.json)

| Setting | Value |
|---------|-------|
| Product name | `zfss` |
| Identifier | `com.forge.zfss` |
| Window title | `ZFSS - Signal Capture` |
| Window size | 600 × 400 |
| Dev URL | `http://localhost:5173` |
| Build output | `../dist` |
| Bundle targets | `deb`, `appimage` |

---

## 6. Frontend

### Technology

Vanilla TypeScript SPA (no framework). Vite 5 for dev server and bundling. Tauri IPC for backend communication.

### Entry Point (main.ts)

The signal capture UI is the primary entry point:
- Text area for raw feedback
- Source selector (email, phone, in-person, app, social, other)
- Submit button → invokes `capture_signal` via Tauri IPC
- Hotkey toggle awareness (window show/hide)

### Router (lib/router.ts)

Hash-based SPA router with simple pattern matching:

| Route | View | Status |
|-------|------|--------|
| `#/capture` | Signal capture form | Implemented |
| `#/signals` | Signals list (pending triage) | Planned |
| `#/signals/:id` | Signal detail + linking | Planned |
| `#/issues` | Issues list | Planned |
| `#/issues/:id` | Issue detail + responses | Planned |
| `#/dashboard` | Overview/statistics | Planned |

### API Layer (lib/api.ts)

25+ TypeScript functions wrapping Tauri `invoke()` calls:

```typescript
// Signal operations
captureSignal(raw_text, source, user_id, device_id)
listSignals()
getSignal(signal_id)
linkSignalToIssue(signal_id, issue_id)

// Issue operations
createIssue(title, description, ...)
listIssues()
getIssue(issue_id)
transitionIssue(issue_id, new_status)

// Decision operations
recordDecision(issue_id, decision_type, body)
getDecision(decision_id)
listDecisionsForIssue(issue_id)
getCurrentDecision(issue_id)

// Artifact operations
createArtifact(issue_id, artifact_type, title, url)
getArtifact(artifact_id)
listArtifactsForIssue(issue_id)
verifyArtifact(artifact_id, verified_by)
hasVerifiedArtifact(issue_id)

// Response operations
draftResponse(signal_id, channel, body)
getResponse(response_id)
listResponsesForSignal(signal_id)
submitResponse(response_id)
approveResponse(response_id, approved_by)
blockResponse(response_id, reason)
markResponseSent(response_id)
```

### Types (lib/types.ts)

TypeScript type definitions mirroring Rust models:
- Status enums: `SignalStatus`, `IssueStatus`, `ApprovalState`
- Model interfaces: `Signal`, `Issue`, `Decision`, `Artifact`, `Response`
- Role enum: `UserRole`

### Styling

Global CSS in `styles.css`. No CSS framework. Minimal UI focused on fast signal capture.

---

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

Command errors are serialized as strings back to the frontend. All errors propagated via `Result<T, String>` return type.

### Global Hotkey

**Ctrl+Alt+Z** — toggles signal capture window visibility. Implemented via `tauri-plugin-global-shortcut` with 100ms debounce to prevent rapid re-triggering.

---

## 8. Backend Internals

### Models (models/)

All models use typed ID prefixes for safety:

| Type | Prefix | Format |
|------|--------|--------|
| SignalId | `sig_` | `sig_{uuid}` |
| IssueId | `iss_` | `iss_{uuid}` |
| DecisionId | `dec_` | `dec_{uuid}` |
| ArtifactId | `art_` | `art_{uuid}` |
| ResponseId | `rsp_` | `rsp_{uuid}` |
| AttachmentId | `att_` | `att_{uuid}` |

All models derive `Serialize`, `Deserialize`, `Clone`, `Debug` for Serde/IPC compatibility.

### Lifecycle State Machines

#### Signal Lifecycle

```
new → linked → responded → closed
  ↘ needs_info ↗
```

#### Issue Lifecycle

```
pending_decision → decided → in_progress → ready_for_verification → closed
```

**Constraint:** Issue cannot transition to `closed` unless `has_verified_artifact()` returns `true`.

#### Response Lifecycle

```
draft → pending → approved → sent
              ↘ blocked
```

### Role-Based Authority

| Action | Steward | Operator | Engineer | AI |
|--------|---------|----------|----------|-----|
| Log Signal | Yes | Yes | Yes | Yes |
| Link Signal to Issue | Yes | Yes | No | No |
| Make Decision | Yes | No | No | No |
| Create Artifact | Yes | No | Yes | No |
| Verify Artifact | Yes | No | No | No |
| Approve Response | Yes | No | No | No |
| Close Issue | Yes | No | No | No |

### Repository Layer (repository/)

Enforces append-only semantics at the code level:
- `append_signal()` — INSERT only
- `append_issue()` — INSERT only
- `append_decision()` — INSERT only (can supersede previous)
- `append_artifact()` — INSERT only
- `append_response()` — INSERT only
- Status transitions use INSERT to history table + UPDATE to current status field

**No `update_*()` or `delete_*()` functions exist.** This is a design invariant, not an oversight.

### Service Layer (service/)

Business logic enforcement:
- Input validation (text length, enum values)
- Role authority checks before mutations
- Lifecycle transition validation
- `close_requires_artifact` rule enforcement

### Constraints (constraints.rs)

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_RAW_TEXT_BYTES` | 10,000 | Signal text size limit |
| `HOTKEY_DEBOUNCE_MS` | 100 | Hotkey re-trigger cooldown |

---

## 9. Database Schema

### Database

PostgreSQL 14+ (local, authoritative). Connected via sqlx async driver with connection pooling.

### Tables (11)

| Table | Purpose | Append-Only |
|-------|---------|-------------|
| `users` | Role assignments (Steward, Operator, Engineer, AI) | No (mutable) |
| `signals` | Raw immutable user expressions | Yes |
| `signal_status_history` | Signal state transition log | Yes |
| `attachments` | Files attached to Signals | Yes |
| `issues` | System's grouped understanding | Yes |
| `issue_status_history` | Issue state transition log | Yes |
| `decisions` | Declared intent (can supersede) | Yes |
| `artifacts` | Proof of learning | Yes |
| `responses` | Controlled outbound communications | Yes |
| `response_approval_history` | Response approval state log | Yes |
| `audit_log` | System-wide audit trail | Yes |

### Views (7)

Database views provide common query patterns (signal counts by status, issues pending decision, etc.).

### Append-Only Enforcement

Migration `002_append_only_enforcement.sql` creates a `zfss_forbid_mutation()` trigger function that blocks UPDATE and DELETE on canonical tables:

```sql
-- Applied to: signals, issues, decisions, artifacts, responses
CREATE TRIGGER forbid_mutation
  BEFORE UPDATE OR DELETE ON {table}
  FOR EACH ROW
  EXECUTE FUNCTION zfss_forbid_mutation();
```

**Exception:** The single `status` field on canonical tables allows UPDATE for state transitions. Status changes are also logged to the corresponding `*_status_history` table via INSERT.

### Migrations

| File | Description |
|------|-------------|
| `001_initial_schema.sql` | 11 tables + 7 views + indexes |
| `002_append_only_enforcement.sql` | Mutation-blocking triggers |
| `003_signal_link_events.sql` | Signal linking history tracking |

### ID Generation

All IDs are prefixed UUIDs generated in Rust:

```rust
SignalId(format!("sig_{}", Uuid::new_v4()))
```

### Key Constraints

- Foreign keys enforce referential integrity (Signal → Issue, Decision → Issue, etc.)
- `issues.closed_at` can only be set when a verified Artifact exists
- `decisions` can supersede previous decisions for the same Issue (append, don't update)
- All timestamps are `timestamptz` (timezone-aware)

---

## 10. Ecosystem Integration

### Forge Ecosystem Position

ZFSS occupies the **feedback boundary** of the Forge ecosystem. It captures external user feedback (Signals) and metabolizes them through a governed lifecycle into verified outcomes (Artifacts, Responses).

### DataForge Authority Model

ZFSS follows the Forge ecosystem's authority doctrine:
- **Local PostgreSQL is the source of truth** (migrated from Render cloud)
- No cloud service holds authoritative state
- The database is the contract — append-only semantics enforced at the trigger level

### Shared Patterns

| Pattern | ZFSS Implementation |
|---------|---------------------|
| Append-only writes | PostgreSQL triggers + repository layer |
| Lifecycle state machines | Signal, Issue, Response state enums |
| Role-based authority | 4 roles (Steward, Operator, Engineer, AI) |
| Typed IDs | Prefixed UUIDs (`sig_*`, `iss_*`, etc.) |
| Tauri v2 desktop | Same framework as Forge:SMITH and ForgeCommand |

### Render-to-Local Migration

ZFSS was originally deployed on Render (cloud PostgreSQL). The authority was cut over to local PostgreSQL with:
- Export tooling (`scripts/export_render_snapshot.sh`)
- Import tooling (`scripts/import_snapshot_to_local.sh`)
- Verification (`scripts/verify_migration.py` — row count comparison)
- Credential rotation and cloud service disconnection

See `docs/local_postgres_authority.md` for the full cutover documentation.

### Future Integration Points

| Service | Integration | Status |
|---------|-------------|--------|
| ForgeCommand | Orchestration of ZFSS health checks | Planned |
| DataForge | Centralized Signal/Issue analytics | Planned |
| NeuroForge | AI-assisted Signal triage and classification | Planned |
| BugCheck | Signal-to-Issue correlation with bug findings | Planned |

---

## 11. Handover

### Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Foundation (Tauri, PostgreSQL, Signal capture, hotkey) | Complete |
| Phase 2 | CRUD Operations (all 5 object repositories + services) | In Progress |
| Phase 3 | Lifecycle Enforcement (state machines, role checks) | Planned |
| Phase 4 | Frontend Views (issues, decisions, artifacts, dashboard) | Planned |
| Phase 5 | Offline Support (SQLite write-behind buffer) | Optional |

### What Works

- Tauri v2 project builds and launches
- PostgreSQL connection with sqlx
- Schema with 11 tables + 7 views + append-only triggers
- Signal capture via IPC (capture_signal command)
- Global hotkey Ctrl+Alt+Z toggles capture window
- Frontend signal capture UI
- 23 IPC command registrations in main.rs

### Known Issues

- Repository and service modules are stubs (signatures exist, implementation pending)
- Frontend views beyond signal capture are planned but not built
- Lifecycle enforcement exists as module structure but not yet wired
- No test suite yet
- No CI/CD pipeline

### Critical Constraints

1. **Never add UPDATE/DELETE to the repository layer** — append-only is a design invariant
2. **Never bypass role checks** — Steward authority is enforced, not suggested
3. **Never close an Issue without a verified Artifact** — `close_requires_artifact` is non-negotiable
4. **Never store secrets in frontend** — all DB access goes through Tauri IPC backend

### Next Priorities

1. Complete repository module implementations (append_issue, append_decision, etc.)
2. Wire service layer with business logic
3. Implement lifecycle state machine enforcement
4. Build frontend views for Issue management
5. Add test coverage for IPC commands and lifecycle transitions

### Dev Quickref

```bash
# Setup
createdb zfss
psql -d zfss -f migrations/001_initial_schema.sql
psql -d zfss -f migrations/002_append_only_enforcement.sql
psql -d zfss -f migrations/003_signal_link_events.sql
echo 'ZFSS_DATABASE_URL=postgresql://localhost/zfss' > .env

# Development
npm install
npm run tauri dev

# Build
npm run tauri build
```
