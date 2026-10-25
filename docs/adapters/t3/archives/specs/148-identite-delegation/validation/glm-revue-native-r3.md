# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-revue-native-r3.md](../../../../../../../specs/148-identite-delegation/validation/glm-revue-native-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Pas de fuite credential ni d'historique parent.** La carte contient l'identité figée et des faits Git seulement : `wrapper.rs:329-392` (nom, type, protocole, digest, worktree, branche, tête, 40 fichiers max). Pas de conversation parent. Le fixture affirme l'absence d'environnement T3 (`tests/fixtures/native_delegation_148.py:13`). La recette réelle interdit les secrets dans le registre (`tests/native_delegation_real_glm.rs:310-317`).

<a id="extrait-2"></a>

## Extrait 2

**Docs.** `docs/delegation-native.md:47-51` décrit exactement le comportement : faits natifs joints au premier tour, ni conversation parent ni credentials T3. `specs/148-identite-delegation/contracts/delegation.md:38-41` (SendIdempotent, capture avant accusé) et lignes 15-20 (périmètres `same_project`/`root`) restent exacts. Aucun doc obsolète ne décrit l'ancien comportement à deux prompts.
