# Audit d'implémentation - SPEC-080

Date : 2026-08-31
Verdict : PASS pour les contrôles automatisés, le relais déployé et la construction du paquet macOS. La clôture reste bloquée uniquement par la validation visuelle humaine de la finition et le remplacement explicite de l'application macOS ouverte.

## Conformité constatée

- PASS : l'accès se fait par une roue seule en bas de la barre d'agents et ouvre un overlay, pas un panneau de détails de conversation.
- PASS : la navigation et la recherche couvrent les huit rubriques annoncées.
- PASS : les préférences de présentation sont locales, normalisées et bornées. Les tests couvrent le stockage corrompu, le fuseau IANA, les polices et tailles invalides.
- PASS : la page serveur conserve le catalogue fermé, la prévisualisation, la confirmation et la génération attendue de la politique de racines.
- PASS : la page Usage distingue les jetons observés d'une estimation API indisponible.
- PASS : les champs de la page Typographie utilisent la grille, les dimensions, les libellés, les sous-titres et les états de focus portés de T3 Code, tout en conservant le fond Bridget et l'overlay Bridget.

## Sécurité et intégrité

- PASS : aucune préférence locale n'est envoyée au serveur.
- PASS : aucun shell, secret, variable d'environnement ou chemin libre n'est ajouté au catalogue modifiable.
- PASS : les écritures serveur passent encore par prévisualisation, génération attendue et reçu durable.
- PASS : le binaire réellement installé correspond au release construit et les deux services utilisateur sont actifs.

## Qualité et preuves

- PASS : 97 tests JavaScript de l'interface, 2 tests Rust ciblés du daemon et 1 test ciblé Desktop passent.
- PASS : formatage Rust, construction release, contrôles syntaxiques JavaScript et `git diff --check` passent.
- PASS : le port de styles est contrôlé par le test UI : grille de réglage, largeurs des deux sélecteurs et bordure de contrôle sont explicitement attendues.
- NON PRÉTENDU : aucune validation visuelle utilisateur n'est enregistrée pour la version de finition nouvellement servie.

## Risque restant et sortie

Le risque restant est uniquement esthétique et de livraison native : vérifier l'overlay ouvert sur Bridget, puis remplacer explicitement l'application macOS après sa fermeture avec le bundle déjà construit. Les limites fonctionnelles non livrées de la SPEC, dont tarification datée et autres écritures serveur, restent signalées dans les tâches plutôt que déclarées terminées.
