// Recette 149 - client RPC WebSocket réel (même pile Effect que le serveur T3).
// Outil de TEST : aucune logique de production, aucun faux serveur. Il parle au
// vrai serveur T3 de recette (127.0.0.1:14773) avec un jeton bearer du cache privé.
//
// Usage (Node 24, cwd = n'importe où ; ui-recipes/node_modules -> deps partagées du WT) :
//   node rpc149.ts setup <tokfile-operate>
//   node rpc149.ts call <tokfile> <méthode> '<json payload>' [--take N] [--wait-ms M]
//   node rpc149.ts http <tokfile|none> <GET|POST> <chemin> ['<json>']
// Sortie : une ligne JSON par résultat/erreur. Jamais le jeton.
import * as NodeCrypto from "node:crypto";
import * as NodeFS from "node:fs";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as Stream from "effect/Stream";
import { Socket } from "effect/socket";
import { RpcClient, RpcSerialization } from "effect/rpc";
import {
  ORCHESTRATION_PROTOCOL_VERSION,
  ORCHESTRATION_PROTOCOL_HEADER,
  ProjectId,
  ThreadId,
  WsRpcGroup,
} from "@t3tools/contracts";

const CACHE = process.env.RECIPE149_CACHE ?? "/Users/moi/.cache/bridget149-ui";
const ORIGIN = process.env.RECIPE149_ORIGIN ?? `http://127.0.0.1:${process.env.RECIPE149_T3_PORT ?? 14773}`;
const STREAM_METHODS = new Set(["bridget.lineage.watch", "bridget.lineage.journal", "bridget.watch"]);

const [, , mode, tokArg, ...rest] = process.argv;
const readToken = (p: string) => (p === "none" ? null : NodeFS.readFileSync(p, "utf8").trim());
const out = (o: unknown) => process.stdout.write(JSON.stringify(o) + "\n");
const flag = (name: string, def: number) => {
  const i = rest.indexOf(name);
  if (i < 0) return def;
  const v = Number(rest[i + 1]);
  rest.splice(i, 2);
  return v;
};

