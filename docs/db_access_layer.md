# ZFSS-Safe Database Access Layer

This module is the curated front door for the ZFSS operational PostgreSQL store: only append-only operations are exposed and all reads use derived views over append tables. The legacy physical name `dataforge` conveys no DataForge authority. See `zfss/src-tauri/src/repository/mod.rs` for the implementation that the IPC layer now depends on.

## Append-only writer APIs

- `append_signal` – inserts a new `signals` row and immediately records the `'new'` status in `signal_status_history`. It never touches existing rows, so the canonical ledger stays immutable.
- `link_signal_to_issue` – records the transition by inserting into `signal_status_history` (new `'linked'` status), writing a new row in `signal_links`, and logging the event in `audit_log`. No `UPDATE` or `DELETE` statements are issued.

Every writer takes a `PgPool` reference, validates the target IDs exist, and runs inside an explicit transaction so partial writes never surface.

## Read-only readers

- `list_signals` – exposes the most recent status and linked issue by joining against the append-only history tables (`signal_status_history`, `signal_links`). It only runs `SELECT`s.
- `get_signal` – echoes the same derived status/link information for a single signal, so callers never have to reason about the underlying history tables.

By routing both capture and issue-linking IPC commands through these functions (`zfss/src-tauri/src/ipc/signal_cmds.rs`), we keep the read/write separation explicit: the Tauri UI can read freely, and writes happen only through the append APIs documented above.

## Guardrails for generated code

1. **No implicit mutation:** New helper functions must never execute `UPDATE`/`DELETE` on canonical tables; if the business need looks like a mutation, model it as an append to a history/log table and derive the current state via a query.
2. **Typed IDs:** Use the typed ID helpers (e.g., `SignalId`) so every insertion satisfies the prefix/length checks enforced by the schema constraints.
3. **Status transitions:** Any transition must go through `signal_status_history` (or `issue_status_history` when implemented) so the trigger layer can prove the change was append-only. Repository code already guarantees the `old_status` is pulled from the latest history row.
4. **Audit/trace logging:** Critical operations insert into `audit_log` so humans and AI agents can prove intent—don’t skip the audit row even if the business flow seems obvious.

Treat this module as the canonical contract; if you need new operations (e.g., decisions, artifacts), follow the same pattern: expose an append-only write helper, its companion read helper, and document the guardrails here before wiring the IPC or service layers.
