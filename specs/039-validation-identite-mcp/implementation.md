# Mise en œuvre 039 — Valider l'identité MCP

## Gel et portée

- base : `90802b0377741b509f3743c5675544315b6f0f29` ;
- branche : `session-039-validation-identite-mcp` ;
- objectif : `3742f4a7-0def-4636-8a1c-e36dcb6a399a` ;
- production autorisée : `crates/bridget-daemon/src/mcp_identity.rs` ;
- oracle autorisé : `crates/bridget-daemon/src/mcp.rs` ;
- CLI, daemon central, destinataire, domaine et protocole : inchangés.

## Mesure initiale

La garde canonique accepte exactement `[A-Za-z0-9_-]` sur 1 à 100 octets.
La grammaire MCP historique accepte en plus toute lettre ou tout nombre
Unicode. La relation est une inclusion stricte de la canonique dans MCP ;
l'ensemble inverse est vide.

Au 27 août 2026 à 11:16:34 UTC, Bridget listait 15 noms, dont 14 connectés.
Les 15 passent la garde canonique ; aucun nom du parc visible n'est retiré.

## Mise en œuvre

`read_name` ne possède plus de grammaire privée. Il normalise le contenu du
fichier comme auparavant, appelle `bridget_core::router::validate_agent_name`,
puis construit l'identité seulement après acceptation. Cette fonction unique
est empruntée par les deux sources productives : le fichier dynamique et le
fichier nommé par un marqueur d'ancêtre.

La façade MCP porte deux témoins de bout en bout. Le premier résout `分析`,
appelle réellement `dispatch_with_executor` et exige zéro appel de
l'exécuteur avant de contrôler `identity_not_found`. Le second résout
`rc5-test`, contrôle l'identité et les arguments reçus par l'exécuteur, puis
exige exactement un appel et le contenu nominal.

Avant le correctif, l'univers exact de ces deux témoins rendait 1 passé,
1 échoué et 0 ignoré : le témoin Unicode mourait à l'assertion de
non-exécution avec `left: 1` et `right: 0`. Le refus n'est donc pas produit par
une autre étape du pipeline.

## Univers et comparaison base/tête

Les deux côtés ont été compilés par `cargo test -p bridget-daemon --no-run`
avant inventaire, dans des worktrees, targets et répertoires temporaires
physiquement distincts. Les comptes obtenus sur les bases antérieures ont été
exclus lorsque `main` a bougé ; leurs targets ont été supprimés avant cette
mesure finale.

- base : 642 tests listés ;
- tête : 644 tests listés ;
- différence des listes : exactement les deux témoins MCP ajoutés.

Le test isolé
`stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels` liste un
univers de 1 mais ne rend aucune ligne de résultat après une borne externe de
30 secondes, sur la base comme sur la tête. Aucun processus ne subsiste après
la borne. La comparaison opposable applique donc le même filtre explicite aux
deux côtés :

- base : 623 passés, 7 échoués, 11 ignorés et 1 filtré, soit 642 ;
- tête : 625 passés, 7 échoués, 11 ignorés et 1 filtré, soit 644.

Les sept rouges sont identiques par leur nom sur les deux côtés : deux tests
unitaires de présence, un test Claude natif, deux tests Codex natifs et deux
tests de reprise MCP dans `managed_parity_test`. Aucun rouge n'est ajouté par
le lot. Le test non terminal filtré n'est pas qualifié par ces comptes et
reste une limite déclarée.

Un premier tir ciblé combinant un filtre préfixe avec `--exact` a exécuté zéro
test ; il est exclu. Après relisting de l'univers exact à 2, la commande
corrigée rend 2 passés, 0 échoué et 0 ignoré. Le module `mcp_identity` liste
6 tests et rend 6/0/0 ; le module MCP complet liste 35 tests et rend 35/0/0.

## Mutant et restauration

Le mutant remplace, dans le vrai `read_name`, l'appel canonique par la
condition Unicode historique. Sur l'univers exact de 2 :

- le contrôle ASCII reste vert ;
- le témoin Unicode meurt à l'assertion de non-exécution, avec `left: 1` et
  `right: 0`.

Le fichier productif restauré retrouve le SHA-256
`63f1dbe033574d18e1feebb89e5dfd338a067af6238a6888c5abd0a3f56bcc16`,
puis les deux témoins rendent de nouveau 2/0/0. Les deux appels productifs de
`read_name` passent donc par le site effectivement muté.

## Gates statiques et revue hostile

- `rustfmt --edition 2024 --check` sur les deux fichiers Rust modifiés : vert ;
- `git diff --check` : vert ;
- Clippy strict complet échoue identiquement sur la base et la tête, avant le
  code du lot, sur 7 diagnostics de `bridget-transport` ;
- Clippy strict avec `--no-deps` échoue identiquement sur la base et la tête
  sur 10 diagnostics préexistants du daemon. Aucun diagnostic supplémentaire
  n'est introduit par la tête, mais le gate strict global reste rouge.

La revue hostile confirme : aucune occurrence de la grammaire Unicode privée
ne subsiste dans `mcp_identity.rs` ; le résolveur précède l'exécuteur dans
`dispatch_with_executor` ; les deux sources d'identité appellent le même
`read_name` ; aucun changement ne touche la CLI, le daemon central, le
protocole, le destinataire ou le domaine.

## Non-mesuré

- workspace complet, réservé à la candidature d'intégration ;
- macOS ;
- déploiement contre un daemon vivant ;
- l'issue fonctionnelle du test de terminaison filtré, qui ne rend pas de
  résultat en 30 secondes sur aucun des deux côtés Linux.
