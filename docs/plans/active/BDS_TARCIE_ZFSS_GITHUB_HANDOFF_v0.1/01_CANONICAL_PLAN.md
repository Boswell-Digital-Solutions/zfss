# BDS-TARCIE-ZFSS-GITHUB-HANDOFF-v0.1 — Canonical Plan

## 1. Executive decision

Proceed with design and fixture proof for a governed Tarcie-to-ZFSS-to-GitHub handoff.

The governing flow is:

Tarcie capture → ZFSS Signal → ZFSS internal Issue → GitHub issue proposal → one human decision surfaced through Forge_Command → bounded GitHub executor → creation receipt → optional Forge:SMITH engineering intake.

This is not a direct Tarcie-to-GitHub integration. Tarcie supplies observations and artifacts but never chooses severity, groups findings, selects a repository, holds credentials, or performs a GitHub mutation.

## 2. Problem

A beta-testing session may concern Forge_Command, Forge:SMITH, or ZFSS. The operator needs to capture a note or screenshot quickly, preserve the original observation, group related observations into a coherent feedback Issue, and deliberately publish an engineering issue to the correct repository without duplication or accidental disclosure.

Tarcie already supplies fast local note capture and a durable queue. ZFSS already defines append-only Signals, internal Issues, Decisions, Attachments, Artifacts, and role-based feedback governance. Forge_Command already defines proposal review and operator decision patterns. Forge:SMITH already defines governed engineering intent and evidence production.

The missing capability is the contract-bound handoff among them.

## 3. Repository routing allowlist

The first slice admits only these targets:

| application_id | Repository | Default branch at source lock | Visibility | Initial posture |
| --- | --- | --- | --- | --- |
| forge-command | Boswell-Digital-Solutions/Forge_Command | main | private | allowed only through fixtures until authorized |
| forge-smithy | Boswell-Digital-Solutions/forge-smithy | master | private | allowed only through fixtures until authorized |
| zfss | Boswell-Digital-Solutions/zfss | master | public | blocked from artifact export until public-content review passes |

Repository identity must come from the BetaSession binding or an operator-selected registered application mapping. Tarcie tags and free text cannot select the target.

Unknown, renamed, transferred, archived, disabled-issues, or non-allowlisted repositories fail closed.

## 4. Authority model

| Component | Authorized responsibility | Prohibited responsibility |
| --- | --- | --- |
| Tarcie | Capture operator-authored notes, markers, explicit screenshots, session/build identity, and local delivery state | Triage, diagnosis, severity, repository selection, GitHub credentials, GitHub writes |
| ZFSS | Preserve Signals, group Signals into internal Issues, record Steward decisions, author evidence-bound GitHub issue proposals, maintain append-only lineage | Code repair, autonomous approval, silently rewriting raw feedback, direct release claims |
| Forge_Command | Display the exact proposal, capture the authorized human decision, invoke a bounded GitHub executor, verify the response, display/store receipts | Authoring source evidence, inventing proposal content, auto-approval, expanding the target or payload |
| Forge:SMITH | Consume a created engineering issue, create governed intent, plan remediation, execute only separately authorized work, produce proof artifacts | Feedback intake authority, rewriting ZFSS truth, creating a GitHub issue in the first slice |
| DataForge | Store cross-system evidence and receipt projections when separately admitted | Feedback-domain decisions or mutation authority |
| Human Steward/operator | Group, classify, select disposition, approve or reject the exact external publication | None within the owned decision scope |

The human decision is singular. Forge_Command is the decision surface; ZFSS remains the feedback-domain record. A second approval in Forge:SMITH is not required to create the issue. Any later code mutation uses Forge:SMITH's separate governed authority path.

## 5. Information classes

The system must keep these classes distinct:

1. Tarcie observation — immutable operator-authored text or deliberate artifact capture.
2. ZFSS Signal — accepted feedback-domain record derived from one observation.
3. ZFSS Issue — a Steward-governed understanding that may group multiple Signals.
4. GitHub issue proposal — a non-executed, evidence-bound external publication candidate.
5. Human decision — approve, reject, hold, or request more evidence.
6. GitHub mutation result — created, rejected, unavailable, or outcome-uncertain.
7. Creation receipt — immutable linkage to repository ID, issue ID, number, URL, and approved payload hash.
8. Engineering work — separately governed Forge:SMITH planning, mutation, verification, and release evidence.

A ZFSS Issue is not a GitHub issue. A proposal is not an issue. An HTTP response without a verified GitHub issue identity is not a creation receipt.

## 6. Contract candidates

### TarcieZFSSSignalSubmission.v1

Required fields:

- submission_id
- tarcie_observation_id
- beta_session_id
- application_id
- build_id
- repository_id
- commit_sha when the build is repository-bound
- captured_at
- raw_text
- artifact_manifest
- privacy_profile
- canonical_payload_hash
- idempotency_key
- schema_version

### ZFSSSignalAcceptanceReceipt.v1

Required fields:

- receipt_id
- submission_id
- zfss_signal_id
- accepted_artifact_ids and hashes
- rejected items with reason codes
- durable_store_reference
- canonical_payload_hash
- received_at
- receipt_hash
- schema_version

### EngineeringHandoff.v1

Required fields:

