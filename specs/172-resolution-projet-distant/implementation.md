# Implémentation 172

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-1).

Hypothèse confirmée : aucune sonde distante lorsque project_id est absent,
car la boucle fait continue avant la lecture du mapping existant.
Correction : résolution unique depuis le registre, refus explicites,
identifiant conservé dans RoutingCandidateObservation puis Delegate.
Tests : en cours. Activation et recette réelle : non effectuées.

Validation : 2 nouveaux tests PASS, 35 tests spec168 PASS et 1 test incident171 PASS.
Premier lancement de régression : 26 PASS/9 FAIL avec le vieux Git système ;
reprise avec le Git 2.56.0 du worker : 35 PASS. Aucun changement de test pour cela.


## Recettes du 18 octobre et correction des délais

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-4).

Négociation ancienne/nouvelle capacité et refus des doublons : 1 PASS.
Le premier ajout au test réutilisait une connexion déjà authentifiée ;
la garde StaleConnection l'a refusée. Le scénario utilise désormais une
nouvelle connexion, comme lors d'un remplacement réel du worker.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-5).

Validation : 69 tests de reconnexion PASS, 1 ignoré ; reproduction RED puis GREEN.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-6).

Recette Revue r5 réussie, task800dae46-f45a-4e7c-bc51-9d1e7dabba7d,
SHAac8f9295, revue0.1.0, marqueur exact, ACK et rejeu même tâche/résultat.
Deux tâches finales cleanup_done=true, un lancement chacune.
Recette supplémentaire parent natif local refusée resource_saturated avant
reçu/tâche ; aucune autre route. Limite conservée au reçu de livraison.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-8).

Le relais ancien162 ne reconnaissait pas ReleaseLeaseV2. Il masque maintenant
cette capacité pour ses cas historiques ; le nouveau cas négocie explicitement
la capacité moderne. Contrôle ancien : 1 test / 4 cas PASS en39,36s,
dont expiration du permis5s et libération de capacité après échec.
Ce lot ne change que le banc et ses documents ; aucune compilation release ni
activation supplémentaire. La recette avec un vrai modèle parent natif reste
à réaliser après retour de la capacité et de l'attestation.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-9).

Recette parent natif réel Cartae : request recette172-parent-natif-cartae-20261018-r2,
task7ed49c85-2860-4ef3-ad31-8d63d1a3184b. Parent lancé ; enfant refusé
workspace_base_preflight_failed avant création. Résultat lu et acquitté.
Cause dans validate_routing_source : lecture locale inconditionnelle du chemin
/home/moi/10.Scripts/64.bridget par le maître Mac. La source est sur Cartae.


## Retour de résultat et délai de l’hôte (18 octobre)
Une vraie tâche de codage Cartae a produit un test, dix tests Python PASS et
le marqueur BRIDGET172_CODAGE_CARTAE_OK dans son worktree isolé e125950f.
Task e125950f-9ceb-4dc3-92e6-2841e0566f05. Deux lectures MCP ont d’abord
expiré (« budget JSONL dépassé »). Le résultat complet est ensuite arrivé
par task_status ; ACK puis rejeu strict : même Task, même résultat, aucun doublon.
Ce succès différé n’a pas été présenté comme une correction de la panne.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-10).

Autre défaut reproduit : daemon_supports_wait convertissait toute panne de
transport en false, donc en wait_unsupported. Les outils d’attente, ACK et
la sonde d’admission conservent maintenant les erreurs techniques. Un rapport
de capacités valide reste nécessaire pour annoncer l’absence d’une capacité.
Cinq nouveaux cas RED sur l’ancien comportement, puis neuf cas172 PASS,
y compris la variante d’admission ; 19 régressions du client PASS.

Les essais des vrais parents ont révélé une autre coupure, côté hôte Codex :
Revue r6 a expiré deux fois à 30 s, sans reçu. Aucun reçu Revue r6 en base au
contrôle suivant. Opus2D r2 a été admis sous Task b00ae037-ed93-4106-9e4b-68fbee5af3d2,
puis a terminé failed avec remote_start_failed:permission_attestation_unavailable.
Ces essais ne sont pas comptés comme réussis. Les deux profils Codex du Mac
configuraient réellement le serveur Bridget à 30 s.

Régressions : 19 tests du client et 19 de délégation PASS. Les attentes MCP
ont donné16 PASS/1 ancien oracle : il attendait wait_unsupported sur une socket
absente. Cet oracle est aligné sur daemon_unreachable ; les cas anciens avec
rapport de capacités valide restent distincts. HTTP/attestation : huit PASS.


## Derniers défauts révélés par les vrais parents
Après installation40e4, Revue r6 reprise strictement a répondu en une seconde
expected_base_commit_mismatch : son HEAD avait changé depuis ac8f9295.
C’est le refus attendu pour une demande ancienne non admise ; aucun nouveau
SHA n’a été substitué dans sa clé. Opus2D r2 a été lu/ACK par son propriétaire.
Opus2D r3, SHAe3bb4066fbb4513d682cf156049445c658c52cd8, a été refusé
worker_unavailable pour dix candidats. Le service Cartae était actif et une
reconnexion apparaît deux secondes après le refus. Aucun résultat Opus2D final
n’est annoncé. Les rapports existent dans les demandes Bridget suivies ; deux
lectures parallèles depuis notre vieux MCP ont ensuite échoué au HTTP3s.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-12).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-13).


## Activation et recettes finales réussies
Source124dc47a9fe2ad2717d06c53053b71d9f0f7dc26 intégrée et poussée.
Build Linux101,11s RC0 ; Mac142,30s RC0, signé. Maître81018 et worker4049619
actifs ; T3main637/serveur1068/pont898 inchangés. Aucun schéma changé.
Avant arrêt du maître51581, aucune mission native active/cleanup en cours.
Après redémarrage, les rejeux officiels de c5d2017b et e125950f ont rendu les
mêmes résultats/Tasks/ACK malgré le changement de HEAD. Quick_check=ok.
Le banc natif par le parcours natif Bridget sur le code courant est PASS :1test/2cas,25,57s.

Nouvelle mission réelle Bridget : a62d8fb1-9fef-4c92-9d80-0d1e1b475790,
SHA124dc47a, version0.4.1, résultat833octets, marqueur exact, ACK et rejeu
identiques. La première sonde réelle est mesurée à4,426s pour dix candidats.

Revue r7 : Task30a2525b-7d62-4be8-a3a6-b5b750382009, sans project_id,
SHAd9d29fe69c9969af913cb99ce70fd249fc723b67, revue0.1.0,
BRIDGET172_REVUE_FINAL_OK. Le parent Rapport a lu le résultat complet,
confirmé ACK et comparé le rejeu strict : mêmes Task/résultat. Le dépôt géré
initialement créé sans déclaration manuelle est actualisé au commit demandé.
L’agent de recette a seulement lu package.json et .git/HEAD détaché.

Opus2D r4 : Taskea8d9680-dd4e-4a71-a22d-1589ee028f9f, sans project_id,
SHAed82547ff65fd23307f63ee6683c9e9e7b22d420, opus2d0.1.0,
BRIDGET172_OPUS2D_CARTAE_OK. Le vrai parent trajets bus a lu le résultat,
confirmé ACK et comparé le rejeu strict : mêmes Task/résultat1007octets.
L’enfant a lu package.json, .git/HEAD et refs/heads/main sous le dossier Cartae.
Les sondes initiales Revue et Opus2D ont pris4,317s et4,303s ; ces mesures
n’attribuent pas rétrospectivement une durée précise à l’ancien refus r3.
Contrôle durable : six clés de recettes réussies, chacune une seule Task,
state=result_available, error=null, cleanup_done=true, acknowledged_at présent.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-14).

