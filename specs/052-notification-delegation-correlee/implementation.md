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
