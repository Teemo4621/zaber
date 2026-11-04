import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t } from "$lib/i18n.svelte";
import { settings } from "$lib/settings.svelte";
import { cheerCompanion } from "$lib/notch/companionCue.svelte";
import { dismissPeek, peeks, showPeek, type Peek } from "$lib/notch/peeks.svelte";

export type Field = { name: string; title: string; kind: "string" | "number" | "boolean" | "enum"; options: string[]; required: boolean };
export type Request = {
  id: number;
  kind: "permission" | "question";
  project: string;
  session: string;
  tool: string | null;
  summary: string;
  message: string | null;
  fields: Field[];
  canAlways: boolean;
};
export type Answer =
  | { kind: "allow"; always: boolean }
  | { kind: "deny" }
  | { kind: "accept"; content: Record<string, string | number | boolean> }
  | { kind: "decline" }
  | { kind: "pass" };
type Note = { project: string; session: string; text: string };

export type Session = { id: string; project: string; state: "working" | "waiting" | "done"; detail: string; since: number; seen: number };

export type Limit = { used: number; resetsAt: number };
export type Usage = { session: Limit | null; week: Limit | null };

export const claude = $state<{ queue: Request[]; sessions: Session[]; usage: Usage | null }>({ queue: [], sessions: [], usage: null });

invoke<Usage | null>("claude_usage").then((u) => {
  claude.usage ??= u;
});
listen<Usage>("claude:usage", ({ payload }) => (claude.usage = payload));

const DONE_KEEP = 10 * 60_000;
const STALE = 30 * 60_000;
let staleTimer: ReturnType<typeof setTimeout> | undefined;

function track(n: { session: string; project: string }, state: Session["state"], detail = "") {
  const now = Date.now();
  const old = claude.sessions.find((s) => s.id === n.session);
  const since = old && old.state !== "done" && state !== "done" ? old.since : now;
  const next: Session = { id: n.session, project: n.project, state, detail: detail || (state === old?.state ? old.detail : ""), since, seen: now };
  claude.sessions = [next, ...claude.sessions.filter((s) => s.id !== n.session)];
  prune();
}

export function prune() {
  const now = Date.now();
  const keep = (s: Session) => (s.state === "done" ? now - s.since < DONE_KEEP : s.state === "waiting" || now - s.seen < STALE);
  if (!claude.sessions.every(keep)) claude.sessions = claude.sessions.filter(keep);
  clearTimeout(staleTimer);
  const next = Math.min(...claude.sessions.filter((s) => s.state === "working").map((s) => s.seen + STALE));
  if (Number.isFinite(next)) staleTimer = setTimeout(prune, next - now + 1000);
}

export const busy = () => claude.sessions.filter((s) => s.state !== "done");

listen<Note>("claude:working", ({ payload }) => track(payload, "working", payload.text));
listen<Note>("claude:ended", ({ payload }) => (claude.sessions = claude.sessions.filter((s) => s.id !== payload.session)));

function note(n: Omit<Extract<Peek, { kind: "claudeNote" }>, "kind">, seconds: number | null) {
  if (seconds) showPeek({ kind: "claudeNote", ...n }, seconds);
}

function showHead() {
  if (claude.queue.length) showPeek({ kind: "claude", count: claude.queue.length }, Math.max(settings.value.peekSeconds, 8));
  else dismissPeek("claude");
}

listen<Request>("claude:request", ({ payload }) => {
  claude.queue.push(payload);
  track(payload, "waiting", heading(payload));
  showHead();
});

listen<number>("claude:expired", ({ payload: id }) => {
  const r = claude.queue.find((q) => q.id === id);
  claude.queue = claude.queue.filter((q) => q.id !== id);
  showHead();
  if (r) track(r, "working");
  if (r) note({ tone: "handed", title: t("claude.handedBack"), sub: r.project }, claude.queue.length ? null : 4);
});

listen<Note>("claude:done", ({ payload }) => {
  track(payload, "done", payload.text);
  cheerCompanion();
  note({ tone: "done", title: t("claude.done", { project: payload.project }), sub: payload.text }, claude.queue.length ? null : 5);
});

listen<Note>("claude:waiting", ({ payload }) => {
  const old = claude.sessions.find((s) => s.id === payload.session);
  if (old?.state !== "done") track(payload, "done", old?.detail);
  const quiet = claude.queue.length || peeks.current?.kind === "claude";
  note({ tone: "waiting", title: t("claude.waiting"), sub: payload.project }, quiet ? null : 5);
});

export async function answer(r: Request, a: Answer) {
  claude.queue = claude.queue.filter((q) => q.id !== r.id);
  track(r, "working");
  try {
    await invoke("claude_answer", { id: r.id, answer: a });
  } catch {
  }
  if (a.kind === "pass") {
    if (claude.queue.length) showHead();
    else dismissPeek("claude");
    return;
  }
  const ok = a.kind === "allow" || a.kind === "accept";
  note({ tone: ok ? "ok" : "no", title: ok ? t("claude.allowed") : t("claude.denied"), sub: `${heading(r)} · ${r.project}` }, claude.queue.length ? null : 2);
  if (claude.queue.length) showHead();
}

export function heading(r: Request) {
  if (r.kind === "question") return r.message || t("claude.question");
  return t("claude.allowTool", { tool: r.tool ?? "?", project: r.project });
}
