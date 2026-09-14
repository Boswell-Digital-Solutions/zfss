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
