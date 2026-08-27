# Journal d'implémentation 052

## Métadonnées

- **Spec** : 052-notification-delegation-correlee
- **Branche** : session-052-notification-delegation-correlee
- **Démarré** : 2026-08-27
- **Terminé** : En cours

## Progression

### T5201 — Formaliser le contrat et l'inventaire

- **Statut** : Complété
- **Faits mesurés** : construction immédiate et différée, sérialisation durable,
  reprise sans reconstruction et surfaces réellement injectées au destinataire.
- **Décision** : notification autonome portant les trois identifiants ; la
  préannonce est rejetée car elle conserve le second mandat et le délai.
- **Tests** : non applicable, documentation seule ; `git diff --check` requis
  avant commit.

### T5202 — Fermer le chemin immédiat

- **Statut** : Complété
- **Oracle rouge avant production** :
  `spec_052_delegation_immediate_transmet_les_trois_identifiants_dans_le_message_reel`
  atteint l'assertion finale puis rend `0 passed / 1 failed`, car le corps lu
  dans le `PublicMessage` ne contient encore que l'instruction.
- **Raccord** : le `message_id` est créé avant l'outbox, puis une règle du
  domaine finalise l'instruction avec les trois identifiants durables.
- **Contrôle nominal** : oracle ciblé `1 passed / 0 failed`; le témoin de cible
  de revue reste `1 passed / 0 failed` et conserve son texte initial.

### T5203 — Fermer le chemin différé

- **Statut** : Complété
- **Oracle rouge avant production** :
  `f37_depends_on_cree_arete_et_deblocage_a_la_cloture` ferme un vrai dernier
  prérequis, trouve l'outbox créée, puis rend `0 passed / 1 failed` sur le corps
  final encore limité au but.
- **Raccord** : la transaction de déblocage crée le `message_id`, finalise la
  délégation, persiste ce nouvel état et sérialise les mêmes octets dans
  l'outbox.
- **Contrôles nominaux** : chemin différé `1 passed / 0 failed`; chemin immédiat
  rejoué `1 passed / 0 failed`.
