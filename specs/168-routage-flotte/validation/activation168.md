# T044 — activation 0.4.0 exécutée

## État courant — 15/10/2026 à 17:13 CEST

**In Progress, 53/56 tâches cochées par ROOT.** T045 et T046 sont closes. T047, commit documentaire C1, est en cours ; T048 et T049 restent ouvertes. Le nettoyage final de la source appartient à ROOT. Aucun statut Livré ou Implemented n’est annoncé.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

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

- CLI native status RC0 : daemon en ligne et Build-id source A.
- Catalogue JSON RC0 : exactement 27 options, dont 17 locales et 10 Cartae ; entitlement reste unknown, pas de droit modèle universel déduit.
- CLI t3 status RC0 : pont running, 34 fils au moment du reçu.
- Identité native : instance `52d64cf9-a40e-47e4-b1a6-3eaacd4215d1`, capacités auto_delegate_v2, worker_candidate_probe_v2 et routing_catalogue_v1 publiées.
- Lecture MCP who réussie après l’activation ; elle établit le rattachement observé, pas une recette de délégation 168.
- Worker : bon exécutable et hash, socket actif et trois threads observés. Aucune époque de connexion de production n’est exposée dans ces lectures ; aucune valeur n’est devinée. Cette preuve ne remplace pas une Task distante T045.
- DB Mac : migrations idempotence 7 / exécutions 12, quick_check=ok, FK0 ; overview après activation total_active=0. Le hash du schéma change normalement avec la DDL 168, de `9d735455f7ec7aa12c050d067233732007b8434d8397f504752d7f897413e331` à `ce291503bf9b09228775ba8825b25ff7e194d920788a56c51f3cc3ddbb1b329f`.

Le statut CLI a aussi signalé 19,7 Gio libres, sous le seuil de 20 Gio. L’avertissement est conservé ; le futur nettoyage doit respecter ses gardes et ne justifie aucune suppression non contrôlée.

## T3, tunnel et historique protégés

T3 conserve PID 65544 né à 04:07:14 et serveur 65608 né à 04:07:15 le 15 octobre ; tunnel 60206 né le 13 octobre à 20:14:11. PID, naissances et chemins des ancres restent ceux du préflight. Aucun signal ou redémarrage T3/tunnel n’est effectué. La sortie naturelle du renderer 34864 a fait refuser le premier freeze avant mutation ; le correctif de garde des ancres stables est conservé et contre-relu, sans effacer l’échec historique.

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

Les préqualifications sans modèle T043 sont distinctes des recettes réelles T045 encore à faire. La release n’est pas publiée à ce checkpoint. La FULL historique reste 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux passent après reprise, trois annulations non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global ni clôture complète de livraison n’est déduit.


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

Les ancres T3 65544/65608, nées le 15 octobre à 04:07:14/15, et le tunnel 60206, né le 13 octobre à 20:14:11, sont préservés avec leurs chemins et naissances. Aucun redémarrage T3. La lecture MCP who réussit et porte project_warnings=[]. Ce rattachement et le worker actif ne constituent pas une recette de Task distante ; aucun connection_epoch de production non exposé n’est déduit.

Preuves canoniques :

- /Users/moi/.cache/bridget-delivery168/activation/B-preflight-command.json ; SHA-256 `0dec216d0ebc179c5ba0cadd7034d4f61bcefa88ac7564308541f0da5303cc83`.
- /Users/moi/.cache/bridget-delivery168/activation/B-freeze-install-command.json ; SHA-256 `a49affa634eec8fad51aa27d524dcaed63b0386eea62355a1e9a96e71702095b`.
- /Users/moi/.cache/bridget-delivery168/activation/B-activate-command.json ; SHA-256 `15722d3a2dae73f0b1c83905936b975fcc47cff11c8eab34ce498839d999cc82`.
- /Users/moi/.cache/bridget-delivery168/activation/backup-431b78f72bdd-B/activation.json ; SHA-256 `a55769f921f1cdd6e1fa51075c726a55309aa399a67f6cca5583180688369ba2`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-native-checks168.json ; SHA-256 `85d0f462f859412862d2e8ce773b762e3f3e4f0665584dc5ac30b563682f2bc0`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-mcp-who168.json ; SHA-256 `ac5f773abf3444baab60fdb24bbd8e601b57c6b9a9b0488a493e0d8eb57dd447`.
- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json ; SHA-256 `6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e`.

État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global. Aucun service ou modèle n’est lancé par la rédaction de ce checkpoint.
