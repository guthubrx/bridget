# Plan 025 — Carte de criticité auto-élue et régime de revue

**Branche** : `session-025-carte-criticite-regime`
**Spec** : `specs/025-carte-criticite-regime/spec.md`
**Statut** : conception gelée avant code

## Résumé technique

La session ajoute un calculateur déterministe au plugin Maicie. Il mesure deux
objets Git immuables, construit un index des chemins suivis, applique le germe
et les trois voies F38, puis persiste dans la même transaction la carte
enrichie et la proposition du lot. Une seconde requête fermée enregistre le
régime retenu. Les métriques sont des projections SQL des lignes durables,
jamais des compteurs mémoire.

L’intégration se fait en deux temps : le noyau pur et son adaptateur Git sont
développés sur la base `b6eea77`; le guichet et la migration v20 sont rebasés
après absorption de v17, v18 et v19. Le branchement des faits de verdict
consomme ensuite le contrat 021. Aucune variante locale du verdict n’est créée.

## Contexte technique

- Rust 2024, sans nouvelle dépendance ;
- `serde`/`serde_json` pour les contrats fermés ;
- `sha2` pour l’identifiant déterministe de soumission ;
- `rusqlite` pour les transactions et projections ;
- commandes Git de lecture seules pour les arbres et diffs ;
- Linux pour l’implémentation et les mesures de référence, avec matrice de
  rouges explicitement reproduite au gate final.

## Constitution et gardes

| Garde | Décision |
|---|---|
| SpecKit avant code | spec, plan, modèle, contrat, tâches et Gherkin créés avant production |
| Minimalisme | deux nouvelles opérations ; aucun éditeur de carte ; aucune dépendance ajoutée |
| Validation aux frontières | enveloppes fermées, SHA complets, référence complète, dépôt configuré |
| Effets de bord | mesure Git en lecture ; transaction unique carte + soumission ; décision séparée et idempotente |
| Confidentialité | seules preuves structurées et empreintes sont persistées ; aucun contenu source |
| Complexité | index construit une fois ; lookup exact/suffixe sans scan répété du dépôt |
| Migration | v20 seulement après DDL v17–v19 présent et vérifié |
| Approbation ADR 011 | aucune approbation de profil/routine par CLI distant ou guichet |

## Architecture

### 1. Domaine pur de criticité

`plugins/maicie/src/review.rs` porte :

- les régimes et leur ordre ;
- les quatre règles de germe ;
- l’index de chemins et la résolution des citations ;
- l’agrégation Bloquant/2 Majors ;
- la proposition de régime ;
- les preuves structurées, ambiguïtés et limites de taille.

Le calcul reçoit un `RepositorySnapshot` déjà mesuré. Il n’ouvre ni fichier,
ni processus, ni base. Cette séparation permet aux mutants de viser la règle
et non la mise en place Git.

### 2. Adaptateur Git borné

`plugins/maicie/src/review_git.rs` construit le snapshot. Toutes les commandes
emploient la racine configurée et les SHA complets :

```text
git -C <racine> rev-parse --verify <ref>^{commit}
git -C <racine> cat-file -e <sha>^{commit}
git -C <racine> merge-base --is-ancestor <base> <tête>
git -C <racine> ls-tree -r -z --name-only <tête>
git -C <racine> diff --name-status -z -M <base> <tête> --
git -C <racine> diff --no-ext-diff --no-textconv --unified=0 <base> <tête> --
```

L’adaptateur neutralise les aides de diff externes, n’effectue aucun checkout
et borne le nombre de chemins, la taille cumulée des blobs textuels et la
sortie du diff. Les renommages rendent le chemin avant et après. Toute borne
dépassée devient un refus fermé et durable ; aucune troncature ne produit une
proposition partielle.

### 3. Lecture du registre et des contrats

Le catalogue déclaré est ouvert en lecture verrouillée et projeté avec la
fonction existante. Seuls les constats ouverts participent. Le calcul conserve
leurs identifiants, sévérités et sources, pas leur texte.

