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

## Livraison effective —31octobre2026
Source fonctionnelle0259ce31644e23292f11ff6ba818f2820e531bb1 intégrée/poussée, main distant vérifié ; code candidat identique à l'archiveb9befcb5.
Recette réelle LinuxR4 :4778entrées,4671fichiers,63584948octets,48,116s ; audit propre, source inchangée, aucun dépôt/worktree Git ni fournisseur, arrêt du groupe confirmé. Sortie :
/home/moi/.local/share/bridget-projects/bridget/.bridget-commit-views/bd6b3b8151a65633914901771804e41c7610691c48880b1ae8f1bbb28d8f04af
Reçu privé :
/home/moi/.local/share/bridget-projects/bridget/c19-sync-private-r3/receipt.json
Trois refus de recette conservés : optionlibtest incorrecte avant test, lecture Git non admise avant helper, ancien placement de recette refusé avant export. Lecture corrigée vers la forme locale/no-includes existante ; placement new_native identique à la production. Aucun changement des protections ou du binaire pour ces corrections de test.
Audit source final13/13 applicables, decisionpass, release_blockedfalse, validate=true ; notes63–75, limites de source conservées. Emplacement privé :
/home/moi/.cache/bridget221-c19-sync/audit-source-final

Build221-view-sync-r1 réellement chargé :
- Cartae maître2336251, mobile2336267, worker2339037.
- SYS worker1002993 ; KS worker1414011.
- Mac worker52603, pont52698.
Les cinq images Linux portent SHA5d04e3df… et les deuxMac SHA114c5939… ci-dessus. Coupure maître2,16s.
Superviseurs Cartae2500061/naissance270585460 et SYS977109/naissance63507446 observés conservés. Aucun service métier ni ancienne mission relancé.
Retours B existants vérifiés par SHA et preuves de helpers antérieures ; rapports formats recalculés dans les deux sens avant installation. rollback natif Cartae/Mac écrit sous verrou par atomic_json existant, backups avant/après conservés, updaters repris.
Lien natif revenu : session run634/révision2, catalogue45options, gel=false. Première lecture après passage non exploitable comme JSON, puis lecture native réussie ; aucune cause compte/réseau inventée.
Le client MCP déjà chargéB reste utilisable : même schéma C19, seules opérations serveur de fichiers et guidage changées. Pas de nouvelle boucle de fermeture de l'application.

## Dernière instruction utilisateur : priorité au développement
Les deux qualifications R2 sont préparées, NONenvoyées, sans tâche. L'utilisateur demande d'arrêter les campagnes répétées et de reprendre les développements par lots cohérents. Conserver ces enveloppes pour la validation ciblée du lot ; aucun remplacement d'issue inconnue.
Les originales restent cancelled. Claude conserve source_stopped/closed confirmés. Codex garde conservationunconfirmed : pas de preuve créée depuis l'absence de fichier, aucun nouvel essai.
T030/T174 restent ouvertes ; programme46/182 In Progress. Ne pas présenter l'activation de ce correctif comme la clôture de la catégorie ou du programme.
Notifications de maintenance envoyées aux trois parents connus, remise in_flight ; aucune lecture par les pairs revendiquée. Après passage, message de reprise distinct, sans demander de relancer une ancienne mission.

## Entretien borné
installed appelé sur SYS/Mac avec leurs images installées : purge refusée, propriétaire de cache lié à un autre staging enregistré. Aucune marque de propriété réécrite, aucune purge forcée. Conserver les caches et sources encore utilisés par le programme ; reprendre par leur propriétaire vérifié.
Actif et retour distincts conservés. Anciennes images encore utilisées par les MCP/superviseurs, vues partielles et preuves conservées. Aucune suppression sur observation incomplète. Ce correctif technique est prouvé par tests et recette native ; aucun diaporama d'interface n'est requis.