## Continuation distante après renouvellement du tour parent

Incident réel VOITURES : la tâche54569150 garde une attestation du tour850,
révision17. Les deux contrôles de Continue comparaient le fait entier avec celui
du tour courant. Un renouvellement valide pouvait donc produire
permission_not_inherited sans changement de droits.

## Sortie fatale du worker pendant une reconnexion

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-15).


## Complément quota du 18 octobre — cause réelle de BUS et de la reprise VOITURES

La Task BUS8dcb54b1 et la reprise VOITURESb9d8b49c ont échoué avec api_error.
Les journaux natifs Claude donnent la cause structurée : error=rate_limit,
isApiErrorMessage=true. Le texte annonce « resets 1:20pm (UTC) », soit15h20
heure de Paris ce18octobre. Ce n’est ni une panne SSH ni un défaut de droits.
La reprise b9d8b49c n’a pas produit de code ; son worktree propre reste conservé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-16).

Limite explicite : aucune heure de retour n’est attestée par le code rate_limit seul.
Le texte humain n’est pas converti en fait machine et n’ouvre aucun hold160.
Une observation périmée redevient inconnue selon le TTL existant. L’authentification
et l’acceptation d’une nouvelle mission restent à prouver par l’essai réel.

Preuves ciblées privées, sous /home/moi/.cache/bridget-delivery172 :
quota-classification-red172.log (rate_limit rendu api_error avant correction),
quota-recovered-response-red172.log (notice incorrectement gardée comme réponse),
quota-routing-red172.log (fait perdu après sortie de l’enfant),
quota-remote-red172.log (sonde distante ignorant la limite), puis leurs GREEN.
34tests du pilote Claude,14tests de mémoire quota,13tests de routage,
1test de sonde distante,1test de liaison stricte et2tests de continuation PASS.
Le test de liaison refuse digest, instance, machine, gestionnaire ou compte ambigu.
Son cas artificiel qui tentait de remettre remote_prepared=false sur une Task déjà
préparée a été retiré : le magasin conserve ce fait de façon monotone. Donnée lue
avant retrait : valeur demandée false, valeur persistée true ; garde produit conservée.

Le défaut de fixture hérité env_unfit est résolu sans modifier le produit :
config-149 était créé0770 sous le masque réel du serveur, malgré le contrôle0700.
La fixture fixe maintenant0700. Les14tests native153_ronde_ passent.

La relecture conserve les mêmes limites de stockage, d’identité et de fraîcheur.
Aucun nouveau schéma, magasin, outil, paramètre ou dépendance. Le test adversarial
ajoute des crédits synthétiques au même historique : ils restent absents du fait
rendu après la sortie de l’instance. Les crédits Codex ne traversent pas ce cas.


## Complément après les essais réels r4 et r5

r4 Task14c382ea-c831-4374-b6b8-c9f77fe456a7 et r5 Taskc9693bca-4304-4e72-abe2-442677bc8c14
ont toutes deux fini usage_limit, puis ont été lues et acquittées par leur propriétaire.
Aucun working confirmé, aucun code produit. La décision conservée prouve le choix
claude-cartae / claude-haiku-5-5 dans les deux demandes. La sonde r5 a lu ses faits
44 à47secondes après le refus r4 ; le TTL de30secondes avait déjà supprimé le refus.
Les autres modèles du même compte ont aussi des digests de mission différents.

Correction en validation : empreinte privée de connexion issue du registre existant
(connexion, définition de base, révision de politique, identité/provisionnement worker).
Les modèles de la mission ne changent pas cette empreinte de compte. Un refus Claude
non levé reste restrictif après nettoyage et délai de décision, sans rajeunir son
observed_at. Les crédits et capacités positives restent soumis au digest complet,
à une instance vivante et au TTL. Une nouvelle réponse assistant sans erreur est
une preuve d'acceptation, sans prouver de crédit ou de pourcentage restant.
Une fenêtre structurée ne reçoit pas en plus un refus unknown sans date depuis
le code assistant. Aucun texte humain n'est transformé en date machine.
La mémoire reste volatile et bornée, sans schéma ni outil public supplémentaire.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-17).


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-18).

Le refus de rattachement garde t3_session_unavailable. Il distingue désormais
primary_absent, credential_mismatch, instance_mismatch, primary_not_live et
connection_already_registered. Les gardes sont identiques et les preuves privées
ne sont jamais imprimées.


## Complément du refus remote_result_slot:StoreUnavailable

Le signalement métier r7 porte la clé
opus2d-voitures-cartae-sol61-reprise-pietons-20261018-r7.
La lecture seule du maître retrouve pourtant une admission durable :
Task350d5c53-3b0d-4b1e-90b2-150002b6f51a, puis failed/remote_start_failed,
cleanup_done=true et guard_released=true. Le premier appel ne rendait pas
cet identifiant. Le propriétaire a reçu cet identifiant pour relire l’existant ;
aucune nouvelle admission métier n’a été créée.

Le store réel contient 73 commandes, 103 slots et 17 reçus de préparation.
Ses fichiers occupent 612964 octets. Le calcul ancien compte 15788644 octets
avec les réservations ; ajouter un slot exige 2228224 octets et dépasse
le plafond inchangé de 16777216 octets. Les anciens reçus soldés continuent
de réserver chacun 64 Kio, même après collecte de leur blob. La collecte existe
déjà et garde les reçus sept jours : la supprimer ou vider le store aurait
détruit des preuves de rejeu sans corriger ce calcul.

Le worker distingue maintenant le refus typé WorkerStoreError::Quota :
WorkerRefusal::ResourceLimit, au lieu de StoreUnavailable. Les erreurs techniques
conservent leur refus fermé sans publier leur détail privé. Aucun nouveau champ
de protocole, schéma de base ou outil n’est ajouté.

Trois régressions RED sont conservées avant correction : saturation au 106e reçu
terminé, quota masqué par StoreUnavailable, et perte de la réponse après admission.
Les premiers contrôles GREEN172 donnent 23 PASS. Le test du store conserve
120 résultats et reçus, vérifie leur relecture après réouverture et leur rejeu,
puis admet sept résultats actifs et refuse le huitième. Les séries de régression
complémentaires sont conservées dans les reçus de validation.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-19).


## Activation après clôture BUS et contrôle réel

Le propriétaire BUS a terminé la ronde2 et fermé sa garde. La lecture seule du
maître confirme cleanup_done=true, guard_released=true et résultat conservé.
Le groupe320286 est absent. Aucun autre mandat natif actif ni admission auto
ouverte : le paquet préparé72a8e8f8 peut être activé sans interrompre un calcul.

Maître7825 actif depuis21:15:02CEST ; worker624845 actif, NRestarts0.
Source72a8e8f8823c5f23b463a45693bb9b27079e41f3, build172-72a8e8f8.
Les PID T330609/30660 et pont898 restent inchangés. Les anciens binaires
et l’unité worker sont conservés pour retour arrière. Aucun schéma modifié.

Le premier appel officiel après la relance du maître refuse le rattachement
avec le motif fermé primary_absent. Le pont existant se reconnecte puis réinscrit
les26fils ; les appels officiels suivants status et capabilities réussissent.
Aucun credential emprunté, aucune identité réécrite, aucune nouvelle route.
Ce cas transitoire observé ne prouve pas la cause historique de la capture Claude.
Le journal du nouveau maître conserve les avertissements avec RUST_LOG=warn.

