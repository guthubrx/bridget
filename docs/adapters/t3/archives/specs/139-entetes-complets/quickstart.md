# Archives techniques de l’adaptateur T3

Document d’origine : [specs/139-entetes-complets/quickstart.md](../../../../../../specs/139-entetes-complets/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Recette139
Depuis apps/web du worktree T3 : vp test run les deux fichiers MessagesTimeline.
Comparer les paragraphes de l'enveloppe dans un aperçu isolé clair/sombre.
Attendu : premier paragraphe12px et gris ; contenu14px couleur normale.
Vérifier que noms, UUID et corps restent complets, copie source inchangée.
Construire l'archive par scripts/build-desktop-artifact.ts, cible mac/zip/arm64.
Ne pas lancer --install pendant que T3 fonctionne.
