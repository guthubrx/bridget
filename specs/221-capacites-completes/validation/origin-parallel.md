# Lectures simultanées : attestation d'origine
Statut : correction active sur les sept services ; clients MCP déjà ouverts encore anciens.

## Signalement et cause
Message monuments opus201-incident-attestation-statuts-paralleles-20261031-01 :
quatre task_status simultanés, deux succès et deux refus
permission_attestation_unavailable / origin_session_reattest.

Le dispatcher worker refusait immédiatement au-delà de deux contrôles
d'origine, alors que le maître conservait jusqu'à 32 attentes. Le refus
ResourceLimit du worker était aplati en origin_proof_unavailable, puis en
permission_attestation_unavailable. Ce défaut reproduit la forme du signalement.
Les journaux antérieurs n'ont pas conservé la cause de chacun des deux appels :
leur attribution historique exacte reste inconnue. Les avertissements du pont
sur le volume des fils constituent un fait séparé, pas la preuve de cette cause.

## Correction
Une borne commune de 32 contrôles pour le maître et les lectures worker.
La préparation de projet garde son créneau séparé de un. Les compteurs worker
restent en dehors de la boucle de reconnexion. Il n'existe aucune file ou cache
d'attestations nouveau. Chaque appel exécute son contrôle indépendant avec ses
identifiants, son époque, sa preuve et son échéance ; les gardes de droits et
de rattachement avant/après réponse sont conservées.

Une saturation explicitement reçue ResourceLimit garde le code public
permission_attestation_unavailable et expose la cause fermée origin_proof_busy.
Le message demande d'attendre les contrôles en cours puis de relire les mêmes
task_id. Une opération mutante exige de vérifier son admission durable avant
tout rejeu strict. Aucun changement de compte, droits ou rattachement conseillé.
La portée reste cet appel ; les autres travaux suivent leur propre état.
Une cause différente ou libre n'est jamais projetée comme une saturation.

## Vérification ciblée
Quatre tests PASS : vrai dispatcher worker et socketpair avec deux contrôles
déjà retenus puis quatre lectures indépendantes ; plafond 32 et libération ;
classification du refus ResourceLimit ; portée et non-fuite daemon/MCP.
Aucune mission fournisseur, aucune tâche métier, aucune écriture dans leurs
reçus ou résultats. La fixture de lecture utilise une racine absente privée.
La première compilation a détecté un import et un type de lecteur manquants ;
ils ont été corrigés avant le passage réussi. Pas de campagne générale.
Log : /home/moi/.cache/bridget221-provider-lot/origin-targeted-r2.log.

32 est une borne de contrôles d'origine, pas un nombre garanti de missions.
Un programme ou un lien indisponible peut encore refuser son attestation.
Aucune réutilisation d'une preuve réussie pour autoriser un appel refusé.


## Revue indépendante
534fce05-a1ed-4165-a827-dcb85f44cf76 : pas d'objection bloquante.
Le chemin de reprise de démarrage effaçait aussi le motif busy. Ce complément
préserve uniquement ce code fermé et réutilise exactement le même refus guidé ;
toute autre cause garde le comportement antérieur. Le test vérifie l'égalité
de cette projection avec le refus direct. Aucun démarrage ni rejeu ajouté.

La réserve sur la fidélité du test est précisée : deux contrôles sont simulés
par leur compteur retenu, puis les quatre suivants passent par le vrai
dispatcher et le socket. Aucun fournisseur n'est simulé comme ayant attesté
des droits. L'affirmation de famine de préparation par les lectures est réfutée :
run_worker utilise deux compteurs distincts, origin_slots et
origin_preparation_slots, sélectionnés avant dispatch. Les bornes CPU/mémoire
de 32 lectures n'ont pas été mesurées. L'échéance est contrôlée par execute
avant l'opération puis après sa réponse, sans modification dans ce correctif.


## Chargement constaté
Source exécutable 357b3586e6e5e59f25f4ecbb9432ce7a8b69bfe9,
build 221-origin-parallel-r2. Image Linux commune construite sur SYS :
SHA256 d2645eda07e5c5404f4ab8b8b848c2c396f80747921087bf1e893bf5969301b8,
minimum GLIBC 2.34, 41274600 octets. Le candidat construit sur Cartae exigeait
GLIBC 2.39 : il a été refusé à la distribution, sans activation sur SYS.
Image Mac SHA256 d08217f7aa81b7ec55ce5316b140d701bfed9394ebe921b080c3ab3848ce8f7e,
34079312 octets. Les 291 fichiers Rust des deux sources sont identiques.

Images réellement chargées et hachées :
maître Cartae3061880, mobile3061917, worker3062574 ;
workerSYS1094196 ; workerKS1479787 ; workerMac78784 et pont78916.
Coupure maître 2,13s. Superviseur Cartae2500061/birth270585460 et
travail SYS1086003/birth65306369 observés conservés pendant leurs passages.
Aucun service métier ni mission ancienne relancé.

Retour : ancienne image active capacity-accounts-r1, compatible dans les
deux sens avec les formats conservés (dont budget256Mio). Pointeurs privés
Cartae/Mac écrits par l'écrivain atomique natif sous verrou, updaters repris,
activation=false relu. SHA Linux de retour632c4f2d389cdb5792baf354812c73917803f898bde5ea1dc5eebd4e500362e3 ;
Mac2c19c5c0292eb8b040373d10815dd7db86cd1ca278bd6bac9175f24d083999ca.

Après chargement du worker Mac, une seule série de quatre task_status
réels simultanés du même résultat de revue a réussi : quatre result_available,
aucun refus d'attestation, aucune mission créée. Après passage du maître,
bridget_session a réussi ; le premier catalogue a rendu primary_absent pendant
la reconnexion. Une lecture de la même tâche a ensuite réussi. Aucun nouveau
restart ni nouvelle mission pour ce transitoire. Les avertissements de volume
du pont n'ont pas été pris pour une preuve de révocation.

Limite d'activation : cinq processus MCP observés utilisent encore view-sync-r1
(42033,79830,80750,85165,90084). Leurs lectures passent par le worker corrigé,
mais leur projection locale du nouveau motif busy n'est pas déclarée chargée.
Leur configuration pointe sur le lien actif. Aucun port natif de rechargement
MCP n'est exposé dans ce fil ; le contrôle du bureau est indisponible
(CUA_REPL_ENABLED_SURFACES is required). Le geste utilisateur restant est
de quitter complètement l'application hôte puis rouvrir le même fil.
Ne pas annoncer tous les clients activés, ni faire une boucle de reprises.

Résultat envoyé à monuments par bridget221-origine-parallele-resultat-20261031-01,
delivery66866c89-c1be-48f6-ba6e-439925972335, in_flight sans preuve de lecture.
Les deux autres destinataires prévenus ont reçu l'avis de fin de maintenance.
Les admissions, résultats et clés métier restent identiques.

## Rangement borné
Transferts et archives devenus inutiles retirés :
Mac193863855octets, Cartae193851984, SYS35658695, KS41286471.
Actif, retour, preuves, sources de compilation et worktrees conservés.
installedMac refuse le staging non enregistré pour cette copie du dépôt ;
installedSYS garde les caches faute d'observation complète des processus.
Aucune réécriture de propriétaire ni purge forcée. Le worktree de revue est
conservé : résultat lu/ACK et clôture demandée, usage global non établi.
Le worktree221 porte la suite du programme, toujours52/182 InProgress.
Cette correction ne clôt aucune catégorie supplémentaire.
