# Contrôles — SPEC-166

Date : 2026-10-15.

## Résultats acquis

| Contrôle | Résultat |
|---|---|
| Oracle rotation A vers B | 1 PASS, code retour 0 |
| Adaptateur Codex, gestionnaire de sessions et registre privé | 233 PASS, code retour 0 |
| Attestation Codex ciblée | 8 PASS, 13 filtrés, code retour 0 |
| Format des trois fichiers affectés | code retour 0 |
| Lint ciblé | code retour 0, deux avertissements préexistants hors diff |
| `git diff --check` | code retour 0 |

## Typecheck

Le typecheck complet du serveur retourne 1 sur des erreurs héritées dans les bancs d’interop Bridget et dans `CodexMcp.ts`. La première exécution a aussi détecté une erreur de type dans le helper ajouté. Cette erreur a été corrigée. La seconde exécution ne signale plus aucune erreur dans les fichiers affectés. Elle conserve seulement des suggestions préexistantes dans `CodexAdapterV2.ts`.

## Suite d’attestation complète

La suite complète `BridgetPermissions149.test.ts` conserve 11 échecs Claude `catalogue_unavailable`. Les 8 tests Codex du même fichier passent. Ces échecs ne passent pas par le code modifié et ne sont pas présentés comme GREEN.

## Invariants prouvés

- B est remontée avant le premier tour après rotation.
- A ne peut plus lire les droits du nouveau run.
- B reçoit les droits du nouveau run.
- Le même identifiant natif est repris avec `excludeTurns: true`.
- Une reprise refusée empêche toute requête `turn/start` suivante.
- Une preuve inchangée ne produit pas un second remontage.
