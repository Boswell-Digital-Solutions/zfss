# ZFSS Local PostgreSQL Operations

Purpose: document the legacy local PostgreSQL binding used as the operational source for the ZFSS feedback domain. The physical database and role names below predate the current authority ruling; they confer no DataForge authority.

## Authority boundary

- ZFSS owns its operational Signals, Issues, lifecycle events, response drafts, and local artifact references.
- Forge Memory may derive candidate memory, preserve contradictions, and construct memory receipts. No ZFSS producer route is admitted in this slice and promotion remains disabled.
- Forge_Command presents evidence and records the operator's bounded authorization.
- SMITH applies only the exact digest-bound action the operator approved.
- Cloud DataForge owns admitted durable BDS evidence, canonical shared memory, decisions, and receipts.
- DataForge Local provides bounded offline continuity where separately admitted; it is not a competing cloud authority.

This document does not authorize a rename, migration, credential change, runtime reconnection, consumer admission, or promotion.

## 1. Create the legacy-bound ZFSS database

Run from a bootstrapper shell with a superuser connection (e.g., `psql postgres`):

```sql
-- database and admin role
CREATE ROLE dataforge_admin LOGIN PASSWORD '<strong-password>' NOINHERIT;
CREATE DATABASE dataforge WITH OWNER dataforge_admin TEMPLATE template0;
```

Then connect to `dataforge` for the remaining grants:

```sql
\c dataforge

-- schema usage
GRANT USAGE ON SCHEMA public TO dataforge_admin;

-- allow admin to create future tables, extensions, and manage pg logical objects
GRANT CREATE, TEMPORARY ON DATABASE dataforge TO dataforge_admin;
GRANT ALL ON SCHEMA public TO dataforge_admin;
```

## 2. Create legacy-named least-privilege roles

```sql
CREATE ROLE dataforge_reader LOGIN PASSWORD '<read-password>' NOINHERIT;
CREATE ROLE dataforge_append LOGIN PASSWORD '<append-password>' NOINHERIT;
CREATE ROLE dataforge_signal_writer LOGIN PASSWORD '<signal-password>' NOINHERIT;
```

Each role must connect through the local instance but cannot escalate privileges:

```sql
GRANT CONNECT ON DATABASE dataforge TO dataforge_reader, dataforge_append, dataforge_signal_writer;
GRANT USAGE ON SCHEMA public TO dataforge_reader, dataforge_append, dataforge_signal_writer;
```

### 2.1 Canonical table permissions

Only the append role may insert records into canonical tables (per `zfss/migrations/001_initial_schema.sql`). Clear append-only rights with:

```sql
GRANT INSERT ON TABLE
    issues,
    signals,
    decisions,
    artifacts,
    responses
TO dataforge_append;
```

History/support tables are also insert-only append structures:

```sql
GRANT INSERT ON TABLE
    issue_status_history,
    signal_status_history,
    attachments,
    response_approval_history,
    audit_log
TO dataforge_append;
```

### 2.2 Legacy remote signal-only role

Legacy remote clients may only supply `signals`. Limit them to that table (and its append-only history if needed). This role does not establish an admitted Cloud DataForge, Forge Memory, or external-write route:

```sql
GRANT INSERT ON TABLE signals TO dataforge_signal_writer;
GRANT INSERT ON TABLE signal_status_history TO dataforge_signal_writer;
GRANT SELECT ON TABLE signals TO dataforge_signal_writer; -- to confirm insert success
```

No UPDATE/DELETE privileges are granted to readers, appenders, or cloud clients.

### 2.3 Read-only analytics

```sql
GRANT SELECT ON ALL TABLES IN SCHEMA public TO dataforge_reader;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO dataforge_reader;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT INSERT ON TABLES TO dataforge_append;
```

This ensures future tables inherit the same append-only model without manual grants.

## 3. Environmental configuration

Per `zfss/src-tauri/src/config/settings.rs`, the frontend/Tauri app reads `ZFSS_DATABASE_URL` (falling back to `DATABASE_URL`) and must point to the authorized append role:

```bash
export ZFSS_DATABASE_URL="postgresql://dataforge_append:<append-password>@localhost:5432/dataforge"
export DATABASE_URL="$ZFSS_DATABASE_URL" # if other tooling uses the default name
```

When performing migrations, schema changes, or admin maintenance, use the admin credentials:

```bash
export DATAFORGE_ADMIN_DATABASE_URL="postgresql://dataforge_admin:<strong-password>@localhost:5432/dataforge"
psql "$DATAFORGE_ADMIN_DATABASE_URL" -f zfss/migrations/001_initial_schema.sql
```

Legacy services that previously wrote to Render must switch to the signal-only role when they submit `signals`; this prevents anyone but local appenders from changing ZFSS operational records.

## 4. Boundary notes

- The local Postgres instance is the operational source for ZFSS objects only. It is not canonical ecosystem memory or admitted BDS evidence.
- SQLite or Render Postgres becomes a read/replay buffer; writes are routed through the append role above.
- Keep connection strings secret. Any credential rotation or cutover requires separate authorization.

## 5. Immutability enforcement (Prompt 3)

The migration `zfss/migrations/002_append_only_enforcement.sql` registers the `zfss_forbid_mutation` trigger and attaches it to every canonical table (`issues`, `signals`, `decisions`, `artifacts`, `responses`). Any `UPDATE` or `DELETE` now raises a clear `'ZFSS doctrine violation'` exception so violations are loud and provable.

This means current code paths that mutate canonical rows (for example `link_signal_to_issue` in `zfss/src-tauri/src/ipc/signal_cmds.rs` which `UPDATE`s `signals` and `issues`) will fail until they move to append-only patterns (e.g., inserting history rows and deriving status from views). Treat trigger errors as proofs that the code must honor the doctrine rather than bypass it.

The append role keeps `INSERT` privileges only, while `dataforge_admin` may still apply schema changes (extensions, new tables). Signal-only legacy clients are limited to inserting into `signals`/`signal_status_history`, keeping the ZFSS operational ledger append-only without granting cross-system authority.
