# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-tests-sonnet-r8.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-tests-sonnet-r8.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Méthode : copie jetable `/tmp/b149t-r8/mut` (supprimée), cible Cargo **séparée** (`bridget149-target-mut`, supprimée). Un motif unique est remplacé par mutation. L'arbre de travail n'est jamais modifié. Outil : `native-r8-mutate.py`. Résumés : `native-r8-mutations-summary.txt` (F1 à F10 valides ; ses lignes O, T2 et T4 sont périmées, banc moins strict), `-2.txt` (F10, O1 à O4, T4 avec le banc final) et `-3.txt` (T0 à T3 avec le faux serveur réactif) ; un journal par mutation `native-r8-mutation-*.log`.

<a id="extrait-2"></a>

## Extrait 2

| Id | Mutation | Tests qui échouent |
|---|---|---|
| F1 | `prepare_restart` ne ferme plus l'exécution | 5 (descendant perdu, capture réelle, hors ligne, mission engagée, terminal) |
| F2 | `prepare_restart` rouvre aussi les terminaux | terminal |
| F3 | exécution d'un autre agent acceptée | mauvais agent |
| F4 | autorité sans contrôle du corps | autorité retenue |
| F5 | autorité sans contrôle du destinataire | autorité retenue |
| F6 | autorité sans contrôle de révocation | autorité retenue |
| F7 | propriétaire hors ligne : faux `result_sent` | hors ligne |
| F8 | garde expéditeur par défaut relâché | message externe |
| F9 | autorité sans contrôle du propriétaire vivant | autorité retenue |
| F10 | `descendants_busy` ignore les missions fermées | ligne de reprise (test ajouté après un premier survivant) |
| O1 | wrapper sans gestionnaire SIGTERM | SIGTERM natif |
| O3 | wrapper natif se reconnecte sur EOF | EOF natif |
| T1 | arrêt Codex sans `turn/interrupt` | tour exact, idempotence |
| T2 | arrêt Codex sans fermeture d'admission | drainage (test durci après un premier survivant, voir plus bas) |
| T3 | arrêt Codex interrompt un autre tour | mission étrangère |
