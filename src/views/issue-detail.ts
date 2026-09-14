/**
 * Issue Detail View
 *
 * View a single issue with decisions, artifacts, and linked signals.
 */

import { el, setContent } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";
import type { Issue, IssueStatus, DecisionType, ArtifactType } from "../lib/types";

export async function render(container: HTMLElement, params: Record<string, string>): Promise<void> {
  const issueId = params.id;

  setContent(container, el("div", { className: "loading" }, ["Loading issue..."]));

  try {
    const issue = await api.getIssue(issueId);

    if (!issue) {
      setContent(
        container,
        el("div", { className: "error-state" }, [
          el("h3", {}, ["Issue not found"]),
          el("a", { className: "btn btn-secondary", href: "#/issues" }, ["Back to issues"]),
        ])
      );
      return;
    }

    // Fetch related data
    const [decisions, artifacts, hasVerified] = await Promise.all([
      api.listDecisionsForIssue(issueId),
      api.listArtifactsForIssue(issueId),
      api.hasVerifiedArtifact(issueId),
    ]);

    const currentDecision = decisions.find((d) => d.is_current);

    const view = el("div", { className: "issue-detail" }, [
      // Header
      el("header", { className: "view-header" }, [
        el("a", { className: "back-link", href: "#/issues" }, ["< Issues"]),
        el("h2", {}, [issue.title]),
      ]),

      // Issue info card
      el("div", { className: "detail-card" }, [
        el("div", { className: "detail-header" }, [
          el("span", { className: `badge ${api.getStatusColor(issue.status)}` }, [issue.status.replace(/_/g, " ")]),
          el("span", { className: `badge ${api.getSeverityColor(issue.severity)}` }, [issue.severity]),
          el("span", { className: "issue-classification" }, [issue.classification]),
          el("span", { className: "detail-id" }, [issue.id]),
        ]),

        issue.description
          ? el("div", { className: "detail-section" }, [
              el("label", {}, ["Description"]),
              el("p", {}, [issue.description]),
            ])
          : el("span", {}, []),

        el("div", { className: "detail-row" }, [
          el("div", { className: "detail-section" }, [
            el("label", {}, ["Signals"]),
            el("span", {}, [String(issue.frequency)]),
          ]),
          el("div", { className: "detail-section" }, [
            el("label", {}, ["Created"]),
            el("span", {}, [api.formatDate(issue.created_at)]),
          ]),
          el("div", { className: "detail-section" }, [
            el("label", {}, ["Requires Artifact"]),
            el("span", {}, [issue.close_requires_artifact ? "Yes" : "No"]),
          ]),
        ]),
      ]),

      // Status transition actions
      renderStatusActions(issue, hasVerified),

      // Decision section
      el("section", { className: "detail-section-block" }, [
        el("div", { className: "section-header" }, [
          el("h3", {}, ["Decision"]),
          issue.status === "pending_decision"
            ? el("button", { className: "btn btn-primary", id: "record-decision-btn" }, ["Record Decision"])
            : el("span", {}, []),
        ]),
        currentDecision
          ? el("div", { className: "decision-card" }, [
              el("div", { className: "decision-header" }, [
                el("span", { className: "badge status-info" }, [currentDecision.decision_type]),
                el("span", { className: "decision-by" }, [`by ${currentDecision.decided_by}`]),
                el("span", { className: "decision-time" }, [api.formatRelativeTime(currentDecision.decided_at)]),
              ]),
              el("p", { className: "decision-rationale" }, [currentDecision.rationale]),
            ])
          : el("p", { className: "empty-state" }, ["No decision recorded yet."]),

        decisions.length > 1
          ? el("details", { className: "decision-history" }, [
              el("summary", {}, [`View history (${decisions.length} decisions)`]),
              el("div", { className: "history-list" }, decisions.map((d) =>
                el("div", { className: `history-item ${d.is_current ? "current" : ""}` }, [
                  el("span", { className: "badge" }, [d.decision_type]),
                  el("span", {}, [d.rationale.slice(0, 50) + (d.rationale.length > 50 ? "..." : "")]),
                  el("span", { className: "history-time" }, [api.formatDate(d.decided_at)]),
                ])
              )),
            ])
          : el("span", {}, []),
      ]),

      // Artifacts section
      el("section", { className: "detail-section-block" }, [
        el("div", { className: "section-header" }, [
          el("h3", {}, ["Artifacts"]),
          el("button", { className: "btn btn-secondary", id: "create-artifact-btn" }, ["+ Add Artifact"]),
        ]),
        artifacts.length > 0
          ? el("div", { className: "artifacts-list" }, artifacts.map((a) =>
              el("div", { className: `artifact-card ${a.verified ? "verified" : ""}` }, [
                el("div", { className: "artifact-header" }, [
                  el("span", { className: "badge" }, [a.artifact_type]),
                  a.verified
                    ? el("span", { className: "badge status-success" }, ["Verified"])
                    : el("button", { className: "btn btn-sm", "data-artifact-id": a.id }, ["Verify"]),
                ]),
                el("h4", {}, [a.title]),
                el("span", { className: "artifact-time" }, [api.formatRelativeTime(a.created_at)]),
              ])
            ))
          : el("p", { className: "empty-state" }, ["No artifacts created yet."]),
      ]),
    ]);

    setContent(container, view);

    // Setup event listeners
    setupEventListeners(issue);
  } catch (error) {
    setContent(
      container,
      el("div", { className: "error-state" }, [
        el("h3", {}, ["Error loading issue"]),
        el("p", {}, [formatIpcError(error)]),
      ])
    );
  }
}

