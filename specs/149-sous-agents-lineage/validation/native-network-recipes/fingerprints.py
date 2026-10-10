#!/usr/bin/python3
"""Empreintes des sources effectivement exercées (lecture seule). Aucun secret, aucune écriture hors results/."""
import hashlib, json, os, subprocess, sys, time

WT = "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage"
T3 = "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage"
BIN = "/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-ec6b18b5d468"
RECEIPT = f"{WT}/specs/149-sous-agents-lineage/validation/native149-debug-receipt-r5.json"  # reçu du binaire IMMUTABLE testé (archivé quand le reçu canonique est passé à r6)
CURRENT_RECEIPT = f"{WT}/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json"

def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

def rust_files():
    out = []
    for root, dirs, files in os.walk(f"{WT}/crates"):
        dirs[:] = [d for d in dirs if d not in ("target", ".git")]
        for name in files:
            if name.endswith((".rs", ".toml")) or name == "Cargo.lock":
                out.append(os.path.join(root, name))
    out += [f"{WT}/Cargo.toml", f"{WT}/Cargo.lock"]
    return sorted(set(out))

def fingerprint(files):
    lines = sorted(f"{os.path.relpath(p, WT)}\0{sha(p)}" for p in files if os.path.exists(p))
    return hashlib.sha256("\n".join(lines).encode()).hexdigest(), len(lines)

receipt = json.load(open(RECEIPT))
files = rust_files()
full, n_full = fingerprint(files)
prod_files = [p for p in files if "/tests/" not in p and not p.endswith(("_test.rs", "_tests.rs"))]
prod, n_prod = fingerprint(prod_files)
binary_mtime = os.path.getmtime(BIN)
newer = [os.path.relpath(p, WT) for p in prod_files if os.path.getmtime(p) > binary_mtime]
newer_all = [os.path.relpath(p, WT) for p in files if os.path.getmtime(p) > binary_mtime]
# Le manifeste HISTORIQUE du binaire (reçu r5) est figé : il décrit les sources compilées. Une dérive
# FUTURE des sources (ex. Cargo.toml modifié par un autre propriétaire) ne l'invalide pas : elle est
# listée fichier par fichier contre les empreintes consignées au reçu, sans toucher au binaire.
recorded = {**receipt["sourceFileDigests"]["keyProductionFiles"], **receipt["sourceFileDigests"]["allChangedOrUntrackedSources"]}
drift = {}
for rel, digest in sorted(recorded.items()):
    path = os.path.join(WT, rel)
    now = sha(path) if os.path.exists(path) else None
    if now != digest:
        drift[rel] = {"receipt": digest[:16], "now": None if now is None else now[:16]}
cargo_now = {rel: sha(os.path.join(WT, rel))[:16] for rel in ("Cargo.toml", "Cargo.lock") if os.path.exists(os.path.join(WT, rel))}
t3_files = [
    "packages/provider-core/src/server/mcpSession.ts", "apps/server/src/mcp/OrchestratorMcpService.ts",
    "apps/server/src/mcp/McpSessionRegistry.ts", "apps/server/src/mcp/McpHttpServer.ts",
    "apps/server/src/bridget/BridgetReader.ts", "apps/server/src/bridget/BridgetLineage.ts",
    "apps/server/src/ws.ts", "apps/server/src/server.ts", "packages/contracts/src/bridgetLineage.ts",
    "packages/contracts/src/bridgetPermissions.ts",
]
result = {
    "binary": {"path": BIN, "sha256": sha(BIN), "matchesReceipt": sha(BIN) == receipt["binary"]["sha256"]},
    "rustSources": {"fingerprintNow": full, "fingerprintReceipt": receipt["sourceFingerprint"]["value"], "equal": full == receipt["sourceFingerprint"]["value"], "files": n_full,
                    "productionNow": prod, "productionReceipt": receipt["productionSourceFingerprint"]["value"], "productionEqual": prod == receipt["productionSourceFingerprint"]["value"],
                    "productionSourcesNewerThanBinary": newer, "allSourcesNewerThanBinary": newer_all,
                    "sourcesChangedSinceReceiptDigests": drift, "cargoManifestNow": cargo_now,
                    "interpretation": "binaire immuable = référence ; sourcesChangedSinceReceiptDigests liste uniquement une dérive FUTURE des sources, jamais un défaut du binaire"},
    "t3": {"head": subprocess.check_output(["git", "-C", T3, "rev-parse", "HEAD"], text=True).strip(),
           "files": {p: sha(f"{T3}/{p}") for p in t3_files}},
    "recipeTools": {n: sha(os.path.join(os.path.dirname(os.path.abspath(__file__)), n)) for n in sorted(os.listdir(os.path.dirname(os.path.abspath(__file__)))) if n.endswith((".ts", ".mjs", ".py")) and not n.startswith("explore") and n != "smoke_host.ts"},
    "receiptBinarySha256": receipt["binary"]["sha256"],
    "currentCanonicalReceipt": (lambda c: {"binarySha256": c["binary"]["sha256"], "productionFingerprint": c["productionSourceFingerprint"]["value"], "productionFingerprintEqualsNow": c["productionSourceFingerprint"]["value"] == prod, "sourceDigestsDifferingFromR5": sorted(k for k in set({**receipt["sourceFileDigests"]["keyProductionFiles"], **receipt["sourceFileDigests"]["allChangedOrUntrackedSources"]}) | set({**c["sourceFileDigests"]["keyProductionFiles"], **c["sourceFileDigests"]["allChangedOrUntrackedSources"]}) if {**receipt["sourceFileDigests"]["keyProductionFiles"], **receipt["sourceFileDigests"]["allChangedOrUntrackedSources"]}.get(k) != {**c["sourceFileDigests"]["keyProductionFiles"], **c["sourceFileDigests"]["allChangedOrUntrackedSources"]}.get(k)), "note": "reçu canonique plus récent (binaire différent, NON rejoué ici) : sert uniquement à situer la dérive future des manifestes"})(json.load(open(CURRENT_RECEIPT))), "checkedAt": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
}
os.makedirs(os.path.join(os.path.dirname(os.path.abspath(__file__)), "results"), exist_ok=True)
with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "results", "fingerprints.json"), "w") as f:
    json.dump(result, f, indent=1)
print(json.dumps({k: result[k] for k in ("binary", "rustSources")}, indent=1)[:3000])
