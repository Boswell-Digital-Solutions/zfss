## 8. Backend Internals

### Models (models/)

All models use typed ID prefixes for safety:

| Type | Prefix | Format |
|------|--------|--------|
| SignalId | `sig_` | `sig_{uuid}` |
| IssueId | `iss_` | `iss_{uuid}` |
| DecisionId | `dec_` | `dec_{uuid}` |
| ArtifactId | `art_` | `art_{uuid}` |
| ResponseId | `rsp_` | `rsp_{uuid}` |
| AttachmentId | `att_` | `att_{uuid}` |

All models derive `Serialize`, `Deserialize`, `Clone`, `Debug` for Serde/IPC compatibility.

### Lifecycle State Machines

#### Signal Lifecycle

```
new → linked → responded → closed
  ↘ needs_info ↗
```

#### Issue Lifecycle

```
pending_decision → decided → in_progress → ready_for_verification → closed
```

**Constraint:** Issue cannot transition to `closed` unless `has_verified_artifact()` returns `true`.

#### Response Lifecycle

```
draft → pending → approved → sent
              ↘ blocked
```

### Role-Based Authority

| Action | Steward | Operator | Engineer | AI |
|--------|---------|----------|----------|-----|
| Log Signal | Yes | Yes | Yes | Yes |
| Link Signal to Issue | Yes | Yes | No | No |
| Make Decision | Yes | No | No | No |
| Create Artifact | Yes | No | Yes | No |
| Verify Artifact | Yes | No | No | No |
| Approve Response | Yes | No | No | No |
| Close Issue | Yes | No | No | No |

### Repository Layer (repository/)

Enforces append-only semantics at the code level:
- `append_signal()` — INSERT only
- `append_issue()` — INSERT only
- `append_decision()` — INSERT only (can supersede previous)
- `append_artifact()` — INSERT only
- `append_response()` — INSERT only
- Lifecycle transitions use INSERT-only history/event tables and current-state projections

**No `update_*()` or `delete_*()` functions exist.** This is a design invariant, not an oversight.

### Service Layer (service/)

Business logic enforcement:
- Input validation (text length, enum values)
- Role authority checks before mutations
- Lifecycle transition validation
- `close_requires_artifact` rule enforcement

### Constraints (constraints.rs)

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_RAW_TEXT_BYTES` | 10,000 | Signal text size limit |
| `HOTKEY_DEBOUNCE_MS` | 100 | Hotkey re-trigger cooldown |
