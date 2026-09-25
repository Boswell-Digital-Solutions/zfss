## 9. Database Schema

### Database

PostgreSQL 14+ (local ZFSS operational store). Connected via sqlx async driver with connection pooling. It is not canonical ecosystem memory or admitted BDS evidence.

### Tables (13)

| Table | Purpose | Append-Only |
|-------|---------|-------------|
| `users` | Role assignments (Steward, Operator, Engineer, AI) | No (mutable) |
| `signals` | Raw immutable user expressions | Yes |
| `signal_status_history` | Signal state transition log | Yes |
| `signal_links` | Signal-to-Issue link events | Yes |
| `attachments` | Files attached to Signals | Yes |
| `issues` | System's grouped understanding | Yes |
| `issue_status_history` | Issue state transition log | Yes |
| `decisions` | Declared intent (can supersede) | Yes |
| `artifacts` | Proof of learning | Yes |
| `artifact_verifications` | Artifact verification events | Yes |
| `responses` | Controlled outbound communications | Yes |
| `response_approval_history` | Response approval state log | Yes |
| `audit_log` | System-wide audit trail | Yes |

### Views (3)

Database views provide common query patterns (signal counts by status, issues pending decision, etc.).

### Append-Only Enforcement

Migration `002_append_only_enforcement.sql` creates a `zfss_forbid_mutation()` trigger function. Migration `004_append_only_lifecycle_projections.sql` extends its UPDATE/DELETE guards to every history and event table. Only `users` is mutable.

```sql
-- Applied to: signals, issues, decisions, artifacts, responses
CREATE TRIGGER forbid_mutation
  BEFORE UPDATE OR DELETE ON {table}
  FOR EACH ROW
  EXECUTE FUNCTION zfss_forbid_mutation();
```

There is no mutation exception. Lifecycle changes are INSERTs into history/event tables; reads project the latest state while canonical rows retain their creation-time values.

### Migrations

| File | Description |
|------|-------------|
| `001_initial_schema.sql` | Base tables, views, and indexes |
| `002_append_only_enforcement.sql` | Mutation-blocking triggers |
| `003_signal_link_events.sql` | Signal linking history tracking |
| `004_append_only_lifecycle_projections.sql` | Lifecycle event tables and current-state views |

### ID Generation

All IDs are prefixed UUIDs generated in Rust:

```rust
SignalId(format!("sig_{}", Uuid::new_v4()))
```

### Key Constraints

- Foreign keys enforce referential integrity (Signal → Issue, Decision → Issue, etc.)
- `issues.closed_at` can only be set when a verified Artifact exists
- `decisions` can supersede previous decisions for the same Issue (append, don't update)
- All timestamps are `timestamptz` (timezone-aware)
