# Reçu final - session 149 (sous-agents natifs et Lineage T3)

**Résultat du contrôle : PASS**, avec les réserves de la section 7.
Date : 2026-10-10. Contrôle : Haiku 5.5 high (T045). Détail : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/livraison-controle-haiku149.md`.

## 1. Commits et branches

| Dépôt | Branche | Commit | Poussé |
|---|---|---|---|
| Bridget | `main` | `072161262a7d4d6ea2e1305d546f115c634506d4` (`07216126`) | `github/main` (vérifié par `ls-remote`) |
| T3 | `local/main-20261009` | `f6f3d1998c686b20e316f304aff7f5df6a14592c` | `fork` seulement (vérifié par `ls-remote`) |

Aucun push vers `upstream` ou `origin` par ce contrôle.

## 2. Versions et empreintes installées

| Élément | Version | Empreinte | Vérification |
|---|---|---|---|
| Binaire Bridget `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget` | bridget 0.1.3, buildId `072161262a7d` | sha256 `6df86239b638dd83e29eb99640606e7e1f3ecb7b62e75cf33e6e781fdbc7a6f8` (17 433 504 octets) | sha256 relu ; `codesign --verify --strict` rc 0 ; buildId présent dans le binaire |
| App `/Applications/T3 Code (Local).app` | 0.0.45-local.149 (Info.plist) | `app.asar` sha256 `a66c7f2a13d9e32ca2260f188248bf91e265ed7789a143ea8fd84fb03ea0bd0e` | `codesign --verify --strict --deep` rc 0 ; commit intégré `f6f3d1998c68` |

Le paquet final est `/Users/moi/.cache/t3-final149.K5dHqa/artifacts/T3-Code-0.0.45-local.149-arm64.zip`, sha256 `5b1ca673480aee3ff708841e187adb6f7ef978c42dc7bd336879a1bd173153e6`.

## 3. Preuves de build et de paquet

- Contrôle final du paquet : 2576 PASS, 0 FAIL.
- Signatures : 38 objets signés, `strictDeepVerifyRc` 0 (reçu de signature).
- Octets installés identiques au paquet (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-installed-receipt.json`).
- Smoke SQLite : electron 44.4.5, node 24.21.0, modules 149, lecture durable OK, `quickCheck` ok. Pas d'interface, pas de serveur, pas d'appel de modèle.

## 4. Processus et lancement

| Élément | État constaté |
|---|---|
| `58394` (`bridget daemon`) | Naissance `Fri Oct 9 09:03:20 2026`, identique avant et après |
| `58396` (`bridget t3 serve`) | Naissance `Fri Oct 9 09:03:20 2026`, identique avant et après |
| `58468` (T3 Code Alpha) | Naissance `Fri Oct 9 09:03:22 2026`, identique avant et après |

- Aucun redémarrage, aucun kill, aucune installation par ce contrôle.
- Le code chargé en mémoire reste l'ancien. Les versions 149 sur disque serviront à la prochaine relance par l'utilisateur.
- Chemin de lancement : `/Users/moi/.local/bin/bridget` est un lien vers `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`, qui existe.
- Jobs ciblés : `com.bridget.daemon.plist` et `com.bridget.t3.plist` existent dans `/Users/moi/Library/LaunchAgents`. Les deux sont chargés, avec les PID ci-dessus. Aucun job d'activation différée créé par la livraison.

## 5. Git, worktrees et sauvegarde

- Bridget : seul le worktree `main` reste. Aucune branche `*149*`. Dans `git status`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/tasks.md` est modifié et les fichiers de `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/` sont non suivis. Le commit et le push de la documentation restent au principal.
- T3 : aucun worktree `149`. Les branches `session-149-sous-agents-lineage` ont été retirées (cleanup).
- Sauvegarde de retour arrière : `/Users/moi/.cache/bridget-install149.8atzwz` contient `T3 Code (Local).app` (0.0.45-local.148) et le binaire `bridget148`. Pas de base de données.

## 6. Nettoyage

Reçu : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/cleanup-delivery149.json`.

- Environ 13,6 GiB supprimés (14 619 975 680 octets estimés).
- Supprimés : clones privés, stage de build, cibles Cargo 149, worktrees et branches fusionnés 149.
- Préservés : worktrees étrangers, `node_modules` partagé, sauvegarde 148, bases de données (non touchées).
- Les chemins de stage, de clones et de worktrees cités dans les anciens rapports sont historiques. Ils n'existent plus.
- Tâches planifiées T3 du projet : 0. Aucun job d'activation créé.

## 7. Réserves et points ouverts

1. **Job étranger chargé** : `com.t3local.install20261009` (`/Users/moi/.cache/t3-upgrade-20261009.tueYFG/installer.plist`). État : non actif, `RunAtLoad` false, `KeepAlive` false, dernier code de sortie 1 (installation du 9 octobre interrompue). Il n'a pas été créé par la livraison 149. Je ne l'ai pas analysé plus loin. Décision du principal : conserver ce job non actif hors du périmètre149.
2. **Jobs étrangers non analysés** : `com.bridget.federation.cartae-core`, `com.t3.spec139.delivery`, `com.t3.spec140.delivery`, `com.t3.spec143.delivery`. Aucun n'est revendiqué ici.
3. **Rust** : 1840 PASS, 0 FAIL final (native r9, agrégé). SC005 non conclu : l'ancien banc de latence n'est pas concluant (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/hash-latency149.md`).
4. **T3** : 16 erreurs serveur et 10 erreurs web préexistantes, selon le brief. Je n'ai pas relancé le typecheck. Le journal UI indique 10 erreurs web dans sa section 1 et 0 dans sa section 5. La formulation de la section 5 est incohérente. Ces erreurs ne viennent pas des fichiers 149 (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md`).
5. **Reconnexion du parent** : environ 27 s. Durée déduite du ledger, non mesurée (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/native-real-restart-sonnet-r7.md`, lignes 88 et 103). Le délai est borné et n'aboutit à aucun résultat faux. À mesurer proprement lors d'une relance automatique.
6. **Pas de vert global** : ce reçu ne prononce pas de validation globale du dépôt.

## 8. UI : 25 tests

Le total est de 25 tests passants sur les 4 fichiers UI 149 : 24 de la baseline, plus 1 nouveau (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md`, lignes 16 et 80). Le chiffre 24 des anciens documents était la baseline avant ce test. Le total courant est 25. Écart résolu.

La revue Sonnet qui couvre l'empreinte `4abac314...` du composant est `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/native-restart-final-source-review-sonnet-r2.md`. Verdict : SOURCE_ONLY_APPROVE.

## 9. Périmètre de ce contrôle

- Aucun test de modèle. Aucune régression globale. Aucun sous-agent lancé.
- Aucun redémarrage, kill ou installation par ce contrôle.
- Non modifiés : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/tasks.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md`, code, Git, builds, services. Modifiés : statuts en tête de `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/spec.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/plan.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/implementation.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/quickstart.md`, et ce reçu.
- Chemins communiqués : absolus.
- Prochaine étape pour le principal : commit et push des reçus sur main ; tâches validées et cochées. Les jobs étrangers sont conservés.

Clôture du principal : T001–T045 validées et cochées. Les reçus sont intégrés à la livraison documentaire sur main. Le commit de code du binaire reste 072161262a7d4d6ea2e1305d546f115c634506d4 ; les changements suivants portent uniquement sur la documentation et les preuves.