Recette officielle bridget_auto_delegate :
request_id recette172-store72-cartae-live-20261018-r1,
Task3a825355-37b1-45e1-8bdf-9aef6de39dd4,
enfantb58e755e-920c-4be7-9a42-887c577079cf.
Cartae est imposé ; les autres axes sont choisis par Bridget :
claude/claude-haiku-5-5. Posturedevelopment pour conserver un enfant continuable,
mission explicitement limitée aux lectures. Worktree demandé et créé par Bridget,
sans project_id : /home/moi/10.Scripts/64.bridget/.worktrees/3a825355.
Le SHA lu est5c2b4635e3bdeda78f3687f1b0a04b8bd6831744 ; Cargo.toml donne0.4.1.
Marqueur BRIDGET172_STORE72_LIVE_OK ; résultat complet891octets lu puis ACK.
Le rejeu exact garde TaskID et résultat byte-identique.

La continuation officielle sur le même enfant, clé
recette172-store72-cartae-live-20261018-c2, ouvre la ronde2 et lit la borne
MAX_WORKER_STORE_BYTES=16*1024*1024. Marqueur BRIDGET172_STORE72_CONTINUE_OK,
résultat complet853octets lu puis ACK. Le rejeu de cette continuation conserve
la même Task, la ronde2 et le même résultat. Aucune nouvelle exécution au rejeu.
Ce contrôle utilise le tour parent courant ; le renouvellement automatique
d’un tour parent distinct n’est pas qualifié par ces deux rondes.

Après la première ronde, le store réel conserve74commandes,104slots et18reçus
de préparation :619207octets physiques,1184455octets comptés avec les réserves
du nouveau calcul. Plafond inchangé16777216octets. L’admission et la publication
réelles réussissent sans purge manuelle des anciens reçus.

La garde du test est ensuite fermée par bridget_task_cancel. Résultat conservé,
cleanup_done=true et guard_released=true ; commande terminale et groupe626253vide.
BUS conserve aussi son résultat, ses deux preuves de clôture et son worktree.
Le propriétaire a reçu l’information de levée du blocage, sans nouvelle recette
demandée. La reprise métier VOITURES lui appartient et n’est pas une livraison
de code revendiquée par cette recette de lecture.


## Incident du 19 octobre — plafond MCP invalide après échec fournisseur

Trajets bus a conservé le refus de bridget_task_status sur la Task
62363500-c0cf-409b-b669-5549f33cfa7b. L’appel exact est daté du18octobre à
23:45:35.364UTC, refus à23:45:35.604UTC. Le constat a ensuite été consigné à
23:46:17.781388UTC dans le reçu métier externe. Les outils send et status du
même fil ont aussi rencontré ce refus. Aucune nouvelle admission n’a été créée.

La lecture seule du maître retrouve un événement durable provider_failed,
non acquitté, à22:48:29UTC, pour l’instance du parent trajets bus. Sa Task est
déjà failed/terminal Codex:failed, cleanup_done=true et guard_released=true.
L’erreur de plafond est distincte de cet échec fournisseur, dont la cause fine
n’est pas attribuée à un quota ou au contexte sans autre preuve.

Le serveur rejoue les avis DelegatedRuntimeEvent après tout Registered,
y compris RegisterAuxiliary. Le client MCP ouvre une connexion auxiliaire pour
CallerScope et attend un objet fermé version/caller_read_only. Il reçoit donc
cet avis d’incident avant la réponse à sa requête et refuse le protocole.
La présence du fait durable et le code du rejeu expliquent le défaut réel.

Le correctif reprend la garde déjà utilisée pour la remise d’incident en direct :
seule la connexion primaire reçoit le rejeu ; l’auxiliaire MCP n’en reçoit aucun.
Il ne supprime ni n’acquitte l’avis durable, ne saute aucun contrôle de droits,
et ne change aucun protocole ou schéma. La modification concerne le maître seul.

Le nouveau test utilise le véritable handle_connection avec des sockets Unix et
une inscription auxiliaire attestée. Avant correction il reçoit bien
DelegatedRuntimeEvent au lieu de CallerScope (RED). Après correction il vérifie
le plafond exact pour Warning et Failed, puis la remise et l’ACK par le parent
primaire. L’avis reste en attente tant que ce parent ne l’acquitte pas.
Reçu RED : /home/moi/.cache/bridget-delivery172/auxiliary-scope172-red.log.


## Erreurs de préparation distante lisibles — 21 octobre 2026

Demande utilisateur : laisser le nettoyage Cartae à son responsable et rendre le
refus de préparation compréhensible. Aucun fichier de cache distant supprimé par
ce correctif ; aucun dossier métier, reçu ou résultat modifié.

Cas réel : Task ba369015-da43-45c4-b70a-dae9509ea2db, reçue durablement avant
l’échec `remote_workspace_prepare:StoreUnavailable`. Son reçu de préparation
contient record=null ; aucun worktree ni enfant n’existe. À l’observation,
Cartae offrait environ 1,1 Go pour une extraction de 7 981 663 707 octets.
L’ancien code avait jeté le diagnostic Git ; cette erreur historique ne sera
pas réécrite en un nouveau diagnostic présenté comme observé.

Adaptation : Git est lancé avec LC_ALL=C. Les diagnostics fatals/error de
l’extraction distinguent espace insuffisant et accès disque refusé. Le worker
transmet seulement DiskFull, FilesystemAccessDenied, WorkspacePrepareFailed ou
WorkspacePrepareIncomplete. Les erreurs OS ENOSPC/EDQUOT sont distinguées des
plafonds de réservation ResourceLimit. Les erreurs de stockage non classées
restent StoreUnavailable ; aucune cause n’est inventée depuis ce seul code.

La réponse native ajoute error_message en français pour un échec de préparation.
Le champ error, l’identifiant de tâche, le status durable, la clé et le rejeu
restent inchangés. Les vieux refus StoreUnavailable reçoivent une explication qui dit
explicitement que la cause précise manque. Aucun stderr brut n’est transmis.

Tests et activation : voir le reçu de livraison et les journaux privés sous
/home/moi/.cache/bridget-message-errors172. Le travail s’effectue dans le
worktree existant de la session172 ; les services restent inchangés pendant
les tests et la préparation.

Validation initiale avant revue r1 : 10 tests distincts PASS, aucune suite complète. La simulation
Git ENOSPC passe par le vrai exécuteur Git et le nettoyage conditionnel de branche.
Les tests du worker couvrent les refus OS, les codes sérialisés et les détails
privés exclus. Le test de reçu maître vérifie les messages, le même TaskID et
le rejeu strict sans nouveau lancement. Les régressions d’extraction, de
préparation/rejeu d’un vrai worktree, d’admission et de réserve du store passent.
Un filtre vide a été exclu du compte, puis remplacé par les deux tests du store.


Complément après revue indépendante r1 : l’import local du commit utilise lui
aussi la classification Git. Les motifs doivent terminer la ligne de diagnostic,
ce qui évite de prendre un nom de fichier pour une erreur système. Le test ENOSPC
crée maintenant réellement sa branche avant l’échec et prouve son retrait.

14 tests distincts PASS après ce complément. La première exécution d’un nouveau
test de flux a refusé une forme status incomplète ; le test emploie maintenant
la forme déjà autorisée. La liste blanche Git n’a pas changé. Les paquets161df06a
préparés avant cette revue ne seront pas activés. Les nouveaux motifs du protocole
fermé imposent d’activer le maître corrigé avant le worker corrigé ; le retour
arrière doit suivre l’ordre inverse. Les anciens workers restent lisibles par le
nouveau maître. Aucune compatibilité ancien maître/nouveau worker n’est annoncée.


