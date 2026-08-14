# Board Review 1 Packet — BDS-TARCIE-ZFSS-GITHUB-HANDOFF-v0.1

## Requested decision

Authorize, rework, or reject documentation-only Phase 0 source lock and Phase 1 contract/fixture design.

No live GitHub operation is requested.

## Decision options

### AUTHORIZE BOUNDED DESIGN

Authorize only:

- source and protocol pinning
- repository ownership reconciliation
- schema candidates
- canonical hash vectors
- synthetic fixtures
- simulated GitHub endpoint
- threat model and RED Board review

### REWORK

Return the packet for changes without granting implementation authority.

### REJECT

Close the proposal. Tarcie remains capture-only and no GitHub handoff lane is pursued.

## Board questions

1. Is ZFSS the canonical owner of feedback Signals, internal Issues, triage decisions, and the GitHub issue proposal?
2. Is Forge_Command the correct surface for the single human decision and bounded GitHub executor?
3. Is Forge:SMITH correctly limited to downstream engineering intake and governed remediation?
4. Are the three target repositories correct for the first allowlist?
5. Must all new contract families enter forge_contract_core before fixture work, or may they remain repo-local until the proving slice?
6. Does the human act as ZFSS Steward through Forge_Command, preserving one decision rather than two approvals?
7. Is screenshot export prohibited through the first private-repository pilot?
8. Is outcome_uncertain plus no automatic retry the required response to an ambiguous GitHub create?
9. What retention period and spool cap apply to Tarcie/ZFSS screenshot artifacts?
10. Is ZFSS local PostgreSQL canonical for feedback records while DataForge holds only admitted cross-system receipt projections?

## Primary threats

| Threat | Required control |
| --- | --- |
| Wrong repository | Registered application-to-repository mapping; exact repository ID |
| Duplicate GitHub issue | One decision/one execution; no retry after ambiguity; reconciliation receipt |
| Public evidence leak | Visibility-aware review; screenshot export blocked by default |
| Secret in issue body | Deterministic secret scan and human preview |
| Tarcie scope creep | Capture-only contract; no credentials or routing decision |
| Forge_Command scope creep | Display, decision capture, execution, and receipt only |
| Forge:SMITH scope creep | Downstream work intake only |
| ZFSS truth rewrite | Append-only Signals, links, decisions, and receipts |
| Label/assignee drift | Preflight allowlist and post-response verification |
| Stale build identity | Build and commit binding shown at decision time |

## Zero-tolerance acceptance

- No GitHub write without exact human approval.
- No repository selected from free text.
- No automatic retry after ambiguous create.
- No public artifact export without explicit approval.
- No receipt without verified issue identity.
- No mutation or release authority granted by this packet.

## Required Phase 0 evidence

- Current heads for all four repositories.
- ZFSS schema and lifecycle inventory.
- Tarcie capture and queue inventory.
- Forge_Command proposal/approval surface inventory.
- Forge:SMITH authority and remediation-posture inventory.
- GitHub App permission and credential-owner decision.
- Contract-family ownership decision.
- Privacy/retention decision log.

## Recommendation

AUTHORIZE BOUNDED DESIGN, limited to Phase 0 and Phase 1. Keep all live intake, database migration, GitHub credentials, GitHub writes, screenshots, and engineering execution unauthorized.
