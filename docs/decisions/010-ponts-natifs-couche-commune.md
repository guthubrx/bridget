# ADR 010 — Ponts natifs et couche commune de session

**Date** : 2026-08-24
**Statut** : Accepté
**Portée** : bloc G du catalogue de coordination v2

## Contexte

Bridget pilote ses agents gérés en écrivant lui-même les messages du
protocole ACP en JSON sur les flux d'un processus fils. Il ne dépend
d'aucune bibliothèque ACP : `acp.rs` implémente le protocole à la main
(~2510 lignes), sous un canon interne déjà neutre (`protocol.rs`, ~1900
lignes) qui porte `Register`, `Send`, `TurnState`, `Runtime`,
`Availability`, `AgentInfo` et le journal d'attache.

Le processus fils est fourni par un tiers : `@zed-industries/codex-acp` et
`@zed-industries/claude-code-acp`. Ces paquets **embarquent une copie figée**
du CLI du fournisseur.

Dans la nuit du 23 au 24 août 2026, deux agents gérés Codex sont morts à la
réception de leur premier message, refusés par l'API : « The 'gpt-5.6-terra'
model requires a newer version of Codex ». La version 0.16.0 est la dernière
publiée du paquet : il n'existe pas de mise à jour. Le CLI Codex installé
localement (0.149.0) fait tourner ce même modèle sans difficulté, ce qui
explique que les agents en panes tmux n'aient jamais été touchés. Le même
défaut, en plus discret, affecte le pont Claude : un agent relancé sur
« Opus 5 » a rendu son verdict en annonçant Opus 4.6, sans qu'aucun signal
ne remonte la substitution.

Le diagnostic a coûté deux hypothèses fausses avant d'être établi par le
journal de l'agent, alors que le message d'erreur y figurait dès la première
minute.

## Décision

**1. Sortir des ponts tiers pour Codex et Claude, au profit des accès natifs.**
Codex expose `codex app-server` — JSON-RPC délimité par newline, sans champ
`jsonrpc`, séquence `initialize`/`initialized` puis `thread/start` et
`turn/start`, saturation signalée par le code -32001. Claude expose le mode
de flux bidirectionnel de son CLI (`--input-format stream-json
--output-format stream-json`). Les deux parlent au binaire installé, donc à
jour.

**2. Le critère n'est pas « natif contre ACP », c'est « maintenu par le
fournisseur contre pont tiers figé ».** Cursor expose lui-même un serveur
ACP (`cursor-agent acp`) : c'est sain, et notre pont existant le sert sans
une ligne de code supplémentaire. ACP est donc conservé pour Cursor et
Gemini, et inscrit comme obsolète pour les deux autres.

**3. Adopter une couche commune MINCE au niveau de la session fils**
(option (a) de l'instruction G1), plutôt qu'un code par fournisseur sans
abstraction. Un trait `ManagedSession` — livrer, drainer des événements,
activer le journal, annuler, arrêter — et un type d'événement canonique
dérivé de l'existant, **préservant les octets bruts** et la source de chaque
événement.

**4. Poser le trait AVANT le second adaptateur.** Sans cela on obtient
l'option (b) de fait, et `wrapper.rs` — déjà couplé à `AcpEvent` — devient
un fourre-tout où la boucle principale, la reconnexion et le journal sont
dupliqués par protocole.

**5. Généraliser le gate d'attache.** `attach` exige aujourd'hui
`PresenceMode::Acp`. Migrer Codex sans lever cette contrainte tuerait
l'attache pour ces agents — c'est-à-dire l'outil qui a permis, la même nuit,
de retrouver trois livraisons perdues et de diagnostiquer la panne. Le
critère devient « le pilote expose un journal append-only », pas « le pilote
parle ACP ».

**6. Ne pas inventer de canon unifié pour ce qui ne l'est pas.** Le modèle,
l'effort, la consommation et les quotas restent des enrichissements
optionnels portés par source, selon le mécanisme `RuntimeSource` existant.
Aplatir ces signaux ferait perdre de l'information réellement disponible —
ce serait une régression déguisée en simplification.

## Conséquences

**Positives.** Les modèles récents redeviennent accessibles aux agents
gérés. Le flux natif porte des informations que le pont tiers ne remonte
pas : l'état d'occupation, le modèle exact, la consommation par tour, et les
limites de quota avec leur instant de réinitialisation — cette dernière
aurait permis d'anticiper la panne du 23 août à 22h27 au lieu de la subir.
Côté Codex, on peut en outre *interroger* les limites avant d'affecter une
mission longue, et détecter le reroutage silencieux d'un modèle.

**Négatives.** Deux à trois pilotes à maintenir au lieu d'un. Le coût est
concentré côté Claude, où les permissions interactives dominent le budget —
estimé trois à cinq semaines-équivalent, contre une à deux pour Codex, sur
la foi des tailles observées chez T3 Code (adaptateur Claude 4597 lignes,
Codex 2010, Cursor 1188). Risque de régression sur l'attache et
l'idempotence pendant le refactor.

**Réversibles.** Le pont ACP reste en place pendant toute la migration ;
aucune étape ne coupe l'existant avant que son remplaçant soit prouvé en
production.

## Alternatives écartées

**Attendre une mise à jour du pont tiers** : 0.16.0 est la dernière version
publiée, il n'y a rien à attendre.

**Coder par fournisseur sans couche commune** : économise environ 800 lignes
au démarrage, mais duplique la boucle principale, la reconnexion et le
journal ; fragmente `who` et le greffe par fournisseur ; et condamne
l'attache aux seuls agents ACP.

**Reprendre le code de T3 Code** (MIT) : leur code est en TypeScript, le
nôtre en Rust. On reprend leur *discipline* — interface étroite, événements
canoniques, octets bruts préservés, capacités déclarées, adaptateur lourd
isolé — pas leurs lignes.

## Sources

Instruction G1 rendue le 2026-08-24 (lecture seule, agent tiers). Mesures
propres : schéma du protocole app-server généré localement (37 fichiers,
579 définitions, version 2) ; exécution réelle du flux Claude en JSON ;
inventaire des adaptateurs de T3 Code. Le catalogue v2 porte le détail des
constats, blocs G1 à G11.
