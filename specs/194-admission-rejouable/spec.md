# Session 194 — Admission rejouable après refus de sonde

Correction autorisée le 28 octobre 2026. Un refus de revalidation après choix laisse actuellement le reçu à `decision_started`. Le rejeu de même époque ne le reprend pas.

## Résultat attendu
Le choix validé et son effort sont persistés avant les E/S de revalidation. Un refus temporaire garde `chosen` et permet le rejeu strict, sans nouveau décideur ni changement de cible. Toutes les gardes source, droits, identité, époque et préparation restent avant création de tâche. Un ancien reçu `decision_started` reste observable ; aucune modification manuelle ou réparation par redémarrage forcé.

## Limites
Ne pas rejouer la demande métier depuis ce chantier, ne toucher ni service Opus2D ni calcul en cours. Chargement seulement par maintenance native admise. Le maître 183 est bloqué par une ancienne clôture non attestée ; la livraison source ne prouve pas son activation.

## Validation
Reproduire un refus de sonde après décision, vérifier absence de tâche, rouvrir le store, rejouer la même clé après correction de la cause puis vérifier une seule tâche et le même choix. Vérifier aussi effort différent, rejeu concurrent, enveloppe différente et droits modifiés.
