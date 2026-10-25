# Archives techniques de l’adaptateur T3

Document d’origine : [specs/118-memoire-des-messages/implementation.md](../../../../../../specs/118-memoire-des-messages/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Correction (`t3code.rs`)
- `ThreadState::forget_invisible` : au-delà de SEEN_BOUND, n'oublie que les identifiants que T3 ne
  montre plus (messages, et faits d'activité par leur activité) ; `remember` n'évince plus.
- Index `HashSet` des messages recopiés dans `project_journal` : O(S + M) au lieu de O(S × M).
- Annonce « prête » anticipée gardée pour les seules lectures qui portent une lacune : le daemon
  ignore la lacune d'une source non déclarée (test `spec101_http_capacites_honnetes…`).

<a id="extrait-2"></a>

## Extrait 2

## Livraison
- 03:36 : fusion `e63a7abd`, construction, relance du pont seul (`kickstart -k com.bridget.t3`).
- Après relance : `seen` du fil = 1 363, relus depuis le journal existant, aucune recopie ; journal
  +0 octet en 2 min ; source `ea69b65b` stable avec ses deux événements ; deux notifications à la
  relance elle-même, plus aucune ensuite.