Revue r2 (Task832e891b) : APPROVE pour le code, avec précision documentaire.
Le total 14 désigne les tests exécutés, nouveaux tests et régressions ciblées,
et non le nombre de fonctions ajoutées. Liste exacte des 14 résultats PASS :
- workspace_git::tests::worktree172_disk_full_keeps_a_distinct_reason : PASS.
- workspace_git::tests::worktree172_failure_classification_uses_only_git_diagnostics : PASS.
- workspace_git::tests::worktree172_fetch_disk_full_is_classified_without_moving_checkout : PASS.
- workspace_git::tests::worktree172_stdout_flood_does_not_hide_stderr_reason : PASS.
- remote_worker::tests::worker172_workspace_refusals_preserve_cause_without_private_details : PASS.
- daemon::native_delegation::tests::spec172_remote_workspace_error_message_preserves_receipt_and_replay : PASS.
- workspace_git::tests::worktree171_failed_extraction_releases_only_its_unused_branch : PASS.
- remote_worker::tests::worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications : PASS.
- daemon::native_delegation::tests::spec172_remote_admission_error_conserve_identifiant_erreur_et_rejeu : PASS.
- workspace_git::tests::native154_git_environnement_et_options_effectives : PASS.
- remote_worker::tests::spec172_worker_store_conserve_replays_sans_reserver_les_recus_termines : PASS.
- remote_worker::tests::spec172_worker_store_quota_distincte_dune_panne_de_stockage : PASS.
- workspace_git::tests::native154_git_sorties_cumulees_bornees : PASS.
- workspace_git::tests::worktree171_branch_mirror_import_isolation_and_refusals : PASS.

Les commandes cargo compilent les tests une seule fois ; le même binaire de
bibliothèque exécute ensuite les dix filtres du reçu tests-r2.json. Les noms ont
été relus via --list et chaque compte comparé au résumé réel de son exécution.
Le journal complet et la liste calculée se trouvent dans
/home/moi/.cache/bridget-message-errors172/test-proof-r2.json.

Choix demandé par la revue sur les statuts : status reste l’état durable v1.
Le client lit work_state et error_message pour connaître l’échec de préparation.
Après cet échec dans le reçu d’admission, aucun résultat enfant n’est à attendre :
il faut corriger la cause et annuler cette tâche avant une nouvelle demande.
Le correctif ne transforme pas une erreur de préparation en relance automatique
et ne modifie pas les données durables pour corriger un libellé. Ce choix est
couvert par spec172_remote_workspace_error_message_preserves_receipt_and_replay.

Choix demandé sur le retour arrière : remettre d’abord le worker ET son client
sur l’ancien binaire, restaurer son unité, puis seulement l’ancien maître.
Un ancien maître ne sait pas décoder les quatre nouveaux motifs fermés.
Aucun mélange ancien maître/nouveau worker n’est annoncé comme compatible.
Les scripts d’activation et les sauvegardes de ce lot restent privés.

Réserves faibles r2 : les noms Debug existants restent le code technique v1 ;
les valeurs inconnues reçoivent un texte générique. Aucun nettoyage de répertoire
partiel n’a été ajouté. Le retrait conditionnel d’une branche demeure interdit
si un worktree l’utilise ; les données et résidus sont conservés. Le nouveau test
ENOSPC prouve uniquement le retrait de sa branche créée et inutilisée, sans
prétendre valider la suppression d’un worktree partiel.


## Canal worker et réservation du résultat — 21 octobre 2026

Cas réel : Task837972ad-ad58-4512-9a71-fd2843ab5be5, clé
opus158-facades-production-cartae-20261021-03. Le dossier distant a été créé.
Le journal maître constate la fermeture du canal à 07:32:38 UTC puis un nouveau
rattachement à 07:32:39. Le service worker n’a pas redémarré : PID624845,
NRestarts=0. La cause fine de cette coupure n’est pas prouvée.

La réconciliation existante a terminé l’échec : état failed, cleanup_done=true,
guard_released=true ; reçu worker terminal, permit=null, native_ready=false.
PID416678 absent et groupe416678 vide au contrôle. Aucun fournisseur n’a reçu
la mission. Le worktree et sa base f9db3810812a7369808cf930d5a4de5e999ed5dc
restent conservés. Aucun cancel, rejeu ou remplacement effectué par ce lot.

Défaut distinct trouvé dans le lecteur : sa file bornée contient huit demandes.
try_send(...).is_err() confondait saturation et disparition du consommateur.
La neuvième demande fermait donc le canal et provoquait abandon_held.
Deux tests ont reproduit ce comportement avant correction (RED2FAIL attendus).

Les fins de lecture du canal reçoivent des diagnostics fermés : eof,
frame_read_failed, unexpected_frame, inbound_receiver_closed. Aucun corps de
trame, jeton, stderr ou configuration n’est imprimé. Ces diagnostics aideront
à distinguer les prochaines coupures ; ils ne reconstruisent pas celle de837972ad.

Autre défaut concret : le nouveau texte lisible couvrait la préparation du
dossier, mais pas remote_result_slot. Ce refus dispose maintenant de son message
et de son action. WorkerUnavailable/StaleConnection signifient que la réservation
n’a pas été confirmée ; le texte ne prétend plus prouver une panne du service.
Un refus ResourceLimit reste distinct d’un défaut du disque.

Validation GREEN :17 tests distincts PASS, soit les14 contrôles précédents
rejoués et trois nouveaux tests. Aucune suite complète. Les deux nouveaux tests
de file sont worker172_inbound_backpressure_preserves_all_requests et
worker172_inbound_backpressure_stops_during_shutdown. Le troisième est
spec172_remote_result_reservation_error_messages_wait_for_cleanup.
La liste exacte, les12 filtres et leurs résumés réels sont conservés dans
/home/moi/.cache/bridget-channel172/tests-r3.json.
Le RED est dans /home/moi/.cache/bridget-channel172/queue-red.log.

Une première sélection GREEN s’est arrêtée sur un filtre vide après un renommage
involontaire du nom du test existant. Son nom initial est conservé ; le filtre
vide ne compte pas comme résultat. La sélection complète17PASS est celle
du reçu tests.json.

Code validé par les tests ; revue indépendante et paquets encore à préparer
à ce checkpoint. Aucune activation pendant les autres missions Cartae.
Le défaut de file est reproduit, mais il n’est pas présenté comme la cause
prouvée de l’incident837972ad.


### Contre-revue r3 et correction de la surcharge

La revue r3 (Task6bddf1b0-88e8-45c7-ac5d-c4ac098bb137) a interdit
l’activation de1c5d8b6e. Son résultat complet a été lu et acquitté.
Les paquets Mac/Linux1c5d8b6e sont marqués non activables. Aucun service changé.

L’attente de10ms est retirée. La file conserve huit demandes. Une neuvième
demande reçoit immédiatement un refus ResourceLimit corrélé, sans entrer dans
la file et sans fermer le canal. Le lecteur continue donc à lire les annulations.
Le marqueur d’annulation est posé avant la remise ou le refus, comme auparavant.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-20).

Le test de saturation utilise la vraie sérialisation et une paire de sockets Unix.
Huit demandes sont retenues, la réservation supplémentaire est refusée ; le
consommateur reste bloqué au-delà de trois secondes, puis sa file est drainée.
Aucun slot n’a été créé et la même connexion accepte ensuite une autre demande.
Les retours des sondes v1/v2 et une annulation passent aussi par ce canal réel.

