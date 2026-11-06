# Validation 238 — 6 novembre 2026

Le reçu du demandeur donne 18 candidats mapping_unavailable sur monordinateur
et 27 candidats workspace_dirty sur les trois hôtes Linux. Ce sont des refus
de cette admission, sans tâche ni machine finale choisie.

## Cause et correction Mac

Le placement existe pour auto-5c29175617db400dab394fa74bc97948. Son identité
Git attendue était le chemin du miroir Linux, alors que le dépôt Mac a pour
origin git@github.com:Xavier91800/revue.git. La garde Rust compare exactement
ces valeurs avant le contrôle du workspace. Le HEAD Mac est déjà le commit
demandé. Cinq fichiers de consignes/documentation sont modifiés ; ils restent
inchangés. Shared admet ces modifications quand HEAD est déjà la base attendue.

Le registre maître et le registre worker Mac passent à la révision 2 de ce seul
mapping. Seuls expected_git_identity et mapping_revision changent. Origines,
racines, modèles, droits, comptes, autres mappings et répertoire additionnel
restent identiques. Écriture native scripts/prepare-project.py, verrou
project-preparation.lock, sauvegarde privée et comparaison des octets avant
écriture ; aucune modification directe de base ou de tâche.

Registre Mac :
/Users/moi/.cache/bridget-core/worker-monordinateur-agents.json

Nouvelle empreinte SHA-256 :
cacf9ba06704faf0699e13a6827f556d799282f96fda02b6e4dfb643e8667374

Elle remplace le repère bdee09c363292d1be730eecffad779a942cd75a0d45b69725472867ff3f3587f
pour les prochains contrôles. Ne pas restaurer l'ancien registre à l'aveugle.

Le helper natif remote_inspect, avec objects_only et le commit exact, réussit
en lecture seule : révision 2, needs_bundle=false, registre inchangé durant la
lecture. HEAD, configuration Git et SHA des cinq fichiers modifiés sont
inchangés. Les chemins et les gardes Rust correspondantes ont été relus.
Ce contrôle ne constitue pas une admission ni une preuve de capacité future.
Le maître et le worker relisent le registre au prochain parcours natif ; aucun
redémarrage n'est requis.

## Refus Linux attendus

| Machine | HEAD du checkout partagé | État Git observé |
|---|---|---|
| Cartae | d9d29fe69c9969af913cb99ce70fd249fc723b67 | ?? .worktrees/ |
| KS | 0ea495d8cfa4b6b62b57959f935add2ce5473731 | ?? .worktrees/ |
| SYS | 0ea495d8cfa4b6b62b57959f935add2ce5473731 | ?? .worktrees/ |

Le commit demandé est présent dans les trois dépôts. Leurs HEAD diffèrent.
shared_refresh_needed refuse donc leur déplacement dès que Git voit des
fichiers non suivis. Le mode discovery limite les écritures de l'enfant ; il
ne supprime pas les conditions de préparation du checkout partagé. Aucun
reset, stash, clean, déplacement de HEAD ou retrait de worktree Revue n'est fait.

Parcours normal : commit_view avec discovery pour lire un commit figé ; cette
vue ne contient pas les modifications non commitées ni les fichiers hors Git.
Pour lire aussi ces fichiers sur la machine source, shared conserve le dossier
existant quand son HEAD est déjà la base demandée, sous les droits hérités.
Les images et le rapport situés dans /Users/moi/Downloads ne sont pas transférés
par le seul placement du dépôt sur Linux. Un agent doit déclarer les sources
inaccessibles ; il ne doit pas prétendre les avoir vues.

Changer de stratégie modifie l'enveloppe. Ne pas réutiliser la clé d'origine
avec d'autres paramètres. Après fin de tout appel concurrent, absence de reçu
encore constatée et accord pour une intention différente, utiliser une nouvelle
clé. Aucun changement ni rejeu n'est exécuté par cette maintenance.

## Catalogue de maintenance

La recherche native dans Bridget retrouve bridge1016, rôle mainteneur-bridget,
scope.projects=[bridget], transversal=false, mandat vide, sans désignation
officielle. La présence native est busy/connectée. Revue n'a que son propre
projet dans caller_projects ; list/search filtrent les spécialistes étrangers,
et get renvoie role_scope_denied. Ce refus est donc conforme au périmètre
actuellement enregistré, sans lien avec un arrêt du mainteneur.

La désignation humaine conservée permet le signalement direct de cet incident.
La déclaration d'un mandat de maintenance transversal exige le parcours humain
natif d'administration des spécialistes. Aucun élargissement de mandat ou de
droits n'est effectué par cette correction de placement.

## État et preuves

Lecture administrative du registre d'admission : aucune ligne pour la clé
d'origine après correction ; aucune tâche créée ou rejouée. Services 0.8.5,
images de retour, tâches métier protégées et étape cliente 237 conservés.
T3 reste ouvert selon le choix de François.

Reçu et preuves privés :
/Users/moi/.cache/bridget238-revue-placement

Le binaire ne change pas. Les vérifications ciblées sont les lectures natives,
les empreintes et la comparaison des registres. Aucun test applicatif, build
ou nouvelle mission de revue n'est lancé pour cette configuration.

Diagnostic envoyé au demandeur sous mcp-42708-6aee236a-1, issued_at 1793991530,
delivery bef8714d-3d12-4a6d-b32c-1b1ce8496d97. Reçu in_flight : admission de
transport attestée, lecture du demandeur non attestée. Aucun accusé séparé.

Le premier envoi échoue avant admission avec binding_transport_unavailable.
La lecture administrative ne retrouve ni enregistrement idempotent ni remise
pour ce diagnostic. Une seule lecture native bridget_session réussit, puis
l'envoi aboutit. Aucun redémarrage ni changement de permissions n'est fait.
La cause initiale de cette interruption du lien reste inconnue ; aucune cause
de compte, réseau ou version n'est inventée. Les références du premier refus
et du reçu suivant sont conservées dans le dossier privé ci-dessus.
