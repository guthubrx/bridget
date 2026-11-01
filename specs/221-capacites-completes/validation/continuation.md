# Statut de reprise cohérent avec la commande native

Statut : correction intégrée, publiée et activée. Programme 52/182, In Progress.

## Incident et faits conservés
Signalement opus201-continuable-remote-ended-20261101-01 :
la tâche 5b00f328-14fa-4c05-9ce2-e33bec70ebb3 affichait
result_available/continuable=true ; Continue refusait remote_execution_ended.
Clé de ronde 6 : opus201-resoudre-alias-textures-rejets-ronde6-20261101-01.

Après clôture propriétaire : ronde 5, guard_released=true, cleanup_done=true,
idle_deadline_at=null. La commande worker était terminale, instance
9feb797d-5afd-4a98-89db-1f4b39b5ef29, génération 1746.
La raison historique exacte de fin reste inconnue. Le délai supposé de 900 s
n'est pas attesté par cette lecture après clôture.
Aucune mission, clé ou ronde historique n'a été rejouée ni reclassée.
Les produits et résultats restent conservés.

## Contrat livré par ce changement
L'ancien prédicat continuable regardait la présence du délai, sans comparer
l'heure et sans exiger une observation courante de la commande distante.

Le prédicat structurel ancien est conservé sous continuation_guard_open pour
la rétention et la conservation des vues. L'inéligibilité de reprise ne donne
aucun nouveau droit de nettoyage. continuable_at(now) vérifie explicitement
garde, remise, réparation, plafond de huit rondes et délai strictement futur.
L'égalité au délai est expirée. L'horloge reste fournie par l'appelant.

Continue et la projection partagent leur contrôle de présence native maître,
sans changer le contrôle ni la remise de Continue.
Un Status distant éligible demande une observation ciblée ExecutionStatus,
avec capacité ExecutionStatusV1 négociée. Délai total 2 s, écriture 250 ms,
hors mutex maître. Propriété vérifiée avant et après, puis comparaison de
mission, ronde, commande, instance, génération et époque avant projection.
Un ancien worker ne reçoit pas cette trame : reprise non confirmée, action
de mise à jour ciblée. Aucun nouveau format persistant, compte ou droit.

Le worker lit uniquement le reçu et l'identité OS du processus : naissance
et état non zombie. Aucun signal, wait, récolte, clôture de réservation,
rapport Git ou appel fournisseur. process_birth conserve sa sémantique pour
le drainage existant ; process_matches_live est une lecture distincte.
L'ancien appel à reconcile, relevé en revue R2, a été retiré de ce port.

continuable=true distant exige la présence maître et une réponse courante
Released/native_ready. Terminal et Refused ont des messages distincts.
Une erreur de lecture, une absence de preuve ou un ancien pair rend
unconfirmed, sans inventer un arrêt.
Les projections sans observation ciblée ne promettent pas une reprise distante.

continuation expose portée this_task, date, cause fermée, prochaine action,
décision humaine et requires_recheck=true. Une commande peut finir après
la réponse : Continue vérifie encore les faits avant remise.
Ne pas appeler Continue comme sonde : ce parcours peut fermer une garde
lorsque la présence manque. Les autres travaux suivent leur propre état.

## Contrôles ciblés
Trois contrôles spec221_continuable PASS sur Linux et Mac couvrent délai/rondes/présence,
RPC frais négocié et corrélé, ancienne capacité, changement d'époque,
Terminal/Refused/Released et absence de mutation de la tâche.
Le test worker emploie un vrai processus attendant stdin. Il constate :
lecture Released vivante, naissance étrangère refusée, sortie normale puis
zombie/absence non déclarés prêts. Reçu inchangé à chaque observation.
La fixture est libérée normalement par stdin, puis récoltée par le test.

Régressions utiles : native153_ronde_meme_enfant_rejeu_et_plafond,
native153_fondations_payloads149_150_et_predicats et
native175_conservation_expliquee_sans_mutation_ni_autorisation_de_retrait.
Les sources des assemblages Linux et Mac sont comparées sur 356 fichiers.
Pas de campagne générale ni de nouvelle mission métier.
Correction technique : pas de nouveau parcours d'interface ; preuves par tests.
Les reçus de tests, revue, installation et chargement sont privés dans
/home/moi/.cache/bridget221-continuable et
/Users/moi/.cache/bridget221-continuable.

