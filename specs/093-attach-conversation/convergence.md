# Convergence 093 — close, APPROVE

## Revue préventive indépendante

2026-09-06, reviewer `review_attach_093`, lecture seule du socle et du contrat.
Les points ci-dessous sont des critères de livraison, pas un verdict imputé à un
diff encore en cours. Aucun APPROVE de code 093 n'est revendiqué.

1. Masquer le raisonnement ne crée pas un bloc vide ; un succès silencieux clôt
   néanmoins le tour. Inconnu/échec/annulation visibles, non-TTY inchangé.
2. Retenir un seul dernier bloc terminé pour resize sans événement. Contradiction
   du research corrigée : le seul bloc actif ne suffisait pas au contrat.
3. Frontière de débordement indépendante des lignes physiques ; reflow de la
   queue gérée sans répétition/perte, jamais promesse sur le scrollback externalisé.
4. Refus canonique conservant motif/couche/preuve, pas simple message générique.
5. Mesurer le texte neutre ; injecter uniquement les SGR locaux après disposition.
6. Noms résolus par ID exact depuis ListAgents existant ; adresse et corrélation
   continuent d'utiliser l'UUID, jamais une chaîne `from` déclarée arbitrairement.
7. Coût O(F × n) explicite et cadence/budget vérifiable, pas une borne proclamée.

Transmission à l'auteur : message `mcp-67952-6a9d8e8c-6`.

## Première lecture du WIP par le pilote

Le module Markdown isolé est présent. Point supplémentaire transmis dans
`mcp-67952-6a9d8ef9-7` : l'assainissement avant parsing n'est pas suffisant si
CommonMark décode ensuite une entité numérique en contrôle terminal. L'oracle
doit exercer les entités décimales/hex et l'assainissement des textes émis par le
parseur. Le repli aux frontières de mots doit aussi être vérifié.

État : en cours de traitement par l'auteur. Tests et contre-revue finale à venir.

## Clôture du cycle

Contre-revue finale indépendante : APPROVE. Les mentions précédentes constituent
la chronologie. Les entités ANSI sont neutralisées après parsing ; les colonnes
sont calculées sur le texte neutre. Les refus gardent leurs détails. Le label
d'un bloc engagé est gelé pour conserver son curseur logique. Un succès non
corrélé reste visible. Les listes gardent une indentation de continuation.
Le wrapping parcourt un vecteur avec indices, sans retrait/copie de tête répétée.
Coût annoncé : O(n) par rendu et O(F × n) sur F rafraîchissements ; pas de SLO de
performance inventé. La cadence regroupe les événements et observe la géométrie.
Les 95 tests attach et le gate clippy passent. Suite workspace séquentielle PASS ;
collision parallèle préexistante consignée dans implementation.md, hors delta.
