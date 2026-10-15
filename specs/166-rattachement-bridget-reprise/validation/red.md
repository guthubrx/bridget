# Oracle RED puis GREEN — SPEC-166

## RED

Commande ciblée :

`pnpm exec vp test run apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts -t "remounts a rotated private identity"`

Résultat avant correction : code retour 1.

Le rejeu attendait `config/read`, puis `thread/resume` avec la preuve B. Le code a envoyé directement `turn/start` avec l’identifiant natif existant. Cette différence reproduit le défaut de rattachement.

Un premier passage a révélé un chemin temporaire trop long dans la fixture. La fixture a été raccourcie avant de retenir la preuve RED. Aucune conclusion fonctionnelle n’a été tirée de ce premier passage.

## GREEN

La même commande retourne 0 après correction : 1 test réussi, 162 ignorés par le filtre.

Un second contrôle ciblé retourne 0 : 2 tests réussis. Il couvre aussi le refus d’une preuve issue d’une autre session fournisseur.

Le rejeu observe un seul remontage avec B, puis `turn/start`. La lecture des droits retourne `unavailable` pour A et `valid` pour B.