Les contrats sont lus depuis l’arbre du commit de tête, jamais depuis le
worktree. Un fichier sous `specs/*/contracts/` peut ancrer seulement un chemin
complet exact. Les citations non exactes sont rendues comme dettes de contrat.

### 4. Greffe v20

La migration v20 ajoute les tables décrites dans `data-model.md`. L’écriture
de soumission suit une transaction `IMMEDIATE` :

1. rejouer ou refuser l’idempotence ;
2. enrichir les zones et preuves monotones ;
3. écrire les ambiguïtés nouvelles ;
4. écrire la soumission et sa proposition ;
5. écrire le reçu guichet ;
6. commit puis réponse depuis les octets persistés.

La sélection de régime verrouille la soumission, vérifie le référent et écrit
la décision. Un deuxième choix divergent est un conflit idempotent, jamais une
réécriture de l’histoire.

### 5. Guichet et surfaces de lecture

Après v19, le vocabulaire fédéré reçoit :

- `review_lot_submit` ;
- `review_regime_select`.

Le premier est admis à tout participant enregistré et fixe l’auteur depuis
`from`. Le second est admis seulement à l’identité de référent configurée. Les
sorties structurées sont consultables par une commande de lecture
`maicie review list|metrics`; aucune sous-commande `add`, `edit`, `remove`,
`force` ou `approve` n’existe.

## Complexité

Soit `P` le nombre de chemins suivis, `R` le nombre de constats ouverts, `T`
le nombre de jetons candidats et `D` la taille bornée du diff.

- index des chemins : O(P log P), nécessaire pour un rendu déterministe ;
- résolution : O(T log P), sans recherche linéaire par citation ;
- agrégation registre : O(R log P) ;
- scan des règles : O(D) ;
- persistance : O(Z log Z) pour `Z` zones élues, avec clés uniques.

Le dépôt n’est parcouru qu’une fois par soumission. Aucune requête SQL ni
commande Git n’est lancée dans une boucle par citation.

## Recherche et choix

La documentation Git confirme que le diff entre deux commits compare leurs
arbres et que `--no-ext-diff`/`--no-textconv` neutralisent les conversions
externes. La documentation SQLite impose la transaction comme frontière de
commit ; la version applicative ne doit donc avancer qu’avec le DDL dans la
même transaction. Les recommandations de journalisation consultées convergent
sur un point utile ici : conserver identifiants et catégories, mais exclure
les contenus sensibles et données externes brutes.

Décision : aucune bibliothèque d’analyse, aucun moteur de règles et aucun
parseur de diff supplémentaire. Les primitives déjà présentes suffisent et
réduisent la surface de dépendance.

## Stratégie de tests

1. Gherkin métier avant les tests Rust.
2. Tests unitaires du résolveur et du calculateur sur snapshots synthétiques.
3. Tests d’intégration avec vrais dépôts Git jetables : worktree divergent,
   ref déplacée, renommage, binaire, limites et configuration de diff hostile.
4. Tests store v20 : idempotence, crashs aux frontières, refus durable,
   confidentialité et migration v19 réelle/factice.
5. Tests guichet producteur↔consommateur après v19.
6. Mutants : quatre germes, élection de tous les homonymes, Major compté deux
   fois dans un constat, décision par défaut, refus non persisté et migration
   marquant une version sans DDL.
7. `cargo test --workspace --no-run` avant tout compte, puis workspace complet
   avec passes, rouges et ignorés annoncés séparément.

## Ordre d’intégration

1. noyau pur + adaptateur Git + docs ;
2. absorption 021/v17, v18 et 026/v19 ;
3. rebase avant toute mesure de jury ;
4. migration v20 et store ;
5. opérations guichet et CLI ;
6. consommation du verdict 021 ;
7. gates, revue hostile et livraison gelée.

## Risques restant hors session

- le capteur de disponibilité des relecteurs ;
- l’attribution d’un incident ultérieur si le constat ne cite ni tête ni
  soumission ;
- les sorties de carte et leur approbation locale ;
- les dépôts non Git ou dont les objets requis ne sont pas présents localement.
