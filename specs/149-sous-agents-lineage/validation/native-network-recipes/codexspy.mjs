// Outil de recette 149 : enveloppe le faux pair app-server Codex de T3 (testFixtures/codexCollabMockPeer.mjs, copié en lecture
// seule dans le cache privé de la fixture). Il consigne chaque ligne JSON-RPC reçue de T3 (sans l'afficher) puis la transmet.
// Aucun modèle : le pair rejoue des réponses de session capturées. Le journal est privé (0600) et supprimé avec la fixture.
import * as ChildProcess from "node:child_process";
import * as FS from "node:fs";
import * as Path from "node:path";
import * as Readline from "node:readline";

const here = Path.dirname(new URL(import.meta.url).pathname);
const log = process.env.SPY_LOG;
FS.appendFileSync(log, JSON.stringify({ spy: "argv", argv: process.argv.slice(2), pid: process.pid, t: Date.now() }) + "\n", { mode: 0o600 });
const peer = ChildProcess.spawn(process.execPath, [Path.join(here, "codexCollabMockPeer.mjs")], { stdio: ["pipe", "inherit", "inherit"], env: process.env });
Readline.createInterface({ input: process.stdin }).on("line", (line) => {
  FS.appendFileSync(log, line + "\n", { mode: 0o600 });
  peer.stdin.write(line + "\n");
});
process.stdin.on("end", () => peer.stdin.end());
peer.on("exit", (code) => process.exit(code ?? 0));
process.on("SIGTERM", () => { peer.kill("SIGTERM"); });
