# Recherche141 — Décisions et preuves

Date : 2026-10-07. Recherche préparatoire ; aucune preuve de livraison revendiquée.

## Format et réutilisation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/research.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/research.md#extrait-2).

Décision : rejeter un découpage ambigu. Les corps sont du texte libre ; ils peuvent contenir un séparateur apparent. Un compteur seul ne suffit donc pas à certifier les limites. Le repli entier évite de perdre du texte ou de l'attribuer au mauvais expéditeur. Le mainteneur peut vérifier cette règle avec une fixture sans consulter de données actives.

Décision : afficher le nom reçu puis un extrait littéral du début du corps. Les sujets de /Users/moi/.cache/bridget-grouped-mockup-20261007/index.html sont illustratifs. Ils ne sont pas des données du transport. Le panneau source de la maquette est exclu par la demande explicite de l'utilisateur.

Précision issue de la revue indépendante APPROVE_WITH_CHANGES : un expéditeur fourni sous forme d'UUID seul conserve cet UUID visible. Le libellé historique « Bridget » des messages directs unitaires ne doit pas devenir un nom inventé dans une section de lot.

## Accessibilité et tests

Le principal a consulté les sources primaires suivantes le 2026-10-07 :

- https://www.w3.org/WAI/ARIA/apg/patterns/accordion/ : état ouvert, commandes de section et navigation clavier.
- https://web.dev/learn/html/details : repli natif et comportement accessible.
- https://testing-library.com/docs/guiding-principles/ : tests fondés sur le comportement visible.
- https://playwright.dev/docs/best-practices : recette isolée et assertions sur les résultats utilisateur.

Décision : conserver les boutons natifs déjà présents, leurs attributs d'accessibilité et l'ancrage. Une conversion intégrale en details n'apporte pas de bénéfice prouvé à la liste virtualisée. Les tests portent sur ce que l'utilisateur lit et copie.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/research.md#extrait-3).

## Alternatives examinées

Un nouveau moteur de cartes dupliquerait la timeline et ses actions. Une nouvelle API structurée modifierait le transport pour un besoin de présentation. Un résumé généré inventerait des sujets. Un panneau source contredirait la demande. Un nouveau store global compliquerait l'isolation existante. Ces alternatives sont écartées.

## Charge future et incertitudes

La règle est locale, pure et testable. Les invariants de copie et de repli sont écrits dans le contrat. Le prochain mainteneur pourra retirer la présentation groupée sans modifier stockage ou daemon.

Aucune incertitude fonctionnelle ne bloque le plan. Les preuves de clavier et de défilement restent à produire avec le composant réel. La contre-revue par un autre fournisseur reste conditionnée à sa disponibilité ; Bridget ne fournit actuellement qu'un agent Codex joignable.
