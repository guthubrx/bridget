# Plan 039 — Valider l'identité MCP

## Pourquoi ce dessin

Le défaut vient de deux sources de vérité. Ajouter une garde à chaque outil
fermerait les outils présents tout en laissant la prochaine extension recréer
le trou. Le correctif remplace donc la grammaire privée au point où le fichier
d'identité devient un principal MCP.

La garde reste définie une seule fois dans `bridget-core`. La façade MCP
continue de traiter une résolution invalide comme une identité absente ; son
chemin existant garantit déjà qu'aucun exécuteur n'est appelé dans ce cas.

## Algorithme

1. Lire puis normaliser le fichier d'identité comme aujourd'hui.
2. Passer la chaîne normalisée à la garde canonique du routeur.
3. Construire l'identité résolue uniquement si cette garde accepte.
4. En cas de refus, poursuivre la recherche d'un marqueur d'ancêtre valide ;
   à défaut, rendre `identity_not_found` avant tout outil.
5. Ne modifier ni la validation du destinataire, ni le filtre de domaine, ni
   le transport métier.

## Oracles et mutant

L'oracle de bout en bout injecte le vrai résolveur dans la façade MCP :

- fichier `分析`, marqueurs absents : résultat `identity_not_found`, compteur
  de l'exécuteur égal à zéro ;
- fichier `rc5-test`, même appel : résultat nominal, compteur égal à un.

Après correctif, le mutant remplace temporairement l'appel canonique par
l'ancienne condition Unicode. L'oracle invalide doit mourir sur le compteur
d'exécution ; le contrôle ASCII doit rester vert. La restauration est vérifiée
sur le fichier productif avant les gates finaux.

## Portée de tests

Le seul paquet productif modifié est `bridget-daemon`, dont aucun paquet ne
dépend. La closure ciblée est donc ce paquet seul. La livraison rend séparément
l'univers listé, les comptes passés/échoués/ignorés, le test MCP exact et le
mutant. Aucun workspace complet n'est nécessaire avant la revue du lot ; il
sera joué une fois seulement si la tête devient candidate à intégration.

## Complexité, minimalisme et sécurité

La validation parcourt O(n) octets, n étant borné à 100 avant acceptation.
Aucune allocation, dépendance, état ou abstraction nouvelle n'est introduit.
Le changement productif remplace une condition locale par une fonction
partagée ; le volume net peut rester nul ou diminuer.

Le contrôle positif interdit une façade toujours fermée. Le compteur de
l'exécuteur interdit un refus tardif après effet. Le mutant vise l'unique appel
emprunté par `read_name`, jamais une occurrence voisine.

## Validation

- liste brute de l'univers complet `cargo test -p bridget-daemon -- --list`
  avant exécution ;
- comparaison base/tête avec targets physiquement distincts ;
- cible exacte des deux oracles MCP ;
- mutant Unicode, contrôle ASCII et restauration ;
- `rustfmt --check` sur les deux fichiers Rust modifiés, comparaison base/tête
  de `cargo clippy -p bridget-daemon --lib -- -D warnings` et
  `git diff --check` ;
- revue hostile du diff complet, notamment effets avant refus et régression
  du parc vivant.
