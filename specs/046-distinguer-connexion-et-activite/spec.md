# Spécification 046 — Distinguer connexion et activité

**Statut** : Prêt pour revue

**Base gelée du lot** : `bc745335530985ce305e82fea4007071c752d5b0`

**Objectif** : `8b5da55d-e37a-4d65-9e68-eb00e70d1ef3`

## Problème mesuré

Le 27 août 2026, la ronde a classé dix agents connectés comme muets avec des
âges `last_seen_secs` compris dans une fenêtre de neuf secondes. Le ledger
montrait pourtant neuf activités plus récentes que le seuil de trente minutes.

Une sonde réelle depuis `rc5` a confirmé le défaut : l'âge valait 3 874 s
avant l'envoi, 3 874 s juste après et 3 877 s après 2,5 s. Le message était
pourtant présent dans le ledger sous l'identifiant
`mcp-3245475-6a905620-58`. L'instrument voyait donc bien une activité durable,
mais l'annuaire ne la datait pas.

Le champ public ne mesure pas un âge de connexion pur. Il expose
`capacity_seen`, initialisé à l'enregistrement puis rafraîchi par les tours, le
runtime, le modèle servi, les limites et l'usage. Le heartbeat possède une
horloge séparée, `link_seen`. Les deux chemins d'envoi de message sont les
écrivains de capacité manquants.

## Propriété

Un message réellement accepté au nom d'un agent enregistré atteste une
capacité actuelle de cet agent. Il rafraîchit `last_seen_secs` après
l'acceptation, sur l'envoi idempotent comme sur l'envoi historique.

Un heartbeat, la seule réception d'un mandat, un expéditeur non enregistré ou
un envoi refusé ne constituent pas cette preuve et ne rafraîchissent pas
l'horloge.

## Scénarios

### US1 — Agent ancien mais actif maintenant

Une présence enregistrée depuis plus de trente minutes émet un message accepté.
La projection suivante rend `last_seen_secs < 2`, sans réenregistrement.

### US2 — Refus sans faux rajeunissement

La même présence émet vers une cible invalide. Le refus ne doit pas faire
passer une activité inexistante pour une capacité récente.

### US3 — Deux voies d'envoi cohérentes

L'envoi idempotent du canal MCP et l'envoi historique produisent la même mise à
jour d'activité après leur propre point d'acceptation durable.

### US4 — Réception sans activité émise

Une présence ancienne reçoit réellement un mandat sur sa connexion. La remise
arrive au destinataire, mais son `last_seen_secs` reste ancien tant qu'il
n'émet aucune activité propre.

## Exigences fonctionnelles

- **FR-4601** : `SendIdempotent` rafraîchit la présence de l'expéditeur logique
  seulement après la préparation durable de la remise.
- **FR-4602** : `Send` rafraîchit la présence de l'expéditeur logique seulement
  après l'écriture durable dans le ledger.
- **FR-4603** : le rafraîchissement résout une présence enregistrée par son nom ;
  il ne crée aucune présence et ne modifie aucun transport.
- **FR-4604** : `Heartbeat` continue de ne toucher que l'horloge de lien.
- **FR-4605** : les deux outils de ronde continuent de consommer
  `last_seen_secs` sans nouveau contrat filaire.
- **FR-4606** : une activité émise par une connexion MCP ou CLI auxiliaire ne
  rafraîchit jamais `link_seen` et ne prolonge donc pas le retain du wrapper.
- **FR-4607** : la seule remise d'un message au destinataire ne rafraîchit
  jamais sa capacité ; réception et émission restent deux faits distincts.

## Critères de succès

- **SC-4601** : une présence vieillie à 1 900 s puis un envoi idempotent accepté
  rend `last_seen_secs < 2`.
- **SC-4602** : le même scénario par `Send` historique rend
  `last_seen_secs < 2`.
- **SC-4603** : un envoi refusé conserve un âge d'au moins 1 800 s.
- **SC-4604** : retirer le rafraîchissement post-envoi fait mourir les témoins
  d'activité sans faire mourir le contrôle de refus.
- **SC-4605** : après l'envoi, l'âge du lien principal reste supérieur à
  soixante secondes tandis que l'âge public d'activité devient inférieur à
  deux secondes.
- **SC-4606** : un destinataire vieilli à 1 900 s reçoit effectivement un
  mandat, puis conserve un âge d'au moins 1 800 s.

## Consommateurs et portée

Les décisions de silence vivent dans `scripts/bridget-idle.py` et
`scripts/bridget-ronde.py`. Le protocole, la CLI JSON et MCP exposent le champ
sans prendre de décision. Maicie le désérialise sans le lire ; l'UI calcule une
autre mesure mêlant messages entrants et sortants, impropre à la ronde.

L'annuaire atteste une connexion d'agent, jamais l'ouverture d'un lot. L'outil
de ronde joint cette source avec les participants actifs de Maicie ; une
branche Git n'atteste pas le cycle de vie du lot. Cette distinction est
documentée mais n'élargit pas le correctif.

## Hors périmètre

- Ajouter un âge de connexion stable : cette information n'est pas conservée
  aujourd'hui et demanderait une horloge `connected_since` distincte.
- Compter un message reçu comme activité de son destinataire.
- Modifier le heartbeat, le routeur, le ledger ou le transport de remise.
- Déduire l'état d'un lot depuis l'annuaire ou une branche distante.
