# Spécification 119 - Les avis d'état de source ne réveillent plus pour rien

## Fiche synthèse
Spec: 119-avis-de-source-apaises | Statut: Implemented | Priorité: P1 | Date: 2026-09-25
Branche: session-119-avis-de-source-apaises | Suite de la 118, demandée par l'utilisateur.

## Problème observé
Chaque avis d'observation est remis comme un message et démarre un tour chez son destinataire.
Dans la nuit du 25/09, une source qui clignotait (défaut corrigé en 118) a produit 222 avis vers
`sol_city_ai` : 197 tours démarrés en trois heures sur un modèle en réflexion élevée, 37 réponses
qui ne faisaient que commenter « interrompu », « rétabli ». Le daemon émettait un avis à chaque
changement du nombre de sources compatibles, sans aucun amortissement.

## Exigences
- **FR-001** : l'état consultable (`events list`) change aussitôt ; l'avis à l'abonné n'est remis
  qu'après 30 secondes de stabilité.
- **FR-002** : un aller-retour bref vers l'état déjà annoncé ne produit aucun avis.
- **FR-003** : une source qui ne se stabilise pas est signalée « instable », avec le nombre de
  bascules, au plus une fois toutes les 5 minutes.
- **FR-004** : les fins de tour, permissions, écritures et lacunes sont remises comme avant.

## Hors périmètre
- Remettre des avis sans démarrer de tour : T3 n'offre qu'une activité (`thread.activity.append`),
  invisible pour l'agent ; l'avis doit lui parvenir.

## Critères de succès
- **SC-001** : dix minutes de bascules toutes les 3 s produisent un seul avis, « source instable ».
- **SC-002** : une interruption durable produit un avis, une seule fois.
- **SC-003** : recette complète verte.
