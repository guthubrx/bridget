# Plan — Session 026, première tranche

## Décision

Étendre la porte existante du guichet, sans accès direct au store. La première
tranche transporte `delegate` assez loin pour confronter sa déclaration aux
objectifs centraux, puis rend uniquement un refus terminal durable.

Le chemin fédéré appelle la fonction existante
`citation::unclassified_known_citations`. Le chemin local reste la propriété de
rc1 ; aucune logique locale, table d'audit ou heuristique branche/SHA n'est
ajoutée par 026.

## Chaîne d'admission

1. v17/session 021, tête `2623772`, transporte le neuvième motif composé.
2. v18/rc1 crée `local_delegate_refusals`, son index et ses triggers
   append-only.
3. v19/026 vérifie comportementalement v18 puis reconstruit uniquement
   `guichet_refusal_receptions`.

Le code 026 est composé sur le commit v18 gelé de rc1. Son test de composition
ajoute `review_target: None`, champ introduit par 021, au nouveau fixture rc1.
Il ne modifie pas la sémantique du refus local.

## Couches modifiées

- `crates/bridget-transport/src/protocol.rs` : opération, charge fermée et deux
  motifs de refus.
- `crates/bridget-daemon/src/daemon.rs` : validation bornée avant dépôt.
- `crates/bridget-daemon/src/store.rs` : absence de corrélation externe pour
  cette charge.
- `plugins/maicie/src/guichet.rs` : parseur canonique `deny_unknown_fields`.
- `plugins/maicie/src/app.rs` : lookup des UUID connus, appel de la règle unique,
  choix du motif puis persistance.
- `plugins/maicie/src/domain.rs` : source unique des noms SQL.
- `plugins/maicie/src/store.rs` : v19 et lecture fail-closed.

## Témoins

1. Compilation workspace avant tout comptage.
2. Admission réelle par le daemon sans confusion entre dépôt et succès.
3. Contradiction UUID connue, persistance, rejeu exact et limite branche/SHA.
4. Round-trip exhaustif des vocabulaires et corruption SQL inconnue.
5. v18 réelle→v19, fausse v18, fausse v19, parcours privé v14 et bootstrap.
6. Suites Maicie/guichet existantes, puis workspace complet.
7. Formatage, `git diff --check` et clippy ciblé.

Les deux témoins d'approbations interdites restent ignorés avec un motif
explicite : ils appartiennent à la seconde tranche et ne comptent pas comme une
propriété livrée.

## Risques et gardes

- Une réponse perdue est rejouée avec le même `request_id` et les mêmes octets.
- Une erreur SQLite ou une corruption n'est jamais transformée en refus métier.
- Les sondes de migration rollbackent dans toutes les branches.
- La source v14 du témoin est une fixture privée en `0700/0600`, jamais la base
  active.
- Le variant composé de 021 n'est ni renommé ni reprojeté par 026.
