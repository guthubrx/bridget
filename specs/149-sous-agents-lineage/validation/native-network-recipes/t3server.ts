// Serveur T3 COMPLET réel (apps/server/src/bin.ts) sur base privée + client RPC WebSocket réel.
// Adapté de ui-recipes/run_recipe_server.sh et ui-recipes/rpc149.ts (copie d'outil, adaptations :
// ids de fil choisis par la recette, daemon Bridget natif réel au lieu du CLI fixture).
import * as ChildProcess from "node:child_process";
import * as Crypto from "node:crypto";
import * as FS from "node:fs";
import * as Net from "node:net";
import * as Path from "node:path";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as Stream from "effect/Stream";
import { Socket } from "effect/socket";
import { RpcClient, RpcSerialization } from "effect/rpc";
import { ORCHESTRATION_PROTOCOL_VERSION, ORCHESTRATION_PROTOCOL_HEADER, ProjectId, ThreadId, WsRpcGroup } from "@t3tools/contracts";
import { BIN, pause } from "./fx.mjs";

export const T3_WT = "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage";
export const NODE_BIN = "/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin";

export interface T3Server {
  origin: string;
  home: string;
  pid: number;
  tokens: Record<string, string>;
  stop(): Promise<string>;
  http(token: string | null, method: string, path: string, body?: unknown): Promise<{ status: number; text: string }>;
  call(token: string, method: string, payload: unknown, opts?: { take?: number; waitMs?: number }): Promise<any>;
  dispatch(token: string, command: unknown): Promise<any>;
  sql(query: string): any[];
  cli(args: string[]): string;
}

const describeError = (e: unknown) => {
  const x = e as any;
  return { tag: x?._tag ?? x?.name ?? "error", code: x?.code, message: typeof x?.message === "string" ? x.message.slice(0, 200) : undefined, raw: x?._tag ? undefined : String(e).slice(0, 200) };
};

