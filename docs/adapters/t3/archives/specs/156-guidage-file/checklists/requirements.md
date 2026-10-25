# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/checklists/requirements.md](../../../../../../../specs/156-guidage-file/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Les noms d'outils (`bridget_send`, `bridget_queue`), de valeurs (`steer`,
  `interrupt`) et de refus (`invalid_intent`, `intent_not_permitted`,
  `interrupt_rate_limited`, `mission_active`, `referent_paused`,
  `steer_unavailable`) sont le contrat public du produit.
- Les chemins de code de « Besoin et périmètre » sont des preuves de constats.
- Cas limites couverts : trame brute avec intention, origine `Human` forgée,
  preuve enfant, cible d'un autre projet, soi-même, boucle d'interruptions,
  pause (dont l'interruption de pause interne), enfant en mission, tête de file
  du pont bloquée, fournisseur T3 sans guidage natif, rejeu d'une clé quand la
  cible change d'état, base 150 migrée.
- Périmètre réduit après contre-revue : `cancel`, `reorder`, `promote` sont
  hors périmètre (saga durable), justifié dans la spec et l'ADR.
- Trois questions ouvertes avec défaut noté. Aucune ne bloque la planification.
- Cadre de menace explicite (ADR 011) : agent par ses outils dans le modèle ; processus hostile à trames brutes hors modèle, limites prouvées dans la spec.
- Points de sécurité signalés : `Send` et `SendIdempotent` laissent passer
  `intent` et `origin` bruts ; `ControlExecution` n'a pas de contrôle de droits.
