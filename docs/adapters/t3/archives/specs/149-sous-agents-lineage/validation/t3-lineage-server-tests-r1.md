# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-server-tests-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-server-tests-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Session : 149-sous-agents-lineage. Lot : L (serveur).
Auteur : sous-agent test du lot serveur. Périmètre : 5 fichiers test + ce rapport.
Date : 2026-10-10. Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

- Oracles issus du contrat fermé lineage.md. Jamais adaptés au code, sauf deux cas
  documentés : budget journal (§6, constante hors contrat) et branche stop re-ciblée (§9).
- Aucune édition production. Aucun fichier d'un autre agent touché.
- Aucun cochage Git. Aucun commit. Aucun restart. Aucune config/DB de production.
- Aucun `pnpm install`. Aucune copie node_modules. Cache `t3-final-148` intact.
- Aucun succès 149 affirmé au-delà des preuves ci-dessus.
