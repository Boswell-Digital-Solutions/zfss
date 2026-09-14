/**
 * ZFSS API Layer
 *
 * TypeScript wrappers for all Tauri IPC commands.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  Signal,
  SignalStatus,
  Issue,
  IssueStatus,
  Decision,
  Artifact,
  Response,
  Classification,
  Severity,
  DecisionType,
  ArtifactType,
  ResponseChannel,
} from "./types";

// =============================================================================
// SIGNAL API
// =============================================================================

export interface CaptureResult {
  signal_id: string;
  status: string;
  created_at: string;
}

export async function captureSignal(
  source: string,
  rawText: string,
  appKey?: string
): Promise<CaptureResult> {
  return invoke<CaptureResult>("capture_signal", {
    source,
    rawText,
    appKey: appKey || null,
  });
}

export async function listSignals(
  status?: SignalStatus,
  limit?: number
): Promise<Signal[]> {
  return invoke<Signal[]>("list_signals", {
    status: status || null,
    limit: limit || null,
  });
}

export async function getSignal(id: string): Promise<Signal | null> {
  return invoke<Signal | null>("get_signal", { id });
}

export async function linkSignalToIssue(
  signalId: string,
  issueId: string
): Promise<Signal> {
  return invoke<Signal>("link_signal_to_issue", { signalId, issueId });
}

// =============================================================================
// ISSUE API
// =============================================================================

export interface IssueSummary {
  id: string;
  title: string;
  classification: string;
  severity: string;
  status: string;
  signal_count: number;
  created_at: string;
}

export async function createIssue(
  title: string,
  classification: Classification,
  severity: Severity,
  description?: string
): Promise<Issue> {
  return invoke<Issue>("create_issue", {
    title,
    description: description || null,
    classification,
    severity,
  });
}

export async function listIssues(
  status?: IssueStatus,
  limit?: number
): Promise<IssueSummary[]> {
  return invoke<IssueSummary[]>("list_issues", {
    status: status || null,
    limit: limit || null,
  });
}

export async function getIssue(id: string): Promise<Issue | null> {
  return invoke<Issue | null>("get_issue", { id });
}

export async function transitionIssue(
  issueId: string,
  newStatus: IssueStatus,
  reason?: string
): Promise<Issue> {
  return invoke<Issue>("transition_issue", {
    issueId,
    newStatus,
    reason: reason || null,
  });
}

// =============================================================================
// DECISION API
// =============================================================================

export interface DecisionHistoryEntry {
  id: string;
  decision_type: string;
  rationale: string;
  decided_by: string;
  decided_at: string;
  is_current: boolean;
}

export async function recordDecision(
  issueId: string,
  decisionType: DecisionType,
  rationale: string,
  stewardDeadlineDays?: number
): Promise<Decision> {
  return invoke<Decision>("record_decision", {
    issueId,
    decisionType,
    rationale,
    stewardDeadlineDays: stewardDeadlineDays || null,
  });
}

export async function getDecision(id: string): Promise<Decision | null> {
  return invoke<Decision | null>("get_decision", { id });
}

export async function listDecisionsForIssue(
  issueId: string
): Promise<DecisionHistoryEntry[]> {
  return invoke<DecisionHistoryEntry[]>("list_decisions_for_issue", { issueId });
}

export async function getCurrentDecision(
  issueId: string
): Promise<Decision | null> {
  return invoke<Decision | null>("get_current_decision", { issueId });
}

// =============================================================================
// ARTIFACT API
// =============================================================================

export interface ArtifactSummary {
  id: string;
  artifact_type: string;
  title: string;
  verified: boolean;
  created_at: string;
}

export async function createArtifact(
  issueId: string,
  artifactType: ArtifactType,
  title: string,
  description?: string,
  refUrl?: string,
  note?: string
): Promise<Artifact> {
  return invoke<Artifact>("create_artifact", {
    issueId,
    artifactType,
    title,
    description: description || null,
    refUrl: refUrl || null,
    note: note || null,
  });
}

export async function getArtifact(id: string): Promise<Artifact | null> {
  return invoke<Artifact | null>("get_artifact", { id });
}

export async function listArtifactsForIssue(
  issueId: string
): Promise<ArtifactSummary[]> {
  return invoke<ArtifactSummary[]>("list_artifacts_for_issue", { issueId });
}

export async function verifyArtifact(artifactId: string): Promise<Artifact> {
  return invoke<Artifact>("verify_artifact", { artifactId });
}

export async function hasVerifiedArtifact(issueId: string): Promise<boolean> {
  return invoke<boolean>("has_verified_artifact", { issueId });
}

// =============================================================================
// RESPONSE API
// =============================================================================

export interface ResponseSummary {
  id: string;
  signal_id: string;
  channel: string;
  approval_state: string;
  drafted_at: string;
  has_violations: boolean;
}

export async function draftResponse(
  signalId: string,
  responseClass: string,
  channel: ResponseChannel,
  body: string,
  issueId?: string
): Promise<Response> {
  return invoke<Response>("draft_response", {
    signalId,
    issueId: issueId || null,
    responseClass,
    channel,
    body,
  });
}

export async function getResponse(id: string): Promise<Response | null> {
  return invoke<Response | null>("get_response", { id });
}

export async function listResponsesForSignal(
  signalId: string
): Promise<ResponseSummary[]> {
  return invoke<ResponseSummary[]>("list_responses_for_signal", { signalId });
}

export async function submitResponse(responseId: string): Promise<Response> {
  return invoke<Response>("submit_response", { responseId });
}

export async function approveResponse(responseId: string): Promise<Response> {
  return invoke<Response>("approve_response", { responseId });
}

export async function blockResponse(
  responseId: string,
  reason: string
): Promise<Response> {
  return invoke<Response>("block_response", { responseId, reason });
}

export async function markResponseSent(responseId: string): Promise<Response> {
  return invoke<Response>("mark_response_sent", { responseId });
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/** Format ISO date string for display */
export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Format relative time (e.g., "2 hours ago") */
export function formatRelativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return formatDate(isoString);
}

/** Get status badge color class */
export function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    // Signal status
    new: "status-new",
    linked: "status-linked",
    needs_info: "status-warning",
    responded: "status-success",
    closed: "status-closed",
    // Issue status
    pending_decision: "status-warning",
    decided: "status-info",
    in_progress: "status-active",
    ready_for_verification: "status-success",
    // Response status
    draft: "status-draft",
    pending: "status-warning",
    approved: "status-success",
    sent: "status-closed",
    blocked: "status-error",
  };
  return colors[status] || "status-default";
}

/** Get severity badge color class */
export function getSeverityColor(severity: string): string {
  const colors: Record<string, string> = {
    blocker: "severity-blocker",
    major: "severity-major",
    minor: "severity-minor",
    idea: "severity-idea",
  };
  return colors[severity] || "severity-default";
}
