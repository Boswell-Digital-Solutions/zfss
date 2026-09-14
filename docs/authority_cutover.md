# Authority Cutover (Prompt 7)

> **Superseded authority language:** This historical cutover record predates `contracts/authority/zfss-authority.v1.yaml`. It does not establish DataForge authority and authorizes no present credential, migration, or runtime action.

Historical purpose: transition ZFSS writes from Render to its local operational PostgreSQL store and limit legacy remote clients to appending Signals.

## 1. Disable Render writes

1. Rotate or delete the `forge-db` database credentials shown in `render-dataforge-only.yaml` (the `forge_user` account). This ensures any leaked Render secrets cannot write after the cutover.
2. Remove the Render env var that provided `DATABASE_URL` to the `dataforge` service so the Render deployment cannot open new connections. (In practice this means pruning the entry under `envVars` → `DATABASE_URL` in `render-dataforge-only.yaml` and redeploying with the empty/missing key.)
3. Optional: Pause the Render `dataforge` service until you confirm local Postgres and clients are receiving traffic so no race occurs.

## 2. Historical application connection change

- Update every `.env`, Docker/Compose, and deployment configuration that previously relied on Render’s `DATABASE_URL` to instead reference the append-only role defined in `zfss/docs/local_postgres_authority.md`:

```bash
export ZFSS_DATABASE_URL="postgresql://dataforge_append:<append-password>@localhost:5432/dataforge"
export DATABASE_URL="$ZFSS_DATABASE_URL"
```

- Ensure any migration or admin tooling uses the admin connection string (`postgresql://dataforge_admin:<strong-password>@localhost:5432/dataforge`) so schema changes still succeed; reserve the append role solely for regular writes.
- Update hard-coded docs or scripts (e.g., `README.md` references to Render) to point readers at the local instance once the cutover is complete.

## 3. Enforce cloud services as signal-only

- Cloud consumers that previously wrote to Render must now authenticate with the `dataforge_signal_writer` role (`GRANT INSERT ON signals`/`signal_status_history` only) so they can only append new signals and not mutate canonical tables.
- Document these credentials rotation and access expectations in your cloud service runbooks. If cloud services still expect `DATABASE_URL`, replace that secret with `ZFSS_SIGNAL_DATABASE_URL` pointing at the signal-only role and ensure their code only calls `INSERT` into `signals` (no updates).
- Any other read-only workloads (reporting dashboards, observers) should use `dataforge_reader` credentials, which have `SELECT` only on the local schema.
- Record these boundaries and their secrets in your vault/auditing system so future operators understand the append-only guardrails.

## 4. Post-cutover validation

1. Run the verification script (`zfss/scripts/verify_migration.py`, referenced in `zfss/docs/migration_verification.md`) to ensure local row counts/timestamps match the Render snapshot.
2. Confirm Render services no longer report health-checks for the old database—`psql forge-db.<host>.render.com` should fail with `password authentication failed`.
3. Archive the Render snapshot and verification artifacts (see `zfss/docs/render_export_plan.md` and `zfss/docs/local_reingest_plan.md`) for auditability.

Once these steps are complete, the cutover is done: the local Postgres instance holds the canonical ledger, cloud services can only append Signals, and Render is demoted to a historical backup.
