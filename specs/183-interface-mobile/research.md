# Recherche183

## Décisions

- Réutiliser le socle Android et les contrats natifs : les parcours concernés existent en grande partie. Le volet ajoute une présentation de l’annuaire existant. Pas de client desktop ajouté.
- Favoris, repères et brouillons restent locaux et cloisonnés à l’association. La nouveauté nécessite un fait d’activité ; ni une couleur ni la seule connexion n’en donnent la preuve. La couverture des signaux varie suivant les producteurs et doit être visible.
- Recherche sur historique chargé et chargement explicite de pages antérieures : évite une API de recherche/stockage parallèle.
- Dictée système, retour texte éditable, aucun envoi automatique ; pas de nouvelle dépendance vocale.
- Préparation des pièces jointes avant effet réseau, réutilisation des commandes/réconciliations ; ne pas confondre retrait avant envoi et suppression d’un artefact déjà téléversé.
- Interactivité limitée aux documents autonomes. La frontière privilégiée Android est fermée ; iframe à origine opaque et blocage réseau/fichier/permissions. Le mode statique et le refus de moteur non corrigé restent opérationnels. L’absence d’un pont privilégié est une règle, pas un simple avertissement.
- Compactage : le schéma du moteur natif installé a été généré et confirme des items contextCompaction début/fin. L’adaptateur existant dispose aussi d’items de cycle de vie ; vérifier leurs versions/identités avant projection. Aucun parsing de texte « je compacte ».

## Sources consultées le25octobre2026

Baseline utilisateur : research/07-frontend-design-systems.md,06-security-compliance.md,08-testing-quality.md et10-data-privacy.md. Sources officielles vérifiées :
- https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges : absence de pont natif pour contenu non fiable.
- https://web.dev/articles/sandboxed-iframes : origine opaque, permissions sandbox minimales ; ne pas réunir allow-scripts et allow-same-origin.
- https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html : taille et espacement des cibles ; cible native48dp conservée.

## Vérifications encore à apporter dans l’implémentation

Démontrer l’isolation sur le WebView réellement installé, y compris navigation depuis script, document hostile, accès parent et boucles de rendu. Vérifier la reprise/ancienneté des marqueurs et l’identité d’un résultat vocal tardif. Mesurer le rafraîchissement sans appels par agent. Les anciennes preuves d’autres sessions ne sont pas une validation du code183.
