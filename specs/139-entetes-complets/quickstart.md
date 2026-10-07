# Recette139
Depuis apps/web du worktree T3 : vp test run les deux fichiers MessagesTimeline.
Comparer les paragraphes de l'enveloppe dans un aperçu isolé clair/sombre.
Attendu : premier paragraphe12px et gris ; contenu14px couleur normale.
Vérifier que noms, UUID et corps restent complets, copie source inchangée.
Construire l'archive par scripts/build-desktop-artifact.ts, cible mac/zip/arm64.
Ne pas lancer --install pendant que T3 fonctionne.
