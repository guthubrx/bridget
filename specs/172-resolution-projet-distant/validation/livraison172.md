# Livraison 172 — préparation autonome des projets distants

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-1).

## Versions actives après le complément quota

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-2).

## Continuation refusée et incident de reprise

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-3).

L’ancien processus VOITURES54569150-ebef-4f11-9053-f17775a5c909 est perdu.
Sa ronde1, son résultat de diagnostic, son worktree et son commit de note sont conservés :
/home/moi/10.Scripts/69.opus2D/.worktrees/54569150,
c41780f4d8b27d01bda57774da27b639185bfdee. Cette note n’est pas une correction source.
La vérification de conservation des anciens processus après activation995 a échoué.
Le worker quittait sur EOF pendant sa sonde, avec worker_capability_unavailable.
Ce défaut a été reproduit puis corrigé dans527308bb, inclus dans1e5ddd07.
La reconnexion garde maintenant le même runtime ; un refus permanent reste fatal.
Le maintien du PID13183 au redémarrage du maître est prouvé sans modèle actif.
Il ne prouve pas la conservation d’un modèle en cours ni une continuation réelle.

BUS8dcb54b1-bfae-4693-afe9-596b44b375e7 a échoué api_error à11:04:18 UTC,
soit23s avant l’arrêt du maître995. Son journal Claude atteste error=rate_limit.
Aucune relance BUS par ce lot. Le propriétaire garde ses travaux.

Le propriétaire a confirmé qu’aucune consigne humaine applicable n’interdit une
nouvelle admission VOITURES. Sa reprise distincte r3 a été admise, puis a échoué :
Taskb9d8b49c-bf25-4c8b-be20-69d2fb1c128a, api_error, sans travail confirmé.
Le journal Claude donne la même limite de session, annoncée jusqu’à13:20 UTC
(15h20 Paris), et non une panne SSH ou un refus de droits. Son worktree est conservé :
/home/moi/10.Scripts/69.opus2D/.worktrees/b9d8b49c.

## Quota : correction, tests et limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-4).

65tests concernés PASS : pilote Claude34, mémoire quota14, routage13,
sonde distante1, liaison stricte1, continuation2. Les14tests de rondes hérités
passent aussi après correction du profil temporaire0770→0700. Les deux anciens
échecs env_unfit avaient été reproduits sur la baseline ; contrôle produit inchangé.
Aucune FULL ni validation universelle déduite. Autres serveurs que Cartae non testés.

## Essais métier après activation1e5 et complément en préparation

r4 Task14c382ea-c831-4374-b6b8-c9f77fe456a7 et r5 Taskc9693bca-4304-4e72-abe2-442677bc8c14
ont toutes deux fini usage_limit, puis ont été lues et acquittées. Pas de working
confirmé ni de code produit. Les décisions ont choisi claude-cartae / claude-haiku-5-5.
Le refus est mieux classé, mais le routage n'est pas débloqué par1e5 : le délai
r4→sonde r5 de44–47secondes dépasse le TTL30. Aucun succès de routage déduit.

Le complément source courant corrige cette perte. Un refus daté reste restrictif
jusqu'au retour structuré. Un refus sans date conserve un délai de nouvelle tentative
de5minutes ; il redevient ensuite inconnu, sans capacité positive inventée.
Une vraie réponse assistant acceptée peut le lever avant ce délai. Les comptes
restent séparés par connexion, définition de base, politique et identité worker.
Les autres modèles du même compte sont écartés. Crédits et faits positifs gardent
les contrôles du digest de mission, de présence vivante et du TTL. Une réponse
vide ou une autre erreur ne lève pas le refus. Les fenêtres structurées gardent
leur vraie date, sans ajout d'un refus unknown permanent par le code assistant.

Tests finaux concernés : mémoire quota16PASS, sonde distante1PASS, pilote Claude35PASS
(en série). Diagnostic du rattachement2PASS, liaison stricte1PASS, héritage/revocation
auxiliaire1PASS, scope de connexion1PASS et continuation2PASS. Routage168 :35PASS.
Les séries peuvent se recouvrir ; elles ne sont pas additionnées comme une FULL.
La série Claude parallèle a eu34PASS/1FAIL Text file busy ; ce cas a été relu,
rejoué seul PASS, puis la série entière exécutée en série35PASS. Cause système
ponctuelle non établie ; aucun changement produit destiné à masquer ce cas.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-5).

