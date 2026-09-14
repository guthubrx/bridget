# Spécification 098 — Adaptateur t3code

**Branche** : session-098-t3code-adapter · **Date** : 2026-09-14 · **Statut** : Proposée
**Demande** : que les agents pilotés par t3code (Claude, Codex, et les autres fournisseurs qu'il héberge) puissent communiquer par Bridget, avec une installation en une commande, réversible, sans modifier t3code ni « trifouiller » dans son code.
**Dépendances** : 089 (noyau, identités, remise idempotente), 094 (outils MCP Bridget), 096 (commande d'administration embarquée, services autonomes), 097 (présence honnête par canal réel).
**Tests** : à exécuter.

## Pourquoi

t3code pilote les agents en mode headless (SDK Agent pour Claude, app-server pour Codex) depuis un téléphone, un navigateur ou une application de bureau. Ces agents ne passent par aucun wrapper Bridget : ils n'ont ni identité, ni présence, ni moyen de recevoir un message. L'utilisateur veut les inclure dans la même communication que ses sessions de terminal, en gardant t3code intact et à jour par son propre canal de distribution.

## Scénarios utilisateur et tests

### US1 — Installer et retirer en une commande (P1)
L'humain lance une commande Bridget qui configure t3code pour que ses agents voient Bridget, puis peut l'annuler. Rien d'autre n'est modifié dans t3code ; une seconde installation ne change rien.
Test indépendant : sur des réglages t3code de fixture, installation, réinstallation et retrait laissent exactement les réglages attendus ; un t3code absent ou une version inconnue produit un refus nommé, jamais une écriture partielle.

1. Étant donné t3code installé et ses réglages lisibles, quand l'humain installe, alors chaque fournisseur pris en charge reçoit la configuration Bridget nécessaire et un statut le confirme.
2. Quand l'humain retire, alors seules les entrées posées par Bridget disparaissent ; les réglages de l'humain sont préservés.
3. Quand l'installation échoue à mi-chemin, alors les réglages sont ceux d'avant : aucune écriture partielle.

### US2 — Un agent t3code écrit et lit via Bridget (P1)
Un agent lancé par t3code dispose des outils Bridget : il voit l'annuaire, envoie une demande à un autre agent, lit le ledger et répond de façon liée. Il a une identité Bridget stable et une présence honnête indiquant qu'il est hébergé par t3code.
Test indépendant : un agent factice lancé avec la configuration produite par US1 apparaît dans `who` avec le canal réel, envoie un message accusé, et répond avec l'identifiant de la demande.

1. Étant donné un fil t3code ouvert avec la configuration Bridget, quand l'agent appelle l'outil d'envoi, alors la demande est acceptée et attribuée à son identité.
2. Quand l'humain consulte l'annuaire, alors l'agent apparaît avec un canal nommé t3code, jamais `tmux` ni un canal inventé.
3. Quand le fil est archivé ou supprimé, alors la présence est retirée.

### US3 — Un agent t3code reçoit un message dans son fil (P1)
Un message Bridget destiné à un agent t3code arrive dans son fil comme un nouveau tour, visible dans l'application t3code. La remise n'est déclarée réussie que si t3code a accepté le tour ; sinon l'état reste indéterminé et nommé.
Test indépendant : avec un serveur t3code de fixture, une demande suivie produit exactement un tour dans le bon fil, l'accusé suit l'acceptation, un refus de t3code donne un état indéterminé sans doublon au rejeu.

1. Étant donné un fil t3code identifié comme agent Bridget, quand une demande suivie lui est envoyée, alors un tour démarre dans ce fil avec le message et l'accusé est émis.
2. Quand t3code refuse ou ne répond pas dans le délai, alors la remise est indéterminée et un rejeu aux mêmes paramètres ne crée pas un second tour.
3. Quand un tour est déjà en cours dans le fil, alors le message attend ou est refusé explicitement, jamais perdu en silence.

### US4 — Observer et rester authentifié sans intervention (P2)
Le journal Bridget d'un agent t3code reflète ses tours, et `bridget attach` fonctionne. L'adaptateur se ré-authentifie seul auprès de t3code, y compris après un redémarrage de t3code ou de Bridget.
Test indépendant : coupure du serveur de fixture puis reprise : identités conservées, aucun tour doublé, journal continu ou lacune annoncée.

### Cas limites
t3code non installé ou non démarré ; jeton expiré ou révoqué ; fournisseur inconnu de Bridget ; plusieurs environnements t3code ; fil sans répertoire de travail ; message plus long que ce que t3code accepte ; deux messages simultanés vers le même fil ; réglages t3code modifiés à la main entre installation et retrait ; version de t3code dont le contrat change.

## Exigences fonctionnelles

- FR-09801 : une commande d'administration Bridget installe, montre l'état et retire l'intégration t3code ; idempotente, réversible, refus nommé sans écriture partielle.
- FR-09802 : l'installation ne modifie que des réglages que t3code expose à ses utilisateurs ; aucun fichier du code de t3code n'est touché ; t3code se met à jour par son propre canal.
- FR-09803 : chaque fournisseur t3code pris en charge reçoit l'accès aux outils Bridget avec l'état Bridget de l'humain, jamais un namespace inventé.
- FR-09804 : un agent hébergé par t3code possède une identité Bridget stable par fil, une présence dont le canal nomme t3code, et il peut envoyer, lire le ledger et répondre de façon liée.
- FR-09805 : la présence suit le cycle de vie du fil : créée à l'ouverture, retirée à l'archivage ou la suppression.
- FR-09806 : la remise d'un message vers un agent t3code démarre un tour dans son fil ; l'accusé n'est émis qu'après acceptation par t3code ; refus, délai et panne donnent un état indéterminé nommé ; le rejeu idempotent ne crée jamais deux tours.
- FR-09807 : rappels, notifications et messages ordinaires empruntent la même voie et les mêmes règles.
- FR-09808 : l'adaptateur s'authentifie auprès de t3code avec une portée limitée à ses besoins, conserve son identifiant hors des journaux, et se ré-authentifie seul après redémarrage ; un échec d'authentification est un état explicite, pas un silence.
- FR-09809 : le journal Bridget d'un agent t3code contient les tours humains et assistant issus des événements t3code ; `bridget attach` est admis.
- FR-09810 : envoi, suivi, réponse liée, canon, ledger et idempotence existants sont réutilisés ; aucun nouvel outil MCP, aucune nouvelle trame du protocole daemon.
- FR-09811 : tout ce qui dépend du contrat de t3code (commandes, événements, réglages, authentification) est isolé en un seul point de Bridget et versionné ; un contrat inconnu est refusé, jamais deviné.
- FR-09812 : aide, README FR/EN, skill et inventaire des commandes documentent l'intégration, ses limites et la procédure de retrait.

## Entités
Environnement t3code (adresse, jeton à portée, version de contrat), fil t3code (identifiant, fournisseur, répertoire), agent hébergé (identité Bridget, fil, canal `t3code`), remise vers un fil (message, tour, état), réglages posés par Bridget (par fournisseur, réversibles).

## Critères de succès
- SC-09801 : installation, réinstallation et retrait en moins de 10 s chacun, réglages identiques à l'attendu, zéro écriture hors des réglages t3code.
- SC-09802 : un agent t3code apparaît dans `who` avec le canal `t3code` en moins de 10 s après l'ouverture de son fil, et disparaît en moins de 10 s après archivage.
- SC-09803 : un message envoyé à un agent t3code est visible dans son fil en moins de 5 s et la demande devient `answered` quand il répond ; rejeu même clé : un seul tour.
- SC-09804 : après redémarrage de t3code puis de Bridget, aucune intervention humaine, identités conservées, zéro tour doublé.
- SC-09805 : recette réelle avec le t3code installé de l'humain : un agent Codex de terminal envoie une mission à un agent Claude hébergé par t3code, lisible depuis l'application mobile ou web.
- SC-09806 : fmt, clippy et suite complète du workspace verts ; tests de fixture sans réseau ni compte.

## Hypothèses et limites
t3code installé localement sur la même machine que le daemon Bridget, une seule instance à la fois ; l'adaptateur n'est ni un lanceur d'agents t3code ni une interface : t3code garde le pilotage. La contrainte « sans modifier t3code » prime : si une capacité exige un fork, elle sort du périmètre et est consignée. Les fournisseurs pris en charge dans cette session sont ceux que Bridget connaît déjà (Claude, Codex) ; les autres sont documentés comme non couverts.
