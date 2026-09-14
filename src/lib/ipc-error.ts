export const IPC_ERROR_CODES = [
  "ZFSS_VALIDATION",
  "ZFSS_FORBIDDEN",
  "ZFSS_IDENTITY_UNAVAILABLE",
  "ZFSS_NOT_FOUND",
  "ZFSS_CONFLICT",
  "ZFSS_REPOSITORY_UNAVAILABLE",
  "ZFSS_INTERNAL",
] as const;

export type IpcErrorCode = (typeof IPC_ERROR_CODES)[number];

export interface IpcError {
  code: IpcErrorCode;
  message: string;
}

const IPC_ERROR_CODE_SET = new Set<string>(IPC_ERROR_CODES);
const FALLBACK_ERROR: IpcError = {
  code: "ZFSS_INTERNAL",
  message: "An unexpected application error occurred.",
};

export function parseIpcError(error: unknown): IpcError {
  if (typeof error !== "object" || error === null || Array.isArray(error)) {
    return { ...FALLBACK_ERROR };
  }

  const candidate = error as Record<string, unknown>;
  if (
    typeof candidate.code !== "string" ||
    !IPC_ERROR_CODE_SET.has(candidate.code) ||
    typeof candidate.message !== "string" ||
    candidate.message.trim().length === 0
  ) {
    return { ...FALLBACK_ERROR };
  }

  return {
    code: candidate.code as IpcErrorCode,
    message: candidate.message,
  };
}

export function formatIpcError(error: unknown): string {
  const parsed = parseIpcError(error);
  return `${parsed.message} (${parsed.code})`;
}
