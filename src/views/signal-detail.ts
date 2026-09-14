/**
 * Signal Detail View
 *
 * View a single signal and its responses.
 */

import { el, setContent, router } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";
import type { Signal } from "../lib/types";

export async function render(container: HTMLElement, params: Record<string, string>): Promise<void> {
  const signalId = params.id;

  setContent(container, el("div", { className: "loading" }, ["Loading signal..."]));

  try {
    const signal = await api.getSignal(signalId);

    if (!signal) {
      setContent(
        container,
        el("div", { className: "error-state" }, [
          el("h3", {}, ["Signal not found"]),
          el("a", { className: "btn btn-secondary", href: "#/signals" }, ["Back to signals"]),
        ])
      );
      return;
    }

    // Fetch responses for this signal
    const responses = await api.listResponsesForSignal(signalId);

    const view = el("div", { className: "signal-detail" }, [
      // Header with back button
      el("header", { className: "view-header" }, [
        el("a", { className: "back-link", href: "#/signals" }, ["< Signals"]),
        el("h2", {}, ["Signal Details"]),
      ]),

      // Signal info card
      el("div", { className: "detail-card" }, [
        el("div", { className: "detail-header" }, [
          el("span", { className: `badge ${api.getStatusColor(signal.status)}` }, [signal.status]),
          el("span", { className: "detail-id" }, [signal.id]),
        ]),

        el("div", { className: "detail-section" }, [
          el("label", {}, ["Source"]),
          el("span", {}, [signal.source]),
        ]),

        el("div", { className: "detail-section" }, [
          el("label", {}, ["Feedback"]),
          el("p", { className: "raw-text" }, [signal.raw_text]),
        ]),

        signal.app_key
          ? el("div", { className: "detail-section" }, [
              el("label", {}, ["App Key"]),
              el("span", {}, [signal.app_key]),
            ])
          : el("span", {}, []),

        el("div", { className: "detail-section" }, [
          el("label", {}, ["Created"]),
          el("span", {}, [api.formatDate(signal.created_at)]),
        ]),

        el("div", { className: "detail-section" }, [
          el("label", {}, ["Created By"]),
          el("span", {}, [signal.created_by]),
        ]),

        signal.linked_issue_id
          ? el("div", { className: "detail-section" }, [
              el("label", {}, ["Linked Issue"]),
              el("a", { href: `#/issues/${signal.linked_issue_id}` }, [signal.linked_issue_id]),
            ])
          : el("span", {}, []),
      ]),

      // Actions
      el("div", { className: "detail-actions" }, [
        !signal.linked_issue_id
          ? createLinkToIssueButton()
          : el("span", {}, []),
        el("button", { className: "btn btn-secondary", id: "draft-response-btn" }, ["Draft Response"]),
      ]),

      // Responses section
      el("section", { className: "responses-section" }, [
        el("h3", {}, ["Responses"]),
        responses.length > 0
          ? el("div", { className: "responses-list" }, responses.map(renderResponseCard))
          : el("p", { className: "empty-state" }, ["No responses drafted yet."]),
      ]),
    ]);

    setContent(container, view);

    // Add event listeners
    setupEventListeners(signal);
  } catch (error) {
    setContent(
      container,
      el("div", { className: "error-state" }, [
        el("h3", {}, ["Error loading signal"]),
        el("p", {}, [formatIpcError(error)]),
      ])
    );
  }
}

function createLinkToIssueButton(): HTMLElement {
  const btn = el("button", { className: "btn btn-primary", id: "link-issue-btn" }, ["Link to Issue"]);
  return btn;
}

function renderResponseCard(response: api.ResponseSummary): HTMLElement {
  return el("div", { className: "response-card" }, [
    el("div", { className: "response-header" }, [
      el("span", { className: `badge ${api.getStatusColor(response.approval_state)}` }, [response.approval_state]),
      el("span", { className: "response-channel" }, [response.channel]),
      el("span", { className: "response-time" }, [api.formatRelativeTime(response.drafted_at)]),
    ]),
    response.has_violations
      ? el("span", { className: "badge status-error" }, ["Has Violations"])
      : el("span", {}, []),
  ]);
}

function setupEventListeners(signal: Signal): void {
  // Link to issue button
  const linkBtn = document.getElementById("link-issue-btn");
  if (linkBtn) {
    linkBtn.addEventListener("click", async () => {
      const issueId = prompt("Enter Issue ID to link (e.g., iss_xxxx):");
      if (issueId) {
        try {
          await api.linkSignalToIssue(signal.id, issueId);
          router.navigate(`/signals/${signal.id}`);
          location.reload(); // Simple refresh
        } catch (error) {
          alert(`Error: ${formatIpcError(error)}`);
        }
      }
    });
  }

  // Draft response button
  const draftBtn = document.getElementById("draft-response-btn");
  if (draftBtn) {
    draftBtn.addEventListener("click", () => {
      router.navigate(`/responses/new?signalId=${signal.id}`);
    });
  }
}
