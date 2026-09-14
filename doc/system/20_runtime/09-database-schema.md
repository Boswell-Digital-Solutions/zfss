## 9. Database Schema

### Database

PostgreSQL 14+ (local ZFSS operational store). Connected via sqlx async driver with connection pooling. It is not canonical ecosystem memory or admitted BDS evidence.

### Tables (11)

| Table | Purpose | Append-Only |
|-------|---------|-------------|
| `users` | Role assignments (Steward, Operator, Engineer, AI) | No (mutable) |
| `signals` | Raw immutable user expressions | Yes |
| `signal_status_history` | Signal state transition log | Yes |
| `attachments` | Files attached to Signals | Yes |
| `issues` | System's grouped understanding | Yes |
| `issue_status_history` | Issue state transition log | Yes |
| `decisions` | Declared intent (can supersede) | Yes |
| `artifacts` | Proof of learning | Yes |
| `responses` | Controlled outbound communications | Yes |
| `response_approval_history` | Response approval state log | Yes |
| `audit_log` | System-wide audit trail | Yes |

### Views (7)

Database views provide common query patterns (signal counts by status, issues pending decision, etc.).

### Append-Only Enforcement

Migration `002_append_only_enforcement.sql` creates a `zfss_forbid_mutation()` trigger function that blocks UPDATE and DELETE on canonical tables:

```sql
-- Applied to: signals, issues, decisions, artifacts, responses
CREATE TRIGGER forbid_mutation
  BEFORE UPDATE OR DELETE ON {table}
  FOR EACH ROW
  EXECUTE FUNCTION zfss_forbid_mutation();
```

**Exception:** The single `status` field on canonical tables allows UPDATE for state transitions. Status changes are also logged to the corresponding `*_status_history` table via INSERT.

### Migrations

| File | Description |
|------|-------------|
| `001_initial_schema.sql` | 11 tables + 7 views + indexes |
| `002_append_only_enforcement.sql` | Mutation-blocking triggers |
| `003_signal_link_events.sql` | Signal linking history tracking |

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
