/**
 * Signal Capture View
 *
 * Fast signal capture form (<60s target).
 */

import { el, setContent } from "../lib/router";
import * as api from "../lib/api";
import { formatIpcError } from "../lib/ipc-error";

export async function render(container: HTMLElement): Promise<void> {
  const view = el("div", { className: "capture-view" }, [
    el("header", { className: "view-header" }, [
      el("a", { className: "back-link", href: "#/" }, ["< Dashboard"]),
      el("h2", {}, ["Capture Signal"]),
    ]),

    el("form", { className: "capture-form", id: "capture-form" }, [
      // Source selector
      el("div", { className: "form-group" }, [
        el("label", { htmlFor: "source" }, ["Source"]),
        el("select", { id: "source", name: "source" }, [
          el("option", { value: "in_app" }, ["In-App"]),
          el("option", { value: "email" }, ["Email"]),
          el("option", { value: "dm" }, ["DM"]),
          el("option", { value: "call" }, ["Call"]),
          el("option", { value: "internal" }, ["Internal"]),
          el("option", { value: "partner" }, ["Partner"]),
          el("option", { value: "monitoring" }, ["Monitoring"]),
        ]),
      ]),

      // Raw text input
      el("div", { className: "form-group" }, [
        el("label", { htmlFor: "raw-text" }, ["Feedback"]),
        el("textarea", {
          id: "raw-text",
          name: "rawText",
          placeholder: "Capture the user's feedback exactly as expressed...",
          rows: "6",
        }, []),
      ]),

      // App key
      el("div", { className: "form-group" }, [
        el("label", { htmlFor: "app-key" }, ["App Key (optional)"]),
        el("input", {
          type: "text",
          id: "app-key",
          name: "appKey",
          placeholder: "e.g., forge-command",
        }, []),
      ]),

      // Status and actions
      el("div", { className: "form-footer" }, [
        el("div", { id: "status", className: "status" }, []),
        el("div", { className: "form-actions" }, [
          el("span", { className: "hint" }, ["Ctrl+Enter to capture"]),
          el("button", { type: "submit", className: "btn btn-primary" }, ["Capture Signal"]),
        ]),
      ]),
    ]),
  ]);

  setContent(container, view);

  // Setup form handling
  const form = document.getElementById("capture-form") as HTMLFormElement;
  const rawTextEl = document.getElementById("raw-text") as HTMLTextAreaElement;
  const sourceEl = document.getElementById("source") as HTMLSelectElement;
  const appKeyEl = document.getElementById("app-key") as HTMLInputElement;
  const statusEl = document.getElementById("status") as HTMLDivElement;

  let isCapturing = false;

  async function captureSignal(): Promise<void> {
    const rawText = rawTextEl.value.trim();

    if (!rawText) {
      showStatus("Please enter feedback text", "error");
      return;
    }

    if (isCapturing) return;

    isCapturing = true;
    showStatus("Capturing...", "");

    try {
      const result = await api.captureSignal(
        sourceEl.value,
        rawText,
        appKeyEl.value || undefined
      );

      showStatus(`Captured: ${result.signal_id}`, "success");

      // Clear form after brief delay
      setTimeout(() => {
        rawTextEl.value = "";
        appKeyEl.value = "";
        showStatus("", "");
        rawTextEl.focus();
      }, 1500);
    } catch (error) {
      showStatus(`Error: ${formatIpcError(error)}`, "error");
    } finally {
      isCapturing = false;
    }
  }

  function showStatus(message: string, type: string): void {
    statusEl.textContent = message;
    statusEl.className = `status ${type}`;
  }

  form.addEventListener("submit", (e) => {
    e.preventDefault();
    captureSignal().catch(console.error);
  });

  rawTextEl.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Enter" && e.ctrlKey) {
      e.preventDefault();
      captureSignal().catch(console.error);
    }
  });

  // Auto-focus
  rawTextEl.focus();
}
