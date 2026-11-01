# Profil Codex SYS0.166 — qualification ciblée

Statut : profil qualifié et actif, catalogue observé cohérent avec la nouvelle
définition. Programme221 inchangé :52/182 In Progress.

## Refus initial et cause constatée

Signalement opus201-incident-definition-solhigh-20261101-01. Demande
opus201-patrimoine-maisons8-solhigh-20261101-01, SYS/codex/gpt-6.1-sol/high,
base347d7b3b9cf63a7c13b39f0c3ed38bc2eaacc513. Le refus est
routing_candidate_unavailable/definition_unavailable, sans tâche enregistrée.
Aucune reprise métier par ce responsable.

Programme présent : Codex0.166.0,319346888octets, SHA256
b8a11fedf24b2dfb59a70c9e52b030f9a508dcd49cd2dd0d41956f4c785156b8.
Les deux profils codex-sysle2 et codex-sysle2-terra213 conservaient0.164.0,
SHAb93f6811d3f653f0fb83be4050d857fb9abbb111dfa2ce160563327daff7f306.
validate_provider_observation contrôle les octets du programme à chaque
préflight ; le changement est donc refusé. La liste déclarée du catalogue
ne constitue ni un préflight de cette définition ni une preuve d’accès.
L’observation native actuelle annonçait pourtant Sol avec high : cette lecture
métadonnée et le contrôle de programme sont deux faits différents.

## Qualification avant actualisation

La documentation officielle confirme que generate-json-schema produit le
contrat de la version exécutée :
https://learn.chatgpt.com/docs/app-server#message-schema
Les binaires0.164 et0.166 déjà présents ont généré leur schéma, sans installation,
login ni tour de modèle. Neuf formes comparées : initialize, thread/start,
thread/resume, thread/fork, turn/start, turn/steer, turn/interrupt, model/list
requête et réponse. Aucun nouveau champ requis ni champ ancien supprimé dans
ces formes. Les empreintes des schémas restent dans la preuve privée.
Cette comparaison ne prétend pas tester chaque changement interne de Codex.

Les observations des deux profils SYS ont ensuite été actualisées vers le
programme qualifié : version, empreinte et contract_version uniquement, puis
incrément de révision6→7. Contrôle normalisé : tous les autres champs identiques,
notamment arguments, comptes, modèles, efforts, outils et droits. Sauvegarde,
CAS des octets et publication atomique avec fsync, sous project-preparation.lock.
Le registre maître a reçu les mêmes observations, révision12→13. Le pont Mac
ne contient pas ces définitions : aucune définition distante n’y a été ajoutée.
Les copies privées ne sont jamais ajoutées à Git.

Deux essais du contrôle de publication ont arrêté avant mutation : le fichier
candidat ne porte pas le scope du worker, puis la CLI sans --registry cherchait
un registre absent. La donnée réelle a été relue : agents.json privé0600,
UID attendu ; accounts list --registry /home/moi/.config/bridget/agents.json
réussit avec machine_id=sysle2. Le port exécute from_json avant machine_scope :
le refus de scope du candidat atteste son parsing, pas des droits de compte.
Le chemin explicite du registre actif a été utilisé ensuite. Aucun contrôle de
compte n’a été contourné ; aucune commande accounts mutante exécutée.

## Une admission technique native

Clé bridget221-qualif-codex-sys166-solhigh-20261101-01. Premier appel lancé trop
tôt après l’arrêt du contrôle de publication : refus definition_unavailable,
not_recorded, aucune tâche. Après correction et publication constatée, rejeu
strict de la même enveloppe et de la même clé, sans nouvel identifiant.