Réserve r3 sur une réservation déjà exécutée mais inconnue du maître :
le code existant finish_result_slots ferme tous les slots de la commande lors
de la réconciliation terminale, y compris ceux absents de Task.remote_result_slots.
Il est conservé. Un test direct d’un slot tardif puis d’un reçu terminal prouve
producer_terminal=true et closed=Cancelled. La saturation n’est donc pas traitée
par un effacement de reçu, un élargissement du quota ou une fausse disposition.
Cela ne transforme pas l’incident837972ad en cause de saturation prouvée.

Réserve r3 sur le dossier non encore vérifié : une tâche terminale avec
cleanup_done=true mais sans rapport de dossier reste waiting_for_children.
Le message demande maintenant d’attendre cette vérification. Il n’autorise
la nouvelle demande qu’après ce rapport. Le nouveau test pose un vrai
WorkspaceRecord dans le reçu, puis la correspondance workspace_report_mission.
La garde de rapport existante n’est pas retirée.

Les refus locaux remote_result_slot_limit, _mismatch et _stale reçoivent aussi
un texte et une action. Les refus connus de droits ou d’identité ne sont plus
présentés comme une cause inconnue. Le champ error reste volontairement inchangé :
son producteur WorkerRefusal est un enum Copy fermé, sans texte libre. Le suffixe
private-secret du test est une entrée artificielle pour vérifier le texte
générique ; le correctif n’ajoute aucune source de texte privé au code durable.

Validation r4 :19 tests distincts du démon PASS, dont14 régressions précédentes,
quatre tests du lecteur et un test des messages/rapports/rejeux. Un test ciblé
du protocole PASS séparément : routing168_ids_et_probe_v2_sont_fermes.
Aucune suite complète. Les noms et comptes exacts sont conservés dans :
/home/moi/.cache/bridget-channel172/tests-r4.json
/home/moi/.cache/bridget-channel172/transport-test.json

Le premier appel de revue r3 avait renvoyé workspace_base_unavailable avant
toute tâche ou admission durable. Le commit était présent et une lecture Git
durcie a réussi. L’unique reprise strictement identique a admis la revue ; la
cause fine de ce refus ponctuel n’est pas reproduite. Aucune nouvelle clé, aucun
autre moteur et aucune CLI de délégation utilisés pour contourner ce refus.

Code et tests r4 prêts pour contre-revue ; ni activation ni nouvelle mission
métier ni qualification réelle du correctif actif revendiquées à ce checkpoint.


### Contre-revue r4 et échéance de réservation — état courant r5

La revue r4 (Task31923c1f-074c-4e2d-84a6-669735cf5227) refuse l'activation
de6da8857c. La réponse complète est lue et acquittée. Les états r3/r4 ci-dessus
sont des checkpoints historiques, pas la définition du code courant.

Deux reproductions ciblées avant correction ont observé les données réelles
du WorkerStore : une réservation expirée était acceptée et écrivait un slot ;
un rejeu après marqueur d'annulation était aussi accepté. Ces deux tests RED
échouaient comme attendu. Journaux :
/home/moi/.cache/bridget-channel172/deadline-red.log
/home/moi/.cache/bridget-channel172/deadline-red.json

La file continue à refuser immédiatement sa neuvième demande avec ResourceLimit.
Une demande admise dans la file porte maintenant une échéance explicite pour
la réservation : ReserveResultSlotUntil. La capacité ResultSlotDeadlineV1 est
annoncée seulement si le maître propose worker_result_slot_deadline_v1, puis
confirmée au rattachement. Le maître ne transmet jamais cette opération à un
worker qui n'a pas négocié cette capacité. L'ancien message ReserveResultSlot
reste byte-compatible ; il sert pendant la mise à jour progressive. La
protection contre une réservation expirée exige les deux composants corrigés.

Le worker vérifie l'échéance après sortie de file, après acquisition du verrou
du store et après son contrôle de capacité, avant la création du reçu. Le
nouveau refus fermé ResultSlotExpired ne contient aucun texte système. Une
annulation déjà observée interdit toute réservation, y compris le rejeu d'un
slot existant. L'entretien existant traite aussi le marqueur d'annulation si
la demande Cancel n'a pas pu entrer dans la file saturée.

ReadyToRelease signifie que le préparateur est prêt mais encore retenu :
la mission métier n'est remise qu'après Release. Si la réservation échoue à
cet endroit, le maître demande l'arrêt de cette même commande. Il conserve
l'erreur initiale. Une absence de confirmation laisse le nettoyage en attente ;
aucun cleanup_done ou arrêt fictif n'est écrit. Aucun nouvel identifiant métier,
rejeu automatique ni fournisseur de remplacement n'est créé.

Le texte français dit maintenant « Le travail métier n'a pas été remis à
l'agent ». Il ne confond plus ce fait avec le démarrage du préparateur.
Le chemin de réservation des rondes ultérieures conserve sa réconciliation
existante. Un état inconnu ne libère ni garde ni résultat.

Validation ciblée r5 :26 tests distincts du démon PASS et2 du protocole PASS.
Les23 premiers figurent dans /home/moi/.cache/bridget-channel172/tests-r5.json.
Les3 contrôles supplémentaires et les2 tests du protocole sont listés dans
/home/moi/.cache/bridget-channel172/proof-r5.json. Cette preuve conserve les
noms exacts et les résumés réels ; ce n'est pas le nombre de tests ajoutés.
Les scénarios incluent la vraie sérialisation sur socket, une demande reçue
avant échéance puis consommée après celle-ci, le verrou du store retenu au-delà
du délai, une saturation de plus de3s sans slot créé, la même connexion réutilisée,
l'annulation et les sondes v1/v2. Aucun fournisseur ni service lancé par les tests.
Aucune suite complète exécutée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-21).


### Contre-revue r5 et annulation supervisée — état courant r6

La revue r5 (Taskc22c8591-96c4-4785-9090-a9525678d231) a confirmé les
corrections de réservation, mais refusé l'activation pour deux défauts du
nouveau chemin d'entretien. Son résultat complet est lu et acquitté.

Les deux cas ont été reproduits avant correction. Un reçu sans son fichier de
marqueur provoquait StoreUnavailable au tick et empêchait la commande suivante
d'être réconciliée. Un processus de test qui conserve son groupe après SIGTERM
recevait trois signaux pendant trois ticks. Le reçu et son slot restaient ouverts,
ce qui était correct ; la répétition du signal ne l'était pas.
Preuves RED : /home/moi/.cache/bridget-channel172/cancel-red.log et cancel-red.json.
La première compilation du test a signalé un nom de type erroné dans l'assertion ;
la preuve RED retenue est l'exécution suivante,0PASS/2FAIL, pas cette compilation.

L'erreur d'annulation reste maintenant limitée à sa commande. Elle ne fait
plus échouer l'entretien des autres commandes. Les erreurs globales de santé
ou d'inventaire du store gardent leur refus fermé existant. Une identité vérifiée
reçoit une seule tentative de signal pendant la vie du worker. Les appels
suivants observent son drainage. Cette mémoire est bornée par les commandes
encore présentes dans le store et n'accorde jamais une clôture fictive.
Une erreur ou un processus encore vivant reste en attente, sans SIGKILL.

Les deux tests de maintenance passent maintenant par runtime.tick, pas seulement
runtime.reconcile. Ils prouvent que la seconde commande continue après l'erreur,
qu'un seul SIGTERM est envoyé, que le slot reste ouvert tant que le groupe vit,
puis que le reçu et le slot ne ferment qu'après sortie et preuve de drainage.
Le processus de test est fini par fermeture de son entrée, sans signal forcé.

