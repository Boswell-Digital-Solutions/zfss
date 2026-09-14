import assert from "node:assert/strict";
import test from "node:test";
import {
  IPC_ERROR_CODES,
  formatIpcError,
  parseIpcError,
} from "./ipc-error.ts";

test("accepts every admitted backend error code", () => {
  for (const code of IPC_ERROR_CODES) {
    assert.deepEqual(parseIpcError({ code, message: "safe message" }), {
      code,
      message: "safe message",
    });
  }
});

test("formats a structured error for display and support", () => {
  assert.equal(
    formatIpcError({ code: "ZFSS_FORBIDDEN", message: "role cannot close issues" }),
    "role cannot close issues (ZFSS_FORBIDDEN)"
  );
});

test("fails closed without rendering malformed error content", () => {
  const malformed = [
    "postgres://admin:secret@private-host/zfss",
    new Error("credential-bearing internal failure"),
    { code: "UNKNOWN", message: "private database host" },
    { code: "ZFSS_INTERNAL", message: "" },
    null,
  ];

  for (const error of malformed) {
    assert.equal(
      formatIpcError(error),
      "An unexpected application error occurred. (ZFSS_INTERNAL)"
    );
  }
});
