# C19 — synchronisation durable, correction en cours
Statut : développement et qualification en cours, aucune activation nouvelle.
Programme complet :46/182 tâches vérifiées, In Progress. T030/T174 restent
ouvertes ; cette correction ne clôt pas C19 ni le programme.

## Reprise vérifiée du client
Le31octobre2026, après fermeture complète et reprise du même fil, les trois
processus MCP observés37990/38144/38729 utilisent221-view-b-8a80c257.
Le port bridget_read_commit_file est déclaré. Une seule session puis un seul
catalogue : run634/révision2, lien répond,45options et gel=false.
Aucune nouvelle boucle de redémarrage.

## Deux qualifications originales
Les enveloppes conservées ont été émises chacune une seule fois :
- Codex2fb96ec5, clébridget221-qualification-c19-codex-20261031-01 : refus
  metadata_capacity avant fournisseur, annulation propriétaire puis cancelled.
  Publication non attestée, conservation non inventée.
- Claudebfb2db6c, clébridget221-qualification-c19-claude-20261031-01 : refus
  timed_out avant fournisseur, annulation propriétaire puis cancelled.
  Le reçu natif confirme source_stopped=true, closed=true et conservation.
  L'export est partiel :2658fichiers,38434107octets ; il reste conservé.
Aucun résultat de qualification fournisseur, remplacement ou rejeu revendiqué.
Les refus ne désignent ni tout le serveur ni les autres travaux comme arrêtés.

## Cause et mesure
Le manifeste de la baseA d35452df compte4778entrées,4671fichiers sélectionnés,
63584948octets. Le helper s'est arrêté durant l'export, après les premières
synchronisations individuelles. Sur le même volume SYS,100fichiers4Kio ont
demandé2,457s séquentiellement et0,650s avec8barrières simultanées.
Mesure synthétique : elle établit le coût de cette étape, pas encore le temps
complet du projet. Le premier essai de mesure dans le cache a été refusé car
son système de fichiers différait ; il n'a produit aucune mesure substituée.
Les deux dossiers synthétiques ont été retirés après fermeture de leurs fichiers.
Les réservations de résultats actifs et les reçus du worker partagent16Mio.
Le relevé courant après solde d'une autre tâche donne7,08Mo avant une nouvelle
réservation de vue de8,54Mo. Il ne reconstruit pas exactement le budget historique
du premier refus. Le conseil nomme donc aussi les réservations actives, sans
ordonner de supprimer des reçus ou de relever une borne.

## Correction ciblée
Conserver chaque fsync, les droits, les hashes, les liens, le manifeste et les
délais. Grouper au plus8fichiers écrits ; attendre et joindre leurs8threads dans
le helper déjà possédé avant toute publication. Synchronisation des dossiers,
notice et barrière forte finale inchangées. Aucun sync du volume, pool global,
thread détaché, privilège ou dépendance. Un échec conserve les données et refuse
la publication ; le groupe du helper reste sous son contrôleur natif.

## Preuves actuelles et limites
Linux première série :134tests daemonC19 PASS/1recette privée ignorée,
5transport PASS ; deux recettes privées ignorées dans cette suite. La même série134/5 passe sur Mac. Le nominal étendu vérifie65sources, modes0400, liens, audit,
source sale et checkout lié conservés. Le négatif vérifie délai expiré, erreur
de fsync, fermeture des descripteurs et dépassement de8.
Le complément metadata_capacity passe dans le test ciblé, puis dans la suiteC19.
Revue indépendante812957bf Claude/Sonnet KS : APPROVE, résultat entier3405octets lu, ACK/cancel une fois ; aucune P1/P2 concrète.
AuditV15 source13/13 applicables, décisionpass et validationtrue ; qualité par axe63–75. Audit borné aux sources, pas au runtime.
Binaires candidats221-view-sync-r1 construits : Linux SHA5d04e3dfe291eb51ef481c1be5a7cee33e2a355fa06780720fe190171ad08afb, GLIBCmax2.34 ; Mac SHA114c59395dc18eada5e2c92c9d647cabd8da9d56b7893209027c2befd836453c.
Recette complète sur le volume SYS, distribution, activation, qualifications et preuves finales restent requises. Aucune activation nouvelle revendiquée.

## Travail parallèle conservé
C05 est conservé dans le stashf79fddc6b5814e9d3cbf8b171b02c1fc270abfab,
et dans un patch privé SHA256323057db14f26c17fe3056ea47e8bceda2b66c055b6a50351fa61ffadcd532cc.
Avant conservation :8daemon/4transport PASS ; contrôles de stockage4PASS et
régressions168 revérifiés. Aucune activationC05. Restaurer ce travail après la
livraison de cette correction ; préserver les deux ajouts au reuse-audit.