Le test maître supplémentaire appelle réellement reserve_remote_result_slot.
Il lit la trame sur socket et vérifie les deux couples de versions : ancien
worker → ReserveResultSlot inchangé ; capacité négociée → ReserveResultSlotUntil
avec une échéance future en millisecondes, bornée à3s. Un refus corrélé est
rendu au maître sans ajout de slot. Les anciens reçus sont préservés.
Deux essais de montage du test ont révélé les contraintes existantes :
registre privé0600 et interdiction de retirer un ancien slot. La fixture respecte
ces règles ; aucun droit ni invariant de production n'a été assoupli.

Précision sur starting (réserve r5 non revérifiée par le relecteur) : après
l'échec de réservation, l'arrêt explicite de la même commande est demandé
dans start_remote_task. Une Observation terminale est traitée par
handle_worker_message, qui appelle fleet.invalidate_for_stop. L'entretien natif
reconnaît ensuite DesiredLifecycleState::Stopped et passe la tâche starting
sans mission remise à failed, avant la clôture habituelle. Ce chemin ne dépend
pas du délai métier réservé à working. Sans preuve d'arrêt, l'attente demeure
volontaire : aucun état terminé n'est inventé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-22).


### Contre-revue r6 et régressions de l'arrêt — état courant r7

La revue r6 (Taskfaa1352b-4f40-48d1-9d15-69af985084d5) est lue et acquittée.
Elle confirme le chemin starting vers failed après Observation terminale.
Elle a trouvé trois défauts réels restants, reproduits avant modification :
échéance du préparateur avec marqueur absent provoquant un tick en erreur ;
annulation avant marqueur laissant Reserved ; préparateur retenu abandonné sans
signal alors que sa clé était déjà marquée. RED0PASS/3FAIL observé :
/home/moi/.cache/bridget-channel172/stop-r7-red.log et stop-r7-red.json.

La même branche supervise maintenant l'annulation demandée, l'échec de démarrage
et l'échéance du préparateur. Une erreur de cette commande est journalisée puis
son observation reste tentée. UnknownExecution ne supprime plus cette observation.
Les erreurs globales de santé, d'inventaire et de clôture des résultats gardent
le refus fermé antérieur ; elles ne sont pas présentées comme un succès local.

La fermeture du canal RELEASE n'est plus mémorisée comme un signal. Le
préparateur abandonné reste supervisé et peut recevoir SIGTERM au passage suivant.
La clé du signal n'est mémorisée qu'après succès de l'appel d'arrêt. Un refus
permet donc une nouvelle tentative ; après un envoi accepté, seule l'observation
du drainage continue. Le test d'un échec d'envoi utilise un callback contrôlé ;
il ne tente jamais de signaler un processus d'un autre utilisateur.

Le test retenu lance un vrai préparateur de test par spawn_managed_bootstrap,
avec groupe et statut Ready réels. Il ignore la fermeture du canal et écrit la
preuve du signal reçu. Son fichier d'arrêt garantit la fin du test, même en RED.
La fixture d'erreur de marqueur utilise désormais un groupe de test réellement
vivant : le groupe fictif absent pouvait légitimement être clos par la preuve
de groupe de contrôle vide existante. Aucun invariant de production n'a changé
pour adapter cette fixture.

Validation :34 tests distincts du démon PASS (17 filtres non vides),2 tests du
protocole conservés, aucune suite complète. Tous les noms et empreintes des
quatre fichiers Rust sont actualisés dans validation/canal-worker172-tests.json.
La preuve privée post-commit rattache ces mêmes octets au commit exact.
Le test existant spec168_rollback_admission_sql_garde_chosen_puis_reprise_sans_decideur
passe également : après changement d'époque du maître, le choix durable est
repris sans nouvelle décision et crée exactement une Task.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-23).


### Revue r7 approuvée et condition de test levée

Task717dcdea-1c6f-4b11-baa5-792feffc1327 : APPROVE sur le code de production,
aucune régression trouvée. Résultat complet lu et acquitté. Le relecteur demande
une garde de sortie pour le processus held en cas de panic du test. Cette garde
Drop écrit maintenant le fichier de sortie avant tout retour de la fonction.
Le test held seul a été recompilé et rejoué :1PASS. Il recoupe les34 tests
précédents et n'augmente donc pas le total distinct.

Le code avant le module cfg(test) est byte-identique à95c6ec4c, source approuvée.
Aucun nouveau changement de production après cette revue. La couverture D4
reste un défaut de signal injecté dans le callback ; aucun refus réel de killpg
sur un processus étranger n'est revendiqué.

Politique confirmée (réserve3) : l'erreur propre à une commande la laisse non
terminale ou Unknown et reste visible aux lectures explicites et dans le journal.
Elle ne ferme plus le canal des autres tâches. La santé, l'inventaire et la
clôture globale du store continuent à échouer de façon fermée. Aucune corruption
n'est traitée comme un reçu Terminal ou un signal réussi.

T045/T046 sont closes sur source, tests et revue. T044 reste ouverte pour
l'activation réelle et sa lecture MCP. Aucun paquet refusé n'a été activé.


### Activation et qualification réelle du lot

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-24).


### 22 octobre 2026 — Identifiants opaques des fils adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-25).

Preuve RED : 1 PASS / 1 FAIL sur l'identifiant réel mcp:, refus binding_unavailable.
GREEN : 7 tests distincts PASS, dont identifiants UUID/mcp:/thread:mcp:, tailles et
contrôles invalides, version, identité étrangère, auxiliaire, projet non attesté,
ambiguïté, reconnexion et invalidation des suivis. Aucune suite complète, aucun
fournisseur lancé par ces tests. Preuves privées :
/home/moi/.cache/bridget-identifiants172/tests.json

Revue et activation restent à qualifier ; ne pas déduire une activation des tests.


Revue r1 7cd09586 : invariants de sécurité confirmés ; ajustement requis sur les
caractères de format Unicode. Réutilisation de bridget_core::is_disallowed_control
comme lineage (Cc et Cf), sans changement de la borne 2048. Ajout U+202E et U+200B.
RED spécifique Cf : 0 PASS / 1 FAIL. GREEN r2 : 7 tests distincts PASS, identifiants
opaques admis et caractères invisibles refusés. Le commentaire distingue la borne
de SessionProof de son filtre, plus permissif ; aucune modification hors liaison.
Revue r2 et activation encore à qualifier. Preuves :
/home/moi/.cache/bridget-identifiants172/tests-r2.json


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-26).

Recette réelle close : appel officiel unique bridget_capabilities réussi depuis
le fil mcp:f5a66bd1-7cef-4d2c-888b-06dd7d8263d3 à 09:38:18 CEST.
Aucun refus d'attestation ; aucune mission créée. Le reçu ci-dessus est actualisé.


### 22 octobre 2026 — Coupure du maître pendant une mission distante

Incident établi sur la ronde2 de la Task1ff9a4d5 : ACK fournisseur acquis,
mais turn_end/cancelled à 09:36:55 CEST exactement à l'arrêt du maître.
Le worker reste vivant ; son wrapper natif sort sur EOF lorsque child_requests=deny.
La conservation du runtime worker ne suffisait donc pas à conserver le fournisseur.
Le contrôle de livraison précédent ne prouvait que les identités et les compteurs
persistés. Il est insuffisant ; ne pas le présenter comme preuve d'exécution continue.
Worktree e7d6d67b propre et journal original préservés. Aucun résultat2 final publié.

Preuve isolée : vrai daemon, worker, wrapper et fournisseur fermé sans réseau.
Nouveau test spec172_remote_wrapper_survives_master_restart_without_second_provider :
RED 0 PASS/1 FAIL (processus fournisseur absent après restart) ; GREEN 1 PASS.
Il exige le même PID/naissance fournisseur, le même wrapper/worker et la même commande,
puis un résultat unique après reconnexion, sans second lancement ni deuxième tour.
Le test réemploie les contrôles de rejeu et de résultat de real_master_case.
Régressions, revue et activation encore à qualifier à ce checkpoint.
Preuves privées : /home/moi/.cache/bridget-remise1ff172.

