# Plan 052 — Corréler la notification de délégation

## Décision technique

Construire le corps complet au dernier point où les trois identifiants sont
connus, avant la sérialisation durable du `PublicMessage` :

1. la création immédiate connaît déjà l'objectif et la délégation, génère le
   `message_id`, puis finalise l'instruction ;
2. le déblocage différé génère son `message_id` dans la transaction de clôture,
   puis applique exactement la même finalisation ;
3. `PreparedDelegation` sérialise ensuite ces octets comme aujourd'hui ;
4. la reprise continue de relire `message_bytes` sans aucune reconstruction.

La règle vit près du type `Delegation`, car elle assemble ses identifiants et
son instruction. Elle n'introduit ni service, ni nouvelle dépendance, ni état
dérivé persistant supplémentaire.

## Ordre d'implémentation

1. Poser deux oracles rouges sur le vrai `PublicMessage` : immédiat et différé.
2. Ajouter le format fermé partagé et raccorder le chemin immédiat.
3. Raccorder le chemin différé et conserver l'égalité entre instruction
   persistée et corps remis.
4. Rejouer les oracles, la reprise historique et le mutant qui rétablit
   l'instruction brute.
5. Exécuter `--no-run` avant tout comptage, puis base et tête dans la même
   campagne, format, clippy ciblé et composition avec le `main` courant.

## Fichiers prévus

- `plugins/maicie/src/domain.rs` : format fermé partagé.
- `plugins/maicie/src/app.rs` : création immédiate.
- `plugins/maicie/src/store.rs` : déblocage différé.
- `plugins/maicie/tests/contract/delegate.rs` : oracle immédiat existant
  renforcé sur le message sérialisé.
- `plugins/maicie/tests/contract/f36_f37_suite_citations.rs` : oracle différé
  existant renforcé sur le message sérialisé.
- `specs/052-notification-delegation-correlee/` : contrat et preuves.

## Risques et gardes

- **Anciennes outboxes** : aucune validation rétroactive du nouveau suffixe ;
  la reprise conserve ses octets historiques.
- **Chemin différé oublié** : oracle distinct après clôture réelle du dernier
  prérequis.
- **Oracle tautologique** : attentes construites depuis les UUID rendus par le
  store, jamais par le formateur testé.
- **Mutant mort au setup** : le mutant contourne seulement la finalisation du
  corps ; la création doit réussir avant l'assertion finale.
- **047** : composition mesurée séparément ; aucune modification de son contrat
  filaire ni de sa tête gelée.

## Complexité et minimalisme

Le formateur effectue une seule allocation proportionnelle à l'instruction :
O(n) temps et espace. Deux appels réels justifient une règle partagée, car ils
doivent produire un contrat identique. Aucun wrapper, table, colonne ou outil
supplémentaire n'est créé.
