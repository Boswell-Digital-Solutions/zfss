# Render Snapshot Export (Prompt 4)

> **Historical migration record:** `DataForgeDB` below is a legacy physical name. This plan grants no DataForge authority and authorizes no current export, import, or migration.

Historical purpose: capture the Render-hosted ZFSS store exactly as it existed, preserving insertion order, timestamps, and foreign-key dependencies for deterministic re-ingest into the local ZFSS operational store.

## 1. Gather Render connection info

- Open the Render dashboard for the `forge-db` database defined in `render-dataforge-only.yaml`.
- Copy the `connectionString` (e.g., `postgres://forge_user:<pw>@<host>.render.com:5432/forge`).
- On the machine running this migration script, export it for reuse:

```bash
export RENDER_DATABASE_URL="postgres://forge_user:<pw>@<host>.render.com:5432/forge"
```

Leave the Render credentials active only for the export phase; later prompts rotate them.

## 2. Run the export script

Use the helper script that orders tables per dependency and timestamps:

```bash
chmod +x zfss/scripts/export_render_snapshot.sh
./zfss/scripts/export_render_snapshot.sh
```

The script:

1. Writes to `$EXPORT_DIR` (default `zfss_render_snapshot` under the current directory).
2. Queries each table with `ORDER BY` on the canonical timestamp (`created_at`, `decided_at`, or `changed_at`) to maintain the ledger’s insertion chronology.
3. Emits CSVs with headers, so no transformations or ambiguity creep in.

Canonical tables are exported before their dependents:

`users` → `issues` → `issue_status_history` → `signals` → `signal_status_history` → `attachments` → `decisions` → `artifacts` → `responses` → `response_approval_history` → `audit_log`.

If additional support tables surface, append them after their dependencies (e.g., history tables after their canonical parents) and keep `ORDER BY` aligned with the relevant timestamp column.

## 3. Verify snapshot integrity

Run quick counts and last-timestamp queries against Render to confirm the export includes the most recent data:

```bash
for tbl in issues signals decisions artifacts responses; do
  psql "$RENDER_DATABASE_URL" -c "SELECT COUNT(*) AS count, MAX(created_at) AS newest FROM ${tbl};"
done
```

For tables without `created_at`, substitute `decided_at` or `changed_at` as documented in `zfss/migrations/001_initial_schema.sql`.

Store the CSV files securely (e.g., `tar -czf dataforge_render_snapshot.tar.gz zfss_render_snapshot/`) before moving to Prompt 5. The next prompt will replay these files against the new local Postgres spool without altering timestamps.
