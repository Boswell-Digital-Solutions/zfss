# ZFSS — Claude Code Context

Zen Feedback & Service System: capture, triage, and respond to user feedback. Tauri desktop app,
**append-only** feedback store.

Canonical reference: `doc/system/` → root `SYSTEM.md` (`bash doc/system/BUILD.sh`). `SYSTEM.md` is
a build artifact; edit the parts, never the artifact.

---

## Boundaries

- **The feedback record is append-only.** Triage and response are new rows, not edits to the
  captured original. Never rewrite a submitted item.
- Do not invent undocumented APIs, tables, routes, or environment variables.

---

## Verification

There is no CI workflow and no test script in this repo. The real checks are the database scripts,
run against a local Postgres:

```bash
bash scripts/check_verify_prereqs.sh     # confirm prerequisites first
bash scripts/verify_local_postgres.sh    # schema/state verification
bash scripts/db_status.sh
```

`scripts/apply_schema.sh`, `export_render_snapshot.sh`, and `import_snapshot_to_local.sh` move
schema and data between the Render deployment and a local database.

---

## Non-obvious

- Snapshot import/export is the supported path for reproducing production state locally — do not
  hand-craft fixture rows to stand in for it.
