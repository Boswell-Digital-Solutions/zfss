# zfss - Compiled System Reference

**Designation:** ZFS
**Document role:** Canonical compiled technical reference for the Zen Feedback and Service System
**Source:** `doc/system/`
**Build command:** `bash doc/system/BUILD.sh`
**Document version:** 2.0 (2026-06-22) - canonical compliance migration
**Protocol:** BDS Documentation Protocol v2.0; BDS Repo Documentation System Canonical Compliance Standard

> **Generated artifact warning:** `doc/ZFSSYSTEM.md` is assembled output. Edit
> the source modules under `doc/system/` and rebuild. Hand edits to the
> compiled artifact are overwritten by the next build.

Assembly contract:

- Command: `bash doc/system/BUILD.sh`
- Validation: `bash doc/system/validate_snapshots.sh` runs during assembly
- Primary output: `doc/ZFSSYSTEM.md`

This `doc/system/` tree is the canonical source of truth for zfss. It
uses explicit **truth classes**: canonical facts define the repo role, authority
boundaries, runtime behavior, service contracts, and verification doctrine;
snapshot facts are dated, audit-derived counts and current implementation
inventory that may drift between audits.

| Part | File | Contents |
| --- | --- | --- |
| §1 | `00_overview/01-overview-philosophy.md` | 1. Overview & Philosophy |
| §2 | `00_overview/02-architecture.md` | 2. Architecture |
| §3 | `00_overview/04-project-structure.md` | 4. Project Structure |
| §4 | `10_service-contract/06-frontend.md` | 6. Frontend |
| §5 | `10_service-contract/07-tauri-commands.md` | 7. Tauri Command Interface |
| §6 | `10_service-contract/10-product-surface.md` | Product Surface |
| §7 | `20_runtime/08-backend-internals.md` | 8. Backend Internals |
| §8 | `20_runtime/09-database-schema.md` | 9. Database Schema |
| §9 | `20_runtime/20-runtime.md` | Runtime |
| §10 | `30_dependencies/03-tech-stack.md` | 3. Tech Stack |
| §11 | `30_dependencies/10-ecosystem-integration.md` | 10. Ecosystem Integration |
| §12 | `30_dependencies/40-integrations.md` | Integrations |
| §13 | `50_operations/05-config-env.md` | 5. Configuration & Environment |
| §14 | `50_operations/11-handover.md` | 11. Handover |
| §15 | `50_operations/50-operations.md` | Operations |
| §16 | `99_appendices/30-data.md` | Data |
| §17 | `99_appendices/90-appendices.md` | Appendices |
| §18 | `99_appendices/91-bootstrap-overview.md` | Overview |
| §19 | `99_appendices/92-bootstrap-architecture.md` | Architecture |

## Quick Assembly

```bash
bash doc/system/BUILD.sh
```

---

## 1. Overview & Philosophy

### Identity

**ZFSS** (Zen Feedback & Service System) is a Tauri v2 desktop application that captures, triages, and responds to user feedback with append-only PostgreSQL as the operational source for the ZFSS feedback domain.

- **Purpose:** Feedback metabolism — turning raw user signals into verified learning artifacts
- **Paradigm:** Append-only, lifecycle-governed, role-based authority
- **Deployment:** Local desktop app (Tauri), local PostgreSQL
- **Status:** Phase 1 complete, Phase 2 in progress

### Design Commitments

1. **ZFSS PostgreSQL owns ZFSS operational records** — it is not DataForge, canonical ecosystem memory, or admitted BDS evidence
2. **Append-only semantics** — no UPDATE/DELETE on canonical records, enforced at database level via triggers
3. **Cross-system promotion is governed** — Forge Memory derives candidates; Forge_Command records operator authorization; SMITH applies exact approved actions; Cloud DataForge retains admitted records and receipts
4. **SQLite is optional** — only as write-behind buffer for offline Signal capture
5. **Lifecycle enforced in code** — no Issue may close without verified Artifact
6. **Role-based authority** — Steward decides, Operator executes, Engineer builds, AI suggests
7. **Promotion is disabled by default** — no ZFSS → Forge Memory route or external mutation is admitted here

The machine-readable boundary is `contracts/authority/zfss-authority.v1.yaml`.

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
│   └── ZFSSYSTEM.md                 # Generated canonical output
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

# Product Surface

**Document version:** 1.0 (bootstrap scaffold)

User-facing product surface: routes, flows, and entry points.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

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

PostgreSQL 14+ (local ZFSS operational store). Connected via sqlx async driver with connection pooling. It is not canonical ecosystem memory or admitted BDS evidence.

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

# Runtime

**Document version:** 1.0 (bootstrap scaffold)

Runtime topology, process boundaries, and managed state.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

---

# Data

**Document version:** 1.0 (bootstrap scaffold)

Data model, persistence, and schema migration posture.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

---

## 3. Tech Stack

### Runtime

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 2024 edition | Backend language |
| TypeScript | 5.x | Frontend language |
| Node.js | 18+ | Build tooling |
| PostgreSQL | 14+ | ZFSS operational feedback store |

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

## 10. Ecosystem Integration

### Forge Ecosystem Position

