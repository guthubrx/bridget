# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-tests-sonnet-r4.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-tests-sonnet-r4.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `native_permission_observer149_tests` écrit le PATH du processus de test une seule fois (`Once`, `set_var`). Aucune production modifiée. Mitigation vérifiée : le module passe 10/10 en série (`--test-threads=1`) en plus de la batterie parallèle. Un risque de concurrence théorique demeure pour d'autres tests qui lisent le PATH pendant l'écriture. Aucun flocon sur 2 batteries r3 et 1 batterie r4.
- Les fixtures `claude` sont des copies de `/bin/echo` : jamais une preuve de comportement du modèle. T037/T038 restent à faire.
- `wrapper::reconnect_tests::le_domaine_derive_nomme_le_depot_courant` exige `TMPDIR` hors de `/Users/moi` (ce dossier contient un `.git`).
- Non vérifié : HEAD exécuté (preuve statique pour les 3 anciens échecs), recette T3 réelle, hooks Claude réels.
- Lignes que j'aurais pu supprimer et que je garde : aucune.