export async function startT3Server(fx: any, port: number, opts: { pathPrefix?: string; extraEnv?: Record<string, string>; allowedOrigins?: string } = {}): Promise<T3Server> {
  const home = Path.join(fx.root, "t3home");
  FS.mkdirSync(home, { recursive: true, mode: 0o700 });
  const env = {
    PATH: `${opts.pathPrefix ? `${opts.pathPrefix}:` : ""}${NODE_BIN}:/usr/bin:/bin:/usr/sbin:/sbin`,
    ...(opts.extraEnv ?? {}),
    HOME: Path.join(fx.root, "t3-user"),
    TMPDIR: Path.join(fx.root, "tmp"),
    T3CODE_PORT: String(port),
    T3CODE_HOME: home,
    T3CODE_MODE: "web",
    T3CODE_NO_BROWSER: "1",
    T3CODE_DEV_AUTH_TOKEN: Crypto.randomBytes(32).toString("hex"),
    T3CODE_DEV_ALLOWED_ORIGINS: opts.allowedOrigins ?? `http://localhost:${port}`,
    T3CODE_BRIDGET_EXECUTABLE: BIN,
    // Le lecteur lance la CLI Bridget avec cet environnement : daemon de la fixture uniquement.
    BRIDGET_HOME: fx.state,
    BRIDGET_SOCKET: fx.socket,
    HOSTNAME: "native149-isolated",
  };
  FS.mkdirSync(env.HOME, { recursive: true, mode: 0o700 });
  const logPath = Path.join(fx.root, "logs", "t3-server.log");
  const fd = FS.openSync(logPath, "a", 0o600);
  const child = ChildProcess.spawn(`${NODE_BIN}/node`, [Path.join(T3_WT, "apps/server/src/bin.ts")], { env, cwd: fx.tmpRoot, stdio: ["ignore", fd, fd] });
  child.on("error", () => undefined);
  const origin = `http://127.0.0.1:${port}`;
  const until = Date.now() + 90000;
  for (;;) {
    // Prêt = port TCP en écoute (le GET / peut rester sans réponse tant que le client web n'est pas servi).
    const up = await new Promise<boolean>((resolve) => {
      const c = Net.connect(port, "127.0.0.1");
      c.once("connect", () => { c.destroy(); resolve(true); });
      c.once("error", () => resolve(false));
    });
    if (up) break;
    if (child.exitCode !== null || Date.now() >= until) throw new Error(`serveur T3 indisponible (log ${logPath})`);
    await pause(250);
  }
  const issue = (scopes: string[]) =>
    ChildProcess.execFileSync(`${NODE_BIN}/node`, [Path.join(T3_WT, "apps/server/src/bin.ts"), "auth", "session", "issue", "--ttl", "2h", "--token-only", ...scopes.flatMap((s) => ["--scope", s])], { env, encoding: "utf8", cwd: fx.tmpRoot }).trim();
  const tokens = {
    operate: issue(["orchestration:read", "orchestration:operate"]),
    read: issue(["orchestration:read"]),
  };
  const http = async (token: string | null, method: string, path: string, body?: unknown) => {
    const headers: Record<string, string> = { "Content-Type": "application/json", [ORCHESTRATION_PROTOCOL_HEADER]: String(ORCHESTRATION_PROTOCOL_VERSION) };
    if (token) headers.Authorization = `Bearer ${token}`;
    const r = await fetch(`${origin}${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
    return { status: r.status, text: await r.text() };
  };
  const withClient = async <A>(token: string, f: (client: any) => Effect.Effect<A, unknown, never>) => {
    const t = await http(token, "POST", "/api/auth/websocket-ticket");
    if (t.status !== 200) return { transport_error: `ticket_http_${t.status}` } as any;
    const ticket = (JSON.parse(t.text) as { ticket: string }).ticket;
    const layerSocket = Socket.layerWebSocket(`${origin.replace("http:", "ws:")}/ws?orchestrationProtocol=${ORCHESTRATION_PROTOCOL_VERSION}&wsTicket=${encodeURIComponent(ticket)}`).pipe(Layer.provide(Socket.layerWebSocketConstructorGlobal));
    const layerProtocol = RpcClient.layerProtocolSocket().pipe(Layer.provide(layerSocket), Layer.provide(RpcSerialization.layerJson));
    const program = Effect.gen(function* () {
      const client = yield* RpcClient.make(WsRpcGroup);
      return yield* f(client);
    }).pipe(Effect.scoped, Effect.provide(layerProtocol));
    return Effect.runPromise(program as Effect.Effect<A, never, never>);
  };
  const STREAMS = new Set(["bridget.lineage.watch", "bridget.lineage.journal", "bridget.watch"]);
  const server: T3Server = {
    origin, home, pid: child.pid!, tokens,
    http,
    call: (token, method, payload, opts = {}) =>
      withClient(token, (client) => {
        const fn = client[method];
        if (typeof fn !== "function") return Effect.succeed({ unknown_method: method });
        if (STREAMS.has(method)) {
          const items: unknown[] = [];
          return (fn(payload) as Stream.Stream<unknown, unknown>).pipe(
            Stream.take(opts.take ?? 1),
            Stream.runForEach((i) => Effect.sync(() => void items.push(i))),
            Effect.timeoutOption(`${opts.waitMs ?? 4000} millis`),
            Effect.map((o) => ({ items, completed: o._tag === "Some" })),
            Effect.catch((e: unknown) => Effect.succeed({ items, error: describeError(e) })),
          );
        }
        return (fn(payload) as Effect.Effect<unknown, unknown>).pipe(
          Effect.map((value) => ({ value })),
          Effect.catch((e: unknown) => Effect.succeed({ error: describeError(e) })),
        );
      }),
    dispatch: (token, command) =>
      withClient(token, (client) =>
        client["orchestration.dispatchCommand"](command).pipe(
          Effect.map((r: unknown) => ({ ok: r })),
          Effect.catch((e: unknown) => Effect.succeed({ error: describeError(e) })),
        ),
      ),
    cli: (args) => ChildProcess.execFileSync(`${NODE_BIN}/node`, [Path.join(T3_WT, "apps/server/src/bin.ts"), ...args], { env, encoding: "utf8", cwd: fx.tmpRoot }).trim(),
    sql: (query) => {
      const out = ChildProcess.execFileSync("sqlite3", ["-readonly", "-json", Path.join(home, "userdata/statev2.sqlite"), query], { encoding: "utf8" });
      return out.trim() ? JSON.parse(out) : [];
    },
    stop: async () => {
      if (child.exitCode !== null) return "already_gone";
      const ps = ChildProcess.execFileSync("/bin/ps", ["-p", String(child.pid), "-o", "ppid=", "-o", "command="], { encoding: "utf8" }).trim();
      if (Number(ps.split(/\s+/)[0]) !== process.pid || !ps.includes("apps/server/src/bin.ts") || /firefox/i.test(ps)) throw new Error("refus d'arrêter un processus non vérifié");
      child.kill("SIGTERM");
      const stopUntil = Date.now() + 15000;
      while (child.exitCode === null && child.signalCode === null) {
        if (Date.now() >= stopUntil) throw new Error("SIGTERM sans effet sur le serveur T3");
        await pause(50);
      }
      return "stopped_by_sigterm";
    },
  };
  return server;
}

export async function createProjectAndThreads(server: T3Server, workspaceRoot: string, threads: Array<{ id: string; title: string }>) {
  const projectId = ProjectId.make(Crypto.randomUUID());
  const created = await server.http(server.tokens.operate, "POST", "/api/projects/mutate", {
    type: "project.create", commandId: Crypto.randomUUID(), projectId, title: "recette149-native", workspaceRoot,
  });
  const results: any[] = [{ step: "project.create", status: created.status }];
  for (const t of threads) {
    results.push(await server.dispatch(server.tokens.operate, {
      type: "thread.create", commandId: Crypto.randomUUID(), threadId: ThreadId.make(t.id), projectId, title: t.title,
      createdBy: "user", creationSource: "web", modelSelection: { instanceId: "codex", model: "gpt-5" },
      runtimeMode: "full-access", interactionMode: "default", branch: null, worktreePath: null,
    }));
  }
  return { projectId, created, results };
}