Tâche20b6e59c-2681-4de9-ab29-eb49b45e98e5, enfant
cbd27848-ca74-4107-ab3a-dc402d9254b6, mission
1885dd53-c721-40ea-bc96-c5d945ff8b80. SYS, codex-sysle2,
gpt-6.1-sol, high, discovery, worktree à la basec74d4210d0b0a4010e9b1fe23cc8ac4691afac10.
Aucun outil demandé, aucun fichier lu ou écrit par l’enfant. Marqueur rendu :
BRIDGET221_CODEX_SYS166_SOL_HIGH_OK. result_available, erreur nulle, résultat
complet lu ; ACK et clôture demandés une fois. Zéro commit, arbre propre.
Le résultat prouve cet accès court sur cette connexion, pas l’accès à tous les
modèles, ni une compilation, ni un essai development. Terra n’a pas été lancé.
Les opérations interrupt/steer conservent leur forme ; elles n’ont pas été
réexécutées dans cette qualification ciblée.

Aucun build Bridget, installation de fournisseur, connexion de compte,
redémarrage de service ou relance d’ancienne mission. Le worker recharge la
définition pour une nouvelle sonde ; l’admission réussie prouve ce chargement.
Le changement temporaire definition_changed du catalogue après le seul worker
est conservé comme étape de synchronisation, pas annoncé comme état final.


## Constat final et reprise du demandeur

Une nouvelle observation après synchronisation maître : codex-sysle2
reported_models/auth reported_present à1793515692, cause nulle ; Sol annoncé,
high dans les efforts, profil déclaré et options concordants. La connexion
Terra revient également reported_models à1793515695 ; Sol y reste hors politique,
sans admission silencieuse de ce modèle dans ce profil. Le catalogue déclaré
historique n’est pas réécrit en prétendue garantie : la vue observée est datée,
et l’entitlement reste unknown. Aucun accès fournisseur universel déduit.

La clé métier opus201-patrimoine-maisons8-solhigh-20261101-01 n’avait encore
aucun reçu à1793515713. Monuments a reçu les prérequis du rejeu strict : fin de
tout appel concurrent, absence de tâche admise depuis, même clé et enveloppe,
même base/modèle/effort. Aucun rejeu de cette clé par bridge1016. Message
bridget221-codex-sys166-qualified-20261101-01, issued_at1793515758,
delivery6dc93855-58c9-44c3-bed8-f2bf8969403e, in_flight sans lecture attestée.

Le signalement R55 revue14 a sa propre tâche214f8726 : starting et
remote_start_failed:remote_start:WorkerUnavailable, mission_issued_at=null à
la lecture administrative. Le refus guichet idempotency_expired ne prouvait
pas une absence d’admission native. Le correctif worker a été chargé après
son incident, sans relancer cette commande. Diagnostic ciblé envoyé au
propriétaire sous bridget221-r55-review14-worker-diagnostic-20261101-01,
issued_at1793515761, delivery00dc510c-4005-4217-92f4-ffc6ee2ae2f0,
in_flight ; aucune cause historique unique affirmée ni arrêt physique déduit.
La continuation844bacce signalée noncontinuable n’apporte aucun défaut nouveau.
La revue5478888b reçue à nouveau est celle déjà traitée par le lot précédent ;
aucune nouvelle revue ni campagne générale n’a été lancée pour ce doublon.

## Conservation et livraison Git

Worktree technique20b6e59c propre, aucun ignoré, zéro commit, HEAD=c74d4210.
Résultat conservé, cleanup_done=true et idle_deadline_at=null au registre ;
les usages externes de son dossier ne sont
pas entièrement observables, donc dossier et branche conservés sans privilège
nouveau. Les schémas et sauvegardes restent dans le cache privé de qualification.
Aucun ancien programme supprimé :0.164 reste disponible pour un éventuel retour
explicitement décidé. Les deux observations des profils ont été actualisées ;
seul Sol high a fait un tour réel dans ce lot. Aucun nouveau binaire Bridget,
aucun pointeur rollback d’exécutable modifié. La qualification et ses limites
sont intégrées à main et poussées, puis les références main des miroirs sont
synchronisées en conservant leurs HEAD détachés. Pas de preuve visuelle requise
pour cette correction de configuration technique.
