# Feature Specification: Identité d'agent robuste pour les commandes CLI

**Feature Branch**: `session-06-identite-agent-robuste`
**Created**: 2026-08-17
**Status**: Obsolète — besoin couvert par la filiation agent-pids livrée en sessions 008-010 (identité stable vérifiée en production le 2026-08-23) ; commit exploratoire 4e121fc archivé en patch local
**Input**: défaut constaté en usage réel — un agent qui envoie un message par
`bridget send` apparaît parfois sous un nom jetable `cli-send-<pid>`, ce qui rend
toute réponse impossible.

## Contexte

L'identité d'un agent, pour les commandes CLI, repose aujourd'hui uniquement sur
deux variables d'environnement transmises par le wrapper à son processus enfant.
Cette transmission fonctionne pour l'agent lui-même, mais pas pour tous ses
descendants.

Constat mesuré le 2026-08-17 sur un agent Codex réel : le processus agent porte
bien les deux variables, ses serveurs MCP n'en portent aucune. Codex lance ces
serveurs avec un environnement filtré par liste blanche, qui retient `HOME`,
`PATH`, `SHELL`, `TERM`, `USER` mais aucune variable applicative. Le shell d'où
le wrapper a été lancé n'en porte pas non plus, ce qui est normal puisque c'est
le wrapper qui les crée pour son enfant.

Conséquence en chaîne : la commande ne trouve aucune identité, `current_agent_name()`
retombe sur `human`, le daemon constate que `human` n'est pas un agent enregistré
et remplace l'émetteur par le nom de la connexion éphémère, `cli-send-<pid>`.
Le message est routé et livré, mais son émetteur a disparu. Le destinataire ne
peut plus répondre, et personne n'est averti.

Le ledger montre que le défaut est général : sur douze messages récents, six
portent un émetteur jetable.

**Ce défaut casse la réciprocité du protocole.** Une demande sans retour possible
est acceptée comme si elle était complète.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Un message envoyé depuis n'importe quel descendant d'un agent porte son nom (Priority: P1)

En tant qu'agent, j'envoie un message par un outil quelconque — mon shell, un
serveur MCP, un sous-processus — et le destinataire voit mon nom, donc peut me
répondre.

**Why this priority**: c'est le défaut constaté. Sans cela, une revue croisée
entre agents ne peut pas aboutir.

**Independent Test**: exécuter `bridget send` depuis un processus dont
l'environnement a été vidé de toute variable Bridget, mais qui descend d'un
agent, et constater que l'annuaire du destinataire montre le nom de l'agent.

**Acceptance Scenarios**:

1. **Given** un agent connecté, **When** un de ses descendants envoie un message
   sans variable d'environnement Bridget, **Then** l'émetteur affiché est le nom
   de l'agent.
2. **Given** ce même envoi, **When** le destinataire répond, **Then** la réponse
   parvient à l'agent.
3. **Given** un agent renommé, **When** un de ses descendants envoie un message,
   **Then** l'émetteur affiché est le nom courant, pas le nom initial.
4. **Given** un envoi depuis un shell qui ne descend d'aucun agent,
   **When** la commande est exécutée, **Then** le comportement actuel est
   conservé et l'utilisateur est averti que sa demande ne pourra pas recevoir de
   réponse.

---

### User Story 2 - Aucune dépendance à un terminal ou à un multiplexeur (Priority: P1)

En tant que mainteneur, je veux que la résolution d'identité fonctionne pour un
agent lancé hors tmux, sans terminal, ou sur un hôte fédéré par SSH.

**Why this priority**: Bridget est transport-agnostique par conception. Faire
dépendre l'identité d'un pane tmux reviendrait à en faire une dépendance
structurelle, et casserait la fédération et les exécutions non interactives.

**Independent Test**: exécuter la résolution dans un processus sans `TMUX` ni
terminal attaché et constater qu'elle aboutit.

**Acceptance Scenarios**:

