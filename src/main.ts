/**
 * ZFSS - Zen Feedback & Service System
 *
 * Frontend for signal capture and feedback management.
 */

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { formatIpcError } from "./lib/ipc-error";

const win = getCurrentWebviewWindow();

// DOM elements
const rawTextEl = document.getElementById("raw-text") as HTMLTextAreaElement;
const sourceEl = document.getElementById("source") as HTMLSelectElement;
const appKeyEl = document.getElementById("app-key") as HTMLInputElement;
const captureBtn = document.getElementById("capture-btn") as HTMLButtonElement;
const statusEl = document.getElementById("status") as HTMLDivElement;
const captureContainer = document.querySelector(".signal-capture") as HTMLDivElement;

// Types
interface CaptureResult {
  signal_id: string;
  status: string;
  created_at: string;
}

// State
let isCapturing = false;

/**
 * Capture a signal via IPC
 */
async function captureSignal(): Promise<void> {
  const rawText = rawTextEl.value.trim();

  if (!rawText) {
    showStatus("Please enter feedback text", "error");
    return;
  }

  if (isCapturing) {
    return;
  }

  isCapturing = true;
  captureBtn.disabled = true;
  showStatus("Capturing...", "");

  try {
    const result = await invoke<CaptureResult>("capture_signal", {
      source: sourceEl.value,
      rawText: rawText,
      appKey: appKeyEl.value || null,
    });

    showStatus(`Captured: ${result.signal_id}`, "success");
    flashSuccess();

    // Clear form after brief delay
    setTimeout(() => {
      rawTextEl.value = "";
      appKeyEl.value = "";
      showStatus("", "");

      // Optionally hide window after capture
      // win.hide();
    }, 1000);
  } catch (error) {
    showStatus(`Error: ${formatIpcError(error)}`, "error");
    flashError();
  } finally {
    isCapturing = false;
    captureBtn.disabled = false;
  }
}

/**
 * Show status message
 */
function showStatus(message: string, type: string): void {
  statusEl.textContent = message;
  statusEl.className = `status ${type}`;
}

/**
 * Flash success animation
 */
function flashSuccess(): void {
  captureContainer.classList.add("captured");
  setTimeout(() => {
    captureContainer.classList.remove("captured");
  }, 300);
}

/**
 * Flash error animation
 */
function flashError(): void {
  captureContainer.classList.add("error");
  setTimeout(() => {
    captureContainer.classList.remove("error");
  }, 300);
}

// Event listeners
captureBtn.addEventListener("click", () => {
  captureSignal().catch(console.error);
});

rawTextEl.addEventListener("keydown", (e: KeyboardEvent) => {
  // Ctrl+Enter to capture
  if (e.key === "Enter" && e.ctrlKey) {
    e.preventDefault();
    captureSignal().catch(console.error);
  }

  // Escape to hide window
  if (e.key === "Escape") {
    win.hide().catch(console.error);
  }
});

// Focus textarea on window show
win.onFocusChanged(({ payload: focused }) => {
  if (focused) {
    rawTextEl.focus();
  }
}).catch(console.error);

// Initial focus
rawTextEl.focus();

console.log("ZFSS frontend initialized");
