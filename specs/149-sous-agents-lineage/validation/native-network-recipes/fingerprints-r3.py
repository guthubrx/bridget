#!/usr/bin/python3
"""Empreintes r3 (lecture seule) : binaire release r8, sources de production Rust, dérive après le binaire, sources T3, outils de recette."""
import hashlib, json, os, subprocess, time
WT = "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage"
T3 = "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage"
BIN = "/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb"
RECEIPT = f"{WT}/specs/149-sous-agents-lineage/validation/native149-release-receipt-r8.json"
HERE = os.path.dirname(os.path.abspath(__file__))
def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""): h.update(c)
    return h.hexdigest()
files = []
for d, dirs, fs in os.walk(f"{WT}/crates"):
    dirs[:] = [x for x in dirs if x not in ("target", ".git")]
    for n in fs:
        if n.endswith((".rs", ".toml")) or n == "Cargo.lock": files.append(os.path.join(d, n))
files += [f"{WT}/Cargo.toml", f"{WT}/Cargo.lock"]
files = sorted(set(files))
def fp(paths): return hashlib.sha256("\n".join(sorted(f"{os.path.relpath(p, WT)}\0{sha(p)}" for p in paths)).encode()).hexdigest()
is_test = lambda p: "/tests/" in p or p.endswith(("_test.rs", "_tests.rs"))
prod = [p for p in files if not is_test(p)]
receipt = json.load(open(RECEIPT))
bin_mtime = os.path.getmtime(BIN)
newer = sorted((time.strftime("%H:%M:%S", time.localtime(os.path.getmtime(p))), os.path.relpath(p, WT), sha(p)[:16]) for p in files if os.path.getmtime(p) > bin_mtime)
r8_digests = {c["path"]: c.get("sha256R8") for c in receipt["sourceChanges"]["sinceR6"]}
out = {
    "binary": {"path": BIN, "sha256": sha(BIN), "equalsReceiptR8": sha(BIN) == "0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6"},
    "productionFingerprintNow": fp(prod), "productionFingerprintReceiptR8": receipt["productionSourceFingerprint"]["value"],
    "productionEqualsReceipt": fp(prod) == receipt["productionSourceFingerprint"]["value"],
    "sourcesNewerThanBinary": newer,
    "t3": {"head": subprocess.check_output(["git", "-C", T3, "rev-parse", "HEAD"], text=True).strip(), "diffHeadSha16": hashlib.sha256(subprocess.check_output(["git", "-C", T3, "diff", "HEAD"])).hexdigest()[:16], "expectedDiffHeadSha16": "59a54267ce7c1d2a"},
    "recipeTools": {n: sha(os.path.join(HERE, n))[:16] for n in sorted(os.listdir(HERE)) if n.endswith((".ts", ".mjs", ".py", ".sh")) and not n.endswith("-r2.ts")},
    "checkedAt": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
}
os.makedirs(f"{HERE}/results-r3", exist_ok=True)
json.dump(out, open(f"{HERE}/results-r3/fingerprints.json", "w"), indent=1)
print(json.dumps({k: out[k] for k in ("binary", "productionEqualsReceipt", "sourcesNewerThanBinary", "t3")}, indent=1))
