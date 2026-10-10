// Recette native 149 - petit outillage de fixture (T036/T039). Pas de framework, pas de dépendance.
// Adapté de BridgetRustInterop.testkit.ts (R4 148) : mêmes principes (vrai daemon, credentials
// délivrés par le vrai Register, jamais forgés ; arrêt SIGTERM d'un PID vérifié ; aucun -9).
import * as ChildProcess from "node:child_process";
import * as Crypto from "node:crypto";
import * as FS from "node:fs";
import * as FSP from "node:fs/promises";
import * as Net from "node:net";
import * as Path from "node:path";
import * as Readline from "node:readline";
import * as Util from "node:util";

const exec = Util.promisify(ChildProcess.execFile);
export const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

// Binaire sous test (ronde r3) : release r8 IMMUABLE bridget-0a29ad9b2cdb (reçu native149-release-receipt-r8.json).
// NATIVE149_BIN permet de rejouer sur un autre binaire (r2 : debug ec6b18b5d468) ; NATIVE149_BIN_SHA256 donne alors l'empreinte attendue.
const DEFAULT_BIN = "/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb";
const DEFAULT_SHA = "0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6";
export const BIN = process.env.NATIVE149_BIN ?? DEFAULT_BIN;
export const BIN_SHA256 = process.env.NATIVE149_BIN_SHA256 ?? (process.env.NATIVE149_BIN ? Crypto.createHash("sha256").update(FS.readFileSync(BIN)).digest("hex") : DEFAULT_SHA);
export const RESULTS_DIR = process.env.NATIVE149_RESULTS ?? "results-r3";
export const HERE = Path.dirname(new URL(import.meta.url).pathname);
export const PROVIDER = Path.join(HERE, "provider149.py");
export const CODEX_PROVIDER = Path.join(HERE, "codex149.py");

/** Réplique de t3code::stable_uuid (R4 : bridgetThreadUuid). */
export const stableUuid = (name) => {
  const namespace = Buffer.from("098b71d3c0de4a119b1d73636f646501", "hex");
  const bytes = Crypto.createHash("sha1").update(namespace).update(name).digest().subarray(0, 16);
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = bytes.toString("hex");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
};

export class Checks {
  constructor(title) {
    this.title = title;
    this.items = [];
  }
  /** layer : REEL | SIMULE ; la couche est celle de la PREUVE, pas du scénario. */
  add(id, layer, description, pass, observed) {
    this.items.push({ id, layer, description, pass: Boolean(pass), observed });
    process.stdout.write(`${pass ? "PASS" : "FAIL"} ${id} [${layer}] ${description}\n`);
    if (!pass) process.stdout.write(`     observé: ${JSON.stringify(observed)}\n`);
  }
  summary() {
    const pass = this.items.filter((i) => i.pass).length;
    return { title: this.title, pass, fail: this.items.length - pass, total: this.items.length };
  }
}

class JsonLines {
  constructor(stream) {
    this.values = [];
    this.waiters = [];
    this.error = undefined;
    Readline.createInterface({ input: stream }).on("line", (line) => {
      try {
        const value = JSON.parse(line);
        // delivery_generation est un u64 > 2^53 : JSON.parse l'arrondit. On garde les chiffres exacts.
        const generation = /"delivery_generation":(\d+)/.exec(line);
        if (generation) Object.defineProperty(value, "__generationExact", { value: generation[1], enumerable: false });
        this.values.push(value);
        for (const w of this.waiters.splice(0)) w();
      } catch {
        this.error = new Error("ligne de protocole non JSON");
      }
    });
  }
  async next(ms = 20000) {
    const until = Date.now() + ms;
    while (this.values.length === 0) {
      if (this.error) throw this.error;
      if (Date.now() >= until) throw new Error("réponse de protocole expirée");
      await pause(10);
    }
    return this.values.shift();
  }
}

