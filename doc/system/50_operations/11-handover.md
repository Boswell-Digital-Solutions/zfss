## 11. Handover

### Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Foundation (Tauri, PostgreSQL, Signal capture, hotkey) | Complete |
| Phase 2 | CRUD Operations (repositories and authority service implemented; broader service layer pending) | In Progress |
| Phase 3 | Lifecycle Enforcement (state machines, role checks) | Planned |
| Phase 4 | Frontend Views (issues, decisions, artifacts, dashboard) | Planned |
| Phase 5 | Offline Support (SQLite write-behind buffer) | Optional |

### What Works

- Tauri v2 project builds and launches
- PostgreSQL connection with sqlx
- Schema with 13 tables + 3 views + append-only triggers
- Signal capture via IPC (capture_signal command)
- Global hotkey Ctrl+Alt+Z toggles capture window
- Frontend signal capture UI
- 23 IPC command registrations in main.rs

### Known Issues

- Repository operations and centralized role authorization are implemented; broader service-layer business logic remains pending
- Frontend view modules exist, but the active entrypoint still exposes only signal capture
- The dedicated lifecycle module remains a placeholder; transition logic currently lives outside that layer
- Rust coverage includes typed IDs, fail-closed IPC input validation, redacted repository-error translation, model transitions, role capabilities, IPC authority decisions, fail-closed role resolution, and a PostgreSQL-backed end-to-end repository lifecycle; SQL contracts cover append-only projections
- CI covers frontend build, documentation and authority checks, Rust tests/formatting, migration replay, and the PostgreSQL append-only contract

### Critical Constraints

1. **Never add UPDATE/DELETE to the repository layer** — append-only is a design invariant
2. **Never bypass role checks** — Steward authority is enforced, not suggested
3. **Never close an Issue without a verified Artifact** — `close_requires_artifact` is non-negotiable
4. **Never store secrets in frontend** — all DB access goes through Tauri IPC backend

### Next Priorities

1. Expand the service layer beyond its centralized role-authority checks
2. Implement the dedicated lifecycle state-machine layer
3. Connect the existing router and management views to the active frontend entrypoint
4. Add frontend handling for the structured IPC error envelope
5. Add command-level tests for serialized Tauri failure responses

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