## Refus de rattachement après Claude

Le fil17f50a6d-ec2f-494a-b9a3-0def5b14475b a rencontré usage_limit/api_error_429
sur Claude (tours169–172), avec retour structuré13:20UTC/15:20Paris. Le message
binding_registration_refused est apparu dans la reprise Codex (tour173,11:56:05UTC).
Aucune limite de taille de contexte n'est établie par ces traces. Le rattachement
local est distinct du quota. Son lien actuel est présent dans l'inventaire.
L'unique demande d'une lecture depuis ce fil n'a pas obtenu de réponse dans son
échéance ; ce n'est pas une preuve de succès ou de refus actuel.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-6).

## Paquets du complément prêts, activation différée

Source codebb83316c68b828fe4ae0c747d81830ac34ee9eea, publiée sur main.
Build172-bb83316c, sans nouvelle release ni changement de version commerciale.
93noms de tests distincts PASS dans9séries ciblées, dédupliqués dans le reçu privé ;
aucune FULL. Le correctif de quota a d'abord reproduit le défaut en RED.

- Artefact Mac : /Users/moi/.cache/bridget-delivery172/quota-delay172/bridget.
  SHA256d5281180383ddaf9d886de276a52c93084a06c3ae9c7482c54cb0a16f40ac9ea.
  Compilation71,81s RC0, signature ad hoc vérifiée strictement.
- Artefact Cartae : /home/moi/.cache/bridget-delivery172/quota-delay172/bridget.
  SHA2564a6164d0bdce8b58c033dd079c2f1294b6c1ba1e580625897e666f51d4117ead.
  Compilation64,82s RC0. Copie privée vérifiée, distincte du cache de compilation.

Le préflight à1792326544 conserve l'activation différée : la Taskc998d2d5 est
working, cleanup_done=false. Aucun binaire actif remplacé, aucun service relancé.
Maître50706, worker13183, T3637/1068 et pont898 conservés. Le statut officiel
annonce toujours172-1e5ddd07, en ligne. Le propriétaire a reçu une demande unique
de signalement après clôture complète, sans polling ni GO pour son travail.
La remise de cette note est in_flight ; aucune réponse ou lecture n'est supposée.

Reçus Mac :
/Users/moi/.cache/bridget-delivery172/quota-delay172/build-receipt.json
/Users/moi/.cache/bridget-delivery172/quota-delay172/activation-preflight.json
/Users/moi/.cache/bridget-delivery172/quota-delay172/CURRENT.json
Reçus Cartae :
/home/moi/.cache/bridget-delivery172/quota-delay172/build-receipt.json
/home/moi/.cache/bridget-delivery172/quota-delay172/tests.json

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-7).

## Reçus privés du complément

/Users/moi/.cache/bridget-delivery172/quota172/build-receipt.json.
/Users/moi/.cache/bridget-delivery172/quota172/activation.json.
/home/moi/.cache/bridget-delivery172/quota172/build-receipt.json.
/home/moi/.cache/bridget-delivery172/quota172/activation.json.
/home/moi/.cache/bridget-delivery172/quota172/after-master.json.
Les anciens checkpoints ci-dessous restent historiques.

## Checkpoint124 — historique

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-8).

## Recettes complémentaires prouvées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-9).

## Échecs réels qui ont déclenché le complément

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-10).

## Validation ciblée du dernier delta

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-11).

## Reçus du complément

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-12).

## Complément de sondes activé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-13).

## Recettes après activation124dc47a

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-14).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-15).

## Revue et Opus2D : recettes finales réussies

Les deux appels partent des vrais parents demandés par l’utilisateur, sans
project_id, autres axes automatiques ; shared/discovery, échéance180s.

