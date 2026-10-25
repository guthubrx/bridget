# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/final.md](../../../../../../../specs/149-sous-agents-lineage/validation/final.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Complément du 2026-10-11 : sources d’identité durables validées et poussées. Bridget `9f2ccbf23195` signé, installé et chargé par daemon30562/pont30974 à02:17:41/48 Europe/Paris. T3 `d833387203` signé et prêt, application encore ancienne PID60116/60228.
Tests ciblés : T3124PASS/1SKIP, Rust25PASS/1ignored puis pont85PASS ; revues approuvées. 121états compacts observés après relance, sans nouvelle erreur de nom trop long dans les journaux capturés.
Le MCP de ce fil répond encore `identity_not_found` avant relance T3. L’utilisateur demande d’attendre la fin des autres fils. Aucune délégation native de ce fil réussie n’est déclarée à ce stade.
Reçu actuel : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/identite-durable149.md. Les checkpoints suivants restent historiques.

<a id="extrait-2"></a>

## Extrait 2

**Complément actif du 2026-10-11 : saturation des fichiers ouverts corrigée.**
Code `0d5107afcf186bacc57aefd1925cd98f076806b8`, tests `8d3eff76`, intégrés sur `main` et poussés vers `github/main`.
Binaire signé installé : build `0d5107afcf18`, sha256 `b0dd9b9040604128ad28efd1680c0ff08ed08e6f19f52129963aba1a4e76ead9`.
Après la demande utilisateur « corrige », seuls le daemon et le pont ont été relancés, individuellement.
Daemon PID10624 et pont PID14501 : limite souple4096, plafond dur inchangé ; 4tests isolés PASS, 1fixture ignorée.
Deux observations réelles : 322fils contre63 avant, daemon983descripteurs et pont1295–1296, zéro erreur de saturation après leur démarrage corrigé.
T3 PID60116/60228 conserve sa naissance du 11 octobre à01:13:35/36 ; application non relancée par ce correctif.
Le contrôle MCP hérité du principal répond encore `identity_not_found` ; ses variables d’attestation sont absentes.
Les avertissements `thread_detail/threadId` antérieurs subsistent. Ce complément valide le correctif de saturation uniquement.
Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/limite-fichiers149-apres.json`.
Sauvegarde du binaire149 remplacé : `/Users/moi/.cache/bridget-fd149.eksbzxke/bridget149-before`.

<a id="extrait-3"></a>

## Extrait 3

| Dépôt | Branche | Commit | Poussé |
|---|---|---|---|
| Bridget | `main` | `072161262a7d4d6ea2e1305d546f115c634506d4` (`07216126`) | `github/main` (vérifié par `ls-remote`) |
| T3 | `local/main-20261009` | `f6f3d1998c686b20e316f304aff7f5df6a14592c` | `fork` seulement (vérifié par `ls-remote`) |

<a id="extrait-4"></a>

## Extrait 4

| Élément | Version | Empreinte | Vérification |
|---|---|---|---|
| Binaire Bridget `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget` | bridget 0.1.3, buildId `072161262a7d` | sha256 `6df86239b638dd83e29eb99640606e7e1f3ecb7b62e75cf33e6e781fdbc7a6f8` (17 433 504 octets) | sha256 relu ; `codesign --verify --strict` rc 0 ; buildId présent dans le binaire |
| App `/Applications/T3 Code (Local).app` | 0.0.45-local.149 (Info.plist) | `app.asar` sha256 `a66c7f2a13d9e32ca2260f188248bf91e265ed7789a143ea8fd84fb03ea0bd0e` | `codesign --verify --strict --deep` rc 0 ; commit intégré `f6f3d1998c68` |

<a id="extrait-5"></a>

## Extrait 5

Le paquet final est `/Users/moi/.cache/t3-final149.K5dHqa/artifacts/T3-Code-0.0.45-local.149-arm64.zip`, sha256 `5b1ca673480aee3ff708841e187adb6f7ef978c42dc7bd336879a1bd173153e6`.

<a id="extrait-6"></a>

## Extrait 6

- Contrôle final du paquet : 2576 PASS, 0 FAIL.
- Signatures : 38 objets signés, `strictDeepVerifyRc` 0 (reçu de signature).
- Octets installés identiques au paquet (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/t3-installed-receipt.json`).
- Smoke SQLite : electron 44.4.5, node 24.21.0, modules 149, lecture durable OK, `quickCheck` ok. Pas d'interface, pas de serveur, pas d'appel de modèle.

