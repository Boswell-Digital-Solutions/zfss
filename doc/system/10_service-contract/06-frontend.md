## 6. Frontend

### Technology

Vanilla TypeScript SPA (no framework). Vite 5 for dev server and bundling. Tauri IPC for backend communication.

### Entry Point (main.ts)

The signal capture UI is the primary entry point:
- Text area for raw feedback
- Source selector (email, phone, in-person, app, social, other)
- Submit button → invokes `capture_signal` via Tauri IPC
- Hotkey toggle awareness (window show/hide)

### Router (lib/router.ts)

Hash-based SPA router with simple pattern matching:

| Route | View | Status |
|-------|------|--------|
| `#/capture` | Signal capture form | Implemented |
| `#/signals` | Signals list (pending triage) | Planned |
| `#/signals/:id` | Signal detail + linking | Planned |
| `#/issues` | Issues list | Planned |
| `#/issues/:id` | Issue detail + responses | Planned |
| `#/dashboard` | Overview/statistics | Planned |

### API Layer (lib/api.ts)

25+ TypeScript functions wrapping Tauri `invoke()` calls:

```typescript
// Signal operations
captureSignal(raw_text, source, user_id, device_id)
listSignals()
getSignal(signal_id)
linkSignalToIssue(signal_id, issue_id)

// Issue operations
createIssue(title, description, ...)
listIssues()
getIssue(issue_id)
transitionIssue(issue_id, new_status)

// Decision operations
recordDecision(issue_id, decision_type, body)
getDecision(decision_id)
listDecisionsForIssue(issue_id)
getCurrentDecision(issue_id)

// Artifact operations
createArtifact(issue_id, artifact_type, title, url)
getArtifact(artifact_id)
listArtifactsForIssue(issue_id)
verifyArtifact(artifact_id, verified_by)
hasVerifiedArtifact(issue_id)

// Response operations
draftResponse(signal_id, channel, body)
getResponse(response_id)
listResponsesForSignal(signal_id)
submitResponse(response_id)
approveResponse(response_id, approved_by)
blockResponse(response_id, reason)
markResponseSent(response_id)
```

### Types (lib/types.ts)

TypeScript type definitions mirroring Rust models:
- Status enums: `SignalStatus`, `IssueStatus`, `ApprovalState`
- Model interfaces: `Signal`, `Issue`, `Decision`, `Artifact`, `Response`
- Role enum: `UserRole`

### IPC Error Boundary (lib/ipc-error.ts)

All command failures pass through one frontend parser before display. It accepts only the seven documented `ZFSS_*` codes with a non-empty message, formats valid envelopes with their support code, and maps strings, ordinary `Error` objects, unknown codes, empty messages, arrays, and null values to a generic `ZFSS_INTERNAL` failure without rendering their contents.

Node contract tests cover all admitted codes, user-facing formatting, and fail-closed handling of malformed or credential-bearing values. CI runs these tests and a strict application typecheck before the production build.

### Styling

Global CSS in `styles.css`. No CSS framework. Minimal UI focused on fast signal capture.
