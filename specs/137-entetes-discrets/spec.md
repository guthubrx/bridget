# SPEC137 — En-têtes Bridget discrets
Statut: Implemented (code et aperçu ; non installé) | Date: 2026-10-06 | Tests: 200/200 ciblés
Branches: session-137-entetes-discrets (Bridget), session-137-entetes-bridget (T3).

## Pourquoi et scénarios
US1 (P1): le contenu doit attirer l'œil avant l'en-tête technique.
US2 (P1): nom, identifiants, corps, sélection, copie et historique restent exacts.
Acceptation: expéditeur nommé/UUID seul, notification, titre de lot groupé.
Messages ordinaires et citations d'en-têtes gardent leur présentation.

## Exigences
- FR01: en-tête gris et plus petit que le corps, thèmes clair/sombre.
- FR02: nom et identifiants complets, sélectionnables et lisibles.
- FR03: seul le premier paragraphe d'une enveloppe complète est atténué.
- FR04: aucun changement du texte, de la copie ou du contenu fournisseur.
- FR05: enveloppe incomplète, citée ou placée dans le corps: rendu ordinaire.
- FR06: tests et aperçu isolés, aucun redémarrage des agents en production.

## Succès et périmètre
Tests positifs/négatifs et timeline existante PASS ; lint/types/build web PASS.
Recette visuelle isolée autorisée explicitement le06/10.
Web et desktop local0.0.45 ; mobile natif distinct hors demande.
En-têtes individuels internes d'un lot inchangés. Aucun transport/protocole modifié.
Une installation doit être distinguée d'un code seulement construit.
Hypothèse: couleurs du thème réutilisées, sans opacity globale ni animation.