<a id="extrait-7"></a>

## Extrait 7

| Élément | État constaté |
|---|---|
| `58394` (`bridget daemon`) | Naissance `Fri Oct 9 09:03:20 2026`, identique avant et après |
| `58396` (`bridget t3 serve`) | Naissance `Fri Oct 9 09:03:20 2026`, identique avant et après |
| `58468` (T3 Code Alpha) | Naissance `Fri Oct 9 09:03:22 2026`, identique avant et après |

<a id="extrait-8"></a>

## Extrait 8

- Aucun redémarrage, aucun kill, aucune installation par ce contrôle.
- Le code chargé en mémoire reste l'ancien. Les versions 149 sur disque serviront à la prochaine relance par l'utilisateur.
- Chemin de lancement : `/Users/moi/.local/bin/bridget` est un lien vers `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`, qui existe.
- Jobs ciblés : `com.bridget.daemon.plist` et `com.bridget.t3.plist` existent dans `/Users/moi/Library/LaunchAgents`. Les deux sont chargés, avec les PID ci-dessus. Aucun job d'activation différée créé par la livraison.

<a id="extrait-9"></a>

## Extrait 9

- Bridget : seul le worktree `main` reste. Aucune branche `*149*`. Dans `git status`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/tasks.md` est modifié et les fichiers de `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/` sont non suivis. Le commit et le push de la documentation restent au principal.
- T3 : aucun worktree `149`. Les branches `session-149-sous-agents-lineage` ont été retirées (cleanup).
- Sauvegarde de retour arrière : `/Users/moi/.cache/bridget-install149.8atzwz` contient `T3 Code (Local).app` (0.0.45-local.148) et le binaire `bridget148`. Pas de base de données.

<a id="extrait-10"></a>

## Extrait 10

- Environ 13,6 GiB supprimés (14 619 975 680 octets estimés).
- Supprimés : clones privés, stage de build, cibles Cargo 149, worktrees et branches fusionnés 149.
- Préservés : worktrees étrangers, `node_modules` partagé, sauvegarde 148, bases de données (non touchées).
- Les chemins de stage, de clones et de worktrees cités dans les anciens rapports sont historiques. Ils n'existent plus.
- Tâches planifiées T3 du projet : 0. Aucun job d'activation créé.

<a id="extrait-11"></a>

## Extrait 11

1. **Job étranger chargé** : `com.t3local.install20261009` (`/Users/moi/.cache/t3-upgrade-20261009.tueYFG/installer.plist`). État : non actif, `RunAtLoad` false, `KeepAlive` false, dernier code de sortie 1 (installation du 9 octobre interrompue). Il n'a pas été créé par la livraison 149. Je ne l'ai pas analysé plus loin. Décision du principal : conserver ce job non actif hors du périmètre149.
2. **Jobs étrangers non analysés** : `com.bridget.federation.cartae-core`, `com.t3.spec139.delivery`, `com.t3.spec140.delivery`, `com.t3.spec143.delivery`. Aucun n'est revendiqué ici.
3. **Rust** : 1840 PASS, 0 FAIL final (native r9, agrégé). SC005 non conclu : l'ancien banc de latence n'est pas concluant (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/hash-latency149.md`).
4. **T3** : 16 erreurs serveur et 10 erreurs web préexistantes, selon le brief. Je n'ai pas relancé le typecheck. Le journal UI indique 10 erreurs web dans sa section 1 et 0 dans sa section 5. La formulation de la section 5 est incohérente. Ces erreurs ne viennent pas des fichiers 149 (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md`).
5. **Reconnexion du parent** : environ 27 s. Durée déduite du ledger, non mesurée (`/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/native-real-restart-sonnet-r7.md`, lignes 88 et 103). Le délai est borné et n'aboutit à aucun résultat faux. À mesurer proprement lors d'une relance automatique.
6. **Pas de vert global** : ce reçu ne prononce pas de validation globale du dépôt.