Validation finale du delta : quatre tests E2E distincts PASS, sans suite complète.
La recette historique162 annonçait le bail de slot172 sans le simuler : refus
WorkerHello avant tout scénario. Son relais masque maintenant cette capacité,
et son cas d'expiration attend le vrai bail60s au lieu des anciennes5s. Les cas
maître réel conservent la négociation complète. Aucune tolérance ajoutée au runtime.
Clôture façades effectuée une seule fois par son propriétaire : cancelled,
cleanup_done=true, guard_released=true, six commits propres conservés sur e7d6d67b.
Aucun résultat2 ni mission de remplacement. Revue et activation restent attendues.


Contre-revue r1 893bfc18 : le défaut est confirmé, trois réserves avant activation.
1. Reconnexion désormais bornée à cinq minutes pour un contexte distant. Le délai est
contrôlé pendant le backoff, avant et après l'inscription ; le signal d'arrêt garde
priorité. Une panne prolongée ne peut donc maintenir cette attente sans fin.
2. Commentaire du relais précisé : seuls les cas SANS relais gardent toute négociation.
3. RED rejoué avec le fichier de test final identique à celui du GREEN (empreintes
conservées), en retirant seulement les deux conditions du correctif. Le fournisseur
est absent à l'assertion de naissance APRÈS l'arrêt/restart, même avec le harnais final.
Le premier RED reste historique, ses numéros de lignes ne sont pas une preuve finale.
La naissance de marker.pgid et le PPID du fournisseur prouvent aussi le wrapper réel.
Disconnect explicite demeure un arrêt dans la boucle managed, distincte de l'interactif.

GREEN r2 : sept tests distincts PASS. Ajouts : échéance de reconnexion sur la fonction
réelle, fournisseur simulé qui termine pendant la coupure puis résultat unique,
redémarrage du worker sans second fournisseur. Les natifs locaux, expiration avant
release et annulation sont aussi rejoués. Aucune suite complète ni réseau fournisseur
réel dans ces sept tests. Source et journaux :
/home/moi/.cache/bridget-remise1ff172/final-tests-r2.json
Le remplacement du worker préserve donc ses fournisseurs déjà relâchés ; le maître
reste en place pendant cette livraison. Une nouvelle validation MCP réelle en lecture
seule est prévue après activation, sans fabrication métier. Revue r2 attendue.


Contre-revue r2 9d063b81 : APPROVE avec réserves mineures, aucun défaut bloquant
sur le code. Résultat complet lu puis ACK. Avant activation : commentaires de grâce
précisés ; assertion daemon.is_none tautologique et assertion sur is_alive constant
du mock retirées. Ces changements ne changent pas la décision de production testée.

Décision du responsable de cette correction : grâce distante de300s retenue dans
le périmètre autorisé. Le redémarrage réel observé du maître a pris environ4s ;
cette marge tolère son démarrage et la reconnexion réseau sans attente infinie.
Ce n'est pas une garantie de continuité pendant une panne prolongée. Une tentative
bloquante déjà engagée peut dépasser l'échéance d'environ20s ; chaque coupure
réinitialise la grâce. Après expiration, la sortie existante arrête le fournisseur.
Cela change le comportement de panne ; aucun droit fournisseur n'est ajouté.

Limites de preuve conservées : le RED/GREEN porte sur le tour actif ; la fin de tour
pendant coupure est GREEN, sans RED séparé. L'annulation durant une coupure et l'arrêt
du fournisseur après300s ne sont pas qualifiés par un modèle réel. Aucun succès global
ni preuve de tous les scénarios. Le paquet reprend la source de production50b0673f
revue et testée ; le commit suivant précise des commentaires et retire deux assertions
inertes seulement. Compilation et activation doivent être vérifiées par leur reçu.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-27).


### 22 octobre 2026 — Refus de continuation lié à un processus distant terminé

Le signalement Notre-Dame d7ddb20c révèle un diagnostic trompeur dans `remote_dispatch_proof` : `Terminal` était converti en `permission_not_inherited`. Le worker enregistre cette fin à 07:36:56 UTC ; la continuation refusée date de 07:45:42 UTC. L’ancienne rupture du wrapper explique l’exécution terminée ; elle ne justifie pas de présenter cet état comme un nouveau besoin de droits.

Correction isolée : codes `remote_execution_ended` et `remote_execution_unconfirmed`, messages constants, contrôle de condensat inchangé. Trois tests ciblés PASS au premier checkpoint : requête Reconcile sur socket Unix, réponses simulées dans le canal interne, comparaison du seul enregistrement de tâche. Ce checkpoint ne qualifie ni le décodage entrant worker ni l’invalidation du bail. Pas de FULL ni de fournisseur dans ces tests. Revue indépendante et activation encore en attente à ce checkpoint.

Preuve : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md.


Contre-revue r1 du complément : APPROVE sur la classification, cinq réserves traitées dans r2. Le texte n’affirme plus qu’un processus a tourné quand le lancement est Refused. Les conseils passent aussi par detail.message, voie déjà transmise par le client MCP chargé. Le condensat absent reste un état non confirmé, sans cause de droits inventée. Quatre tests distincts PASS, dont un test du client MCP sur socket et huit cas de preuve distante ; aucun fournisseur ni FULL. La réponse worker du test de preuve est simulée dans le canal interne ; l’invalidation réelle du bail reste celle du code antérieur. Les deux empreintes de source sont dans green-r2.json ; RED r2 sur la même fixture finale.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-28).


## 22 octobre 2026 — Préparer les worktrees liés distants

La sonde traduit le chemin du worktree Mac, mais le helper importe seulement les objets du dépôt principal. Le dossier distant manque.

Le helper reconnaît cette identité, prépare un checkout détaché au commit exact et garde le mapping existant. Aucun reset ni transfert des fichiers non committés. 15 tests Python ciblés PASS, RED final discriminant conservé. Activation et recette réelle à qualifier.

Reçu : validation/worktrees-lies-20261022.md.


### Contre-revue worktrees : bornage au checkout reel

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-29).


### Worktrees r4 — refus propre rejouable et avance du checkout lie

Objections c51603b8 traitees : retirer le recu uniquement sur absence prouvee de dossier ET inscription apres refus synchrone ; garder les issues inconnues fermees. 21 Python PASS, nouveau RED discriminant, test worker etendu a cinq cas dont avance rapide C0 vers B sans toucher le principal. Les trois autres Rust de r3 sont conserves, production worker identique. Contre-revue finale avant activation.


### Livraison worktrees et Git — 22 octobre 2026

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-30).


## Correction du 22 octobre — transfert différentiel sur branches divergentes

- Périmètre : helper de préparation Git et tests ciblés ; aucune nouvelle mission Notre-Dame ni rejeu automatique.
- Constat : si le HEAD distant n'est pas ancêtre de la base demandée, l'export existant abandonne toute exclusion et copie l'historique complet. Le timeout Notre-Dame n'est pas encore attribué à cette phase.

- Auto-revue : le nouveau refus nommé de délai passait dans le nettoyage des refus synchrones. Test RED par TimeoutExpired injecté au vrai subprocess.run : reçu supprimé. Correction : conserver tout reçu après un refus de délai ; test GREEN via le vrai wrapper Git. Les erreurs Git et SSH testées restent des codes exacts sans argument privé.
- Qualification sans mission métier : export réel fd4 ^e89, 7 610 389 octets en 0,504 s ; bundle verify réel sur Cartae réussi, HEAD distant inchangé. Aucun import dans le projet ni agent Notre-Dame lancé par cette qualification.