Revue : request recette172-revue-cartae-controle-final-20261018-r7,
Task30a2525b-7d62-4be8-a3a6-b5b750382009.
SHA attendu et lu d9d29fe69c9969af913cb99ce70fd249fc723b67,
nom revue/version0.1.0, BRIDGET172_REVUE_FINAL_OK.
Lectures réelles :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/package.json
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/.git/HEAD.
HEAD détaché, pas de référence supplémentaire. Le projet initialement absent
avait été créé automatiquement ; son dépôt géré a reçu le nouveau commit.
Le parent Rapport a lu le résultat, ACK, puis comparé le rejeu : même Task et
texte identique. Aucun travail de code demandé à l’enfant ; la préparation Git
reste l’action du framework.

Opus2D : request recette172-opus2d-trajets-cartae-20261018-r4,
Taskea8d9680-dd4e-4a71-a22d-1589ee028f9f,
SHA attendu et lu ed82547ff65fd23307f63ee6683c9e9e7b22d420,
nom opus2d/version0.1.0, BRIDGET172_OPUS2D_CARTAE_OK.
Lectures réelles :
/home/moi/10.Scripts/69.opus2D/package.json
/home/moi/10.Scripts/69.opus2D/.git/HEAD
/home/moi/10.Scripts/69.opus2D/.git/refs/heads/main.
Le parent trajets bus a lu le résultat1007octets, ACK, puis comparé son unique
rejeu : même Task, même résultat textuel. Aucune modification demandée à l’enfant.

Les sorties réelles worker sont exit_code0/failure=null ; générations557/558.
Sondes initiales observées : Revue4,317s ; Opus2D4,303s pour dix candidats.
SQL en lecture seule confirme pour les six clés réussies : une Task chacune,
result_available, error=null, cleanup_done=true, ACK présent. Les rapports
parents sont conservés dans :
/Users/moi/.cache/bridget-delivery172/project-final-recipes172.json.
Les deux rapports transportés sont bornés et leur fin est tronquée ; les
résultats natifs complets et leurs états durables ont été relus séparément.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-16).

## Liaison intermittente après les recettes finales

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-17).

## Mission VOITURES : refus réel puis reprise corrélée

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-18).

## Premier reçu conservé pour l’historique

Les états et chemins ci-dessous décrivent le premier lot, avant les compléments
ci-dessus ; ils ne décrivent pas les processus courants.

Date : 18 octobre 2026. Correction active, recettes Cartae réussies.
Périmètre : résolution implicite du projet, préparation Git/SSH, démarrage et retour.

## Résultat utile

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-19).

## Versions du premier reçu (historiques)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-20).

## Recettes réelles

### Projet Bridget connu

Request : recette172-bridget-cartae-connu-20261018-r3.
Task : 68f69ce3-62bb-4b17-8bbc-853399a6346f.
Sans project_id, machine Cartae, posture discovery, espace partagé.
Lecture réelle de /home/moi/10.Scripts/64.bridget/.git/HEAD, de sa référence
et de /home/moi/10.Scripts/64.bridget/Cargo.toml.
SHA a53b05c23426160dd34c5987613e7d0d878e273f, version0.4.1.
Résultat : BRIDGET172_CARTAE_CONNU_OK.
Résultat lu, ACK, rejeu strict : même tâche, même résultat.
État durable result_available, error=null, cleanup_done=true, un lancement.

### Projet Revue initialement absent

Source : /Volumes/8TB2/50-repos-archives/11.Repositories/revue.
Aucune déclaration manuelle avant la recette r1.
La préparation r1 a créé automatiquement :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository.
Le premier lancement et son résultat ont confirmé la création ; son brief
demandait par erreur un shell interdit en discovery, donc la lecture finale
n'était pas validée à ce stade.

Recette finale :
Request : recette172-revue-cartae-neuf-20261018-r5.
Task : 800dae46-f45a-4e7c-bc51-9d1e7dabba7d.
SHA ac8f9295b89d1475f32af85f9e3e9d12d3f89008 transféré automatiquement.
Ce commit a remplacé le SHA initial pendant les essais à la suite d'une fusion
faite par une autre session. L'appel r4 avec l'ancien SHA a été correctement refusé.
Fichiers lus :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/.git/HEAD
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/package.json
Nom revue, version0.1.0, SHA exact, BRIDGET172_REVUE_NEUF_OK.
Résultat lu et acquitté par le parent Revue. Rejeu strict : même tâche,
même résultat, acknowledged. État durable result_available, error=null,
cleanup_done=true, un lancement.
Aucun fichier Revue modifié par les agents de recette.
La création initiale et la lecture finale sont deux essais distincts, pas
présentés comme un seul lancement ininterrompu.

