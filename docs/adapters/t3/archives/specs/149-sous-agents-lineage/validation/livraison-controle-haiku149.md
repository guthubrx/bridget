# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/livraison-controle-haiku149.md](../../../../../../../specs/149-sous-agents-lineage/validation/livraison-controle-haiku149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Contrôle | Méthode | Résultat |
|---|---|---|---|
| 1 | Lecture des JSON de livraison | `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/installed-bridget149.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/build-bridget149.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-installed-receipt.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-signing-receipt.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json` (extraction ciblée, 618 Ko), `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-sqlite-smoke-final.json` | Cohérents entre eux |
| 2 | Empreinte du binaire | `shasum -a 256` sur `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget` | `6df86239b638dd83e29eb99640606e7e1f3ecb7b62e75cf33e6e781fdbc7a6f8`, identique au JSON |
| 3 | Signature du binaire | `codesign --verify --strict` | rc 0 (signature ad hoc) |
| 4 | buildId | recherche de `072161262a7d` dans le binaire | présent (2 occurrences) |
| 5 | Empreinte de `app.asar` | `shasum -a 256` | `a66c7f2a13d9e32ca2260f188248bf91e265ed7789a143ea8fd84fb03ea0bd0e`, identique au JSON |
| 6 | Info.plist de l'app | `plutil -p` | `CFBundleShortVersionString` et `CFBundleVersion` = `0.0.45-local.149` |
| 7 | Signature de l'app | `codesign --verify --strict --deep` | rc 0 |
| 8 | Commit T3 | `git rev-parse` dans `t3code-local` | `f6f3d1998c686b20e316f304aff7f5df6a14592c` ; `commitEmbedded` = `f6f3d1998c68` |
| 9 | Empreinte du composant UI | `git show f6f3d1998...:apps/web/src/components/BridgetTaskJournal.tsx` | `4abac314...`, conforme au journal UI |
| 10 | PIDs | `ps -o pid,ppid,lstart` | `58394`, `58396` = `Fri Oct 9 09:03:20 2026` ; `58468` = `09:03:22` ; identiques avant et après |
| 11 | Chemin de lancement | `command -v bridget`, `readlink` | `/Users/moi/.local/bin/bridget` → `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget` (existe) |
| 12 | Jobs ciblés | `ls ~/Library/LaunchAgents`, `launchctl list` | `com.bridget.daemon.plist`, `com.bridget.t3.plist` présents et chargés (PID ci-dessus). Pas de scan global. |
| 13 | Job étranger | `launchctl print` puis lecture du plist et de `install.log` | `com.t3local.install20261009` : non actif, `RunAtLoad` false, `KeepAlive` false, sortie 1 le 9 octobre. Non revendiqué. |
| 14 | Git Bridget | `git log`, `git ls-remote github main`, `git worktree list`, `git branch` | `main` = `07216126`, identique à `github/main` ; seul le worktree `main` ; aucune branche `*149*` |
| 15 | Git T3 | `git ls-remote fork 'refs/heads/*20261009*'`, `git worktree list` | `local/main-20261009` = `f6f3d1998c...` sur `fork` ; aucun worktree 149 |
| 16 | Sauvegarde 148 | `ls` puis `plutil` | `T3 Code (Local).app` (0.0.45-local.148) et `bridget148` ; pas de base de données |
| 17 | Paquet final | `shasum -a 256` | `5b1ca673480aee3ff708841e187adb6f7ef978c42dc7bd336879a1bd173153e6`, identique au JSON |
| 18 | Journal UI | lignes 16 et 80 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md` | 25 / 25 = 24 de la baseline + 1 nouveau test |
| 19 | Revue du composant | `grep 4abac314` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/` | Couverte par `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/native-restart-final-source-review-sonnet-r2.md` (SOURCE_ONLY_APPROVE) |
| 20 | Nettoyage | `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/cleanup-delivery149.json` | 14 619 975 680 octets estimés ; 2 worktrees 149 et 2 branches retirés ; `node_modules` partagé préservé ; pas de base touchée ; 0 job d'activation |
| 21 | Mesure de reconnexion | `grep` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/native-real-restart-sonnet-r7.md` | 27 s déduits, non mesurés. Réserve notée. |
