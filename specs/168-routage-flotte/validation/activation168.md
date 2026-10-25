# T044 — activation 0.4.0 exécutée

## État courant — 15/10/2026 à 17:24 CEST

**Implemented — 56/56 tâches cochées. Pipeline : Livré, avec conservations explicites T048.** ROOT a validé le contrôle final des 35 exigences, 11 critères et 7 histoires. Aucune tâche ne reste ouverte.

La récupération documentaire T047 est prouvée : C1 `db46c7816777199da7c41bb9bc10063cd361dae7` intégré et poussé, main Mac/Cartae alignés et propres aux reçus. Source binaire B et release v0.4.0 inchangées ; aucun rebuild. Preuves : /home/moi/.local/state/bridget-delivery168/git/integration-pushC1-cartae.json, SHA-256 `e32d1529d986fff8af4441928353d8b1a5fd163e4aefaf7e3b90651d9ce90454` ; /Users/moi/.cache/bridget-delivery168/git/integration-pullC1-mac.json, SHA-256 `4cf769d0707b2b28535a6f91052d786ff5c5b1c861738e645f6a20df84111766`.

L’examen T048 est terminé avec retraits et conservations motivées. Le worktree source Cartae et sa branche sont conservés : C1 fusionné/poussé, arbre propre et zéro ignoré ; garde lsof RC0 mais 4 337 octets stderr, donc observation incomplète. Aucun match observé ne vaut preuve d’absence d’usage. Aucun contournement ni suppression inventée. Les 18 anciens candidats conservent leurs motifs. Reçu : /home/moi/.local/state/bridget-delivery168/cleanup/t048-source-final-guard.json, SHA-256 `e2410c3dc3d838b1fb49cdb065f0a9c067cda717a105cce4d0458b94ea25cfc3`. Bilan : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/nettoyage168.md. Le contrôle final T049 est validé. Le SHA du commit documentaire final est vérifié hors de son propre contenu dans les reçus Git privés.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-1).

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-2).

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

> État courant : source B 431b78f72bdd73eac4772af17bf36879d7a629d2 ; checkpoints A historiques.
> Référence : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md (section « Checkpoint B courant »).

Date : 15 octobre 2026. **Activation utile vérifiée**, source `269de64e172f71f8856373fe8b09cb2d8642da52`. La session complète reste In Progress : recettes réelles T045, release et nettoyage non terminés. Ce reçu ne coche aucune tâche.

## Installation et services chargés

Le préflight, freeze-install et activate r2 terminent chacun RC0. Après quiescence, le worker est préparé avant l’arrêt des services Mac ; sauvegardes cohérentes et protections de processus sont conservées. Les binaires et le registre fusionné sont installés, puis les services nécessaires sont relancés. Le reçu d’activation s’arrête honnêtement à active_pending_mcp_checks ; le suivi natif et MCP exécuté ensuite porte active_verified.

| Service | Identité nouvelle | Exécutable |
|---|---|---|
| Daemon Mac | PID 47823, né le 15 octobre à 15:09:07 CEST | /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget |
| Pont Mac | PID 47926, né le 15 octobre à 15:09:09 CEST | /Users/moi/.local/bin/bridget |
| Worker Cartae | PID 3712026, naissance Linux 158490770 ; 13:09:09 UTC | /home/moi/.local/lib/bridget-communication/0.4.0-269de64e172f/bin/bridget |

Les services remplacés étaient daemon 89231, pont 89553 et worker 3144803. La CLI répond bridget 0.4.0 ; le daemon publie le Build-id source A. Le binaire Mac installé porte SHA-256 `17ec685eb9c80be7e91dc5350119f0404cbf8ddf57fe1e5e8710d00e5d3d6b69`. Le worker Linux porte `a03e86f84851f81d082bc4c70f5fbd53f63c31d65482f1e785a43e05f3c53887`.

Le registre ON installé est celui qualifié : Mac `d104f2e82ec93ef8aae487e73326ba3da7f3e8adb9ad043ed3587beb54ce67e7`, worker `bac0293a069ed4d90a3fcff7feb439c42fd5c49d5b1a3d86ad605a975bfe1083`. Le credential worker reste inchangé. Les plists Mac et le fichier fleet restent inchangés ; l’unité worker pointe vers le nouveau préfixe binaire.

## Contrôles après activation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-3).

Le statut CLI a aussi signalé 19,7 Gio libres, sous le seuil de 20 Gio. L’avertissement est conservé ; le futur nettoyage doit respecter ses gardes et ne justifie aucune suppression non contrôlée.

## adaptateur de bureau, tunnel et historique protégés

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-4).

Sauvegardes conservées :

/Users/moi/.cache/bridget-delivery168/activation/backup-269de64e172f-r2

/home/moi/.local/state/bridget-delivery168/activation/backup-269de64e172f-r2

Aucun rollback automatique ou restauration de DB. Les lignes metadata_version=2 interdisent le retour direct à un ancien binaire.

## Preuves exécutées

