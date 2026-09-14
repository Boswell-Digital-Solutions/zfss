# Local Re-ingest Plan (Prompt 5)

> **Historical migration plan:** This document predates `contracts/authority/zfss-authority.v1.yaml`; it authorizes no present migration and grants no DataForge authority.

Purpose: replay the Render-exported ZFSS ledger CSVs into the local ZFSS operational PostgreSQL instance while keeping timestamps, foreign keys, and immutability intact.

## 1. Preconditions

- Ensure the Render snapshot exists (see `zfss/docs/render_export_plan.md`), e.g., `zfss_render_snapshot/{users,issues,...}.csv`.
- Confirm the local Postgres schema is ready (`zfss/migrations/001_initial_schema.sql`) and the append-only enforcement migration (`zfss/migrations/002_append_only_enforcement.sql`) has run.
- Set `LOCAL_DATABASE_URL` to the append role connection string (same as `ZFSS_DATABASE_URL`), and export it for the import script:

```bash
export LOCAL_DATABASE_URL="postgresql://dataforge_append:<append-password>@localhost:5432/dataforge"
export EXPORT_DIR="${EXPORT_DIR:-$(pwd)/zfss_render_snapshot}"
```

## 2. Import script

Use the deterministic loader:

```bash
chmod +x zfss/scripts/import_snapshot_to_local.sh
./zfss/scripts/import_snapshot_to_local.sh
```

Highlights:

- Loads tables in the dependency-safe order (`users → issues → history → signals → attachments → decisions → artifacts → responses → audit log`).
- Uses `\copy` so each row keeps its exported values and timestamps.
- Resets the `BIGSERIAL` sequences for append-only history/audit tables so future inserts continue with higher IDs (`issue_status_history`, `signal_status_history`, `response_approval_history`, `audit_log`).
- Never issues `UPDATE`/`DELETE`, so triggers in `zfss/migrations/002_append_only_enforcement.sql` stay satisfied.

If you need to rerun, manually `TRUNCATE` the tables (obeying FK order) before reimporting to avoid duplicate PKs.

## 3. Verification

After the script succeeds, run these checks to prove the replay is accurate:

```bash
for tbl in users issues signals decisions artifacts responses; do
  psql "$LOCAL_DATABASE_URL" -c "SELECT COUNT(*) AS count, MAX(created_at) AS newest FROM ${tbl};"
done

psql "$LOCAL_DATABASE_URL" -c "SELECT COUNT(*) AS count, MAX(changed_at) AS newest FROM issue_status_history;"
psql "$LOCAL_DATABASE_URL" -c "SELECT COUNT(*) AS count, MAX(changed_at) AS newest FROM signal_status_history;"
psql "$LOCAL_DATABASE_URL" -c "SELECT COUNT(*) AS count, MAX(changed_at) AS newest FROM response_approval_history;"
```

Compare against equivalent queries executed against Render (from Prompt 4) to ensure the row counts/timestamps line up. If mismatched, re-export/reimport after confirming the Render source has not mutated since the export.

For offline execution guidance (when PyPI is unreachable) see `zfss/docs/ops/verify_migration_offline.md`.

Finally, archive the snapshot CSVs (e.g., `tar -czf dataforge_render_snapshot.tar.gz -C zfss_render_snapshot .`) for auditing what data was replayed. Keep the archive read-only as historical migration evidence; retention does not promote it to canonical ecosystem memory or admitted BDS evidence.