## Défauts révélés et corrigés pendant les essais

1. Identifiant projet absent : résolution unique depuis le dossier attesté.
2. Projet absent : préparation Git/SSH et publication atomique des registres.
3. Réattestation après Ready plus longue que cinq secondes :
   bail de60s négocié ReleaseLeaseV2, ancien worker limité à5s ;
   contrôles de droits/révision/époque conservés.
4. Enregistrement du wrapper limité à1s : échec réel os error11 reproduit
   avec Registered retardé de1,5s. Attente initiale10s puis lecture de session1s.

Les échecs historiques sont conservés dans le journal d'implémentation.
Aucun refus de droits ou de commit n'a été transformé en réussite.

## Vérifications et limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-21).

## Reçus et conservation

Reçus privés Mac : /Users/moi/.cache/bridget-delivery172.
État final des tâches :
/Users/moi/.cache/bridget-delivery172/final-task-receipts.json.
Reçu worker :
/home/moi/.cache/bridget-delivery172/linux172-register.json.
Ancien binaire conservé :
/Users/moi/.cache/bridget-delivery172/bridget-before172.
Unités et binaires précédents conservés sur Cartae ; aucun effacement de base.
Worktree de correction conservé :
/home/moi/10.Scripts/64.bridget/.worktrees/172-resolution-projet-distant.


[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-22).


## Correctif du store distant préparé le 18 octobre 2026

Incident réel : remote_result_slot:StoreUnavailable sur la demande voitures r7.
Cause confirmée par lecture seule du store Cartae : 612964 octets réels contre
15788644 octets réservés. Les anciens reçus soldés retenaient chacun 64 Kio.
La Task350d5c53-3b0d-4b1e-90b2-150002b6f51a avait pourtant été admise ;
son identifiant a été transmis au propriétaire pour consultation de l’existant.

Trois tests RED prouvent le défaut avant correction. Après correction :
série172 23 PASS ; module worker 24 PASS ; résultats distants 46 PASS ;
routage168 35 PASS. Ces séries se recouvrent :
125 noms de tests distincts PASS, sans suite complète.
Reçu privé : /home/moi/.cache/bridget-delivery172/store-admission172-tests.json.
Les tests sont en série, avec un seul thread, sur Cartae. Une première commande
sans dossier Cargo et une erreur de syntaxe de fixture n’ont lancé aucun test ;
la première compilation du séparateur typé a nécessité de distinguer les erreurs
techniques des erreurs du store. Les trois échecs RED produits restent séparés
de ces erreurs de préparation du banc.

Le store conserve 120 résultats et reçus, leur relecture après réouverture et le
rejeu du premier. Sept résultats actifs restent possibles et le huitième est
refusé. Les gardes des orphelins, des liens symboliques, des identités, des
permissions et des rejeux passent dans les régressions. La réponse après
admission garde désormais TaskID et erreur ; une modification d’enveloppe reste refusée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-23).


Les deux paquets sont désormais construits depuis la source
72a8e8f8823c5f23b463a45693bb9b27079e41f3, build172-72a8e8f8, sans activation.
Mac : compilation76,28s, signature adhoc et vérification stricte conformes.
Binaire /Users/moi/.cache/bridget-delivery172/store172/bridget,
SHA256 2d9a543ade02f12e669d6a94e45ccc2cae8c843067ff00294712ef5f82d04485.
Cartae : compilation99,96s,
binaire /home/moi/.cache/bridget-delivery172/store172/bridget,
SHA256 2d2df02dacb28e72cf25273f5e7d558ab2ed25e0c5add5029588b28b459b6a92.
Le paquet intègre aussi le complément quota bb83316c préparé précédemment.

