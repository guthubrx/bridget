# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/adversarial-review-codex.md](../../../../../../specs/159-edition-file/adversarial-review-codex.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : sûreté d'une saga de retrait, de promotion et de déplacement répartie entre le daemon et les porteurs de file.
- **Verdict reçu** : BLOCKED (11 points, dont 4 critiques). La matrice de droits auteur / parent / humain est jugée cohérente.
- **Diagnostic de l'architecte** : les 11 points ont une racine commune. La file vit dans la mémoire de chaque porteur, donc toute édition est une saga entre deux processus. T3 évite cela parce que sa file est durable côté serveur. D'où l'étude d'une file tenue par le daemon (`research.md` R14).
- **Vérification** : chaque point est relu dans le checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`, non modifié).

<a id="extrait-2"></a>

## Extrait 2

- 159a : file retenue par le daemon, ouverte par un réglage par type d'agent (défaut : fermé). Aucune édition.
- 159b : `cancel`, `reorder`, `promote` sur les messages retenus, une transaction chacun.
- Hors lots : cibles T3 (le pont n'envoie pas `TurnState` et possède le regroupement de la session 114, `t3code.rs:44`, `:3126`, `:3150`). Option 159c : confier la file des fils T3 à la file durable de T3 (`queued-run.*`).

<a id="extrait-3"></a>

## Extrait 3

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : points 1, 2, 3, 5, 10 de la ronde 1 sans objet pour `held` ; point 7 cohérent ; points 4, 6, 8, 11 partiellement ouverts ; point 9 bien conçu mais tests seulement prévus. Exclusion des fils T3 jugée cohérente (regroupement 114, remise Claude/Cursor). Garde 156 requise pour toute promotion. 10 points nouveaux.
- **Principe** : chaque point est une hypothèse, relue dans le checkout principal. Quand une promesse était fausse, elle a été réduite plutôt que laissée en l'état.
