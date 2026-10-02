# Known issues

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