Reçu courant : /Users/moi/.cache/bridget-delivery172/store172/CURRENT.json.
Pointeur de préparation : /Users/moi/.cache/bridget-delivery172/prepared-current.json.
Le paquet quota-delay172 est signalé comme remplacé par cette préparation.

Le dernier contrôle en lecture seule trouve encore BUS453c75be-503b-4934-9311-e382733498aa
working, cleanup_done=false. La garde interdit donc l’activation.
Maître50706, worker13183 avec NRestarts0, pont898 et T3637/1068 inchangés.
Le binaire actif Mac garde le SHA d585bb22d60265b59133adcb451fa12ff5a9c20e687fd80744212d8537a38dd4.
Le binaire actif Cartae garde le SHA bc5daf985c5472e825291bb568596238463c544a2135fb367ac3168af1e2c97c.
La source active reste1e5ddd076b3d610c864e5d37dbe66d28dc780edc.
L’objectif reste paused et aucun réveil ni redémarrage automatique n’est programmé.

T037/T038/T039 sont closes pour le code, ses tests et ses paquets.
T036 reste ouverte pour l’activation puis le parcours réel. Aucun succès métier
sur cette nouvelle source n’est déduit de ses tests ou de sa compilation.


## Relance adaptateur de bureau effectuée par l’utilisateur à 20 h 39

Application30609 et serveur30660 démarrés depuis le paquet0.0.45-local.172.
Serveur né à18:39:56UTC. ASAR inchangé :
4d57cee627d68079504e613cf93d532fa6399c7026a14c2b8f24fc26aa52a9b3.
L’appel officiel bridget_status puis bridget_capabilities réussit depuis le fil
courant après la relance : rattachement, attestation, développement et worktree
autorisés sur le projet Bridget. Ce catalogue ne teste pas l’authentification des
fournisseurs et ne prouve pas une nouvelle délégation ou une continuation métier.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-24).


## Source72a8e8f8 active et parcours distant réel validé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-25).

- Maître Mac7825, naissance21:15:02CEST, build172-72a8e8f8.
- Worker Cartae624845, NRestarts0, build172-72a8e8f8.
- T330609/30660 et pont898 préservés.
- SHA Mac2d9a543ade02f12e669d6a94e45ccc2cae8c843067ff00294712ef5f82d04485.
- SHA Linux2d2df02dacb28e72cf25273f5e7d558ab2ed25e0c5add5029588b28b459b6a92.

Un refus initial de rattachement primary_absent pendant la reconnexion du pont
a disparu après la réinscription existante des fils. Les appels officiels ont
ensuite réussi. Aucun réattachement forcé ni emprunt de preuve. Cette observation
ne clôt pas le diagnostic historique T035 de la capture après quota Claude.

Recette réelle sur Cartae : Task3a825355-37b1-45e1-8bdf-9aef6de39dd4,
clérecette172-store72-cartae-live-20261018-r1, puis continuation-c2 sur le même
enfant. Bridget a choisi claude/claude-haiku-5-5. Espace isolé créé par Bridget,
sans project_id fourni. SHA exact et version lus, premier résultat891octets ;
borne du store relue à la ronde2, second résultat853octets. Deux lectures complètes,
deux ACK et deux rejeux stricts, Task/ronde/résultat identiques, sans doublon.
Worktree propre, zéro commit métier. Les deux marqueurs sont conservés.

La garde du test est fermée après le résultat : cleanup_done=true,
guard_released=true, commande terminale et groupe626253absent.
Le worktree de recette reste propre et conservé avec son reçu. Aucun espace
ou produit Opus2D n’est supprimé. Le propriétaire a reçu la levée du blocage ;
la remise de cette note est in_flight, sans preuve de lecture revendiquée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-26).

Preuves privées :
/Users/moi/.cache/bridget-delivery172/store172/activation.json
/home/moi/.cache/bridget-delivery172/store172/activation.json
/Users/moi/.cache/bridget-delivery172/store172/live-delegation.json
/Users/moi/.cache/bridget-delivery172/store172/CURRENT.json

