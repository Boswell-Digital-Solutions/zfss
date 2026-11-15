# Known issues

## 2026-11-15: six placeholder chapters are in the compiled spine, and `40-governance.md` is not in the index

- Source: BDS-ORG-ARCH-AUDIT-2026-11.
- What is wrong: six chapters hold only the registry bootstrap text "Replace this placeholder with real authored content", and the compiled reference includes them. `40-governance.md` is a real chapter (3110 bytes), and `_index.md` does not list it. A reader of the index cannot find the governance chapter. The index also does not follow the file order: §3 is `04-project-structure.md` and §10 is `03-tech-stack.md`.
- Evidence (Observed (doc tree)): `zfss@6f376c98a3:doc/system/10-product-surface.md`, `20-runtime.md`, `30-data.md`, `40-integrations.md`, `91-bootstrap-overview.md` and `92-bootstrap-architecture.md` are 354 to 377 bytes each. Each holds the placeholder text. `doc/system/90-appendices.md` (349 bytes) is a carry-forward stub. `doc/system/_index.md:L28-L46` lists 19 parts for 20 chapter files, and `40-governance` is not among them. Chapters `91` and `92` repeat the purpose of the real chapters `01` and `02`.
- Root cause: not determined. This matches the cause of the 2026-10-02 entry on placeholders and duplicate prefixes, which names `50-operations.md`, `10-ecosystem-integration.md`, `10-product-surface.md`, `40-governance.md` and `40-integrations.md`. That entry does not name the other four placeholders or the missing index row.
- Fix: none proposed. Recommendation (audit): remove or fill the six placeholder chapters, add `40-governance` to the index, and put the parts in file order. Then run `bash doc/system/BUILD.sh`. The duplicate `10-` and `40-` prefixes stay tracked in the 2026-10-02 entry.
- Status: open.

## 2026-11-15: the documentation says the app has 23 commands and lists 24

- Source: BDS-ORG-ARCH-AUDIT-2026-11.
- What is wrong: three chapters say 23 commands. The source registers 24.
- Evidence (Observed (source)): `zfss@6f376c98a3:src-tauri/src/main.rs:L171-L200` lists 24 names in `tauri::generate_handler![...]` (4 signal, 4 issue, 4 decision, 5 artifact, 7 response). `doc/system/07-tauri-commands.md:L5` says "23 IPC commands". `doc/system/11-handover.md:L21` says "23 IPC command registrations". `doc/system/02-architecture.md:L18` says "23 handlers". The tables in `doc/system/07-tauri-commands.md:L13-L56` list 24 rows.
- Root cause: not determined. A command was added and the count was not updated.
- Fix: none proposed. Recommendation (audit): change the three counts to 24 and run `bash doc/system/BUILD.sh`.
- Status: open.

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
