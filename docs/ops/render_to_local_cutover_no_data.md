# Render-to-Local Cutover (No Data)

> **Historical runbook:** This document predates `contracts/authority/zfss-authority.v1.yaml`; it authorizes no present cutover, credential change, or runtime reconnection.

Purpose: document the historical schema-only cutover from Render Postgres to the local ZFSS operational store when no production data needed migration.

## 1. Assumptions

- Render Postgres never saw production traffic; the migration is schema-only.
- Local Postgres will hold ZFSS feedback-domain operational objects. It does not own canonical ecosystem memory or admitted BDS evidence.
- Append-only/insertion guards are defined via `zfss/migrations/002_append_only_enforcement.sql` and described in `zfss/docs/local_postgres_authority.md`.

## 2. Local Postgres bring-up

1. Install/start PostgreSQL 14+ (`systemctl start postgresql` or equivalent container command).
2. Create the `dataforge` database and admin role if they do not exist:
   ```bash
   sudo -iu postgres psql <<'SQL'
   CREATE ROLE dataforge_admin LOGIN PASSWORD '<strong-password>' NOINHERIT;
   CREATE DATABASE dataforge OWNER dataforge_admin TEMPLATE template0;
   \c dataforge
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   SQL
   ```
3. Give the admin schema privileges and CREATE rights (see `zfss/docs/local_postgres_authority.md:15-26`).

## 3. Schema initialization

- Apply the canonical schema+append-only migrations:
  ```bash
  psql postgresql://dataforge_admin:<strong-password>@localhost:5432/dataforge -f zfss/migrations/001_initial_schema.sql
  psql ... -f zfss/migrations/002_append_only_enforcement.sql
  psql ... -f zfss/migrations/003_signal_link_events.sql
  ```
- These scripts create tables (`signals`, `issues`, histories, `signal_links`, etc.) plus the immutability triggers.

## 4. Role creation + grants

- Follow `zfss/docs/local_postgres_authority.md:28-87` to create:
  - `dataforge_reader` (SELECT-only)
  - `dataforge_append` (INSERT on canonical/history tables)
  - `dataforge_signal_writer` (INSERT on `signals`/`signal_status_history`)
- Ensure default privileges and future tables inherit the same grant set.

## 5. Guard verification checks

Run:
```bash
psql postgresql://dataforge_append:<append-password>@localhost:5432/dataforge -c "UPDATE signals SET status='linked' WHERE false;"
```
—should fail with the trigger error `'ZFSS doctrine violation'`.
```bash
psql postgresql://dataforge_reader:<read-password>@localhost:5432/dataforge -c "\dt"
```
—should see the canonical tables listed.
```bash
psql postgresql://dataforge_signal_writer:<signal-password>@localhost:5432/dataforge -c "INSERT INTO signals (id, source, raw_text, status, created_by) VALUES ('sig_test', 'in_app', 'test', 'new', 'system');"
```
—should succeed if the table is empty; confirm the row exists via the reader role.

## 6. App configuration changes

- Update `.env`, `zfss/src-tauri/.env`, and any Docker/Compose files to point to the append role:
  ```
  ZFSS_DATABASE_URL=postgresql://dataforge_append:<append-password>@localhost:5432/dataforge
  DATABASE_URL="$ZFSS_DATABASE_URL"
  ```
- Migration tools use `DATAFORGE_ADMIN_DATABASE_URL=postgresql://dataforge_admin:<strong-password>@localhost:5432/dataforge`.
- Cloud services that merely submit signals should use the signal writer URL `postgresql://dataforge_signal_writer:<signal-password>@localhost:5432/dataforge`.

## 7. Smoke test checklist

1. **Signal capture**: use the append URL to `INSERT` a test signal and confirm `signal_status_history` recorded `'new'`.
2. **Read verification**: use the reader URL to `SELECT count(*) FROM signals;` and confirm the count reflects your test insert.
3. **Immutability test**: attempt an `UPDATE` or `DELETE` via append role and ensure the trigger blocks it (error message referencing `ZFSS doctrine violation`).
4. **Link guard**: insert into `signal_links` and ensure `audit_log` receives an entry (this may be part of repository logic).

If any check fails, revisit the trigger migration or role grants before proceeding.

## 8. HUMAN-REQUIRED

- **Render credential retirement** – Rotate or revoke the Render `forge-db` credentials; ensure the old Render `DATABASE_URL` is removed from `render-dataforge-only.yaml` and any live deployment. This cannot be executed inside this repo/sandbox.
- **Secrets vault update** – Seed the new append/signal/read/admin passwords into your secrets manager/vault before updating services.
- **Postgres service scope** – Ensure CC4 operations know that Render now acts strictly as an offline backup; do not re-enable writes on the cloud DB.
