# Governance

**Truth class:** canonical doctrine

This documentation system governs ZFSS repo-local implementation truth. It does
not define shared Forge ecosystem doctrine or DataForge cloud authority beyond
the explicit integration and handoff surfaces ZFSS owns.

## Authority Boundary

- `doc/system/` is the canonical authored source tree for ZFSS system truth.
- `doc/ZFSSYSTEM.md` is generated output and must not be edited by hand.
- Supporting docs, plans, and archives outside `doc/system/` are subordinate to
  the compiled system reference when they describe current behavior.
- Runtime behavior and verification evidence override stale prose; when they
  disagree, update the source chapter and rebuild the compiled artifact.

## Change Control

Changes that alter signal capture, issue lifecycle, database schema, response
control, integrations, or local authority boundaries must update the relevant
`doc/system/` chapter in the same change as the implementation.

Documentation-only changes must still rebuild `doc/ZFSSYSTEM.md` with:

```bash
bash doc/system/BUILD.sh
```

## Which CI runs for which change

A change that touches only documentation runs the Documentation CI and no code CI.
A change that touches any other file runs the code CI.
A change that touches both runs both.
If the scope is unknown, the code CI runs.

The code CI is `.github/workflows/ci.yml`.
It has a workflow-level `paths` filter on `push` and `pull_request`.
The filter includes `**` and then removes `docs/**`, `doc/**` and `**/*.md`.
The last matching pattern wins, so the re-include comes last.
A change to `.github/workflows/**` is code and runs the code CI.

The filter re-includes one path, because a code check reads it:

- `doc/system/*.sh`. The step "Validate scripts" runs `bash -n` on `BUILD.sh` and `validate_snapshots.sh`. A shell script is code, even when it lives under `doc/`.

The code CI also reads documentation in two steps.
The step "Rebuild canonical system document" builds and diffs `doc/ZFSSYSTEM.md`.
The step "Reject superseded authority claims" scans `README.md`, `CLAUDE.md`, `doc/ZFSSYSTEM.md` and `doc/system` for banned authority wording.
These steps are documentation gates.
The Documentation CI runs the same checks on every documentation change.
The code CI keeps its copies and runs them on every code change.
No gate is weaker.

The Documentation CI is `.github/workflows/documentation.yml`.
It runs on changes to `docs/**`, `doc/**`, `**/*.md` and its own file.
It runs `bash doc/system/BUILD.sh`.
It fails if `git diff --exit-code -- doc` shows a difference.
It fails if a superseded system document exists.
It fails if a banned authority phrase appears in the scanned files.
The assembled `doc/ZFSSYSTEM.md` must be built from its parts and committed.

No scheduled run exists.
No secret scan or other security scan exists in this repository.
If a scan is added, it must run on every change, documentation included.

Do not add a required check on a path-filtered workflow.
A workflow that does not start leaves the check pending, and the pending check blocks the merge.
