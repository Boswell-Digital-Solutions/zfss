## 10. Ecosystem Integration

### Forge Ecosystem Position

ZFSS occupies the **feedback boundary** of the Forge ecosystem. It captures external user feedback (Signals) and metabolizes them through a governed lifecycle into verified outcomes (Artifacts, Responses).

### Cross-System Authority Model

ZFSS follows the Forge ecosystem's divided-authority doctrine:
- **Local PostgreSQL is the source of truth** (migrated from Render cloud)
- ZFSS PostgreSQL owns ZFSS feedback-domain operational records only
- Forge Memory owns candidate-memory lifecycle and receipts, with promotion disabled by default
- Forge_Command records operator review and bounded authorization
- SMITH applies only the exact digest-bound action approved by the operator
- Cloud DataForge owns admitted durable evidence, canonical shared memory, decisions, and receipts
- DataForge Local provides bounded offline continuity and is not competing cloud authority
- The database is the contract — append-only semantics enforced at the trigger level

### Shared Patterns

| Pattern | ZFSS Implementation |
|---------|---------------------|
| Append-only writes | PostgreSQL triggers + repository layer |
| Lifecycle state machines | Signal, Issue, Response state enums |
| Role-based authority | 4 roles (Steward, Operator, Engineer, AI) |
| Typed IDs | Prefixed UUIDs (`sig_*`, `iss_*`, etc.) |
| Tauri v2 desktop | Same framework as Forge:SMITH and ForgeCommand |

### Render-to-Local Migration

ZFSS was originally deployed on Render (cloud PostgreSQL). Its operational writes were cut over to local PostgreSQL with:
- Export tooling (`scripts/export_render_snapshot.sh`)
- Import tooling (`scripts/import_snapshot_to_local.sh`)
- Verification (`scripts/verify_migration.py` — row count comparison)
- Credential rotation and cloud service disconnection

See `docs/local_postgres_authority.md` for legacy binding operations and `contracts/authority/zfss-authority.v1.yaml` for the current boundary.

### Future Integration Points

| Service | Integration | Status |
|---------|-------------|--------|
| ForgeCommand | Orchestration of ZFSS health checks | Planned |
| Forge Memory | Candidate memory derived from feedback | Not admitted; disabled |
| Cloud DataForge | Admitted evidence, canonical shared memory, decisions, receipts | Route not admitted |
| DataForge Local | Bounded offline continuity | Route not admitted |
| NeuroForge | AI-assisted Signal triage and classification | Planned |
| BugCheck | Signal-to-Issue correlation with bug findings | Planned |