async function http(token: string | null, method: string, path: string, body?: unknown) {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    [ORCHESTRATION_PROTOCOL_HEADER]: String(ORCHESTRATION_PROTOCOL_VERSION),
  };
  if (token) headers.Authorization = `Bearer ${token}`;
  const r = await fetch(`${ORIGIN}${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  const text = await r.text();
  return { status: r.status, text };
}

async function wsTicket(token: string) {
  const r = await http(token, "POST", "/api/auth/websocket-ticket");
  if (r.status !== 200) return { error: `ticket_http_${r.status}`, body: r.text.slice(0, 300) } as const;
  return { ticket: (JSON.parse(r.text) as { ticket: string }).ticket } as const;
}

async function withClient<A>(token: string, f: (client: any) => Effect.Effect<A, unknown, never>) {
  const t = await wsTicket(token);
  if ("error" in t) return { transport_error: t.error, body: t.body };
  const layerSocket = Socket.layerWebSocket(
    `${ORIGIN.replace("http:", "ws:")}/ws?orchestrationProtocol=${ORCHESTRATION_PROTOCOL_VERSION}&wsTicket=${encodeURIComponent(t.ticket)}`,
  ).pipe(Layer.provide(Socket.layerWebSocketConstructorGlobal));
  const layerProtocol = RpcClient.layerProtocolSocket().pipe(
    Layer.provide(layerSocket),
    Layer.provide(RpcSerialization.layerJson),
  );
  const program = Effect.gen(function* () {
    const client = yield* RpcClient.make(WsRpcGroup);
    return yield* f(client);
  }).pipe(Effect.scoped, Effect.provide(layerProtocol));
  return Effect.runPromise(program as Effect.Effect<A, never, never>);
}

const describeError = (e: unknown) => {
  const x = e as any;
  return { tag: x?._tag ?? x?.name ?? "error", code: x?.code, message: typeof x?.message === "string" ? x.message.slice(0, 300) : undefined, missingScope: x?.scope ?? x?.requiredScope, raw: x?._tag ? undefined : String(e).slice(0, 300) };
};

async function main() {
  if (mode === "http") {
    const [method, path, body] = rest;
    const r = await http(readToken(tokArg), method, path, body ? JSON.parse(body) : undefined);
    out({ status: r.status, body: r.text.slice(0, 600) });
    return;
  }
  const token = readToken(tokArg);
  if (token === null) throw new Error("jeton requis");
  if (mode === "raw") {
    // Trame brute : le serveur reçoit EXACTEMENT la charge fournie (pas d'encodage client).
    const waitMs = flag("--wait-ms", 4000);
    const [method, json] = rest;
    const t = await wsTicket(token);
    if ("error" in t) { out({ transport_error: t.error, body: t.body }); return; }
    const url = `${ORIGIN.replace("http:", "ws:")}/ws?orchestrationProtocol=${ORCHESTRATION_PROTOCOL_VERSION}&wsTicket=${encodeURIComponent(t.ticket)}`;
    const frames: unknown[] = [];
    await new Promise<void>((done) => {
      const ws = new WebSocket(url);
      const stop = setTimeout(() => { ws.close(); done(); }, waitMs);
      ws.onopen = () => ws.send(JSON.stringify({ _tag: "Request", id: "1", tag: method, payload: JSON.parse(json ?? "{}"), headers: [] }));
      ws.onmessage = (m) => {
        const parsed = JSON.parse(String(m.data));
        for (const f of Array.isArray(parsed) ? parsed : [parsed]) {
          frames.push(f);
          if ((f as any)._tag === "Exit") { clearTimeout(stop); ws.close(); done(); }
        }
      };
      ws.onerror = () => { frames.push({ _tag: "socket_error" }); };
    });
    const exit = frames.find((f: any) => f._tag === "Exit") as any;
    const failure = exit?.exit?._tag === "Failure" ? exit.exit.cause : undefined;
    out({ exit: exit?.exit?._tag ?? null, causeText: failure ? JSON.stringify(failure).slice(0, 500) : undefined, value: exit?.exit?._tag === "Success" ? "success" : undefined, frames: frames.length });
    return;
  }
  if (mode === "setup" || mode === "setup2") {
    // setup  : projet recette149-proj + fil hôte 00000000-0000-4000-8000-000000000101 (ids.json)
    // setup2 : second projet étranger (root différent du magasin) + fil aléatoire (ids2.json)
    const second = mode === "setup2";
    const projectId = ProjectId.make(NodeCrypto.randomUUID());
    const threadId = ThreadId.make(second ? NodeCrypto.randomUUID() : "00000000-0000-4000-8000-000000000101");
    const created = await http(token, "POST", "/api/projects/mutate", {
      type: "project.create", commandId: NodeCrypto.randomUUID(), projectId,
      title: second ? "recette149-proj2" : "recette149-proj", workspaceRoot: `${CACHE}/${second ? "proj2" : "proj"}`,
    });
    out({ step: "project.create", status: created.status, body: created.text.slice(0, 200) });
    if (created.status !== 200) return;
    const dispatched = await withClient(token, (client) =>
      client["orchestration.dispatchCommand"]({
        type: "thread.create", commandId: NodeCrypto.randomUUID(), threadId, projectId,
        title: second ? "Fil étranger recette 149" : "Fil hôte recette 149", createdBy: "user", creationSource: "web",
        modelSelection: { instanceId: "codex", model: "gpt-5" },
        runtimeMode: "full-access", interactionMode: "default", branch: null, worktreePath: null,
      }).pipe(Effect.map((r: unknown) => ({ ok: r })), Effect.catch((e: unknown) => Effect.succeed({ error: describeError(e) }))),
    );
    out({ step: "thread.create", result: dispatched });
    NodeFS.writeFileSync(`${CACHE}/${second ? "ids2.json" : "ids.json"}`,
      JSON.stringify(second ? { projectId2: projectId, threadId2: threadId } : { projectId, threadId }, null, 2), { mode: 0o600 });
    return;
  }
  if (mode === "call") {
    const repeat = flag("--repeat", 1);
    const take = flag("--take", 1);
    const waitMs = flag("--wait-ms", 4000);
    const [method, json] = rest;
    const payload = json ? JSON.parse(json) : {};
    const res = await withClient(token, (client) => {
      const fn = client[method];
      if (typeof fn !== "function") return Effect.succeed({ unknown_method: method });
      if (STREAM_METHODS.has(method)) {
        const items: unknown[] = [];
        return (fn(payload) as Stream.Stream<unknown, unknown>).pipe(
          Stream.take(take),
          Stream.runForEach((i) => Effect.sync(() => void items.push(i))),
          Effect.timeoutOption(`${waitMs} millis`),
          Effect.map((o) => ({ items, completed: o._tag === "Some" })),
          Effect.catch((e: unknown) => Effect.succeed({ items, error: describeError(e) })),
        );
      }
      const once = (fn(payload) as Effect.Effect<unknown, unknown>).pipe(
        Effect.map((value) => ({ value })),
        Effect.catch((e: unknown) => Effect.succeed({ error: describeError(e) })),
      );
      if (repeat <= 1) return once;
      // N appels séquentiels sur la même connexion : compte les succès et les codes d'erreur
      return Effect.gen(function* () {
        let ok = 0;
        const errors: Record<string, number> = {};
        for (let i = 0; i < repeat; i++) {
          const r: any = yield* once;
          if (r.value) ok++; else errors[String(r.error?.code ?? r.error?.tag)] = (errors[String(r.error?.code ?? r.error?.tag)] ?? 0) + 1;
        }
        return { repeat, ok, errors };
      });
    });
    out(res);
    return;
  }
  throw new Error(`mode inconnu ${mode}`);
}

// Vider stdout avant de quitter : process.exit tronque un pipe au-delà de 64 Ko (constat r2).
const flushExit = (code: number) => process.stdout.write("", () => process.exit(code));
main().then(() => flushExit(0), (e) => { out({ fatal: String(e?.stack ?? e).slice(0, 800) }); flushExit(1); });
