/**
 * Dashboard View
 *
 * Overview of signals, issues, and pending actions.
 */

import { el, setContent } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";
import type { Signal } from "../lib/types";

export async function render(container: HTMLElement): Promise<void> {
  // Show loading state
  setContent(container, el("div", { className: "loading" }, ["Loading dashboard..."]));

  try {
    // Fetch data in parallel
    const [signals, issues] = await Promise.all([
      api.listSignals(undefined, 100),
      api.listIssues(undefined, 100),
    ]);

    // Calculate stats
    const newSignals = signals.filter((s) => s.status === "new").length;
    const pendingIssues = issues.filter((i) => i.status === "pending_decision").length;
    const inProgressIssues = issues.filter((i) => i.status === "in_progress").length;
    const readyForVerification = issues.filter((i) => i.status === "ready_for_verification").length;

    // Recent activity (last 5 signals)
    const recentSignals = signals.slice(0, 5);

    // Render dashboard
    const view = el("div", { className: "dashboard" }, [
      el("header", { className: "view-header" }, [
        el("h2", {}, ["Dashboard"]),
      ]),

      // Stats cards
      el("div", { className: "stats-grid" }, [
        createStatCard("New Signals", newSignals, "stat-new", "#/signals?status=new"),
        createStatCard("Pending Decision", pendingIssues, "stat-pending", "#/issues?status=pending_decision"),
        createStatCard("In Progress", inProgressIssues, "stat-progress", "#/issues?status=in_progress"),
        createStatCard("Ready for Verification", readyForVerification, "stat-verify", "#/issues?status=ready_for_verification"),
      ]),

      // Recent signals section
      el("section", { className: "recent-section" }, [
        el("h3", {}, ["Recent Signals"]),
        recentSignals.length > 0
          ? el("div", { className: "recent-list" }, recentSignals.map(createSignalCard))
          : el("p", { className: "empty-state" }, ["No signals captured yet. Press Ctrl+Alt+Z to capture feedback."]),
      ]),

      // Quick actions
      el("section", { className: "quick-actions" }, [
        el("h3", {}, ["Quick Actions"]),
        el("div", { className: "action-buttons" }, [
          createActionButton("Capture Signal", "#/capture", "btn-primary"),
          createActionButton("View All Signals", "#/signals", "btn-secondary"),
          createActionButton("View All Issues", "#/issues", "btn-secondary"),
        ]),
      ]),
    ]);

    setContent(container, view);
  } catch (error) {
    setContent(
      container,
      el("div", { className: "error-state" }, [
        el("h3", {}, ["Error loading dashboard"]),
        el("p", {}, [formatIpcError(error)]),
        el("button", { className: "btn-primary" }, ["Retry"]),
      ])
    );
  }
}

function createStatCard(
  label: string,
  value: number,
  className: string,
  href: string
): HTMLElement {
  const card = el("a", { className: `stat-card ${className}`, href }, [
    el("span", { className: "stat-value" }, [String(value)]),
    el("span", { className: "stat-label" }, [label]),
  ]);
  return card;
}

function createSignalCard(signal: Signal): HTMLElement {
  const truncatedText = signal.raw_text.length > 100
    ? signal.raw_text.slice(0, 100) + "..."
    : signal.raw_text;

  return el("a", { className: "signal-card", href: `#/signals/${signal.id}` }, [
    el("div", { className: "signal-card-header" }, [
      el("span", { className: `badge ${api.getStatusColor(signal.status)}` }, [signal.status]),
      el("span", { className: "signal-source" }, [signal.source]),
      el("span", { className: "signal-time" }, [api.formatRelativeTime(signal.created_at)]),
    ]),
    el("p", { className: "signal-text" }, [truncatedText]),
  ]);
}

function createActionButton(label: string, href: string, className: string): HTMLElement {
  return el("a", { className: `btn ${className}`, href }, [label]);
}
