# Plan
1. Persister le choix déjà validé depuis la matrice attestée, avant sa revalidation finale. Réutiliser le parcours exclusif `route_selected_admission` qui revalide déjà la source et la cible avant tout effet.
2. Conserver le profil initial dans le snapshot. L’effort décidé reste dans la décision et le canon Delegate ; sa définition effective est vérifiée au même endroit avant Prepare.
3. Tests ciblés avant/après, revue indépendante, candidat cumulé, intégration et push. Aucun contournement de maintenance.

Complexité : suppression d’une revalidation redondante. Même borne de 128 candidats et même exclusion par clé ; aucune nouvelle dépendance ni nouvel état.
