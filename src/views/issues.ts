/**
 * Issues List View
 *
 * Display and filter issues.
 */

import { el, setContent, router } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";
import type { IssueStatus } from "../lib/types";

export async function render(container: HTMLElement, params: Record<string, string>): Promise<void> {
  const statusFilter = params.status as IssueStatus | undefined;

  setContent(container, el("div", { className: "loading" }, ["Loading issues..."]));

  try {
    const issues = await api.listIssues(statusFilter, 50);

    const view = el("div", { className: "issues-view" }, [
      el("header", { className: "view-header" }, [
        el("h2", {}, ["Issues"]),
        el("button", { className: "btn btn-primary", id: "create-issue-btn" }, ["+ Create Issue"]),
      ]),

      // Status filter tabs
      createFilterTabs(statusFilter),

      // Issues list
      issues.length > 0
        ? el("div", { className: "issues-list" }, issues.map(renderIssueRow))
        : el("div", { className: "empty-state" }, [
            el("p", {}, ["No issues found."]),
            el("button", { className: "btn btn-primary", id: "create-first-issue-btn" }, ["Create your first issue"]),
          ]),
    ]);

    setContent(container, view);

    // Add event listeners
    setupEventListeners();
  } catch (error) {
    setContent(
      container,
      el("div", { className: "error-state" }, [
        el("h3", {}, ["Error loading issues"]),
        el("p", {}, [formatIpcError(error)]),
      ])
    );
  }
}

function createFilterTabs(current?: string): HTMLElement {
  const statuses: (IssueStatus | "all")[] = [
    "all",
    "pending_decision",
    "decided",
    "in_progress",
    "ready_for_verification",
    "closed",
  ];

  return el("div", { className: "filter-tabs" }, statuses.map((status) => {
    const isActive = status === "all" ? !current : current === status;
    const href = status === "all" ? "#/issues" : `#/issues?status=${status}`;
    const label = status === "all" ? "All" : status.replace(/_/g, " ");
    return el("a", {
      className: `filter-tab ${isActive ? "active" : ""}`,
      href,
    }, [label]);
  }));
}

function renderIssueRow(issue: api.IssueSummary): HTMLElement {
  return el("a", { className: "issue-row", href: `#/issues/${issue.id}` }, [
    el("div", { className: "issue-row-main" }, [
      el("div", { className: "issue-row-header" }, [
        el("span", { className: `badge ${api.getStatusColor(issue.status)}` }, [issue.status.replace(/_/g, " ")]),
        el("span", { className: `badge ${api.getSeverityColor(issue.severity)}` }, [issue.severity]),
        el("span", { className: "issue-classification" }, [issue.classification]),
      ]),
      el("h3", { className: "issue-title" }, [issue.title]),
    ]),
    el("div", { className: "issue-row-meta" }, [
      el("span", { className: "issue-signals" }, [`${issue.signal_count} signal${issue.signal_count !== 1 ? "s" : ""}`]),
      el("span", { className: "issue-time" }, [api.formatRelativeTime(issue.created_at)]),
    ]),
  ]);
}

async function createIssue(): Promise<void> {
  const title = prompt("Issue Title:");
  if (!title) return;

  const classification = prompt("Classification (Bug, UX, Feature, Limitation):", "Bug");
  if (!classification) return;

  const severity = prompt("Severity (blocker, major, minor, idea):", "major");
  if (!severity) return;

  const description = prompt("Description (optional):");

  try {
    const issue = await api.createIssue(
      title,
      classification as any,
      severity as any,
      description || undefined
    );
    router.navigate(`/issues/${issue.id}`);
  } catch (error) {
    alert(`Error creating issue: ${formatIpcError(error)}`);
  }
}

function setupEventListeners(): void {
  const createBtn = document.getElementById("create-issue-btn");
  if (createBtn) {
    createBtn.addEventListener("click", createIssue);
  }

  const createFirstBtn = document.getElementById("create-first-issue-btn");
  if (createFirstBtn) {
    createFirstBtn.addEventListener("click", createIssue);
  }
}
