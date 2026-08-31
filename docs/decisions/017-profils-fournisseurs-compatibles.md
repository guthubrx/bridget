# ADR 017 - Profils de fournisseurs compatibles

## Statut

Accepté

## Contexte

Bridget peut exécuter Cursor via ACP et Claude Code via claude_stream_json. GLM et DeepSeek exposent des endpoints compatibles Anthropic et sont déjà configurés localement dans des fonctions shell. Les faire passer pour Anthropic rendrait l audit faux et modifier globalement les réglages Claude risquerait de rediriger des agents Anthropic actifs.

## Décision

Le type d agent du registre est le fournisseur sélectionné explicitement : codex, cursor, anthropic, glm ou deepseek.

Le runtime et le transport restent séparés. Cursor utilise le transport ACP commun. Anthropic, GLM et DeepSeek utilisent tous Claude Code et le transport claude_stream_json.

Chaque endpoint compatible possède un répertoire de configuration Claude Code isolé, référencé par un chemin absolu non secret dans la définition résolue. Les jetons et endpoints privés restent dans le settings.json de ce profil, hors dépôt, avec des permissions vérifiées. L absence ou l illisibilité d un profil arrête ce lancement avec un motif explicite, sans repli vers le profil Anthropic.

Le journal reçoit le type explicitement lancé comme provider_kind. Il ne déduit jamais un fournisseur d un modèle, d un nom d agent ou d un runtime.

## Conséquences

### Positives

- Cursor ne nécessite pas un second adaptateur.
- GLM, DeepSeek et un futur endpoint compatible réutilisent le transport déjà testé, ses événements d outils, ses permissions et son interruption.
- L opérateur peut auditer l upstream sélectionné sans confondre produit Claude Code et fournisseur de modèle.
- Un secret GLM ou DeepSeek ne peut pas modifier les agents Anthropic.

### Négatives

- Le registre et la définition résolue portent un champ non secret de plus.
- Chaque nouveau fournisseur compatible réclame une preuve réelle avant d être déclaré actif.
- La présentation UI de runtime reste sous la responsabilité de SPEC-071 ; elle consommera cette provenance lorsqu elle sera disponible.

## Alternatives écartées

- Créer un transport HTTP direct par fournisseur : duplication du cycle de vie, des outils et de l interruption.
- Modifier le profil global Claude : risque de contamination des agents actifs.
- Déduire GLM ou DeepSeek du modèle : faux positif et conflit avec SPEC-071.
- Utiliser un wrapper shell par fournisseur : surface de maintenance et de secret supérieure à un champ de profil limité.