Résultats ciblés : six tests distincts Linux PASS, trois mêmes contrôles sur Mac PASS.
Le premier essai Mac refusait un chemin TMPDIR trop long avant toute opération ;
le passage avec TMPDIR=/tmp réussit. Le test de rétention existant conserve
son admission non gardée : le délai seul ne rend plus son statut reprenable,
mais sa garde structurelle interdit toujours le retrait. Il passe en 63,43 s.
Le dernier ajustement de cette assertion est limité à cfg(test) ; il est copié
dans les snapshots, sans effet sur les exécutables release en cours.

## Revue et dernier complément
R1 : 71659ed0 ; R2 : ba84c4db ; revue ciblée finale 0b5a6415.
La revue R2 a bloqué l'appel à reconcile ; il est remplacé par la lecture pure.
La revue R3 approuve cette correction et l'absence de régression de rétention.
Ses deux remarques sont traitées : la projection brute reste fausse à distance ;
une identité de commande refusée expose worker_check/stage=command_identity et
le WorkerRefusal fermé, avant tout échange réseau. Les trois contrôles ciblés
repassent après ce complément (0,48 s). Aucun quatrième tour de revue.
Les trois revues sont lues, acquittées, clôture demandée une fois chacune.
Compilation et contrôle de processus macOS acquis, hors de la lecture du reviewer.

## Livraison active et limites
Source exécutable finale 06efa57b3bf9804466ec8597b812b025911a6a12,
intégrée à main et poussée. Build 221-continuable-r3, version 0.6.5.
Linux commun construit sur SYS : GLIBC maximum 2.34, 41 389 960 octets,
SHA256 5f98c104bf71c83521ac333a9197b62092ba27416b39513f339125bcf6d0f2b2.
Mac ARM64 : 34 172 608 octets,
SHA256 aa4924611702b6c82f0b90d81acfe6867d936263908002cb499e638dcac52b8b.
Les 356 fichiers d'assemblage sont identiques aux sources finales.
Les trois contrôles Mac restent valables sur les fonctions OS inchangées ;
les deux derniers changements de projection sont revérifiés sur Linux.
Pas de nouvelle campagne.

Sept images chargées et hachées : maître Cartae 3631851, mobile 3631889,
worker Cartae 3632577, worker SYS 1152425, worker KS 1522856,
worker Mac 41845, pont Mac 42142. Coupure maître : 2,18 s.
Wrappers Cartae 3405178/naissance300643209 et
2500061/naissance270585460 observés identiques avant/après.
Aucun service métier, résultat, clé ou mission historique changé.

Retour 221-workspace-busy-r1 vérifié par les lecteurs de formats dans les
deux sens. Pointeurs rollback Cartae/Mac écrits atomiquement sous verrou
natif, sans activation concurrente ; empreintes retour vérifiées, updaters repris.
Les sept services sont actifs. Les MCP déjà ouverts restent 221-view-sync-r1 :
aucun nouveau schéma requis pour ce statut JSON, aucun rechargement prétendu.

Réattestation native révision13 réussie. Première lecture pendant republication
des fils refusée primary_absent, sans doublon ; après constat du fil exposé dans
le journal du pont, même task_status réussi. Nouveaux champs continuation
réellement lus sur la revue close : continuable=false, portée this_task,
requires_recheck=true. capabilities revenu, delegations_halted=false,
45 options déclarées. Aucun nouveau témoin fournisseur ni reprise pour éprouver
une commande distante active : ce chemin est couvert par les tests corrélés,
pas présenté comme une mission réelle réussie.
Monuments et les deux autres parents ont été informés par messages natifs,
reçus in_flight conservés, sans prétendre leur lecture acquise.

Trois copies de transfert hachées retirées : 124 169 880 octets au total.
installed appelé Mac/SYS : staging non enregistré côté Mac et observation
processus incomplète SYS ; caches conservés sans forcer.
Workspace221 conservé pour la suite du programme. Les worktrees de revue,
branches et anciennes images non entièrement observables restent conservés ;
aucune suppression sur une simple absence de processus connu.
Les candidats intermédiaires installés mais non activés sont distincts de
l'actif et du retour ; ne pas les choisir comme retour depuis le seul numéro.
Programme global toujours 52/182 In Progress, aucune catégorie entière close.