- handoff_id
- zfss_issue_id
- ordered_signal_ids
- decision_id
- application_id
- target_repository_id
- tested_build_id
- tested_commit_sha
- evidence_refs
- publication_class
- privacy_disposition
- created_at
- schema_version

### GitHubIssueProposal.v1

Required fields:

- proposal_id
- handoff_id
- target_repository_id and full name
- title
- body_markdown
- requested labels
- requested assignees
- evidence_refs
- artifact_export_refs
- target_visibility
- approved_payload_hash candidate
- created_at
- schema_version

Proposal content is deterministic from accepted ZFSS records plus explicit human edits. AI output, if later admitted, is advisory and must be visibly distinguished.

### GitHubIssueCreateDecision.v1

Required fields:

- decision_id
- proposal_id
- decision
- operator_user_id
- operator_role
- decided_at
- exact_payload_hash
- exact_repository_id
- privacy_acknowledgment
- schema_version

Allowed decisions: approve, reject, hold, request_more_evidence.

### GitHubIssueCreationReceipt.v1

Required fields:

- receipt_id
- decision_id
- proposal_id
- target_repository_id
- github_issue_id
- github_issue_node_id when available
- issue_number
- issue_url
- created_at
- executing_app_identity
- returned_title_hash
- returned_body_hash
- returned_labels
- approved_payload_hash
- source_evidence_refs
- receipt_hash
- schema_version

### GitHubIssueReconciliationReceipt.v1

Required when a request outcome is ambiguous. It records the request identity, proposal, repository, payload hash, observed failure, reconciliation attempts, and final human disposition. It cannot claim that an issue exists without a verified issue identity.

## 7. Privacy and artifact policy

- Screenshots remain local by default.
- No artifact is embedded in a GitHub issue unless the operator approves that exact export.
- Public-repository export requires a separate warning and redaction acknowledgment.
- Password managers, credentials, terminals, private messages, personal information, private manuscripts, tokens, secrets, and unrelated applications are denied or require deterministic redaction.
- Issue bodies contain minimized reproduction information and evidence references, not unrestricted logs.
- Private target visibility does not remove the need for minimization.
- Artifact hashes, byte lengths, media types, redaction decisions, and retention state remain auditable.

## 8. GitHub mutation controls

### Zero tolerance

- Target repository mismatch.
- Missing or invalid human decision.
- Payload hash mismatch.
- Unknown application mapping.
- Non-allowlisted repository.
- Public export without explicit privacy acknowledgment.
- Secret-detection failure.
- Requested label or assignee outside admitted target policy.
- Duplicate proposal execution.
- Automatic retry after an outcome-uncertain create request.
- Creation receipt without a verified GitHub issue ID and URL.

### Ambiguous outcome

GitHub issue creation is treated as non-idempotent. If the executor loses the response after sending the request, it records outcome_uncertain and stops. It does not automatically retry. Reconciliation must search by the proposal trace marker and approved payload hash, then require an operator disposition if the result is not unique.

### Exact response verification

After a successful create response, the executor verifies:

- repository identity
- issue ID, number, and URL
- returned title and body hashes
- applied labels and assignees
- visibility assumptions
- trace marker

A mismatch produces a partial or failed receipt and blocks downstream automatic consumption.

## 9. First proving slice

One Linux workstation, one Tarcie session, one deliberately selected application, one synthetic observation, one synthetic redacted screenshot, one ZFSS fixture database, one Forge_Command fixture review, and one simulated GitHub endpoint.

The slice proves:

- repository selection from an allowlisted application mapping
- idempotent Tarcie-to-ZFSS Signal acceptance
- append-only Signal and Issue lineage
- deterministic issue proposal construction
- exact human approval
- simulated create success, rejection, timeout, and ambiguous outcome
- no automatic retry after ambiguity
- hash-bound creation and reconciliation receipts
- read-only Forge:SMITH handoff fixture

The slice excludes live GitHub writes, credentials, production ZFSS data, personal content, cloud routing, AI classification, repair, deployment, and release.

## 10. Success measures

- Zero lost accepted observations.
- Zero duplicate GitHub issues in fault-injection fixtures.
- Zero issue writes without a human decision.
- Zero public artifact exports without an explicit acknowledgment.
- One-to-one binding among decision, payload hash, target repository, and creation receipt.
- Complete lineage from Tarcie observation through ZFSS Signal and Issue to GitHub issue.
- Forge:SMITH can consume the created-issue fixture without becoming feedback authority.

## 11. Stop conditions

Stop immediately if:

- ZFSS cannot preserve raw Signal immutability.
- A screenshot can reach GitHub without a redaction/export decision.
- Repository routing can be influenced by untrusted note text.
- A create timeout triggers an automatic retry.
- Forge_Command authors or expands proposal content.
- Forge:SMITH is made feedback truth owner.
- A public repository receives private evidence.
- A receipt can be issued without a verified issue identity.
- Missing evidence is represented as accepted.
- Required source revisions cannot be pinned.

## 12. Explicit non-authorization

This plan does not authorize implementation, migrations, live Tarcie intake, screenshot capture, GitHub credentials, GitHub issue creation, issue updates, comments, labels, assignees, production database writes, agent execution, repair, deployment, promotion, release, beta enrollment, or registry activation.
