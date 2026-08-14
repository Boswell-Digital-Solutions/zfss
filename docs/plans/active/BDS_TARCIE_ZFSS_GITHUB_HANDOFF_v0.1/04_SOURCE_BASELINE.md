# Source Baseline — BDS-TARCIE-ZFSS-GITHUB-HANDOFF-v0.1

Verified through the connected GitHub repositories on 2026-08-14.

## Pinned repository heads

| Repository | Branch | Head | Visibility |
| --- | --- | --- | --- |
| Boswell-Digital-Solutions/tarcie | main | 3661ae223d96903f58c29acec7918221ef1abd53 | public |
| Boswell-Digital-Solutions/zfss | master | d004a16c37533cf70ea2efc0b43771f24bbeae40 | public |
| Boswell-Digital-Solutions/Forge_Command | main | fbcf51e1575e4d57152cfa7079f888d191bc7939 | private |
| Boswell-Digital-Solutions/forge-smithy | master | 4b8d431dcbaca3f3fbc99d84d078425b7ce6a674 | private |

A gate session must re-pin from the remote immediately before accepting GATE-00.

## Tarcie facts

- Fast note and marker capture with durable JSONL queue and generic HTTP sink.
- Existing beta-evidence plan proposes session identity, screenshots, artifact hashes, receipts, and Forge_Command intake.
- Automatic GitHub issue proposals are deferred and live issue creation is explicitly unauthorized.
- Current repository has no GitHub issue client or issue template.
- Current CI proof at the pinned head includes 89 Rust tests and 22 frontend tests.
- Tarcie's default sink still does not prove a live ZFSS or Forge_Command receiver.

## ZFSS facts

- ZFSS means Zen Feedback & Service System.
- Canonical objects already include Signal, Issue, Decision, Artifact, Response, and Attachment.
- Signals are intended as raw immutable feedback expressions.
- Internal Issues aggregate multiple Signals.
- Attachments admit screenshot, log, video, file, and link kinds.
- Role model states that the Steward decides, Operator executes, Engineer builds, and AI suggests.
- SignalSource does not include tarcie.
- Signal records include app_key and app_version but lack the proposed session, repository, build, commit, and receipt bindings.
- Attachment records lack required SHA-256, byte length, redaction, and retention fields.
- The service layer remains a placeholder.
- Repository guidance reports no CI workflow and no test script; current database verification relies on local PostgreSQL scripts.
- No GitHub issue integration was found.

## Forge_Command facts

- Operator-facing mission control and proposal review surface.
- Existing doctrine says Forge_Command displays proposals, captures decisions, and issues receipts when permitted, but does not author evidence or proposal content.
- Existing proposal/approval contracts are scoped to other capabilities and cannot be silently reused as GitHub issue authority.
- No Tarcie/ZFSS feedback intake or GitHub create-issue implementation was found.
- Existing GitHub issue #11 demonstrates that the repository uses GitHub Issues for operator/engineering work.

## Forge:SMITH facts

- Desktop governance workbench and authority layer for governed engineering work.
- No inbound REST API; ecosystem communication is outbound from the Rust backend.
- Repository is in remediation posture, so release-ready claims are downgraded.
- No Tarcie or ZFSS feedback intake was found.
- Existing GitHub issue #29 treats GitHub issues as discovery and engineering planning surfaces.
- Correct first role is read-only consumption of an approved engineering handoff, followed by separately authorized planning or mutation.

## Baseline conclusions

1. ZFSS is the strongest canonical feedback-domain home.
2. Forge_Command is the strongest operator review and GitHub execution surface.
3. Forge:SMITH should remain downstream of a created issue.
4. The feature does not exist today.
5. The first work must be contracts, fixtures, privacy, and ambiguity handling rather than live GitHub writes.