/** Connexion Register à la socket du daemon (mêmes trames que le wrapper T3 réel). */
export class Peer {
  constructor(fx, threadId, connection, lines, agentId, instanceId) {
    this.fx = fx;
    this.threadId = threadId;
    this.connection = connection;
    this.agentId = agentId;
    this.instanceId = instanceId;
    this.received = [];
    this.deliveries = [];
    this.acks = [];
    this.ack = true;
    this.ackFilter = () => true; // r3 : ACK sélectif (ex. ne pas accuser une remise précise)
    this.ackDelayMs = 0;
    this.lines = lines;
    this.alive = true;
    connection.on("close", () => (this.alive = false));
    connection.on("error", () => (this.alive = false));
    this.pump();
  }
  async pump() {
    while (this.alive) {
      if (this.lines.values.length === 0) {
        await pause(5);
        continue;
      }
      const frame = this.lines.values.shift();
      this.received.push(frame);
      if (frame.type === "DeliverIdempotent") {
        this.deliveries.push({
          message_id: frame.message?.id,
          from: frame.message?.from,
          to: frame.message?.to,
          in_reply_to: frame.message?.in_reply_to ?? null,
          body: frame.message?.body,
          generation: frame.delivery_generation,
          delivery_id: frame.delivery_id,
        });
        if (this.ack && this.ackFilter(frame)) {
          const sendAck = () => {
            // Trame écrite à la main pour conserver les 19 chiffres exacts du u64.
            this.connection.write(`{"type":"DeliverAcked","delivery_id":${JSON.stringify(frame.delivery_id)},"delivery_generation":${frame.__generationExact}}\n`);
            this.acks.push(frame.delivery_id);
          };
          if (this.ackDelayMs > 0) setTimeout(sendAck, this.ackDelayMs);
          else sendAck();
        }
      } else if (frame.type === "Deliver" || frame.type === "DeliverExecution") {
        const m = frame.message ?? frame;
        this.deliveries.push({ message_id: m.id, from: m.from, to: m.to, in_reply_to: m.in_reply_to ?? null, body: m.body, plain: true });
      }
    }
  }
  send(value) {
    this.connection.write(`${JSON.stringify(value)}\n`);
  }
  async waitFor(predicate, ms = 15000) {
    const until = Date.now() + ms;
    for (;;) {
      const found = this.received.find(predicate);
      if (found) return found;
      if (Date.now() >= until) return undefined;
      await pause(10);
    }
  }
  close() {
    this.alive = false;
    this.connection.destroy();
  }
}

