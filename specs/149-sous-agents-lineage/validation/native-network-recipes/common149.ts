// Aides communes T036/T039 (aucune logique de production, aucun faux serveur).
import * as Crypto from "node:crypto";
import { pause } from "./fx.mjs";

export const THREAD_A = "89000000-0000-4000-8000-000000000a01";
export const THREAD_B = "89000000-0000-4000-8000-000000000a02";
export const THREAD_C = "89000000-0000-4000-8000-000000000a03";
export const THREAD_D = "89000000-0000-4000-8000-000000000a04";

const mcpHeaders = (config: any, session?: string) => ({
  authorization: config.authorizationHeader,
  accept: "application/json, text/event-stream",
  "content-type": "application/json",
  "mcp-protocol-version": "2025-06-18",
  ...(session ? { "mcp-session-id": session } : {}),
});

/** Appel HTTP MCP direct à T3 (la vue d'un client quelconque qui détient ce credential). */
export async function rawTool(config: any, name: string, args: unknown = {}) {
  const url = config.endpoint;
  const init = await fetch(url, {
    method: "POST",
    headers: mcpHeaders(config),
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "native149-raw", version: "1" } } }),
  });
  const initText = await init.text();
  if (init.status !== 200) return { httpStatus: init.status, initBody: initText.slice(0, 200) };
  const sid = init.headers.get("mcp-session-id")!;
  await (await fetch(url, { method: "POST", headers: mcpHeaders(config, sid), body: JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) })).text();
  const response = await fetch(url, { method: "POST", headers: mcpHeaders(config, sid), body: JSON.stringify({ jsonrpc: "2.0", id: 2, method: "tools/call", params: { name, arguments: args } }) });
  const text = await response.text();
  await fetch(url, { method: "DELETE", headers: mcpHeaders(config, sid) });
  let body: any;
  try { body = JSON.parse(text.startsWith("event:") ? text.split("\n").find((l) => l.startsWith("data: "))!.slice(6) : text); } catch { body = { unparsed: text.slice(0, 200) }; }
  const result = body?.result;
  // Les refus d'outil T3 portent leur code nommé dans le texte JSON de l'échec.
  let namedCode: string | undefined;
  if (result?.isError === true) {
    try { namedCode = JSON.parse(result?.content?.[0]?.text ?? "{}").code; } catch { namedCode = undefined; }
  }
  return {
    namedCode,
    httpStatus: response.status,
    bytes: text.length,
    isError: result?.isError === true,
    structured: result?.structuredContent,
    code: result?.code,
    error: body?.error,
    text: result?.content?.[0]?.text,
  };
}

export const rawSession = (config: any) => rawTool(config, "bridget_session");

/** Statut d'une requête MCP dont on ne veut que le code HTTP (credential invalide). */
export async function rawStatus(config: any): Promise<number> {
  const response = await fetch(config.endpoint, {
    method: "POST",
    headers: mcpHeaders(config),
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "native149-raw", version: "1" } } }),
  });
  await response.text();
  const sid = response.headers.get("mcp-session-id");
  if (sid) await fetch(config.endpoint, { method: "DELETE", headers: mcpHeaders(config, sid) });
  return response.status;
}

export const readOnlyFact = (config: any, runId: string, cwd: string, over: Record<string, unknown> = {}) => ({
  version: 1,
  source: "provider_turn",
  run_id: runId,
  provider_session_id: config.providerSessionId,
  provider_instance_id: config.providerInstanceId,
  driver: "codex_app_server",
  cwd,
  runtime_mode: "approval-required",
  interaction_mode: "default",
  provider_policy: {
    kind: "codex",
    approval_policy: "on-request",
    approvals_reviewer: "user",
    sandbox_policy: { type: "readOnly" },
  },
  ...over,
});

export async function waitTask(fx: any, taskId: string, states: string[], ms = 30000) {
  const until = Date.now() + ms;
  for (;;) {
    const task = fx.tasks().find((t: any) => t.task_id === taskId);
    if (task && states.includes(task.state)) return task;
    if (Date.now() >= until) return task;
    await pause(50);
  }
}

export const counts = (fx: any, peers: Record<string, any> = {}) => {
  const evidence = fx.evidence();
  return {
    rows: fx.tasks().length,
    started: evidence.filter((e: any) => e.event === "started").length,
    prompts: evidence.filter((e: any) => e.event === "prompt").length,
    answered: evidence.filter((e: any) => e.event === "answered").length,
    deliveries: Object.fromEntries(Object.entries(peers).map(([k, p]: any) => [k, p.deliveries.length])),
  };
};

export const sha = (value: unknown) => Crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex").slice(0, 16);

/** Attend que toutes les tâches soient terminales, que les remises soient livrées et stables. */
export async function settle(fx: any, peers: any[], quietMs = 1500, ms = 30000) {
  const terminal = ["result_available", "failed", "cancelled"];
  const until = Date.now() + ms;
  let last = "";
  let stableSince = Date.now();
  for (;;) {
    const tasks = fx.tasks();
    const signature = JSON.stringify([tasks.map((t: any) => [t.task_id, t.state, t.result_sent]), peers.map((p) => p.deliveries.length)]);
    if (signature !== last) { last = signature; stableSince = Date.now(); }
    if (tasks.every((t: any) => terminal.includes(t.state)) && Date.now() - stableSince >= quietMs) return true;
    if (Date.now() >= until) return false;
    await pause(100);
  }
}
