# Known issues

## 2026-11-10: the `attachments` table cannot hold evidence custody, and no code writes to it

- What is wrong: `migrations/001_initial_schema.sql` defines `attachments` with `kind`, `filename`, `ref_url` and `created_at` only. It has no content hash, no byte length, no storage location for the bytes and no retention field. No application code inserts a row. Only the snapshot export and import scripts and `scripts/verify_migration.py` name the table. `migrations/002_append_only_enforcement.sql` makes the rows append-only (`trg_attachments_prevent_mutation`).
- Root cause: the table was planned for links and file names, not for evidence that must prove its own bytes.
- Effect: a Tarcie screenshot recorded here cannot be proven unchanged. The tr-manage evidence flow cannot use this table for image custody. Found in a read-only fit review for tr-manage on 2026-11-10.
- Fix: not made. A custody design needs a content hash, a byte length and a storage reference at minimum. It needs an operator decision.
- Status: open.

## 2026-10-02: `CLAUDE.md` says there is no CI workflow

- What is wrong: the Verification section of `CLAUDE.md` says "There is no CI workflow and no test script in this repo." `.github/workflows/ci.yml` exists and runs tests, typecheck, build, Rust tests and a PostgreSQL contract.
- Root cause: the section was written before the workflow was added.
- Fix: not made. Update the section to name `ci.yml` and `documentation.yml`.
- Status: open.

## 2026-10-02: `doc/system/` carries bootstrap placeholders and two part sets

- What is wrong: `50-operations.md` is a "registry-generated bootstrap scaffold" with no real content. Parts `10-ecosystem-integration.md` and `10-product-surface.md`, and `40-governance.md` and `40-integrations.md`, share a number prefix.
- Root cause: unknown. Likely a bootstrap run on top of authored parts.
- Fix: not made.
- Status: open.


## 2026-10-18 — Documentation check-mode limit

**Status: OPEN verification limit.** `doc/system/BUILD.sh` does not parse
`--check`; that invocation rebuilds the output and returns success. This change
uses before/after generated-output hashes to check deterministic rebuilds. It
does not claim a strict stale-document guard. The existing builder was not
changed as part of Debian package inspection.
