# ZFSS Extended Roadmap

**Document version:** 1.0 (2026-04-03) — Baseline protocol adoption

## Current Status

| Phase | Status | Outcome |
| --- | --- | --- |
| Documentation normalization | Complete | Protocol-required baseline surfaces are present |
| Module catalog expansion | Pending | Expand exact routes, tables, and runtime contracts from code |
| QA alignment | In progress | CI baseline and PostgreSQL contract are active; IPC and lifecycle coverage remain |

## Phase 0 — Documentation Normalization

**Goal:** establish the required documentation stack and build surfaces.

**Delivered:**

- root `CLAUDE.md`
- modular `doc/system/`
- `doc/system/BUILD.sh`
- generated `doc/ZFSSYSTEM.md`
- `scripts/context-bundle.sh`

## Phase 1 — Exact Surface Expansion

**Goal:** replace baseline placeholders with exact module, API, schema, and environment documentation.

## Phase 2 — Verification Hardening

**Goal:** align repo-specific testing, QA, and handover documentation with current implementation reality.

**Delivered:** frontend build, documentation/authority validation, Rust unit-test and formatting
gates, full migration replay, a disposable PostgreSQL append-only contract test, fail-closed model
transition matrices, and the complete role-capability matrix.

**Remaining:** direct IPC role-enforcement, database-backed lifecycle-transition, and repository
integration tests.
