# BDS-TARCIE-ZFSS-GITHUB-HANDOFF-v0.1 — Roadmap

Progression is gate-based. Dates do not grant authority.

## Phase 0 — Source lock and ownership decision

- Pin Tarcie, ZFSS, Forge_Command, forge-smithy, and applicable contract/protocol revisions.
- Confirm ZFSS as the feedback-domain authority.
- Confirm Forge_Command as the decision and GitHub-execution surface.
- Confirm Forge:SMITH as the downstream engineering consumer.
- Confirm the three-repository allowlist and visibility.
- Decide whether contract families begin repo-local or require forge_contract_core admission.

Exit: GATE-00 receipt and Board Review 1 decision.

## Phase 1 — Contracts and fixtures

- Define the seven candidate contracts in the canonical plan.
- Define canonicalization, hashing, reason codes, and trace markers.
- Publish positive, negative, replay, conflict, partial, timeout, ambiguous-outcome, and public-leak fixtures.
- Prove deterministic proposal construction without live services.

Exit: schema proof, negative fixtures, canonical hash vectors, and contract-ownership receipt.

## Phase 2 — Tarcie-to-ZFSS fixture intake

- Add an explicit Tarcie Signal source candidate.
- Bind application, build, repository, commit, and session identity.
- Add artifact hash, size, media type, redaction state, and retention metadata candidates.
- Prove idempotent acceptance and append-only lineage in an isolated ZFSS database.

Exit: GATE-01 intake receipt.

## Phase 3 — ZFSS triage and handoff proposal

- Group multiple Signals into one internal ZFSS Issue.
- Record a Steward decision without rewriting raw Signals.
- Construct one deterministic GitHubIssueProposal per approved handoff candidate.
- Require exact target repository resolution from the allowlist.

Exit: GATE-02 proposal and lineage receipt.

## Phase 4 — Forge_Command fixture review

- Add a read-only proposal review fixture surface.
- Display target, visibility, evidence basis, exact title/body/labels, privacy posture, and receipt preview.
- Capture approve, reject, hold, and request-more-evidence decisions.
- Block approval on stale, missing, conflicting, or private-to-public evidence.

Exit: GATE-03 operator comprehension and decision receipt.

## Phase 5 — Simulated GitHub execution

- Use a simulated endpoint only.
- Prove success, validation rejection, permission failure, timeout, response loss, ambiguous outcome, and reconciliation.
- Prove no automatic retry after an ambiguous create.
- Verify returned issue identity and response fields before issuing a creation receipt.

Exit: GATE-04 fault-injection and receipt report.

## Phase 6 — Forge:SMITH read-only engineering intake

- Consume the created-issue fixture and linked ZFSS lineage.
- Present the issue as candidate work.
- Require a separate RunIntent or applicable governed authority before planning or mutation.
- Return proof artifact references to ZFSS without rewriting feedback records.

Exit: GATE-05 boundary and lineage receipt.

## Phase 7 — Security qualification and limited live pilot

Requires a separate reviewed revision and Board Review 2.

Candidate scope:

- one private target repository first
- one synthetic issue
- one operator
- one GitHub App installation with repository-limited Issues permission
- no screenshot export
- explicit cleanup/closure decision
- independent receipt review

Public ZFSS repository publication is not inherited from a private-repository pilot.

Exit: GATE-06 and explicit AUTHORIZE, REWORK, or REJECT decision.

## Deferred

- AI classification or summarization
- automatic deduplication verdicts
- automatic issue closure
- issue comments or updates
- public screenshot publication
- targets outside the three-repository allowlist
- multi-user or team approval
- remote/cloud Tarcie transport
- automatic repair or deployment