- Revue r1 : Task daab8723-2447-4de9-8c58-1d818a91a81d, APPROVE sur 10eaa0e7, trois remarques non bloquantes traitées avant livraison : tous les imports du miroir et du nouveau checkout utilisent bundle_import ; les sept codes de délai sont vérifiés à l'égalité ; la source doit connaître le HEAD distant pour calculer la base. Sans ce fait, le comportement sûr préexistant reste un bundle complet et le gain ne peut pas être promis.
- Le délai Notre-Dame initial reste sans phase rétrospective. Le transfert réduit est mesuré ; sa causalité avec le délai initial n'est pas démontrée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-31).


## Correction du 22 octobre — checkout demandé sous le projet parent

- Cas réel : la couverture a5c3607c-e4d3-42ab-815d-54d0a0b0ac6c tourne sur Cartae depuis le cwd racine. Sa contre-revue depuis /Users/moi/Nextcloud/10.Scripts/69.opus2D/.worktrees/158-monuments-realistes est refusée avant Task avec project_preparation_git_root_outside_parent. Aucun rejeu ni interruption de couverture par ce lot.
- Cause vérifiée : prepare_routing_projects transmet parent.cwd comme source_root. prepare() exigeait top == source_root même si cwd désignait un checkout lié inclus dans le projet autorisé.

- GREEN : 28 tests Python ciblés PASS sur Cartae. Le checkout lié attendu est préparé avec un seul mapping, les deux principaux restent inchangés et les trois refus ont lieu avant tout transport.

- Revue r1 c7c7f9f0 expirée à180s sans verdict, clôturée. Revue r2 329f8a8f : défaut concret sur le premier worktree d'un projet non déclaré ; créer K_W puis K_P rendait la résolution de W ambiguë. La proposition de refuser un projet inconnu n'est pas retenue : l'autonomie de préparation est déjà demandée.
- Correction r3 : source_root reste la racine autorisée pour l'identité du projet et le mapping automatique. checkout_root, champ interne calculé depuis Git puis borné, désigne le checkout réel pour l'export et les reçus. Aucun champ nouveau du port modèle ou du protocole worker. Les deux scopes sont vérifiés côté réception. Les reçus existants gardent leur identité de checkout.
- RED r3 : le premier W puis P produisent deux identités de projet différentes ; le test s’arrête sur cette assertion avant le troisième appel. Le refus ambigu du troisième W est le scénario établi par lecture en revue, pas une trace RED exécutée ici. GREEN : 31 tests Python PASS, dont les deux ordres d'admission sans déclaration avec une seule correspondance et un worktree d'un autre dépôt, .git fichier mais common-dir distinct, refusé avant transport. Ces tests qualifient le helper et le registre, pas une nouvelle remise fournisseur.


### Livraison du checkout sous le parent — 22 octobre 2026

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-32).


## Correction du 22 octobre — réservation du résultat sur un canal occupé

La tâche e6283066-7755-4d6b-be16-54f3f167d6a8 est terminale et nettoyée, sans permis de remise ni résultat. Aucun rejeu métier. Le journal worker montre une sonde de4,318s dans la fenêtre du refus ; la réservation partage ce consommateur mais n’attendait que3s. Le journal ne prouve pas à lui seul la causalité historique.

Reproduction de bout en bout : vrai maître, vrai worker, transport Unix réel ; relais de fixture qui retarde de4s la première demande ReserveResultSlotUntil. Fournisseur simulé hors réseau. Sur la production initiale, le test échoue avec le code exact remote_start_failed:remote_result_slot:WorkerUnavailable avant remise. Le correctif porte le budget à15s, inférieur au bail de démarrage de60s. Aucun réessai ajouté, élargissement de droits ni changement de protocole. Tests affectés, revue et activation en cours.

GREEN : le même test E2E termine avec résultat unique et rejeu strict identique. Deux tests de négociation/erreur et les tests worker172_inbound_ sont PASS, dont les refus expirés après file et verrou, annulation et saturation. Fournisseur simulé, aucun appel API. Les empreintes des sources et le journal RED sont conservés hors Git. Revue et activation à suivre.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-33).


## Correction du 22 octobre — session du parent renouvelée

Le propriétaire Monuments conserve sa conversation et sa politique exacte, mais son provider_session_id passe de56926afc àb9f0f5d0. run_id/révision changent aussi. La continuation28da9844 ronde3 est refusée avant toute admission. Le refus de réservation aaba664f est antérieur au build172-cbc034bd ; il est terminal, nettoyé et sans remise.

La comparaison provider_turn ignore désormais la session technique comme elle ignore déjà run_id et révision. Le fournisseur/driver, les modes, le dossier et la politique restent identiques. NativePermissionFact ne peut pas déclarer provider_turn : validate_permissions(fact,false) exige native_wrapper ; ce chemin garde son égalité stricte. La politique enfant, sa preuve distante et le snapshot durable ne sont pas modifiés.

RED : le renouvellement de session avec politique identique échoue sur le code précédent. Contrôles ciblés et revue en cours ; aucune reprise métier à la place de Monuments.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-34).


## Reprise après renouvellement de session — 22 octobre 2026

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-35).


### Parent à la racine et délégation depuis un worktree lié — 22 octobre 2026

Le refus de préparation c67ba253 est reproduit : le remap réduisait la correspondance au checkout lié et rejetait le parent attesté à la racine. Pour un checkout contenu, la correspondance projet reste maintenant stable et seul le cwd d’exécution change.

Livraison du correctif de portée : source 39a9df21e925ad54b904a2b3c2078eba9f20ef17 poussée sur GitHub/main, worker Cartae 172-39a9df21 actif PID 2828136. Binaire de dépannage non optimisé après échec ENOSPC de la compilation optimisée. Deux tests distincts PASS, aucun verdict de revue obtenu ; arrêt de la revue demandé et livraison immédiate explicitement demandée par l’utilisateur. Aucun métier relancé. Reçu : validation/worktree-portee-parent-20261022.md.

### 22 octobre 2026 — remplacement du worker non optimisé

Source 39a9df21 inchangée, build release 172-39a9df21-opt actif sur Cartae (PID 2948540). Le binaire dev livré en dépannage ralentissait les sondes à 189 s pour un budget de 60 s. Compilation optimisée rc 0 en 95,01 s ; aucun changement de droit ou de délai. Signal d’essai réel envoyé à Monuments, résultat encore attendu. Détails : validation/worktree-portee-parent-20261022.md, section de remplacement optimisé.

### 22 octobre 2026 — refus de statut et canal worker

Correction ciblée du rejet des réponses sans attente pour une commande distante connue. Une sonde Reconcile précédant Start ne doit pas fermer le canal sur UnknownExecution. Un test ciblé RED puis GREEN ; aucune nouvelle mission. Détails et limite de causalité runtime : validation/reconcile-demarrage-20261022.md.

### 23 octobre 2026 — diagnostic du plafond de worktrees

BUS R15 : 48 worktrees sous la racine Opus2D (34 directs, 14 imbriqués),
base présente, branche demandée absente, 161 Go disponibles. Le preflight
refuse à 32 ; la sonde perdait le motif en produisant workspace_unavailable.
Correction : code fermé workspace_limit, message et action dans detail.reasons,
sans stderr ni chemin distant. Plafond, droits et idempotence inchangés.
Le worker ne transmet le nouveau code que si le maître annonce
worker_workspace_diagnostics_v1 ; sinon il garde le refus générique historique.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/implementation.md#extrait-36).
