# T046 — release Bridget 0.4.0 publiée et vérifiée

Publication depuis **Cartae** (`cartae`), le 2026-10-15T15:10:34Z. Source binaire B : `431b78f72bdd73eac4772af17bf36879d7a629d2`.

Release : https://github.com/guthubrx/bridget/releases/tag/v0.4.0

Tag annoté : `v0.4.0`. Objet tag : `43202c9de66ce466b76ba05bd3077699ac88ca3d`. Cible pelée et API GitHub : `431b78f72bdd73eac4772af17bf36879d7a629d2`.

## Actifs publiés et téléchargés

| Actif | Taille en octets | SHA-256 vérifié après téléchargement |
|---|---:|---|
| bridget-0.4.0-darwin-arm64.tar.gz | 9702745 | `3b57805bd4a2b95dedab14e950118d698166c06b2dc0f2085a5f7d799242d687` |
| bridget-0.4.0-darwin-arm64.tar.gz.sha256 | 100 | `7cce08e88757679e7f2aa44f298de2c718ffd8e5d7eaf1e613c69824e381c13a` |
| bridget-0.4.0-linux-x86_64.tar.gz | 10700914 | `ae8b7c17c1abea84a39fd01385961a8daaad7f803f5fea87f8fb0ed64dd30aa2` |
| bridget-0.4.0-linux-x86_64.tar.gz.sha256 | 100 | `7a94e6af6abac1d5005cc50d790320e06928582088f4eb890306160287037322` |

Les quatre actifs distants sont exactement les quatre actifs B attendus. Les tailles, les digests GitHub disponibles et les empreintes des fichiers réellement téléchargés concordent. Le tag annoté distant et sa cible exacte sont vérifiés séparément. Aucun actif A n’est publié.

## Prérequis et preuve réelle

ROOT a clos T045 et donné le GO explicite avant les écritures GitHub. Laya B utilise decision_service sur deux candidats ; le classement déterministe porte 21 candidats ; Haiku OAuth Cartae utilise lightweight_model sur 21 candidats et choisit gpt-5.6-luna avec effort low. Les trois voies rendent leurs résultats au SHA B.

Le rejeu Haiku après redémarrage privé 3804689→3834811 conserve la même Task, le résultat et les trois BLOB canon/metadata/delegate. ACK 1792075876 et cleanup_done=true sont conservés. Le payload brut de Task change ; aucune identité byte-à-byte de ce payload n’est revendiquée. HTTP et nouveaux processus ne sont pas instrumentés.

Les parents, daemons, relais et tunnels privés ont été arrêtés selon les reçus de nettoyage et la clôture ROOT. Cette publication n’effectue aucune action de service. La production B était déjà installée et active. T3 n’est pas relancé ; le serveur T3 97328 est un changement externe signalé par ROOT.

## Limites conservées

FULL unique antérieure au correctif B : 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des échecs initiaux PASS au dernier replay, trois annulations159 non validables sur Linux. La cause initiale T3 reste inconnue. Aucun GREEN global. Audit A V15 r2 PASS et delta T045 documenté restent distingués ; pas de nouveau scan, FULL ou passage Converge. Deux LOW RUSTSEC-2026-0098 et RUSTSEC-2026-0099 sur rustls-webpki0.101.7, et un INFO maintenance RUSTSEC-2024-0449 sur unsafe-libyaml0.2.11 restent conservés.

Après création d’un reçu admission v2, aucun retour direct au binaire 0.3.0. Utiliser un binaire compatible ou une restauration cohérente validée sans perte.

## Reçus canoniques

/home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json

SHA-256 du reçu privé : `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

/home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication-remote-release.json

/home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication-remote-tag-ref.json

/home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication-remote-annotated-tag.json

/home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/download-verification

/home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/after-restart-matrix-receipt.json

SHA-256 : `f6d3d046bfe8cd1c3665b3a583931a3dbc170aa1b9603be20ef239a52c740581`.

/home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/native-mcp-matrix-restart-proof.json

SHA-256 : `aea8a42db1b2eae8265030cb669669cdd31fe9b761cc30f173607873fdc9a003`.

Le nettoyage des worktrees, branches, caches et TMP reste une étape séparée. Aucune tâche n’est cochée ici et aucun commit documentaire n’est créé.
