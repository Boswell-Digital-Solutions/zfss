# ZFSS - Zen Feedback & Service System

**Version:** 1.0.0
**Status:** Phase 1 Complete

Feedback metabolism for the Forge ecosystem. A Tauri v2 desktop application with local PostgreSQL as the authoritative operational source for the ZFSS feedback domain.

## Documentation Contract

- **Repo type:** Desktop/local system
- **Authority boundary:** Local feedback capture and metabolism with repo-local durable state; it is not part of the Forge shared resident-service mesh
- **Deep reference:** `doc/system/_index.md`, generated `doc/ZFSSYSTEM.md`, `../docs/canonical/documentation_protocol_v1.md`
- **README role:** Product overview and local run entrypoint
- **Truth note:** Version lines, phase labels, and implementation totals in this README are snapshot facts unless explicitly marked as canonical doctrine or target values

## Non-Negotiable Architecture Rules

1. **ZFSS PostgreSQL owns ZFSS operational records** - It is not DataForge and does not own canonical ecosystem memory or admitted BDS evidence
2. **Append-only semantics** - No UPDATE/DELETE on canonical records
3. **Cross-system promotion is governed** - Forge Memory may derive candidates; Forge_Command records operator authorization; SMITH applies the exact approved action; Cloud DataForge retains admitted records and receipts
4. **SQLite is optional** - Only as write-behind buffer for offline Signal capture
5. **Lifecycle enforced in code** - No Issue may close without verified Artifact
6. **Role-based authority** - Steward decides, Operator executes, Engineer builds, AI suggests
7. **Promotion is disabled by default** - No ZFSS → Forge Memory producer route, shared-memory promotion, or external mutation is admitted by this repository

See `contracts/authority/zfss-authority.v1.yaml` for the machine-readable authority boundary.

## Canonical Objects

| Object | ID Pattern | Purpose |
|--------|------------|---------|
| Signal | `sig_*` | Raw immutable user expression |
| Issue | `iss_*` | System's understanding (many Signals → one Issue) |
| Decision | `dec_*` | Declared intent (append-only) |
| Artifact | `art_*` | Proof of learning |
| Response | `rsp_*` | Controlled outward communication |

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- PostgreSQL 14+

### Database Setup

```bash
# Create database
createdb zfss

# Run migration
psql -d zfss -f migrations/001_initial_schema.sql
```

### Environment

```bash
# Create .env file
echo 'ZFSS_DATABASE_URL=postgresql://localhost/zfss' > .env
```

### Run Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Test

```bash
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
bash doc/system/BUILD.sh
```

CI also applies every migration to disposable PostgreSQL 16 and runs `tests/postgres_contract.sql`. It proves INSERT succeeds, UPDATE and DELETE fail against a real canonical row, all five mutation guards exist, and the pending-triage view exposes a new unlinked Signal. It never targets a configured production database.

### Build for Production

```bash
npm run tauri build
```

## Global Hotkey

**Ctrl+Alt+Z** - Toggle signal capture window

## File Structure

```
zfss/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # App entry, hotkey, setup
│   │   ├── state.rs              # Shared AppState
│   │   ├── constraints.rs        # Constants and limits
│   │   ├── config/               # Configuration
│   │   ├── db/                   # Database pool
│   │   ├── models/               # Canonical objects
│   │   ├── ipc/                  # Tauri IPC commands
│   │   ├── lifecycle/            # State machine enforcement
│   │   ├── repository/           # Data access
│   │   └── service/              # Business logic
│   └── Cargo.toml
├── src/                          # TypeScript frontend
│   ├── index.html
│   ├── main.ts
│   ├── styles.css
│   └── lib/types.ts
├── migrations/                   # PostgreSQL DDL
│   └── 001_initial_schema.sql
└── package.json
```

## Lifecycle State Machines

### Signal
```
new → linked → responded → closed
  ↘ needs_info ↗
```

### Issue
```
pending_decision → decided → in_progress → ready_for_verification → closed
```

### Response
```
draft → pending → approved → sent
              ↘ blocked
```

## Role-Based Authority

| Action | Steward | Operator | Engineer | AI |
|--------|---------|----------|----------|----|
| Log Signal | ✓ | ✓ | ✓ | ✓ |
| Link Signal to Issue | ✓ | ✓ | ✗ | ✗ |
| Make Decision | ✓ | ✗ | ✗ | ✗ |
| Create Artifact | ✓ | ✗ | ✓ | ✗ |
| Verify Artifact | ✓ | ✗ | ✗ | ✗ |
| Approve Response | ✓ | ✗ | ✗ | ✗ |
| Close Issue | ✓ | ✗ | ✗ | ✗ |

## Implementation Status

### Phase 1: Foundation ✅
- [x] Tauri v2 project structure
- [x] PostgreSQL with sqlx
- [x] Initial schema migration
- [x] Core models and ID types
- [x] Signal capture IPC command
- [x] Global hotkey (Ctrl+Alt+Z)
- [x] Frontend signal capture UI

### Phase 2: CRUD Operations (In Progress)
- [x] Complete repository modules
- [ ] Complete service modules
- [ ] All IPC commands for 5 objects
- [ ] Status history tables

### Phase 3: Lifecycle Enforcement (Pending)
- [ ] State machine enforcement
- [ ] close_requires_artifact rule
- [ ] Role-based authority checks

### Phase 4: Frontend Views (Pending)
- [ ] Issue management UI
- [ ] Decision recording UI
- [ ] Artifact tracking UI
- [ ] Response workflow UI
- [ ] Dashboard

### Phase 5: Offline Support (Optional)
- [ ] SQLite write-behind buffer
- [ ] Sync to PostgreSQL
 
See `docs/ops/render_to_local_cutover_no_data.md` for the no-data Render-to-local Postgres cutover checklist.

## License

MIT
