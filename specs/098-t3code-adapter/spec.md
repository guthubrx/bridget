# Spécification 098 — Adaptateur t3code

**Branche** : session-098-t3code-adapter · **Date** : 2026-09-14 · **Statut** : Implémentée
**Demande** : que les agents pilotés par t3code (Claude, Codex, et les autres fournisseurs qu'il héberge) puissent communiquer par Bridget, avec une installation en une commande, réversible, sans modifier t3code ni « trifouiller » dans son code.
**Dépendances** : 089 (noyau, identités, remise idempotente), 094 (outils MCP Bridget), 096 (commande d'administration embarquée, services autonomes), 097 (présence honnête par canal réel).
**Tests** : à exécuter.

## Pourquoi

t3code pilote les agents en mode headless (SDK Agent pour Claude, app-server pour Codex) depuis un téléphone, un navigateur ou une application de bureau. Ces agents ne passent par aucun wrapper Bridget : ils n'ont ni identité, ni présence, ni moyen de recevoir un message. L'utilisateur veut les inclure dans la même communication que ses sessions de terminal, en gardant t3code intact et à jour par son propre canal de distribution.

## Scénarios utilisateur et tests

### US1 — Installer et retirer en une commande (P1)
L'humain lance une commande Bridget qui prépare l'intégration (jeton t3code, service de pont) et peut l'annuler. Aucun réglage ni fichier de t3code n'est modifié ; une seconde installation ne change rien.
Test indépendant : sur un faux t3code, installation, réinstallation et retrait laissent exactement l'état attendu (jeton, identifiant de session, service) ; un t3code absent, une commande `t3` absente ou une réponse de forme inconnue produisent un refus nommé, jamais un état partiel.

1. Étant donné t3code et sa commande `t3` disponibles, quand l'humain installe, alors un jeton dédié est obtenu, conservé de façon privée avec son identifiant de session et son expiration, et le service de pont est enregistré ; un statut le confirme.
2. Quand l'humain retire, alors le jeton est révoqué par son identifiant, le service retiré, l'état privé effacé ; rien d'autre n'est touché.
3. Quand une étape échoue, alors les étapes déjà faites sont défaites et l'état final est celui d'avant.

### US2 — Chaque fil t3code est un agent visible et joignable (P1)
Chaque fil ouvert dans t3code apparaît dans l'annuaire Bridget comme un agent, avec un canal qui nomme t3code, le fournisseur et le dossier de travail ; il disparaît quand le fil est archivé ou supprimé.
Test indépendant : avec un faux t3code exposant deux fils, `who` montre deux agents `t3code` ; l'archivage d'un fil retire son agent en moins de 10 s ; un redémarrage du pont conserve les mêmes identités.

### US3 — Un agent t3code reçoit un message dans son fil et sa réponse revient (P1)
Un message Bridget destiné à un fil t3code y apparaît comme un nouveau tour, visible dans l'application. La remise n'est déclarée réussie que si t3code a accepté le tour ; si un tour est déjà en cours, la remise attend son terme dans une borne, sinon elle est indéterminée et nommée. Quand l'assistant du fil termine le tour ainsi ouvert, sa réponse est renvoyée à l'émetteur comme réponse liée, par le pont, au nom de l'agent du fil.
Test indépendant : faux t3code : une demande suivie produit exactement un tour dans le bon fil ; l'accusé suit l'acceptation ; un rejeu aux mêmes paramètres ne crée pas de second tour ; un tour actif fait attendre puis, borne dépassée, donne un état indéterminé ; deux demandes concurrentes vers le même fil partent l'une après l'autre ; une déconnexion du daemon pendant l'attente donne un état indéterminé ; la réponse de l'assistant qui suit le message du pont clôt la demande (`answered`) ; un message humain intercalé laisse la demande ouverte avec ambiguïté journalisée ; un refus ou une panne de t3code donne un état indéterminé.

### US4 — Observer et rester authentifié sans intervention (P2)
Le journal Bridget d'un agent t3code reflète les tours humains et assistant du fil, y compris ceux que Bridget n'a pas causés, sans rejouer l'historique antérieur à l'installation et sans doublon après redémarrage ; `bridget attach` fonctionne. Le pont se ré-authentifie seul, au plus une fois par incident, et signale un échec durable.
Test indépendant : coupure du faux t3code au milieu d'un tour puis reprise : identités conservées, aucun tour doublé, journal continu ou lacune annoncée ; fil créé et achevé avant le premier passage, fil créé pendant un arrêt du pont : projetés une fois, dans l'ordre ; 401 répété : un seul renouvellement puis état d'échec explicite.

### Cas limites
t3code non installé ou non démarré ; jeton expiré ou révoqué ; fournisseur inconnu de Bridget ; plusieurs environnements t3code ; fil sans répertoire de travail ; message plus long que ce que t3code accepte ; deux messages simultanés vers le même fil ; réglages t3code modifiés à la main entre installation et retrait ; version de t3code dont le contrat change.

## Exigences fonctionnelles

- FR-09801 : une commande d'administration Bridget installe, montre l'état et retire l'intégration t3code ; idempotente ; chaque étape défaite en cas d'échec ; refus nommé sans état partiel.
- FR-09802 : Bridget ne modifie ni le code, ni les réglages de t3code et n'écrit jamais directement dans ses bases ; seule la commande officielle `t3` peut y créer ou révoquer la session de Bridget, sous un libellé d'installation unique qui permet de la retrouver et de la révoquer même si le reçu de création a été perdu ; t3code se met à jour par son propre canal.
- FR-09803 : le jeton t3code est obtenu par la commande officielle de t3code avec sa sortie structurée, conservé en privé avec son identifiant de session et son expiration, jamais journalisé, révoqué au retrait ; ses portées réelles, plus larges que le besoin, sont documentées comme limite acceptée ; le renouvellement est borné à une tentative par incident.
- FR-09804 : chaque fil t3code ouvert possède une identité Bridget stable entre redémarrages, une présence dont le canal nomme t3code, le fournisseur et le dossier ; l'émission de messages Bridget depuis l'agent t3code n'est pas offerte en v1.
- FR-09805 : la présence suit le cycle de vie du fil : créée à l'ouverture, retirée à l'archivage ou la suppression, restaurée à l'identique après redémarrage du pont.
- FR-09806 : la remise vers un fil démarre un tour dans ce fil ; les remises d'un même fil sont sérialisées dans une file propre au fil, sans bloquer la boucle de connexion au daemon ni les autres fils ; si un tour est actif, la remise attend son terme dans une borne puis, à défaut, devient indéterminée ; l'accusé n'est émis qu'après acceptation par t3code ; refus, délai, panne et déconnexion pendant l'attente donnent un état indéterminé nommé ; le rejeu idempotent ne crée jamais deux tours ; la course résiduelle entre l'observation et le démarrage est documentée et testée.
- FR-09807 : le pont identifie son propre message dans le fil par l'identifiant de message qu'il a choisi ; la réponse de l'assistant qui répond à ce message, identifiée par son rang dans le fil (t3code traite les messages dans l'ordre, un tour par message) et une fois complète, est renvoyée à l'émetteur comme réponse liée, au nom de l'agent du fil ; la corrélation en attente est durable : un redémarrage du pont entre l'accusé et la réponse ne la perd pas ; si un autre message humain s'intercale ou si l'ordre ne peut être prouvé, aucune réponse n'est inventée : la demande reste ouverte et l'ambiguïté est journalisée ; rappels et notifications empruntent la même voie que les messages.
- FR-09808 : le pont découvre t3code par son fichier d'état local et n'accepte qu'une adresse de boucle locale en HTTP, reconstruite vers `127.0.0.1` et le port publié ; toute autre origine, un fichier périmé (processus absent) ou une forme inattendue est un refus nommé ; chaque réponse est validée route par route et champ par champ ; aucune version d'API n'est déduite d'un champ qui ne la porte pas ; le jeton ne quitte jamais la boucle locale.
- FR-09809 : le journal d'un agent t3code est projeté par différence d'états successifs du fil, lus par pages récentes et seulement quand le fil a changé, avec un repère d'installation et une séquence durable par fil : tout ce qui est postérieur au repère est projeté, y compris au premier passage et après un arrêt du pont ; rien d'antérieur n'est rejoué ; aucun doublon ; l'ordre des messages suit leur ordre dans le fil, pas un dernier identifiant ; `bridget attach` est admis.
- FR-09810 : envoi, suivi, réponse liée, canon, ledger et idempotence existants sont réutilisés ; aucun nouvel outil MCP, aucune nouvelle trame du protocole daemon.
- FR-09811 : tout ce qui dépend de t3code (routes, champs, commande `t3`, fichier d'état) est isolé en un seul point de Bridget.
- FR-09812 : aide, README FR/EN, skill et inventaire des commandes documentent l'intégration, ses limites (portées du jeton, absence d'émission en v1, course résiduelle) et la procédure de retrait.

## Entités
Environnement t3code (adresse, fichier d'état, jeton avec identifiant de session et expiration), fil t3code (identifiant, fournisseur, instance, dossier, tour actif), agent hébergé (identité Bridget stable, canal `t3code`), remise vers un fil (message, tour, état), curseur de journal par fil, manifeste d'installation (jeton, service).

## Critères de succès
- SC-09801 : installation, réinstallation et retrait en moins de 10 s chacun ; état final identique à l'attendu ; zéro écriture dans les fichiers de t3code.
- SC-09802 : un fil t3code apparaît dans `who` avec le canal `t3code` en moins de 10 s après son ouverture, disparaît en moins de 10 s après archivage, garde le même UUID après redémarrage du pont.
- SC-09803 : un message envoyé à un fil est visible dans l'application en moins de 5 s hors tour actif ; la demande devient `answered` avec la réponse de l'assistant quand aucun message humain ne s'intercale, sinon elle reste ouverte avec ambiguïté journalisée ; rejeu même clé : un seul tour ; tour actif : attente bornée puis indéterminé nommé ; deux demandes simultanées : deux tours successifs, jamais concurrents.
- SC-09804 : après redémarrage de t3code puis de Bridget, aucune intervention humaine, identités conservées, zéro tour doublé, zéro ligne de journal doublée.
- SC-09805 : recette réelle avec le t3code installé de l'humain : un agent Codex de terminal envoie une mission à un fil Claude de t3code, la lit dans l'application, et reçoit la réponse liée.
- SC-09806 : fmt, clippy et suite complète du workspace verts ; tests de fixture sans réseau ni compte.

## Hypothèses et limites
t3code installé localement sur la même machine que le daemon Bridget, une seule instance à la fois, CLI `t3` disponible (npm) pour l'émission du jeton. Le pont n'est ni un lanceur d'agents t3code ni une interface : t3code garde le pilotage. Le jeton émis par `t3 auth session issue` porte des portées administratives : limite acceptée et documentée, révocable. L'émission de messages Bridget par un agent hébergé (v2) exige une attestation publique fil ↔ processus MCP que t3code n'expose pas aujourd'hui ; aucune identité n'est déduite d'une variable d'environnement. Sans précondition atomique côté t3code, une course entre l'observation « pas de tour actif » et le démarrage du tour reste possible : elle est bornée, documentée et testée, jamais masquée.