function renderStatusActions(issue: Issue, hasVerified: boolean): HTMLElement {
  const actions: HTMLElement[] = [];

  // Determine valid next statuses
  const transitions: Record<IssueStatus, { next: IssueStatus; label: string }[]> = {
    pending_decision: [],
    decided: [{ next: "in_progress", label: "Start Work" }],
    in_progress: [{ next: "ready_for_verification", label: "Ready for Verification" }],
    ready_for_verification: [
      { next: "closed", label: "Close Issue" },
      { next: "in_progress", label: "Needs Rework" },
    ],
    closed: [],
  };

  const available = transitions[issue.status as IssueStatus] || [];

  available.forEach(({ next, label }) => {
    // Special check for closing
    if (next === "closed" && issue.close_requires_artifact && !hasVerified) {
      actions.push(
        el("button", { className: "btn btn-disabled", disabled: "true" }, [
          `${label} (requires verified artifact)`,
        ])
      );
    } else {
      const btn = el("button", {
        className: "btn btn-primary",
        "data-transition": next,
      }, [label]);
      actions.push(btn);
    }
  });

  return actions.length > 0
    ? el("div", { className: "status-actions" }, actions)
    : el("span", {}, []);
}

function setupEventListeners(issue: Issue): void {
  // Record decision button
  const decisionBtn = document.getElementById("record-decision-btn");
  if (decisionBtn) {
    decisionBtn.addEventListener("click", async () => {
      const decisionType = prompt(
        "Decision Type (FixNow, FixLater, DocumentClarify, WontFix, DeEscalate):",
        "FixNow"
      );
      if (!decisionType) return;

      const rationale = prompt("Rationale (min 10 characters):");
      if (!rationale || rationale.length < 10) {
        alert("Rationale must be at least 10 characters");
        return;
      }

      try {
        await api.recordDecision(issue.id, decisionType as DecisionType, rationale);
        location.reload();
      } catch (error) {
        alert(`Error: ${formatIpcError(error)}`);
      }
    });
  }

  // Create artifact button
  const artifactBtn = document.getElementById("create-artifact-btn");
  if (artifactBtn) {
    artifactBtn.addEventListener("click", async () => {
      const title = prompt("Artifact Title:");
      if (!title) return;

      const artifactType = prompt("Type (Code, Logic, Knowledge, Test, Law):", "Code");
      if (!artifactType) return;

      const refUrl = prompt("Reference URL (optional):");

      try {
        await api.createArtifact(issue.id, artifactType as ArtifactType, title, undefined, refUrl || undefined);
        location.reload();
      } catch (error) {
        alert(`Error: ${formatIpcError(error)}`);
      }
    });
  }

  // Status transition buttons
  document.querySelectorAll("[data-transition]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const newStatus = (btn as HTMLElement).dataset.transition as IssueStatus;
      const reason = prompt("Reason for transition (optional):");

      try {
        await api.transitionIssue(issue.id, newStatus, reason || undefined);
        location.reload();
      } catch (error) {
        alert(`Error: ${formatIpcError(error)}`);
      }
    });
  });

  // Verify artifact buttons
  document.querySelectorAll("[data-artifact-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const artifactId = (btn as HTMLElement).dataset.artifactId!;

      try {
        await api.verifyArtifact(artifactId);
        location.reload();
      } catch (error) {
        alert(`Error: ${formatIpcError(error)}`);
      }
    });
  });
}
