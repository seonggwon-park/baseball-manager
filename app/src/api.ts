// One entry point to the engine: Tauri commands in the desktop app, HTTP to the
// development server (`bm serve`) in a browser.

import type { AdvanceRequest, AdvanceView, Delegate, MatchState, MatchView, NewMatch, SubRequest } from "./types";

export class ApiError extends Error {
  code: string;
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
  }
}

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, payload?: unknown): Promise<T> {
  if (inTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    try {
      return await invoke<T>("api", { cmd, payload: payload ?? null });
    } catch (e) {
      const err = e as { code?: string; message?: string };
      throw new ApiError(err.code ?? "internal", err.message ?? String(e));
    }
  }
  const res = await fetch(`/api/${cmd}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload ?? {}),
  });
  const data = await res.json();
  if (!res.ok) {
    throw new ApiError(data.code ?? "internal", data.message ?? res.statusText);
  }
  return data as T;
}

export const api = {
  newMatch: (cfg: NewMatch) => call<{ view: MatchView; state: MatchState }>("new_match", cfg),
  advance: (req: AdvanceRequest) => call<AdvanceView>("advance", req),
  substitute: (req: SubRequest) => call<MatchState>("substitute", req),
  delegate: (d: Delegate) => call<MatchState>("delegate", d),
  state: () => call<MatchState>("state"),
};
