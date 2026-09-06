# Recherche 090 — faits et décisions

## Environnement constaté le 2026-09-05

Codex CLI 0.153.4, Rust 1.92.0. Source de départ 6cfbc4d33ca7.
`codex app-server --help`, `proxy --help`, `queue --help` et schéma généré avec
`app-server generate-json-schema --experimental` lus. Aucun agent connecté dans
`bridget who` : contre-revue autre fournisseur indisponible à ce stade.

## Sonde réelle, sans secret ni fournisseur distant

`python3 specs/090-codex-interactif/probe_shared_session.py` et variante `--approval`.
Le processus Codex est réel ; seule sa réponse de modèle est synthétique, servie
en boucle locale. HOME/CODEX_HOME/endpoint isolés, processus identifié puis arrêté.

1. `app-server proxy --sock` ne parle PAS le WebSocket exposé par `--listen unix://`.
   Deux négociations sans réponse, puis GET Upgrade réel → HTTP 101. Ne pas en
   faire le raccord de production. Le proxy vise un autre canal de contrôle.
2. Deux connexions WS Unix : initialize/initialized acceptés ; thread/loaded/list
   expose le même fil aux deux clients.
3. Avant premier tour, resume échoue « no rollout found ». `thread/name/set`
   avec le nom Bridget matérialise le fil ; resume renvoie alors le MÊME thread.id,
   sans prompt fabriqué, sans second fil, sans accès direct à la base Codex.
4. Après reprise, turn/started, deltas, item/completed et turn/completed sont
   reçus par les deux clients. La TUI native peut être le deuxième client.
5. Une requête réelle exec_command require_escalated produite par la réponse
   synthétique provoque requestApproval chez A ET B (même id=0). La décision
   decline de B suffit. Bridget doit observer, jamais répondre automatiquement
   aux permissions/élicitations en mode interactif.

La sonde seule prouve le protocole, pas l'intégration Bridget/TUI ni l'abonnement
réel. Ces deux preuves ont ensuite été exécutées en T007/T012 ; résultats et
commandes reproductibles dans implementation.md.

## Décision

Réutiliser CodexAppServerTransport et la boucle managed-wrapper ; leur fournir
un flux compatible Read/Write depuis WS sur socket Unix privée. Dépendance
tungstenite synchrone, sans TLS ni runtime async : implémenter RFC6455 à la main
dans le produit serait moins sûr et plus coûteux. La sonde Python est jetable
de conception et n'est ni une bibliothèque ni un transport de production.

Le pilote possède le serveur enfant ; la TUI en hérite via `--remote` et reprend
le fil nommé. Pas de proxy enfant supplémentaire, pas de nouveau daemon métier,
pas de multiplexeur JSON-RPC fabriqué. Pas de port TCP. Les permissions restent
celles du CLI utilisateur, distinctes du réglage headless allow du registre.

Les tours TUI exigent une observation explicite dans le reader existant. Une
notification inconnue reste source brute, pas un état inféré. Le worker Bridget
attend le terminal attesté d'un tour externe avant le prochain turn/start.

## Alternatives écartées

- tmux/PTY injection de texte : contredit la demande, fragile vis-à-vis de la saisie.
- nouvelle TUI Bridget : supprime l'interface native au lieu de l'intégrer.
- queue seule : ne donne pas à elle seule l'identité, l'ACK de consommation et le journal.
- proxy officiel sur la socket WS : incompatible empiriquement.
- DTO/crate de protocole complète : le pilote utilise déjà JSON et le schéma du
  binaire. Aucun intérêt à ajouter une deuxième version de types OpenAI.

## Sources

- https://learn.chatgpt.com/docs/app-server : cycle start/resume/turn, sockets Unix
  WebSocket, approbations corrélées et notifications ; statut expérimental.
- https://learn.chatgpt.com/docs/developer-commands?surface=cli : interface native
  remote/resume. L'aide du binaire installé fait foi pour ses options concrètes.
- ADR 010 du projet : couche commune mince, raw/provenance, journal indépendant du protocole.

Baseline architecture utilisateur consultée ; ses risques de protocole jeune
motivent la sonde locale et la recette versionnée. DevKMS `mem` absent du PATH :
connaissances conservées ici. Mémoire projet Codex absente du répertoire catalogué.
