#!/usr/bin/env python3
"""Vérifie le paquet Android avant de remplacer les actifs de sa release."""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


PACKAGE = "app.cartae.bridget"
REPOSITORY = "guthubrx/bridget"
# Certificat public de production, déjà utilisé par la release officielle 0.7.0.
CERTIFICATE = "0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588"


def version_code(version):
    if not re.fullmatch(r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}", version):
        raise ValueError("Version stable attendue : major.minor.patch")
    major, minor, patch = map(int, version.split("."))
    return major * 1_000_000 + minor * 1_000 + patch


def run(arguments, timeout=120):
    return subprocess.run(arguments, check=True, capture_output=True,
                          text=True, timeout=timeout).stdout


def inspect(apk, build_tools):
    badging = run([str(build_tools / "aapt2"), "dump", "badging", str(apk)])
    package = re.search(
        r"^package: name='([^']+)' versionCode='([0-9]+)' versionName='([^']+)'",
        badging, re.MULTILINE)
    if not package:
        raise ValueError("Identité du paquet illisible")
    # Un code de sortie nul est obligatoire : la sortie seule ne prouve rien.
    signing = run([str(build_tools / "apksigner"), "verify", "--verbose",
                   "--print-certs", str(apk)])
    signers = set(re.findall(
        r"^Signer #[0-9]+ certificate SHA-256 digest: ([0-9a-fA-F]{64})$",
        signing, re.MULTILINE))
    if not signers:
        raise ValueError("Certificat du paquet non vérifié")
    return package.group(1), int(package.group(2)), package.group(3), {
        signer.lower() for signer in signers}


def verify(apk, reference, version, build_tools):
    expected = version_code(version)
    package, code, name, signers = inspect(apk, build_tools)
    old_package, old_code, _, old_signers = inspect(reference, build_tools)
    if package != PACKAGE or old_package != PACKAGE:
        raise ValueError("Identifiant Android inattendu")
    if name != version or code != expected:
        raise ValueError("Version APK différente de la release : publication refusée")
    if code < old_code:
        raise ValueError("Régression de version Android : publication refusée")
    if signers != old_signers or signers != {CERTIFICATE}:
        raise ValueError("Certificat différent du paquet de référence : publication refusée")
    return {"package": package, "version": name, "version_code": code,
            "sha256": hashlib.sha256(apk.read_bytes()).hexdigest(),
            "size": apk.stat().st_size, "signers": sorted(signers)}


def verify_publication(version, result, checksum):
    try:
        release = json.loads(run(["gh", "api", "repos/" + REPOSITORY + "/releases/tags/v" + version]))
    except subprocess.CalledProcessError as exc:
        # GitHub ne résout pas toujours par tag un brouillon dont le tag n'existe pas encore.
        releases = json.loads(run(["gh", "api", "repos/" + REPOSITORY + "/releases?per_page=100"]))
        matches = [item for item in releases if item.get("tag_name") == "v" + version]
        if len(matches) != 1:
            raise ValueError("Release non identifiée ; aucun nouvel upload automatique") from exc
        release = matches[0]
    assets = {asset["name"]: asset for asset in release["assets"]}
    expected = {
        "bridget-android.apk": (result["sha256"], result["size"]),
        "bridget-android.apk.sha256": (
            hashlib.sha256(checksum.read_bytes()).hexdigest(), checksum.stat().st_size)}
    for name, (digest, size) in expected.items():
        asset = assets.get(name, {})
        if asset.get("digest") != "sha256:" + digest or asset.get("size") != size:
            raise ValueError("Actifs publics non confirmés ; relire la release avant tout nouvel envoi")
    if release["tag_name"] != "v" + version:
        raise ValueError("Tag public non confirmé")
    return not release.get("draft", False)



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--apk", type=Path, required=True)
    parser.add_argument("--reference-apk", type=Path, required=True)
    parser.add_argument("--build-tools", type=Path, required=True)
    parser.add_argument("--publish", action="store_true",
                        help="Remplacer seulement APK et checksum après vérification")
    parser.add_argument("--verify-published", action="store_true",
                        help="Relire les actifs publics sans nouvel upload")
    args = parser.parse_args()
    version_code(args.version)
    # La copie inspectée est exactement celle envoyée, hors de l'arbre de build.
    with tempfile.TemporaryDirectory(prefix="bridget-android-release-") as directory:
        apk = Path(directory) / "bridget-android.apk"
        shutil.copyfile(args.apk, apk)
        result = verify(apk, args.reference_apk, args.version, args.build_tools)
        checksum = apk.with_suffix(".apk.sha256")
        checksum.write_text(result["sha256"] + "  bridget-android.apk\n")
        if args.publish:
            run(["gh", "release", "upload", "v" + args.version,
                 str(apk), str(checksum), "--clobber", "--repo", REPOSITORY], timeout=600)
        checked = args.publish or args.verify_published
        public = verify_publication(args.version, result, checksum) if checked else False
        print(json.dumps({**result, "uploaded": args.publish, "published": public,
                          "assets_verified": checked, "public_verified": public}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        raise SystemExit("Publication Android non confirmée : " + str(error)
                         + ". Après un envoi commencé, relire les actifs avant tout nouvel envoi.") from error
