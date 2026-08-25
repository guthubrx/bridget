# Spécification 021 — Verdict lié au SHA réellement mesuré

**Statut** : Prêt à livrer

**Base de conception vérifiée** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`

**Validation du lot** : 10/10 (100 %)

**Suite workspace** : 944 réussis ; 3 rouges hors lot reproduits sur la base ;
16 ignorés

## Problème

Un échec de `git checkout <sha>` laisse aujourd'hui le juré sur son ancien
`HEAD`. Les gates peuvent alors produire des chiffres cohérents et un verdict
favorable sur le mauvais objet. Le rapport de livraison existant porte un hash
de contenu, mais aucun verdict de revue typé ni aucune révision Git mesurée.

## Propriété

Un verdict de revue n'est recevable que si la greffe compare, dans sa propre
transaction, le SHA effectivement observé par la commande de dépôt avec le SHA
gelé dans le mandat. Un verdict sans attestation Git, ou portant une autre
révision, est refusé avant toute transition métier.

## Décisions demandées

### D1 — Lieu de la comparaison

La comparaison vit dans la greffe Maicie, dans le chemin réel
`delivery_report`, après chargement de la délégation et avant toute transition.
Bridget valide la forme canonique et transporte les faits ; il ne possède ni
le mandat ni le droit de décider qu'un verdict s'y rapporte.

Le mandat porte une cible de revue typée optionnelle : référence distante et
SHA attendu. Le dépôt porte un verdict fermé et les observations Git. Les
octets canoniques du dépôt et de la réponse sont la preuve durable.

### D2 — Cible déplacée ou mauvais objet

L'ordre de décision est fermé :

1. mandat fourni différent du mandat greffé → `review_mandate_mismatch` ;
2. tête distante observée différente du SHA gelé → `target_head_moved` ;
3. `HEAD` mesuré différent du SHA gelé → `measured_head_mismatch` ;
4. égalité des trois SHA → verdict recevable.

`target_head_moved` est évalué avant `measured_head_mismatch`. Si la branche a
bougé pendant le jury, le verdict est refusé comme périmé sans imputer une
faute au juré. Il faut geler une nouvelle tête et émettre un nouveau mandat.

### D3 — Garantie exacte

Sur le chemin CLI officiel, le lot garantit l'identité du commit `HEAD` et
l'état de la référence distante telle que la configuration Git locale la
résout au moment du dépôt. Il ne garantit pas : authenticité cryptographique
d'une trame forgée par un autre client, identité de l'URL associée au nom du
remote, arbre de travail propre, artefacts de compilation non pollués,
dépendances identiques, variables d'environnement, état des services, ni
reproductibilité des gates. Ces propriétés restent hors périmètre et ne
doivent pas être suggérées par les messages ou les tests.

## Scénarios

### US1 — Mandat de revue typé

Un `maicie delegate` peut déclarer ensemble `--review-ref` et
`--expected-head`. Fournir un seul des deux est refusé. La cible typée est
persistée dans la délégation et rendue dans l'instruction initiale comme dans
la carte de reprise.

### US2 — Dépôt qui mesure au lieu de croire

Le dépôt `delivery-report` accepte un verdict fermé et mesure lui-même :

- `git rev-parse --verify HEAD^{commit}` pour le commit testé ;
- `git ls-remote --exit-code <remote> refs/heads/<branche>` pour la tête
  distante actuelle de la référence mandatée.

Aucun drapeau ne permet de fournir manuellement ces deux observations. Une
commande Git absente, ambiguë ou en échec interdit le dépôt.

### US3 — Refus mécanique à la greffe

Pour une délégation de revue, un rapport sans verdict est refusé avec
`review_verdict_required`. Un verdict sur une délégation ordinaire est refusé
avec `review_verdict_unexpected`. Les cinq refus de revue sont persistés et
rejouables comme les refus guichet existants.

### US4 — Verdict recevable

Si le mandat, la tête distante et le `HEAD` mesuré concordent, le verdict est
greffé, la délégation passe à `terminee` et l'objectif reste `a_evaluer` pour la
décision explicite du référent. Un verdict ne clôt jamais l'objectif.

## Exigences fonctionnelles

- **FR-2101** : la cible de revue est optionnelle pour les délégations
  ordinaires, mais atomique (`review_ref` + `expected_head`) lorsqu'elle existe.
- **FR-2102** : un SHA Git admis contient exactement 40 caractères
  hexadécimaux minuscules.
- **FR-2103** : les verdicts fermés sont `approve`,
  `approve_with_changes`, `amender` et `stop`.
- **FR-2104** : les observations `measured_head` et `observed_target_head`
  sont produites par le binaire, jamais par une option utilisateur.
- **FR-2105** : aucune transition d'objectif, de délégation ou décision de
  coordination n'est écrite lors d'un refus de revue.
- **FR-2106** : l'absence du bloc de revue conserve octet pour octet la forme
  historique d'un `delivery_report` ordinaire.
- **FR-2107** : les refus sont distincts et durables :
  `review_verdict_required`, `review_verdict_unexpected`,
  `review_mandate_mismatch`, `target_head_moved`,
  `measured_head_mismatch`.
- **FR-2108** : un verdict accepté termine la délégation de revue sans fermer
  l'objectif ni qualifier automatiquement le verdict.
- **FR-2109** : le chemin de production guichet, et non seulement une fonction
  de comparaison appelée directement, est couvert par un oracle.
- **FR-2110** : les limites D3 figurent dans la documentation opérateur et dans
  le commentaire de l'oracle bout en bout.

## Critères de succès

- **SC-2101** : un `HEAD` ancien avec cible distante inchangée produit
  `measured_head_mismatch` et zéro transition.
- **SC-2102** : une branche distante avancée produit `target_head_moved`, même
  si le `HEAD` local diffère aussi.
- **SC-2103** : un verdict sans attestation sur mandat de revue est refusé.
- **SC-2104** : le binaire réel, lancé dans un dépôt Git jetable, dépose le SHA
  de son vrai `HEAD` et la tête d'un remote jetable.
- **SC-2105** : le retrait de l'appel de comparaison dans le chemin de greffe
  fait rougir l'oracle bout en bout.
- **SC-2106** : les rapports ordinaires v1 existants gardent leurs octets et
  leur comportement.
