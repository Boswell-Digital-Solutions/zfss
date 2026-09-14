/**
 * Signals List View
 *
 * Display and filter signals.
 */

import { el, setContent } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";
import type { Signal, SignalStatus } from "../lib/types";

export async function render(container: HTMLElement, params: Record<string, string>): Promise<void> {
  const statusFilter = params.status as SignalStatus | undefined;

  setContent(container, el("div", { className: "loading" }, ["Loading signals..."]));

  try {
    const signals = await api.listSignals(statusFilter, 50);

    const view = el("div", { className: "signals-view" }, [
      el("header", { className: "view-header" }, [
        el("h2", {}, ["Signals"]),
        el("a", { className: "btn btn-primary", href: "#/capture" }, ["+ Capture"]),
      ]),

      // Status filter tabs
      createFilterTabs(statusFilter),

      // Signals list
      signals.length > 0
        ? el("div", { className: "signals-list" }, signals.map(renderSignalRow))
        : el("div", { className: "empty-state" }, [
            el("p", {}, ["No signals found."]),
            el("a", { className: "btn btn-primary", href: "#/capture" }, ["Capture your first signal"]),
          ]),
    ]);

    setContent(container, view);
  } catch (error) {
    setContent(
      container,
      el("div", { className: "error-state" }, [
        el("h3", {}, ["Error loading signals"]),
        el("p", {}, [formatIpcError(error)]),
      ])
    );
  }
}

function createFilterTabs(current?: string): HTMLElement {
  const statuses: (SignalStatus | "all")[] = ["all", "new", "linked", "needs_info", "responded", "closed"];

  return el("div", { className: "filter-tabs" }, statuses.map((status) => {
    const isActive = status === "all" ? !current : current === status;
    const href = status === "all" ? "#/signals" : `#/signals?status=${status}`;
    return el("a", {
      className: `filter-tab ${isActive ? "active" : ""}`,
      href,
    }, [status === "all" ? "All" : status.replace("_", " ")]);
  }));
}

function renderSignalRow(signal: Signal): HTMLElement {
  const truncatedText = signal.raw_text.length > 150
    ? signal.raw_text.slice(0, 150) + "..."
    : signal.raw_text;

  return el("a", { className: "signal-row", href: `#/signals/${signal.id}` }, [
    el("div", { className: "signal-row-main" }, [
      el("div", { className: "signal-row-header" }, [
        el("span", { className: `badge ${api.getStatusColor(signal.status)}` }, [signal.status]),
        el("span", { className: "signal-source" }, [signal.source]),
        signal.linked_issue_id
          ? el("span", { className: "signal-linked" }, [`Linked to ${signal.linked_issue_id}`])
          : el("span", {}, []),
      ]),
      el("p", { className: "signal-text" }, [truncatedText]),
    ]),
    el("div", { className: "signal-row-meta" }, [
      el("span", { className: "signal-time" }, [api.formatRelativeTime(signal.created_at)]),
      signal.app_key ? el("span", { className: "signal-app" }, [signal.app_key]) : el("span", {}, []),
    ]),
  ]);
}
