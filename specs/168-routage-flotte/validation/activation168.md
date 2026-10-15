# T044 — activation 0.4.0 exécutée

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
