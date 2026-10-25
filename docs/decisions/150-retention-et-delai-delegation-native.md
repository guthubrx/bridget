# Décision150 - rétention, admission, pause et délai de la délégation native

Date : 2026-10-11. Statut : accepté (plan validé après trois rondes de
contre-revue Codex, voir `specs/150-socle-delegation/adversarial-review-codex.md`).

## Contexte

La délégation native (148-149) garde toutes ses tâches. L'admission compte
l'historique cumulé : après 4096 délégations, elle refuse pour toujours
(`task_limit`). La pause du référent n'est pas consultée à l'admission. Le
délai de réponse vaut toujours 3600 s.

## Décision

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/150-retention-et-delai-delegation-native.md#extrait-1).

## Conséquences

Positives : la délégation reste disponible sans limite de durée ; aucun
résultat non accusé n'est perdu ; l'admission ne lit et ne décode plus chaque
tâche : elle fait des comptages SQL (actives par l'index d'état, total en O(n)
avec n ≤ 4096) ; aucun contrat existant ne change de forme.

Négatives : une remise indéterminée bloque son arbre ; si ces arbres
remplissent la borne, la délégation est refusée. Les tables d'exécution,
d'idempotence et de flotte liées aux tâches purgées ne sont pas purgées (dette
notée). La suspension des missions en cours pendant une pause est transmise à
la session152.
