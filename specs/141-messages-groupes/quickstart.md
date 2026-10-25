# Validation141 — Guide ciblé

## Préconditions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-1).

## Tests de projection et de rendu

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-2).

Format et lint ciblés sur les fichiers modifiés :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-3).

## Recette du composant réel

1. Monter la timeline avec un lot direct de trois messages synthétiques, dont deux ont le même nom.
2. Vérifier une ligne au repos. Ouvrir le groupe : seule la première section est ouverte. Ouvrir une seconde section. Vérifier les noms, l'ordre, l'extrait littéral borné à 120 et les corps entiers. Refermer puis ouvrir le groupe conserve les choix.
3. Refaire le parcours avec Tab, Entrée et Espace. Vérifier focus et états annoncés.
4. Copier le message complet et comparer au texte original row.message.text. Inclure [x](t3-context://v1/skill/ctx_1) sans contexte structuré : le lien doit rester dans la copie du lot valide ou en repli sûr. Vérifier la copie actuelle des autres familles, puis pièces jointes et actions existantes. Vérifier aussi un UUID seul visible sans nom inventé.
5. Essayer compteur faux, indices répétés, séparateur dans le corps, suffixe inconnu et en-tête trop long. Vérifier le texte entier et l'absence de panneau source.
6. Vérifier thèmes clair/sombre, largeur 320, corps long, nom long, ancrage et changement de fil/message.
7. Vérifier les cinq familles SPEC140. Seul le lot direct change. Les autres détails techniques restent accessibles comme auparavant.

Consigner les résultats et leurs limites. Un rendu de maquette seul ne valide pas le composant. Le dossier Converge doit relier les exigences aux preuves, puis l'audit doit relire le diff ciblé.

Baseline exécutée par le principal : 269 tests PASS dans les deux suites, 4,53 secondes. La commande pnpm exec a tenté une réinstallation et échoué avec ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY. Utiliser directement le binaire déjà installé. Les dépendances sont liées au checkout principal ; ne pas les réinstaller.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-6).

## Livraison autorisée, sans relance de adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/quickstart.md#extrait-8).

Journal de livraison : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/livraison.md