ZFSS occupies the **feedback boundary** of the Forge ecosystem. It captures external user feedback (Signals) and metabolizes them through a governed lifecycle into verified outcomes (Artifacts, Responses).

### Cross-System Authority Model

ZFSS follows the Forge ecosystem's divided-authority doctrine:
- **Local PostgreSQL is the source of truth** (migrated from Render cloud)
- ZFSS PostgreSQL owns ZFSS feedback-domain operational records only
- Forge Memory owns candidate-memory lifecycle and receipts, with promotion disabled by default
- Forge_Command records operator review and bounded authorization
- SMITH applies only the exact digest-bound action approved by the operator
- Cloud DataForge owns admitted durable evidence, canonical shared memory, decisions, and receipts
- DataForge Local provides bounded offline continuity and is not competing cloud authority
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

ZFSS was originally deployed on Render (cloud PostgreSQL). Its operational writes were cut over to local PostgreSQL with:
- Export tooling (`scripts/export_render_snapshot.sh`)
- Import tooling (`scripts/import_snapshot_to_local.sh`)
- Verification (`scripts/verify_migration.py` — row count comparison)
- Credential rotation and cloud service disconnection

See `docs/local_postgres_authority.md` for legacy binding operations and `contracts/authority/zfss-authority.v1.yaml` for the current boundary.

### Future Integration Points

| Service | Integration | Status |
|---------|-------------|--------|
| ForgeCommand | Orchestration of ZFSS health checks | Planned |
| Forge Memory | Candidate memory derived from feedback | Not admitted; disabled |
| Cloud DataForge | Admitted evidence, canonical shared memory, decisions, receipts | Route not admitted |
| DataForge Local | Bounded offline continuity | Route not admitted |
| NeuroForge | AI-assisted Signal triage and classification | Planned |
| BugCheck | Signal-to-Issue correlation with bug findings | Planned |

---

# Integrations

**Document version:** 1.0 (bootstrap scaffold)

External integrations, upstream services, and wire contracts.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

---

# Governance

**Truth class:** canonical doctrine

This documentation system governs ZFSS repo-local implementation truth. It does
not define shared Forge ecosystem doctrine or DataForge cloud authority beyond
the explicit integration and handoff surfaces ZFSS owns.

## Authority Boundary

- `doc/system/` is the canonical authored source tree for ZFSS system truth.
- `doc/ZFSSYSTEM.md` is generated output and must not be edited by hand.
- Supporting docs, plans, and archives outside `doc/system/` are subordinate to
  the compiled system reference when they describe current behavior.
- Runtime behavior and verification evidence override stale prose; when they
  disagree, update the source chapter and rebuild the compiled artifact.

## Change Control

Changes that alter signal capture, issue lifecycle, database schema, response
control, integrations, or local authority boundaries must update the relevant
`doc/system/` chapter in the same change as the implementation.

Documentation-only changes must still rebuild `doc/ZFSSYSTEM.md` with:

```bash
bash doc/system/BUILD.sh
```

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

## 11. Handover

### Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Foundation (Tauri, PostgreSQL, Signal capture, hotkey) | Complete |
| Phase 2 | CRUD Operations (all 5 object repositories implemented; service layer pending) | In Progress |
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

- Repository operations are implemented, but the dedicated service module remains a placeholder
- Frontend view modules exist, but the active entrypoint still exposes only signal capture
- The dedicated lifecycle module remains a placeholder; transition logic currently lives outside that layer
- Rust coverage includes typed IDs, fail-closed model transition matrices, and the role-capability matrix; IPC enforcement and database-backed lifecycle behavior lack direct tests
- CI covers frontend build, documentation and authority checks, Rust tests/formatting, migration replay, and the PostgreSQL append-only contract

### Critical Constraints

1. **Never add UPDATE/DELETE to the repository layer** — append-only is a design invariant
2. **Never bypass role checks** — Steward authority is enforced, not suggested
3. **Never close an Issue without a verified Artifact** — `close_requires_artifact` is non-negotiable
4. **Never store secrets in frontend** — all DB access goes through Tauri IPC backend

### Next Priorities

1. Wire the service layer with business logic and role checks
2. Implement the dedicated lifecycle state-machine layer
3. Connect the existing router and management views to the active frontend entrypoint
4. Add direct tests for IPC validation, role enforcement, and database-backed lifecycle transitions
5. Add repository integration cases beyond the append-only database contract

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

---

# Operations

**Document version:** 1.0 (bootstrap scaffold)

Deployment, observability, incident response, and bounded repair.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

---

# Appendices

**Document version:** 1.0 (carry-forward)

Appendices, glossary, and cross-references.

## Unmapped legacy chapters

The following legacy chapters were carried forward but could not be
deterministically mapped to a class-aware slot. Review and place them by
hand:

- `ZFSS — System Documentation`
- `Setup`
- `Development`
- `Build`

---

# Overview

**Document version:** 1.0 (bootstrap scaffold)

System identity, role, and boundary with the rest of the Forge ecosystem.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.

---

# Architecture

**Document version:** 1.0 (bootstrap scaffold)

High-level architecture, authority posture, and surface ownership.

> This chapter is a registry-generated bootstrap scaffold for a
> `application` class documentation system. Replace this placeholder with
> real authored content. Registry will not invent repo truth that is not
> already present in the repo.
