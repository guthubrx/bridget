# Session 063 — Interruption et pilotage d’un tour humain

**Statut** : en cours  
**Branche** : `session-063-interruption-pilotage-tour-humain`  
**Base gelée** : `d5a362d613191747550460e79475328f3d72f418`

## Objectif

Lorsqu’un humain écrit à un agent pendant un tour actif, le système doit faire
prendre ce message en compte immédiatement : l’agent peut être piloté sans
interruption lorsque le fournisseur le permet, ou interrompu proprement dans
le cas contraire. L’humain ne pilote pas les processus lui-même.

## Scénarios utilisateur

### US1 — Interrompre un tour actif (P1)

Un humain envoie une instruction à un agent qui travaille. Le tour actif se
termine de manière reconnue par le transport et l’instruction humaine devient
traitable sans attendre la fin naturelle du travail précédent.

**Acceptation** : après mise en service, un horodatage de la route réelle
montre l’interruption d’un tour actif et la prise en charge du message humain.

### US2 — Piloter sans interrompre quand c’est possible (P1)

Pour un fournisseur qui accepte le pilotage en cours de tour, le message humain
est injecté dans ce tour sans perdre ni doubler les messages déjà acceptés.

**Acceptation** : le pilotage est borné dans le temps ; un rejet conserve l’ordre
des messages et une acceptation seule ne vaut pas confirmation de traitement.

### US3 — Préserver l’intégrité des transports (P1)

Les sorties tardives d’un tour interrompu, les autorisations en attente et les
états terminaux propres à chaque fournisseur ne polluent pas le tour suivant et
ne bloquent pas l’agent.

**Acceptation** : chaque transport distingue explicitement les états terminaux
attendus et maintient la corrélation avec le tour concerné.

## Exigences fonctionnelles

- FR-001 : un message humain adressé à un agent au tour actif déclenche la
  stratégie de pilotage applicable au transport concerné.
- FR-002 : la stratégie prioritaire pilote le tour quand le fournisseur le
  permet ; l’interruption est le repli explicite.
- FR-003 : les messages acceptés mais non encore traités ne sont jamais perdus
  lors d’une interruption, d’un rejet ou d’une sortie tardive.
- FR-004 : une demande d’autorisation en attente est réglée avant annulation si
  le fournisseur l’exige.
- FR-005 : les états terminaux d’interruption sont enregistrés comme tels, sans
  les confondre avec un succès ni avec une panne de transport.
- FR-006 : l’action de pilotage est bornée ; un fournisseur silencieux ne peut
  pas bloquer indéfiniment les pilotages ultérieurs.
- FR-007 : le changement ne relance pas les campagnes de mesures déjà archivées
  dans le document de chantier ; la vérification finale se limite à l’effet réel
  requis par ce document.

## Hors périmètre

- Arrêter le parc d’agents ou administrer directement ses processus.
- Ajouter une file prioritaire distincte pour les messages humains.
- Déduire un mode de pilotage pour un fournisseur sans preuve disponible.
- Rejouer les quatre campagnes historiques de mesure.

## Critères de succès

- SC-001 : un humain peut interrompre ou piloter un tour actif sur la route
  réelle, avec un horodatage conservé.
- SC-002 : un message humain n’est ni perdu, ni dupliqué, ni attribué au tour
  suivant par erreur.
- SC-003 : les trois transports supportés conservent une fin de tour lisible
  après l’action de pilotage.
- SC-004 : aucune action hors chantier n’est ouverte avant ce résultat.

## Hypothèses et dépendances

- Les formes et délais déjà mesurés dans le document de chantier constituent la
  référence ; seuls les points explicitement non mesurés peuvent justifier une
  nouvelle observation.
- La mise en service reste nécessaire : une intégration Git ne prouve pas un
  effet réel.
- La priorité du chantier suspend les travaux non liés jusqu’à SC-001.
