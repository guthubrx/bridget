# Diagnostic 268 : journaux et observations

Les signalements du 11 novembre sont rapprochés des états durables sans rejouer les missions.

- Les tâches b8a3bce2, 57e85b90 et 0cd79968 sont cancelled, cleanup_done et garde libérée. La liste T3 conserve une projection working/seq708, car la synchronisation du lineage est portée par la lecture ou le suivi du parent visible. Cette projection ne remplace pas l’état durable de la tâche. Aucun changement des tâches ni relancement.
- Les quatre agents refusés AgentUnknown correspondent à des tâches result_available et cleanup_done. Le daemon ne cherchait que la présence volatile, perdue après disparition du routeur. Hypothèse vérifiable : le refus peut être qualifié depuis la tâche durable après arrêt confirmé, parce que la présence en mémoire ne conserve pas tout l’historique.
- La coupure Vie des habitants coïncide avec le chargement de 267 : core à 14:37:44Z, connecteur Mac renouvelé vers 14:38, lien de ce responsable réinscrit à 14:39:02Z. Une seule tâche fd4460c8 existe pour la clé rejouée strictement, encore working. Aucun droit changé ni mission doublée.
- La copie privée de fca64289 contient la base 1892791d demandée, pas les deux autres commits cités dans le corps. Le placement worktree transfère sa base, pas les objets arbitraires cités dans le texte, les réserves externes ou les alias SSH. SYS n’a plus que 4 790 724 Kio disponibles au relevé. Son propriétaire doit fournir les prérequis autorisés ; aucune mission de remplacement ici.
- Il n’existe pas de mode auto pour send. Le parcours entre responsables est queue ; steer exige filiation ou mandat de boucle spécifique. Les reçus de diagnostic mcp-86189-6af482ad-10 et mcp-86189-6af482af-11 sont conservés sans réveil, pas présumés lus.

Périmètre du correctif : diagnostic du journal après disparition des présences. Conserver les gardes d’accès, les tâches actives, les codes du protocole et les résultats. Ne pas qualifier un enfant working ou non nettoyé d’arrêté. Une route active reste prioritaire sur l’historique. Pas de redémarrage de T3.

Le test exécuté avant correction échoue sur AgentUnknown au lieu de AgentStopped, puis passe avec la recherche durable. Le premier passage sans test après copie interrompue ne compte pas comme preuve ; le chemin temporaire macOS a aussi dû être raccourci pour la socket de la fixture.

Revue indépendante acd3a93d-992e-4653-a31d-0b589a5055e0 : accord sur le diagnostic sous réserve de ne pas prétendre que le résultat est livré. Le message existant ne le prétend pas. L’objection sur les homonymes est écartée par l’index UNIQUE native_delegations_child, déjà présent dans la source cumulative, main et la base active. Le test est complété pour vérifier ce refus d’homonyme et l’absence de faux arrêt en cas d’erreur de lecture du magasin. Le résultat est lu, ACK et la garde propriétaire close ; vue conservée.

Le contrôleur n’est plus routable lors du retour mcp-86189-6af483aa-12 (unknown_recipient, effet non confirmé). Ce retour n’est pas présumé reçu et n’est pas renvoyé par une autre route. Son diagnostic est conservé ici et sera partagé au propriétaire racine encore connecté avec le bilan.

## Vérifications et activation

Six scénarios ciblés passent sur chaque plateforme : régression268, gate JournalReady, compatibilité du wrapper historique, refus avant JournalReady, distinction arrêté/inconnu et refus du pilote sans journal. Le scénario268 final comprend aussi l’unicité de l’enfant et l’erreur de lecture du magasin. Clippy strict et builds release passent sur Mac et Debian12. Le binaire Linux exige glibc2.34, sous le plafond2.36. La recette cumulative reproduit exactement les quatre fichiers exécutables ou de tests modifiés ; aucun code du programme réservé n’est activé.

Version0.8.35, build268-journal-durable-r1, code f524eb6259d09a65164d5437029eb041cd1b88d6 poussé. Release publique https://github.com/guthubrx/bridget/releases/tag/v0.8.35 : six actifs téléchargés anonymement et vérifiés. APK8035, certificat de production conservé.

Sept services réellement chargés sur quatre hôtes, empreintes vérifiées : core3186324, mobile3187003 et worker3190539 sur Cartae, worker3618467 sur SYS, worker1293991 sur KS, worker32035 et connecteur32398 sur Mac. Empreinte Linux : 4b1b7de41c0a0c01b138ec3d05fda2906dacf250c56c59518d154fe82021816f. Empreinte Mac : fb46342b0144c5689c6b203964ecda044f22a5aa38c555cfacc0a51f957dedf9. Les quatre pointeurs privés de retour désignent267, avec images et formats qualifiés dans les deux sens ; aucun retour exécuté. Les clients et superviseurs de missions déjà lancés restent compatibles avec ce changement interne du diagnostic du maître.

Coupure core4,86secondes. Neuf superviseurs Cartae, un SYS et un KS ont conservé PID et naissance. T3 conserve PID77762, démarrage11novembre14:10:41CET : aucun arrêt ni remplacement. La modification n’écrit ni les tâches métier ni les registres de télémétrie ; le déblocage267 et la protection mémoire restent actifs.

Le circuit réel MCP est rejoué sur le même agent e880e4f9 : avant correction, AgentUnknown ; après chargement, AgentStopped avec le guidage existant vers l’historique et le résultat de la même tâche, sans prétendre qu’ils ont été livrés. Le lien natif et le statut du maître répondent sur268. Les états T3 en cache ne sont pas réécrits par ce correctif ; leur portée et leur rafraîchissement sont documentés dans l’adaptateur.

Retours ciblés après activation : mcp-86189-6af48799-17, mcp-86189-6af4879c-18, mcp-86189-6af4879e-19 et mcp-86189-6af487a1-1a, tous stored_without_wakeup. Conservation attestée, lecture non présumée. Le propriétaire racine reçoit le diagnostic du contrôleur devenu non routable. Aucun envoi de remplacement au contrôleur ni accusé de réception ajouté.

## Ménage et preuves

547935005octets logiques retirés : exports et copies propres terminés sur Mac et SYS. Les copies temporaires Cartae et KS restent conservées faute d’observation complète de leurs lecteurs. Les anciennes images Mac, SYS et Cartae sont conservées lorsque l’observation native des processus est incomplète ; sur KS,266 reste réellement utilisée. Active268 et retour267 sont qualifiés et conservés. Aucun autre processus n’est arrêté pour nettoyer. Les appels installed ont été effectués après installation vérifiée, avec refus conservateur du nettoyage quand les usages ne peuvent être attestés. Aucun worktree ni branche créé.

Preuves privées : /Users/moi/.cache/bridget268-journal, notamment reproduction.json, review-result.json, journal-before.json, journal-after.json, runtime-final-cartae.json, runtime-final-sysle2.json, runtime-final-ksstore.json, mac-loaded.json, final-notices.json et cleanup-summary.json. Exports reconstructibles retirés après qualification ; recettes, journaux et distribution publique vérifiée conservés.
