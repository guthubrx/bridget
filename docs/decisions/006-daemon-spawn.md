# ADR 006 — Équipiers gérés par le daemon

**Date** : 2026-08-22
**Statut** : Accepté

## Contexte

Un équipier ACP lancé par le wrapper terminal de la session 007 disparaît avec
le terminal qui le possède. La persistance attendue ne consiste pas à laisser
des descendants détachés survivre sans contrôle : le daemon doit devenir
l'unique propriétaire du cycle de vie, conserver les garanties 007 (transport,
journal, annulation et état) et réconcilier explicitement toute trace après un
arrêt brutal.

La conception doit aussi préserver l'idempotence des ordres de spawn, ne pas
réintroduire une seconde boucle ACP dans le daemon, et empêcher qu'un enfant
exécute avant qu'un marqueur durable permette sa réconciliation.

## Décision

### D-501 — Lancer le wrapper existant

Le daemon lance le wrapper 007 existant, et non l'adaptateur ACP directement.
Le wrapper reste responsable du transport, de la reconnexion, du journal et du
chemin d'arrêt propre ; le daemon supervise son groupe de processus. Le hook
`managed-status` fournit les échecs de démarrage structurés : `stderr` reste
un artefact de diagnostic, jamais un protocole.

### D-502 — Retenir le bootstrap jusqu'au marqueur durable

Le daemon démarre un sous-mode `managed-bootstrap` dans un groupe de processus
dédié. Le bootstrap annonce `BootstrapReady`, puis attend exactement l'octet
`RELEASE`. Le daemon écrit et synchronise le marqueur (pgid, naissance,
instance, commande et génération) avant d'écrire cet octet. EOF avant
`RELEASE` fait quitter le bootstrap. Les descripteurs auxiliaires et le pipe
de libération sont `CLOEXEC` ; seul le canal `managed-status` reste hérité
jusqu'à ce que le wrapper le ferme explicitement.

### D-503 — Consommer le socle d'idempotence 012

La session 009 consomme `idempotency.rs` de la session 012 avec
`operation_kind = "spawn"` et `command_id` comme clé. `spawn_commands` ne
porte que l'état de la saga et ses liens métier. Une issue durable est rejouée
telle quelle ; une génération durable sans issue rattache le retry à son vrai
terminal, sans succès synthétique. L'ordre d'un spawn persistant est : état
désiré durable, issue après `Register` réel, réponse client.

### D-504 — Réconcilier avant toute reprise

Au démarrage, le daemon entre en phase visible `Recovering`, valide les
marqueurs contre leur naissance et leur instance, puis termine les groupes
orphelins par `killpg` et polling borné. Il ne supprime un marqueur qu'après
disparition confirmée ou échec terminal consigné. Les reprises persistantes
sont réservées dans un ordre déterministe avant l'ouverture normale ; les
nouveaux spawns sont refusés `DaemonRecovering`, sans casser le rejeu d'un
`command_id` déjà connu.

### D-505 — Construire l'environnement depuis sa source contrôlée

Avant toute construction, la garde `forbidden_env` est évaluée sur
l'environnement source du daemon. L'environnement enfant est ensuite construit
depuis une base portable (`HOME`, `PATH`, `USER`, `LANG`, `TMPDIR`) et le
`pass_env` déclaratif de l'entrée de registre. Le `cwd` absolu fourni par le
client est validé à l'ordre puis au spawn. Cette règle conserve les refus de
facturation et évite de recopier aveuglément l'environnement du client.

### D-506 — Résoudre `stop` dans la table superviseur

La table superviseur en mémoire, nommée par génération, est la source primaire
du `stop` : elle couvre aussi `Reserved` et `Starting`, avant l'existence d'un
marqueur. Le stop invalide d'abord la génération puis utilise le groupe de
processus si présent ; les marqueurs ne servent qu'au secours après crash. Un
wrapper terminal ne figure dans aucune de ces sources et reçoit donc
`NotManaged`.

## Conséquences positives

- Les garanties ACP de 007 sont réutilisées par le même wrapper plutôt que
  réimplémentées dans le daemon.
- Le protocole à octet rend la durabilité du marqueur antérieure à l'exécution
  de l'équipier et rend EOF non ambigu.
- La reprise, le stop concurrent et les retries sont corrélés par une
  génération et par le socle idempotent durable.
- Les environnements interdits sont refusés avant tout filtrage ou spawn.

## Conséquences négatives

- Le daemon devient superviseur de groupes de processus et doit gérer les
  marqueurs, la réconciliation et les délais bornés.
- Le lancement ajoute un bootstrap, un canal de statut et une phase
  `Recovering` visibles.
- Les arrêts brutaux ne permettent pas de garantir un nettoyage immédiat : la
  garantie devient la réconciliation au prochain démarrage.