export class Fx {
  static async create(label, { port }) {
    process.umask(0o077);
    await FSP.mkdir("/Users/moi/.cache", { recursive: true });
    const root = await FSP.mkdtemp(`/Users/moi/.cache/bridget149-native-interop.`);
    await FSP.chmod(root, 0o700);
    const fx = new Fx(root, label, port);
    for (const sub of ["state", "provider", "tmp", "provider/.t3/userdata", "logs", "evidence"]) {
      await FSP.mkdir(Path.join(root, sub), { recursive: true, mode: 0o700 });
    }
    // Le projet vit hors de /Users/moi (qui est lui-même un dépôt git) : sinon la racine de
    // projet résolue serait /Users/moi/.git. Dépôt git privé minimal sous /tmp.
    const tmpRoot = await FSP.realpath(await FSP.mkdtemp("/tmp/b149n."));
    await FSP.chmod(tmpRoot, 0o700);
    fx.tmpRoot = tmpRoot;
    fx.work = Path.join(tmpRoot, "work");
    await FSP.mkdir(fx.work, { recursive: true, mode: 0o700 });
    ChildProcess.execFileSync("/usr/bin/git", ["init", "-q", fx.work], { env: { PATH: "/usr/bin:/bin", HOME: tmpRoot, GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_SYSTEM: "/dev/null" } });
    return fx;
  }
  constructor(root, label, port) {
    this.root = root;
    this.label = label;
    this.port = port;
    this.state = Path.join(root, "state");
    this.socket = Path.join(this.state, "bridget.sock");
    this.t3Home = Path.join(root, "provider/.t3"); // HOME/.t3 : le daemon ne reçoit jamais T3CODE_HOME (garde BillingGuard)
    this.evidencePath = Path.join(root, "evidence/provider.jsonl");
    this.children = [];
    this.peers = [];
    this.daemonStarts = 0;
  }
  env(extra = {}) {
    return {
      HOME: Path.join(this.root, "provider"),
      BRIDGET_HOME: this.state,
      BRIDGET_SOCKET: this.socket,
      TMPDIR: Path.join(this.root, "tmp"),
      PATH: "/usr/bin:/bin:/usr/sbin:/sbin",
      HOSTNAME: "native149-isolated",
      XDG_CACHE_HOME: Path.join(this.root, "provider/.cache"),
      XDG_CONFIG_HOME: Path.join(this.root, "provider/.config"),
      XDG_DATA_HOME: Path.join(this.root, "provider/.local/share"),
      XDG_STATE_HOME: Path.join(this.root, "provider/.local/state"),
      ...extra,
    };
  }
  /** Registre privé de recette : un seul agent fixture, aucun modèle, aucune clé. */
  async writeRegistry(agents = {}) {
    await FSP.writeFile(this.evidencePath, "", { mode: 0o600 });
    const entry = {
      command: "/usr/bin/python3",
      args: [PROVIDER, this.evidencePath],
      protocol: "claude_stream_json",
      permissions: "deny",
      queue_capacity: 4,
      notify_timeout_secs: 900,
      forbidden_env: ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "T3CODE_HOME", "BRIDGET_T3_MCP_ENDPOINT", "BRIDGET_T3_MCP_AUTHORIZATION"],
      pass_env: [],
      capabilities: { execution_paths: ["claude_stream_json"], models: { "fixture-model-149": { efforts: ["high"] } } },
      ...agents,
    };
    const codex = {
      ...entry,
      args: [process.env.NATIVE149_CODEX_PROVIDER ?? CODEX_PROVIDER, this.evidencePath, BIN],
      protocol: "codex_app_server",
      forbidden_env: ["OPENAI_API_KEY", "CODEX_API_KEY", "T3CODE_HOME", "BRIDGET_T3_MCP_ENDPOINT", "BRIDGET_T3_MCP_AUTHORIZATION"],
      capabilities: { execution_paths: ["codex_app_server"], models: { "fixture-model-149": { efforts: ["high"] }, "fixture-model-149-mini": { efforts: ["high"] } } },
    };
    await FSP.writeFile(Path.join(this.state, "agents.json"), JSON.stringify({ agents: { "fixture-claude-149": entry, "fixture-codex-149": codex } }), { mode: 0o600 });
  }
  async publishRuntime() {
    // Le pid publié est celui de la recette (vivant), comme le serveur T3 en production.
    await FSP.writeFile(
      Path.join(this.t3Home, "userdata/server-runtime.json"),
      JSON.stringify({ host: "127.0.0.1", port: this.port, pid: process.pid }),
      { mode: 0o600 },
    );
  }
  spawnOwned(args, env, { stdoutLog, stderrLog } = {}) {
    const out = stdoutLog ? FS.openSync(stdoutLog, "a", 0o600) : "ignore";
    const err = stderrLog ? FS.openSync(stderrLog, "a", 0o600) : "ignore";
    const child = ChildProcess.spawn(BIN, args, { env, cwd: this.root, stdio: ["pipe", stdoutLog ? out : "pipe", stderrLog ? err : "pipe"] });
    child.on("error", () => undefined);
    this.children.push(child);
    return child;
  }
  async startDaemon() {
    this.daemonStarts += 1;
    const log = Path.join(this.root, "logs", `daemon-${this.daemonStarts}.log`);
    const daemon = this.spawnOwned(["daemon"], this.env({ RUST_LOG: "info" }), { stdoutLog: log, stderrLog: log });
    this.daemon = daemon;
    const until = Date.now() + 15000;
    for (;;) {
      const ready = await new Promise((resolve) => {
        const c = Net.connect(this.socket);
        c.once("connect", () => { c.destroy(); resolve(true); });
        c.once("error", () => resolve(false));
      });
      if (ready) return daemon.pid;
      if (daemon.exitCode !== null || Date.now() >= until) throw new Error("daemon isolé indisponible");
      await pause(25);
    }
  }
  async verifiedStop(child, needle) {
    if (!child || child.exitCode !== null || child.signalCode !== null || !child.pid) return "already_gone";
    const observed = await exec("/bin/ps", ["-p", String(child.pid), "-o", "ppid=", "-o", "command="]);
    const text = observed.stdout.trim();
    if (Number(text.split(/\s+/)[0]) !== process.pid || !text.includes(needle) || /firefox/i.test(text)) {
      throw new Error(`refus d'arrêter un processus non vérifié: ${child.pid}`);
    }
    child.kill("SIGTERM");
    const until = Date.now() + 10000;
    while (child.exitCode === null && child.signalCode === null) {
      if (Date.now() >= until) throw new Error(`SIGTERM sans effet après 10 s: ${child.pid}`);
      await pause(25);
    }
    return "stopped_by_sigterm";
  }
  async stopDaemon() {
    const result = await this.verifiedStop(this.daemon, BIN);
    this.daemon = undefined;
    return result;
  }
  /** Parent T3 simulé : enregistrement du wrapper de fil (trames réelles) + credential réel. */
  async registerParent(threadId, { project = this.work, bind = true, ackFilter } = {}) {
    const agentId = stableUuid(threadId);
    const instanceId = stableUuid(`instance:${threadId}`);
    const connection = Net.connect(this.socket);
    this.peers.push(connection);
    const lines = new JsonLines(connection);
    await new Promise((resolve, reject) => { connection.once("connect", resolve); connection.once("error", reject); });
    connection.write(`${JSON.stringify({
      type: "Register", identity_version: 2, agent_type: "fixture", agent_id: agentId, instance_id: instanceId,
      host: "native149-isolated", transport: "t3code", mode: "cli", os: "test", turn_in_progress: false,
      journal_available: false,
    })}\n`);
    const registered = await lines.next();
    if (registered.type !== "Registered" || registered.agent_id !== agentId || !registered.credential) {
      throw new Error(`Register refusé: ${JSON.stringify(registered).slice(0, 200)}`);
    }
    await FSP.mkdir(Path.join(this.state, "agent-names"), { recursive: true, mode: 0o700 });
    const hash = Crypto.createHash("sha256").update(instanceId).digest("hex");
    await FSP.writeFile(
      Path.join(this.state, `agent-names/proof-${hash}.json`),
      JSON.stringify({ agent_id: agentId, instance_id: instanceId, credential: registered.credential }),
      { mode: 0o600 },
    );
    const peer = new Peer(this, threadId, connection, lines, agentId, instanceId);
    if (ackFilter) peer.ackFilter = ackFilter; // synchrone : avant toute trame traitée
    if (bind) {
      const fact = () => peer.send({ type: "CommunicationProjectFact", root: project, source: "t3", host: "native149-isolated", worktree_root: null });
      fact();
      const projectResult = await peer.waitFor((f) => f.type === "ProjectContextResult");
      peer.send({ type: "T3ThreadBindingFact", version: 1, t3_thread_id: threadId });
      fact();
      await pause(50);
      peer.projectResult = projectResult;
    }
    return peer;
  }
  /** Client MCP stdio réel (`bridget mcp`) monté avec la preuve T3 privée (jamais journalisée). */
  async mcp({ endpoint, authorization, extraEnv = {}, label }) {
    const env = this.env({ ...(endpoint ? { BRIDGET_T3_MCP_ENDPOINT: endpoint } : {}), ...(authorization ? { BRIDGET_T3_MCP_AUTHORIZATION: authorization } : {}), ...extraEnv });
    const child = this.spawnOwned(["mcp"], env);
    child.stderr?.resume();
    const lines = new JsonLines(child.stdout);
    const send = (v) => child.stdin.write(`${JSON.stringify(v)}\n`);
    let nextId = 1;
    send({ jsonrpc: "2.0", id: nextId++, method: "initialize", params: { protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: `native149-${label ?? "client"}`, version: "1" } } });
    const init = await lines.next();
    if (!init.result) throw new Error("initialisation MCP Rust refusée");
    send({ jsonrpc: "2.0", method: "notifications/initialized" });
    const api = {
      pid: child.pid,
      call: async (name, args = {}, ms = 30000) => {
        const id = nextId++;
        send({ jsonrpc: "2.0", id, method: "tools/call", params: { name, arguments: args } });
        const response = await lines.next(ms);
        const result = response.result ?? {};
        let payload = result.structuredContent;
        if (payload === undefined && result.content?.[0]?.text) {
          try { payload = JSON.parse(result.content[0].text); } catch { payload = { text: result.content[0].text }; }
        }
        return { id: response.id, error: response.error, isError: result.isError === true, code: result.code, payload, raw: result };
      },
      close: async () => {
        child.stdin.end();
        await this.verifiedStop(child, BIN).catch(() => undefined);
      },
    };
    return api;
  }
  /** Lecture seule de la base privée du daemon de recette. */
  sql(query) {
    const out = ChildProcess.execFileSync("sqlite3", ["-readonly", "-json", Path.join(this.state, "bridget.db"), query], { encoding: "utf8" });
    return out.trim() ? JSON.parse(out) : [];
  }
  tasks() {
    return this.sql("select task_id, owner_instance, request_id, payload from native_delegations").map((r) => ({ ...JSON.parse(r.payload), _task: r.task_id, _owner_instance: r.owner_instance }));
  }
  /** Exécutions durables (TaskExecution) : lecture seule de la base privée. */
  execs() {
    return this.sql("select execution_id, state, reason, revision, generation from executions order by created_at, execution_id");
  }
  /** Arbre de processus de la fixture avec PGID (pour prouver l'indépendance des groupes). */
  async processTree() {
    const ps = await exec("/bin/ps", ["-axo", "pid=,ppid=,pgid=,command="]);
    return ps.stdout.split("\n").filter((l) => l.includes(this.root) && !l.includes("/bin/ps")).map((l) => {
      const m = l.trim().match(/^(\d+)\s+(\d+)\s+(\d+)\s+(.*)$/);
      if (!m) return null;
      const kind = /codex149b?\.py|provider149\.py/.test(m[4]) ? "provider" : /managed-wrapper/.test(m[4]) ? "wrapper" : m[4].includes(BIN) ? "bridget" : "other";
      return { pid: Number(m[1]), ppid: Number(m[2]), pgid: Number(m[3]), kind, command: m[4].replaceAll(this.root, "<ROOT>").slice(0, 140) };
    }).filter(Boolean);
  }
  evidence() {
    try {
      return FS.readFileSync(this.evidencePath, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l));
    } catch { return []; }
  }
  /** Fournisseurs fermés encore vivants (PID), pour compter les exécutions réelles. */
  async liveProviders() {
    return (await this.liveFixtureProcesses()).filter((p) => p.kind === "provider").map((p) => p.pid);
  }
  /** Processus encore vivants liés à cette fixture (chemin racine dans la commande). */
  async liveFixtureProcesses() {
    const ps = await exec("/bin/ps", ["-axo", "pid=,ppid=,command="]);
    return ps.stdout.split("\n").filter((l) => l.includes(this.root) && !l.includes("/bin/ps")).map((l) => {
      const m = l.trim().match(/^(\d+)\s+(\d+)\s+(.*)$/);
      if (!m) return null;
      const kind = /codex149b?\.py|provider149\.py/.test(m[3]) ? "provider" : m[3].includes(BIN) ? "bridget" : "other";
      return { pid: Number(m[1]), ppid: Number(m[2]), kind, command: m[3].replaceAll(this.root, "<ROOT>").slice(0, 160) };
    }).filter(Boolean);
  }
  async cli(args, { env = {}, ms = 30000 } = {}) {
    try {
      const out = await exec(BIN, args, { env: this.env(env), cwd: this.root, timeout: ms, maxBuffer: 4 * 1024 * 1024 });
      return { code: 0, stdout: out.stdout, stderr: out.stderr, json: tryJson(out.stdout) };
    } catch (error) {
      return { code: error.code ?? -1, stdout: error.stdout ?? "", stderr: error.stderr ?? "", json: tryJson(error.stdout ?? "") };
    }
  }
  /** Nettoyage : annulation/arrêts propres déjà faits par le scénario ; ici SIGTERM individuel vérifié. */
  async cleanup() {
    const tmpRoot = this.tmpRoot;
    for (const c of this.peers) c.destroy();
    for (const child of this.children.toReversed()) {
      if (child.exitCode !== null || child.signalCode !== null) continue;
      await this.verifiedStop(child, BIN).catch(() => undefined);
    }
    // Tout processus restant lié à la fixture est listé puis arrêté individuellement (SIGTERM, jamais -9).
    await pause(500);
    const remaining = await this.liveFixtureProcesses();
    const stopped = [];
    for (const p of remaining) {
      if (/firefox/i.test(p.command) || p.kind === "other") continue;
      try { process.kill(p.pid, "SIGTERM"); stopped.push(p.pid); } catch { /* déjà parti */ }
    }
    if (stopped.length) await pause(3000);
    const after = await this.liveFixtureProcesses();
    return { remainingBefore: remaining, sigtermed: stopped, remainingAfter: after };
  }
}

