#!/usr/bin/env python3
"""T033 : objets Git immuables, builds neufs et scénario identique sans flotte.

Produit un rapport dans une racine temporaire privée. Aucun code/compte/service
de l'ancien checkout n'est utilisé. macOS seulement (RSS de /usr/bin/time -l).
"""
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import tempfile
import time

SOURCE = "dfa2134dcfe2a2522e3ae77d93561e6ae72556b3"
REPO = Path(__file__).resolve().parent.parent
CARGO = "/Users/moi/.cargo/bin/cargo"


def run(args, cwd, env, timeout=1200):
    return subprocess.run(args, cwd=cwd, env=env, capture_output=True, timeout=timeout)


def main():
    if sys.platform != "darwin":
        raise SystemExit("Cette mesure utilise explicitement le RSS macOS (time -l).")
    root = Path(tempfile.mkdtemp(prefix="b089-measure-", dir="/private/tmp"))
    print(f"PREUVES {root}", flush=True)
    refs = [SOURCE, subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()]
    report = {"schema_version": 1, "root": str(root), "versions": [], "samples": []}
    env = {"PATH": "/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin",
           "CARGO_HOME": "/Users/moi/.cargo", "RUSTUP_HOME": "/Users/moi/.rustup",
           "CARGO_NET_OFFLINE": "true", "RUSTUP_AUTO_INSTALL": "0"}
    for name in ["home", "state", "tmp"]:
        (root / name).mkdir(mode=0o700)
    env.update(HOME=str(root / "home"), BRIDGET_HOME=str(root / "state"), TMPDIR=str(root / "tmp"))
    report["toolchain"] = run([CARGO, "--version"], root, env).stdout.decode().strip()
    report["rustc"] = run(["/Users/moi/.cargo/bin/rustc", "--version"], REPO, env).stdout.decode().strip()
    report["platform"] = run(["/usr/bin/uname", "-sm"], root, env).stdout.decode().strip()
    executables = []
    try:
        for label, ref in zip(["reference", "core"], refs):
            source = root / label
            source.mkdir(mode=0o700)
            archive = subprocess.check_output(["git", "archive", ref], cwd=REPO)
            subprocess.run(["tar", "-xf", "-", "-C", str(source)], input=archive, check=True)
            local_env = dict(env, BRIDGET_BUILD_ID=ref[:12])
            meta = run([CARGO, "metadata", "--offline", "--locked", "--format-version", "1"], source, local_env)
            (root / f"{label}-metadata.stderr").write_bytes(meta.stderr)
            if meta.returncode:
                raise RuntimeError(f"metadata {label}: voir {label}-metadata.stderr")
            metadata = json.loads(meta.stdout)
            (root / f"{label}-metadata.json").write_bytes(meta.stdout)
            local = [p for p in metadata["packages"] if p["source"] is None]
            modules = {}
            for package in local:
                lib = Path(package["manifest_path"]).parent / "src/lib.rs"
                if lib.exists():
                    modules[package["name"]] = re.findall(r"^(?:pub )?mod (\w+);", lib.read_text(), re.M)
            entry = {"label": label, "commit": ref, "resolved_packages": len(metadata["packages"]),
                     "workspace_members": len(metadata["workspace_members"]), "root_modules": modules}
            report["versions"].append(entry)
            command = [CARGO, "build", "--offline", "--locked", "--jobs", "2", "-p", "bridget-daemon", "--bin", "bridget"]
            print(f"BUILD {label} (target neuf, cache Cargo partagé, 2 jobs)", flush=True)
            before = time.monotonic()
            build = run(["/usr/bin/time", "-l", *command], source, local_env)
            entry["build_seconds"] = time.monotonic() - before
            entry["build_exit"] = build.returncode
            (root / f"{label}-build.log").write_bytes(build.stdout + build.stderr)
            rss = re.search(rb"(\d+)\s+maximum resident set size", build.stderr)
            entry["build_max_rss_bytes"] = int(rss[1]) if rss else None
            if build.returncode:
                raise RuntimeError(f"build {label}: voir {label}-build.log")
            entry["binary_bytes"] = (source / "target/debug/bridget").stat().st_size
            print(f"BUILD {label} OK {entry['build_seconds']:.2f}s", flush=True)
            compile_test = run([CARGO, "test", "--offline", "--locked", "--jobs", "2", "-p", "bridget-daemon",
                                "--test", "execution_scale_test", "--no-run", "--message-format=json"], source, local_env)
            (root / f"{label}-compile-test.stderr").write_bytes(compile_test.stderr)
            if compile_test.returncode:
                raise RuntimeError(f"compile scénario {label}: voir compile-test.stderr")
            messages = [json.loads(line) for line in compile_test.stdout.splitlines() if line.startswith(b"{")]
            executable = next(m["executable"] for m in messages if m.get("target", {}).get("name") == "execution_scale_test" and m.get("executable"))
            executables.append((label, source, local_env, executable))
        # Ordre alterné pour ne pas attribuer toute dérive de charge à un côté.
        # Même test 256 agents/in-memory, mêmes assertions et seuil 3 s.
        for pair in range(20):
            for label, source, local_env, executable in executables[::1 if pair % 2 == 0 else -1]:
                before = time.monotonic_ns()
                result = run(["/usr/bin/time", "-l", executable, "--exact",
                              "projection_execution_reste_bornee_sur_256_agents", "--nocapture"], source, local_env, 30)
                elapsed = time.monotonic_ns() - before
                if result.returncode or b"1 passed; 0 failed" not in result.stdout:
                    (root / f"{label}-sample-{pair}.log").write_bytes(result.stdout + result.stderr)
                    raise RuntimeError(f"scénario {label} #{pair} refusé")
                rss = re.search(rb"(\d+)\s+maximum resident set size", result.stderr)
                report["samples"].append({"label": label, "pair": pair, "wall_ns": elapsed,
                                          "max_rss_bytes": int(rss[1]) if rss else None})
        for entry in report["versions"]:
            samples = [s for s in report["samples"] if s["label"] == entry["label"]]
            values = sorted(s["wall_ns"] for s in samples)
            entry["scenario"] = {"count": len(values), "median_ns": statistics.median(values),
                                 "p95_ns": values[18], "max_ns": values[-1],
                                 "median_max_rss_bytes": statistics.median(s["max_rss_bytes"] for s in samples)}
        report["status"] = "passed"
    except (RuntimeError, subprocess.SubprocessError) as error:
        report["status"] = "incomplete"
        report["error"] = str(error)
        print(f"INCOMPLET {error}", flush=True)
    finally:
        # Résultats générés, pas réécriture d'un fichier de production.
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"RAPPORT {root / 'report.json'}", flush=True)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    os.umask(0o077)
    raise SystemExit(main())