1. **Given** un agent lancé sans tmux, **When** un descendant envoie un message,
   **Then** l'identité est résolue.
2. **Given** un environnement sans `TMUX`, **When** la résolution s'exécute,
   **Then** elle n'interroge aucun multiplexeur.

---

### Edge Cases

- **Agents imbriqués** : un agent lancé depuis le shell d'un autre agent a deux
  ancêtres candidats. Le plus proche dans la chaîne gagne, car c'est celui qui
  décrit réellement le processus courant.
- **PID recyclé** : un identifiant de processus peut être réattribué après la
  mort d'un agent. L'entrée doit être supprimée à l'arrêt du wrapper, et une
  entrée dont le processus n'existe plus doit être ignorée.
- **Chaîne longue ou cyclique** : la remontée est bornée et s'arrête à `1`.
- **Agent renommé** : l'entrée suit le nom courant.
- **Aucun ancêtre agent** : comportement inchangé, plus un avertissement.
- **Coût** : la résolution ne doit pas rendre une commande sensiblement plus
  lente, y compris quand elle échoue.

## Requirements *(mandatory)*

- **FR-001**: Une commande CLI MUST pouvoir déterminer l'agent auquel elle
  appartient même sans aucune variable d'environnement Bridget.
- **FR-002**: La résolution MUST s'appuyer sur la filiation de processus, et MUST
  NOT dépendre de tmux, d'un terminal attaché ou d'une variable d'affichage.
- **FR-003**: L'ordre de résolution MUST être : émetteur explicite, puis
  environnement, puis filiation, puis échec explicite.
- **FR-004**: Le lien entre un processus d'agent et son nom MUST être créé au
  lancement, mis à jour au renommage, et supprimé à l'arrêt.
- **FR-005**: Une entrée dont le processus n'existe plus MUST être ignorée.
- **FR-006**: La remontée de filiation MUST être bornée en profondeur.
- **FR-007**: Quand aucune identité n'est déductible et qu'un daemon est
  joignable, la commande MUST avertir que son message partira sous un nom
  jetable et ne pourra pas recevoir de réponse.
- **FR-008**: L'avertissement MUST aller sur la sortie d'erreur et MUST NOT
  empêcher l'envoi.
- **FR-009**: La résolution MUST rester silencieuse et sans effet de bord en cas
  d'échec, hors avertissement.

## Success Criteria *(mandatory)*

- **SC-001**: Un `bridget send` exécuté avec un environnement vidé de toute
  variable Bridget, depuis un descendant d'agent, affiche le nom de l'agent dans
  le ledger et dans l'annuaire du destinataire.
- **SC-002**: La réponse à un tel message parvient à l'agent.
- **SC-003**: La résolution aboutit sans `TMUX` dans l'environnement.
- **SC-004**: Un envoi hors de tout agent conserve son comportement actuel et
  produit un avertissement sur la sortie d'erreur.
- **SC-005**: Le surcoût d'une commande reste inférieur à 100 ms dans le cas où
  la filiation doit être remontée.
- **SC-006**: Aucune régression : suite de tests verte, comportement inchangé
  quand les variables d'environnement sont présentes.

## Assumptions

- La lecture du parent d'un processus est disponible sur macOS et Linux par la
  commande `ps`, déjà employée par le projet pour l'hôte et pour tmux.
- Le protocole n'est pas modifié : la correction est locale au client et au
  wrapper. Le daemon conserve sa logique de résolution existante, qui traite
  correctement un émetteur nommé.
- Le cas d'un agent dont le nom est usurpé volontairement reste hors périmètre :
  l'authentification de l'émetteur est un sujet transverse déjà consigné.

## Dépendances

- **`001-renommer-agent`** : le lien processus-nom doit suivre un renommage.
- **`002-federation-ssh`** : la résolution doit fonctionner sur un hôte distant,
  sans terminal.
- **`005-domaines-dnd`** : même motif de persistance locale que les domaines.
