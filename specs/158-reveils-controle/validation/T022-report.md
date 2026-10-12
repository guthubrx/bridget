# Relecture T022158A — capture avant correctifs

La lecture couvre les 4 612 lignes, les 137 blocs et les 24 fichiers du diff fourni : 10 fichiers Rust et 14 documents. Les 24 empreintes courantes correspondent encore à la capture. Un écart confirmé reste ouvert : la preuve enfant de bridget_queue peut ouvrir une connexion auxiliaire avant son refus.

Cette revue ne coche pas T022 et ne déclare aucun succès global de la suite complète. ROOT conserve l'application, l'exécution et la clôture.

## Preuves de lecture

Diff : /Users/moi/.cache/bridget-audit158-20261012-r1/diff158A-final.patch

SHA256 : 590c8bbb70c61471552513428c468cc0e3211feaa67ee02acfd61e992d87399b

Inventaire : /Users/moi/.cache/bridget-audit158-20261012-r1/scope-files.json

SHA256 : 5081979d37c5bbe52df6e6cc616547944105430c4f88c99f7dad7ea4e43cb25e

Preuve par fichier et par bloc : /Users/moi/.cache/bridget-finalread158-20261012/read_manifest.json

Base : bd37e927eee0b28f058df9d6a6992e38665a6ad0. La couverture137/137 concerne le diff et son contexte. Elle ne couvre pas tous les fichiers existants ni toute l'architecture. Les nouveaux fichiers sont entièrement présents dans le diff. Les anciennes lignes supprimées sont repérées par leurs coordonnées avant modification. L'inventaire initial reste une preuve d'inventaire, sans lecture sémantique inventée.

## Écart confirmé : ChildProof après connexion

Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs

Aux lignes631–655, execute_tool_at_with_identity ouvre registered_connection et échange CallerScope avant le dispatch. Le refus enfant arrive dans execute_queue, lignes1135–1146. Le test guidance156_mcp_queue_schema est modifié aux lignes2233–2264 du diff : « avant toute connexion » devient « avant toute connexion métier ». Cette adaptation affaiblit la protection antérieure.

Contrat antérieur : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/156-guidage-file/tasks.md

T024, ligne62, impose le refus sur la façade MCP avant connexion. T026, ligne64, conserve cet oracle. Le nouveau calcul du plafond158 ne justifie pas de déplacer cette garde.

Statut : confirmé par lecture indépendante puis par ROOT. Aucun exploit runtime observé. ROOT a demandé la garde enfant avant registered_connection et la restauration du témoin préconnexion. Correctif et rejeu exact restent attendus. Aucune résolution n'est revendiquée sur cette capture.

## Rapprochement code, tests et documents

- La CLI reprend le transport de contrôle existant. Liste active et limite50 par défaut ; maximum100. Les limites invalides, options inconnues et UUID invalides sont couverts. Les mutations conservent le terminal interactif avant émission. Le compteur annoncé désigne les arbres visés par l'annulation, sans promettre un arrêt physique.
- La vue globale publie sept champs par tâche. Elle exclut mission, résultat et chemin de travail. Le compteur actif précède la pagination. Les métadonnées d'arrêt et halted viennent du même snapshot. Les tests couvrent plusieurs propriétaires, ordre, limite et exclusion des corps.
- Les gardes exigent négociation, capacité puis acteur humain. La borne ADR011 reste explicite : le rôle de contrôle existant porte l'identité humaine. Le processus hostile du même compte système ne reçoit pas une garantie nouvelle.
- État et reçu d'arrêt sont écrits sous transaction immédiate. Le rejeu rend la réponse enregistrée et marque replayed. Aucun callback, snapshot ou parcours d'annulation n'est refait. Les témoins A/B/C et la réouverture disque distinguent réponse historique et état actuel. Le callback qui panique au rejeu contrôle cette absence de parcours.
- Delegate et Continue composent arrêt et pause après la recherche de rejeu. Les quatre combinaisons sont étendues à Continue. Demandes d'enfant ouvertes et missions déjà en cours conservent leur périmètre déclaré. L'annulation active choisit les racines sans ancêtre actif. Les liens manquants et la profondeur invalide ferment le traitement.
- Les six variantes de contrôle ont leurs allers-retours. Les refus sont fermés. Delegate conserve ses octets historiques. Aucun outil MCP de contrôle humain n'est ajouté.
- Les portées couvrent les30 outils dans les deux sens. Un outil inconnu ferme avant RPC. Lecture, état propre et écriture des fils restent distincts. Journal sans destinataire ou avec null relève de Reads ; toute autre valeur, même vide, relève de Sends. Les annotations suivent la portée de base.
- CallerScope utilise l'identité inscrite côté daemon et l'index enfant existant. Résultat invalide et panne ferment l'appel. Aucun stockage client ni repli permissif. Les tests distinguent tâche terminale avant nettoyage, nettoyage, rondes, posture effective et parent. Les façades à réponses simulées ne sont pas une preuve de fournisseur réel. Lecture de trois pages et ACK utilisent le store existant.
- Les documents maintiennent158B hors implémentation. Budgets de tours, réveils et secrets restent exclus. T021 et T022 restent ouverts dans le diff lu.

## Autres points transmis par ROOT

F1 : StoreUnavailable peut arriver après le commit d'arrêt, pendant l'annulation. Le handler du diff, lignes420–505, montre cette séquence. Le libellé CLI doit signaler une application partielle possible. ROOT prépare la correction.

F2 : ROOT/Haiku signalent la purge atomique des arbres et tree_settled. Ce point n'a pas été élargi en collecte indépendante ici. Sa preuve et sa correction restent distinctes.

F3 : ROOT demande de documenter le code1 de refus de control status.

F4 : Continue rejoué expose la vue actuelle avec suspended dynamique. Le test du diff, ligne1336, prend la pause courante. La documentation doit distinguer cette vue du reçu historique de halt.

Ces quatre points ne sont pas déclarés résolus dans cette capture.

## Limites

Aucun test, build, fournisseur, service ou mutation Git n'a été exécuté par ce lecteur. ROOT rapporte diffcheckRC0 et47 tests ciblés PASS. Ces informations ne remplacent pas le reçu de la suite complète ni la comparaison des255 noms de référence. La lecture statique ne prouve ni l'absence de course, ni la latence, ni l'arrêt physique.

Toute correction ultérieure exige une empreinte et une relecture du delta. ChildProof exige son rejeu préconnexion exact. Ce rapport ne demande aucun deuxième lancement global.
