# Compiled Plan Set — BDS-TARCIE-ZFSS-GITHUB-HANDOFF-v0.1

## Decision posture

Proposed and documentation-only. Board Review 1 is open. The recommended decision is AUTHORIZE BOUNDED DESIGN for source lock, contracts, fixtures, canonical hashing, and threat modeling only.

## Canonical architecture

Tarcie captures operator observations and deliberate artifacts. ZFSS accepts them as append-only Signals, groups related Signals into an internal feedback Issue, records the Steward disposition, and authors an evidence-bound GitHub issue proposal. Forge_Command shows the exact proposal and privacy posture, captures the single human decision, performs a separately authorized bounded GitHub write, and records a creation or reconciliation receipt. Forge:SMITH consumes a created engineering issue only after publication and requires separate authority for any planning, mutation, deployment, or release.

## First repository allowlist

- Boswell-Digital-Solutions/Forge_Command — private
- Boswell-Digital-Solutions/forge-smithy — private
- Boswell-Digital-Solutions/zfss — public

The tested application mapping selects the repository. Tarcie text and tags never select it.

## Contract spine

1. TarcieZFSSSignalSubmission.v1
2. ZFSSSignalAcceptanceReceipt.v1
3. EngineeringHandoff.v1
4. GitHubIssueProposal.v1
5. GitHubIssueCreateDecision.v1
6. GitHubIssueCreationReceipt.v1
7. GitHubIssueReconciliationReceipt.v1

Every hop binds stable identifiers, canonical payload hashes, evidence references, timestamps, schema versions, and reason codes.

## Non-negotiable controls

- One proposal, one exact repository, one exact payload, one human decision.
- No GitHub credentials in Tarcie.
- No repository routing from untrusted note text.
- Screenshots remain local by default.
- Public publication requires explicit visibility and redaction acknowledgment.
- No creation receipt without verified issue ID, number, and URL.
- No automatic retry after an ambiguous create outcome.
- Forge_Command does not author evidence or proposal content.
- Forge:SMITH does not become feedback truth owner.
- ZFSS raw Signals remain immutable and its feedback lineage remains append-only.

## First proving slice

Use only synthetic data, a ZFSS fixture database, Forge_Command fixture review, and a simulated GitHub endpoint. Prove:

- allowlisted routing
- idempotent Signal acceptance
- many-Signals-to-one-Issue lineage
- deterministic proposal construction
- exact human decision
- success, rejection, timeout, response loss, ambiguity, and reconciliation
- creation and reconciliation receipts
- read-only Forge:SMITH intake

Do not use live GitHub, credentials, production data, personal content, cloud transport, AI classification, repair, deployment, or release.

## Roadmap summary

- Phase 0: source lock and ownership decision
- Phase 1: contracts and fixtures
- Phase 2: Tarcie-to-ZFSS fixture intake
- Phase 3: ZFSS triage and handoff proposal
- Phase 4: Forge_Command fixture review
- Phase 5: simulated GitHub execution
- Phase 6: Forge:SMITH read-only intake
- Phase 7: separately reviewed security qualification and limited live pilot

## Board Review 1 questions

- Confirm ZFSS as feedback-domain authority.
- Confirm Forge_Command as the single decision and execution surface.
- Confirm Forge:SMITH as the downstream engineering consumer.
- Confirm the three-repository allowlist.
- Decide contract-family ownership.
- Decide whether the human acts as ZFSS Steward through Forge_Command.
- Decide screenshot retention and encryption posture.
- Confirm outcome_uncertain plus no retry.
- Confirm ZFSS feedback truth versus DataForge cross-system receipt projections.

## Source baseline

Pinned on 2026-08-14:

- tarcie main at 3661ae223d96903f58c29acec7918221ef1abd53
- zfss master at d004a16c37533cf70ea2efc0b43771f24bbeae40
- Forge_Command main at fbcf51e1575e4d57152cfa7079f888d191bc7939
- forge-smithy master at 4b8d431dcbaca3f3fbc99d84d078425b7ce6a674

A gate session must re-pin before GATE-00 acceptance.

## Explicit non-authorization

This plan does not authorize implementation, migrations, live intake, screenshots, GitHub credentials, GitHub issue creation or mutation, production writes, agent execution, repair, deployment, promotion, release, beta enrollment, or registry activation.
