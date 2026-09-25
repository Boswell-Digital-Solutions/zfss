# zfss - Compiled System Reference

**Designation:** ZFS
**Document role:** Canonical compiled technical reference for the Zen Feedback and Service System
**Source:** `doc/system/`
**Build command:** `bash doc/system/BUILD.sh`
**Document version:** 2.0 (2026-06-22) - canonical compliance migration
**Protocol:** BDS Documentation Protocol v2.0; BDS Repo Documentation System Canonical Compliance Standard

> **Generated artifact warning:** `doc/ZFSSYSTEM.md` is assembled output. Edit
> the source modules under `doc/system/` and rebuild. Hand edits to the
> compiled artifact are overwritten by the next build.

Assembly contract:

- Command: `bash doc/system/BUILD.sh`
- Validation: `bash doc/system/validate_snapshots.sh` runs during assembly
- Primary output: `doc/ZFSSYSTEM.md`

This `doc/system/` tree is the canonical source of truth for zfss. It
uses explicit **truth classes**: canonical facts define the repo role, authority
boundaries, runtime behavior, service contracts, and verification doctrine;
snapshot facts are dated, audit-derived counts and current implementation
inventory that may drift between audits.

| Part | File | Contents |
| --- | --- | --- |
| §1 | `01-overview-philosophy.md` | 1. Overview & Philosophy |
| §2 | `02-architecture.md` | 2. Architecture |
| §3 | `04-project-structure.md` | 4. Project Structure |
| §4 | `06-frontend.md` | 6. Frontend |
| §5 | `07-tauri-commands.md` | 7. Tauri Command Interface |
| §6 | `10-product-surface.md` | Product Surface |
| §7 | `08-backend-internals.md` | 8. Backend Internals |
| §8 | `09-database-schema.md` | 9. Database Schema |
| §9 | `20-runtime.md` | Runtime |
| §10 | `03-tech-stack.md` | 3. Tech Stack |
| §11 | `10-ecosystem-integration.md` | 10. Ecosystem Integration |
| §12 | `40-integrations.md` | Integrations |
| §13 | `05-config-env.md` | 5. Configuration & Environment |
| §14 | `11-handover.md` | 11. Handover |
| §15 | `50-operations.md` | Operations |
| §16 | `30-data.md` | Data |
| §17 | `90-appendices.md` | Appendices |
| §18 | `91-bootstrap-overview.md` | Overview |
| §19 | `92-bootstrap-architecture.md` | Architecture |

## Quick Assembly

```bash
bash doc/system/BUILD.sh
```
