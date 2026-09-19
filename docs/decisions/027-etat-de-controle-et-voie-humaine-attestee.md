# ADR 027 — État de contrôle du référent et voie humaine attestée

**Date** : 2026-09-02
**Statut** : Accepté
**Portée** : SPEC-087, daemon Bridget, plugin le service compagnon, relais UI

## Contexte

Le coordinateur sait s'ouvrir du travail : routines, rondes, réassignations, continuations. Rien ne permet au référent de suspendre cette autonomie, de lui imposer un objectif prioritaire, ni d'être prévenu quand le système attend sa décision. Sur la fenêtre D des métriques d'efficience, 100 objectifs ont été créés pour 26 clos, et le référent a constaté que le système « ne l'écoutait plus ». L'ADR-014 a rendu représentable une origine d'objectif « demandée par l'humain » mais aucun composant n'émet l'attestation qui la conditionne. L'état « intervention humaine requise » existe dans le service compagnon sans canal sortant.

## Décision

1. **L'état de contrôle vit dans le daemon Bridget**, plan de contrôle au sens de l'ADR-015 : une ligne persistante `control_state` (pause, plafond d'objectifs auto-générés, génération) et un journal `control_events`. Il est exposé par le protocole (`ControlStateRead`, `ControlStateSet`). le service compagnon le lit à chaque relève et n'en conserve aucune copie ; un daemon injoignable rend un état inconnu qui **différe** toute ouverture automatique.
2. **Chaque plan possède une garde unique** `admit_autonomous_effect(effet, état)` appelée par tous ses puits d'effet autonome : ronde, relances et continuations côté daemon ; routines, réassignation, rejeu d'outbox, déblocage de dépendance et dispatch différé côté le service compagnon. Un puits qui ne l'appelle pas est un défaut. La reprise des remises déjà acceptées et la réconciliation de flotte restent admises pendant la pause ; un tour fournisseur en cours est interrompu et mémorisé pour reprise à la levée.
3. **Le focus est un attribut d'objectif le service compagnon**, plan de mission : un seul actif, une file derrière. Son ouverture passe par le guichet, avec une origine humaine.
4. **L'attestation d'origine humaine est fabriquée par le daemon**, jamais acceptée d'un client. Quand le principal humain dépose une délégation, le daemon enregistre le message au ledger, scelle son contenu, lie le hash canonique du dépôt et transmet l'origine au service compagnon, qui rejoue les cinq vérifications de `ObjectiveOpeningPermit::human_request` et consomme le message une seule fois.
5. **La boîte de réception humaine vit dans le daemon**, comme le guichet : une table transport où le daemon et le service compagnon déposent, que seul le référent tranche, et dont le service compagnon relève les décisions sans rien marquer. Une décision n'est acquittée qu'après le commit de son application par le service compagnon (`HumanInboxAck { decision_id }`). Un canal externe personnel, une commande configurée dans un fichier 0600, pousse un résumé borné.
6. **Le plafond d'objectifs auto-générés** est un réglage du contrôle, appliqué par le service compagnon à l'ouverture ; les objectifs d'origine humaine n'y entrent pas.

## Borne de la garde, écrite pour ne pas dériver

La reconnaissance du principal humain repose sur l'attribution d'émetteur du daemon et, en ligne de commande, sur un terminal interactif. C'est la borne de l'ADR-011 : elle arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte. La contre-revue du 2026-09-02 a montré que `replace_stale_ui_human_route` laisse toute connexion `ui` sous l'UUID humain évincer la route vivante. Nous ne prétendons pas fermer ce chemin ; nous le rendons visible : une reprise d'une route encore vivante dépose un item `human_route_replaced` poussé sur le canal externe, que l'agent ne lit pas. Le second facteur de l'ADR-011 reste la sortie réelle ; la boîte est conçue pour l'accueillir.

## Conséquences

**Positives.** Le référent dispose d'un frein d'urgence persistant, d'une commande d'une phrase pour imposer son objectif, d'un endroit unique où arrivent les décisions qui l'attendent, et d'une garde permanente contre l'emballement. La voie humaine de l'ADR-014 est fermée sans nouveau mécanisme d'attestation. Les vocabulaires existants sont réutilisés : `AutonomyRuntimeState::Paused`, `RoutineOccurrence::Differee`, `priority_class`, notification native ADR-016.

**Négatives.** Deux tables de plus dans une base sans propriétaire de schéma unique, à reprendre avec la consolidation. Une migration le service compagnon de plus. Une commande externe exécutée par le daemon, bornée par un fichier privé à chemin absolu. Le rejeu des outboxes et le déblocage des dépendances passent par une garde de plus.

## Amendement 2026-09-03

Le référent a décidé que la pause arrête immédiatement les tours fournisseurs en cours. Bridget écrit donc une ligne dans `control_pause_interruptions` avant d'émettre `Interrupt`, puis ne reconstruit que ces exécutions lorsque la pause est levée et que l'agent est connecté et libre, ou à sa prochaine reconnexion. Cette décision ajoute une table et un chemin de reprise. Le cas où deux candidats réclament le même agent reste volontairement refusé, sans arbitrage automatique.

## Alternatives écartées

- État de contrôle dans un fichier partagé ou dans la config le service compagnon : pas d'atomicité entre processus, non modifiable par l'interface.
- Laisser un client déclarer l'origine humaine : forgeable, refusé par l'ADR-014.
- Boîte de réception dans le service compagnon : pas de processus résident pour pousser vers l'extérieur, pas d'accès direct de l'interface.
- Client Telegram dans le daemon : dépendance et secret dans le dépôt.