T036/T040 closes. T016/T032 restent ouvertes pour le cas exact du renouvellement
automatique d’un tour parent distinct ; la continuation du tour courant est validée.
T035 historique et la livraison métier VOITURES ne sont pas déclarés résolus par
cette recette. L’objectif conserve son statut paused ; aucun réveil programmé.


## Complément plafond MCP du19octobre — code validé, activation à suivre

Cause reproduite : un avis provider_failed non acquitté du parent trajets bus
est rejoué sur la connexion auxiliaire MCP après Registered. CallerScope attend
son objet de droits et reçoit cette notification. Correction côté maître :
réserver le rejeu au parent primaire, comme la remise en direct.

17 tests distincts PASS : nouvelle reproduction1, événements0687,
plafondsMCP1587, diagnostics de rattachement1722. Aucun ancien résultat
n’est additionné pour fabriquer une suite complète.
Reçu : /home/moi/.cache/bridget-delivery172/scope172-tests.json.
La reproduction RED et les contrôles GREEN restent séparés.

Task623 est failed/terminal Codex:failed, garde et nettoyage terminés.
La tâche n’est pas relancée par ce correctif de lecture. Le résultat à obtenir
après activation est la vraie réponse officielle depuis son parent trajets bus.
Aucun credential d’un autre fil n’est utilisé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-27).


## Plafond MCP — correctif actif et lecture du cas réel réussie

Source7085b77c21d7c94460d2fabbe1e62d04b9938bd2 publiée et build172-7085b77c
construit en96,69s, signé ad hoc et vérifié strictement.
Binaire préparé /Users/moi/.cache/bridget-delivery172/scope172/bridget,
SHA25609da2c026677ac6789a65a323fa8644ef9e73eed83a82e0dd50e4fef545c4a3e.
Le correctif est propre au maître ; worker Cartae624845/source72a8e8f8 conservé.

La tâche locale46fd5bd7 a fini avant l’arrêt du maître. Préflight frais :
aucune tâche active/non nettoyée gardée, aucune admission auto ouverte.
Binaire remplacé atomiquement puis ancien maître7825 arrêté par SIGTERM.
Le mécanisme de démarrage existant a déjà lancé le nouveau maître16326 ;
la tentative manuelle suivante a été refusée par le verrou exclusif.
Le PID, la commande, le binaire et le build ont ensuite été vérifiés.
Aucun second démon lancé ni nouvelle relance nécessaire.
T330609/30660 et pont898 gardent leurs PID et dates de naissance ;
worker624845 actif avec NRestarts0. Le statut MCP officiel confirme172-7085b77c.

Vérification du cas réel depuis le parent trajets bus, une seule lecture officielle
de62363500-c0cf-409b-b669-5549f33cfa7b demandée après activation.
Réponse à mcp-95175-6ad56373-3 : lecture réussie, status/work_state failed,
error terminal Codex:failed, round4, continuablefalse, resultnull.
Le dossier reste propre et conservé, HEAD396cd40d4d84a2cc46ed31d5d0841fda89b87a2c,
chemin /home/moi/10.Scripts/69.opus2D/.worktrees/62363500.
Aucune admission, continuation, relance ni ACK métier créés par cette vérification.

L’événement provider_failed initial reste présent et non acquitté :
la correction rétablit le bon canal sans effacer l’incident. Elle ne transforme
pas l’échec Codex en succès. Le propriétaire a reçu la décision de reprendre
ses acquis dans une nouvelle mission identifiée, dans son mandat existant.
Aucun autre test Bridge n’a été demandé. La cause fine de l’échec du fournisseur
reste hors de cette preuve et n’est attribuée ni au quota ni au contexte.

Preuves privées :
/Users/moi/.cache/bridget-delivery172/scope172/build-receipt.json
/Users/moi/.cache/bridget-delivery172/scope172/activation.json
/Users/moi/.cache/bridget-delivery172/scope172/diagnostic.json
/Users/moi/.cache/bridget-delivery172/scope172/tests.json
/Users/moi/.cache/bridget-delivery172/scope172/live-owner-read.json
/Users/moi/.cache/bridget-delivery172/scope172/CURRENT.json

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/livraison172.md#extrait-28).