export const tryJson = (text) => {
  try { return JSON.parse(text.trim()); } catch { return undefined; }
};

export async function freePort(port) {
  return new Promise((resolve) => {
    const s = Net.createServer();
    s.once("error", () => resolve(false));
    s.listen(port, "127.0.0.1", () => s.close(() => resolve(true)));
  });
}

/** Flux `bridget lineage watch --json` réel, processus possédé par la recette. */
export function lineageWatch(fx, thread, projectRoot = fx.work) {
  const child = fx.spawnOwned(["lineage", "watch", "--json", "--t3-thread", thread, "--project-root", projectRoot], fx.env());
  child.stderr?.resume();
  const lines = [];
  Readline.createInterface({ input: child.stdout }).on("line", (l) => { try { lines.push(JSON.parse(l)); } catch { lines.push({ raw: l.slice(0, 80) }); } });
  return {
    lines,
    pid: child.pid,
    async waitLines(n, ms = 8000) {
      const until = Date.now() + ms;
      while (lines.length < n && Date.now() < until) await pause(20);
      return lines.length >= n;
    },
    async close() {
      child.stdout.destroy();
      await fx.verifiedStop(child, BIN).catch(() => undefined);
    },
  };
}

export const lineageArgs = (action, thread, root, extra = []) =>
  action === "cancel"
    ? ["lineage", "cancel", "--json", "--t3-thread", thread, "--project-root", root, ...extra]
    : ["lineage", "inspect", "--json", "--t3-thread", thread, "--project-root", root, "--action", action, ...extra];

export async function writeResults(name, payload) {
  const dir = Path.join(HERE, RESULTS_DIR);
  await FSP.mkdir(dir, { recursive: true, mode: 0o700 });
  await FSP.writeFile(Path.join(dir, `${name}.json`), JSON.stringify(payload, null, 1) + "\n", { mode: 0o600 });
}

/** Empreintes de sources lues par la recette (pas de secret). */
export function sha256File(path) {
  return Crypto.createHash("sha256").update(FS.readFileSync(path)).digest("hex");
}
