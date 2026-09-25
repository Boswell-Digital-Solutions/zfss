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