- /Users/moi/.cache/bridget-delivery168/activation/r2-preflight-command.json — SHA-256 `3189b27a31f8b1f61aa36f604a7e6d6bd6863d8422093c6731ad5eeaab0f1f3a`.
- /Users/moi/.cache/bridget-delivery168/activation/r2-freeze-install-command.json — SHA-256 `be04d0b6f596fcd9bfccb3362b71a39bc3609af1dd9724962125141632a1cd10`.
- /Users/moi/.cache/bridget-delivery168/activation/r2-activate-command.json — SHA-256 `f9d9ed2943578289f691c76c24e793565a5ea60f1b327a0d40fb6432638fc645`.
- /Users/moi/.cache/bridget-delivery168/activation/backup-269de64e172f-r2/activation.json — SHA-256 `04b0dcf71522597c36788c47ed573a9aea4bf3c451dbd8a0141f9264e98bee77`.
- /Users/moi/.cache/bridget-delivery168/activation/runtime-native-checks168.json — SHA-256 `2608083d3f8861925522d3781c16116fc54ddd4824df17861c22d1040616b810`.
- /Users/moi/.cache/bridget-delivery168/activation/runtime-identity168.json — SHA-256 `033e2ab622a3764be4acf759b95818ef90fdeaf6125e2dbffa2b73ed3b7f2461`.
- /Users/moi/.cache/bridget-delivery168/activation/runtime-mcp-who168.json — SHA-256 `4ebdfa46c142bccd9001a923d723ee64aa3d24b9fcdde8fa4d82231291b0f1a8`.
- /Users/moi/.cache/bridget-delivery168/activation/runtime-followup168.json — SHA-256 `af31f8e20f63db8283cfb1b2cfde1cd26adb57bbe920cf3c339f1dbe63cdabdc`.

Worker : /home/moi/.local/state/bridget-delivery168/activation/runtime-worker-connection168.json ; SHA-256 `6cb49093703db34dba4da47663c27ce8002a470ff993136215df190a6335a5ae`.

Le suivi runtime relie explicitement les hashes des reçus d’activation, d’identité et de MCP. Les fichiers privés restent les preuves canoniques ; aucune nouvelle commande de service, de modèle ou de test n’est lancée pour cette rédaction.

## Limites et suite

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-5).


## Checkpoint B courant — activation vérifiée

Source désormais chargée : `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0. Les activations A 0.4.0 et 0.3.0 précédentes restent des faits historiques ; elles ne décrivent plus les services courants. Les actions B preflight, freeze-install et activate sont réellement RC0. Le worker est préparé avant l’arrêt maître/pont, avec quiescence et sauvegardes cohérentes. Le reçu initial B conserve sa phase active_pending_mcp_checks ; un suivi séparé prouve ensuite active_verified sans réécrire cet historique.

| Service courant | PID et naissance | Exécutable |
|---|---|---|
| Daemon Mac | 71927 ; 15 octobre 15:57:58 CEST | /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget |
| Pont Mac | 72468 ; 15 octobre 15:58:00 CEST | /Users/moi/.local/bin/bridget |
| Worker Cartae | 3767260 ; naissance Linux 158783879 | /home/moi/.local/lib/bridget-communication/0.4.0-431b78f72bdd/bin/bridget |

Mac SHA-256 : `3502daf850bae303a9ad0f4bdd2dbc073f6b36fdd51ced8ac62f52014aa15bed`. Linux SHA-256 : `3b86b8fd48698e567d5f190a67f946d84d99fecd4526441ba6702bba9c3bf95a`. Le statut natif répond RC0, daemon en ligne et Build-id B exact. Le pont répond running, avec 33 fils au moment de cette lecture ; les 34 fils du reçu A restent une mesure historique.

Le catalogue natif B expose exactement 27 options = 17 locales + 10 Cartae. Registre ON : Mac `59802f1a8e42bdca8fc01935b0638f448f493a8dcf30973bf46d0f846ef7bf47`, worker `bac0293a069ed4d90a3fcff7feb439c42fd5c49d5b1a3d86ad605a975bfe1083`. Le seul delta de politique est le budget local global/service 30 s / 20 s. Credential worker inchangé ; fleet et plists Mac inchangés ; unité worker dirigée vers le préfixe B.

DB de production : quick_check ok, FK0, migrations 7/12 et hash de schéma `ce291503bf9b09228775ba8825b25ff7e194d920788a56c51f3cc3ddbb1b329f` inchangé entre préflight et après activation B. Aucun rollback de DB.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-6).

Preuves canoniques :

- /Users/moi/.cache/bridget-delivery168/activation/B-preflight-command.json ; SHA-256 `0dec216d0ebc179c5ba0cadd7034d4f61bcefa88ac7564308541f0da5303cc83`.
- /Users/moi/.cache/bridget-delivery168/activation/B-freeze-install-command.json ; SHA-256 `a49affa634eec8fad51aa27d524dcaed63b0386eea62355a1e9a96e71702095b`.
- /Users/moi/.cache/bridget-delivery168/activation/B-activate-command.json ; SHA-256 `15722d3a2dae73f0b1c83905936b975fcc47cff11c8eab34ce498839d999cc82`.
- /Users/moi/.cache/bridget-delivery168/activation/backup-431b78f72bdd-B/activation.json ; SHA-256 `a55769f921f1cdd6e1fa51075c726a55309aa399a67f6cca5583180688369ba2`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-native-checks168.json ; SHA-256 `85d0f462f859412862d2e8ce773b762e3f3e4f0665584dc5ac30b563682f2bc0`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-mcp-who168.json ; SHA-256 `ac5f773abf3444baab60fdb24bbd8e601b57c6b9a9b0488a493e0d8eb57dd447`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json ; SHA-256 `6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/activation168.md#extrait-7).
