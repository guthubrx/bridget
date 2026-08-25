# Plan — Session 026

## Décision 1 — Étendre le guichet, ne pas créer une seconde porte

Le daemon Bridget maître reste propriétaire du dépôt durable. Maicie relève et
applique les demandes avec son store, son catalogue, sa configuration et sa
vue d'annuaire centraux. Le protocole existant fournit déjà canon,
idempotence, claim, expiration et corrélation ; la session étend sa matrice et
sa consultation terminale.

Un outil MCP éventuel appellera ce contrat comme le CLI Bridget. Il n'importe
pas Maicie, n'ouvre aucun fichier du greffe et ne traduit pas un accusé de
dépôt en succès métier.

## Décision 2 — Deux paquets, un seul validateur

### Paquet A — Contradiction partagée, urgent

Le validateur entre dans `plugins/maicie/src/app.rs`, au début du cas d'usage
`delegate`. Il consomme les valeurs déjà structurées de `DelegateRequest` et
le résultat déterministe du lecteur de citations. La CLI locale et le futur
traitement guichet appellent `delegate`; aucun branchement parallèle n'est
ajouté dans `main.rs`.

Le résultat `coherent|refused|signaled` est persisté avec une identité dérivée
de la clé idempotente, du canal et de la version de règle. Un refus précède
toute création. Un signal est écrit atomiquement avec la réservation de la
délégation afin qu'une création réussie ne perde pas son doute mesuré.

Le corpus sépare : dépendance structurée, marqueur lexical machine, référence
de revue, formulation naturelle ambiguë, négation et citation non classée. Le
même corpus est appelé par les deux chemins.

### Paquet B — Trois opérations fédérées

Après le paquet A, le protocole ajoute des charges fermées :

- `delegate` réemploie `DelegateRequest`, mais le maître fournit horloge,
  horizons, taille, clé idempotente, configuration, candidats et identité ;
- `registre_add` convertit une charge typée vers `catalogue::AddEntry`, ouvre
  uniquement le journal configuré côté maître et retourne
  `appended|idempotent_noop` ;
- `objective_close` vérifie objectif, délégation, participant et livraison
  terminale, puis appelle la clôture existante avec une base dérivée et fermée,
  jamais avec un motif client.

Les résultats terminaux portent les identifiants créés ou affectés et sont
persistés avant la réponse Bridget.

## Décision 3 — Vocabulaire fermé en Rust, stockage SQL ouvert

v18 reconstruit `guichet_receptions` et `guichet_refusal_receptions` sans
recopier les listes `operation` et `reason` dans des `CHECK`. Les contraintes
d'état, d'unicité, de non-nullité et de corrélation restent en SQL.

Une macro de domaine engendre depuis une seule déclaration :

- l'enum accepté ;
- `ALL` ;
- `as_sql()` ;
- le parseur exact.

Les refus d'opération inconnue conservent séparément le nom demandé, borné et
validé, sans le convertir en variante autorisée. À la lecture d'une réception
acceptée, une chaîne inconnue rend `StoreError::Corrupt`. Le compromis accepte
qu'une écriture SQL sauvage atteigne le disque, car la base est privée et n'a
aucun écrivain supporté hors Maicie ; il interdit qu'elle soit interprétée.

Le gate couvre quatre oracles exacts : valeur SQL inconnue fail-closed,
round-trip de toutes les variantes, approbations absentes de `ALL` mais refusées
durablement, et bijection entre `ALL` et le corpus. Aucune assertion
`contains()` ne valide un vocabulaire.

## Décision 4 — Consultation terminale et coupures

`guichet_lookup` est étendu pour rendre : état de transport, issue terminale
éventuelle et charge de réponse canonique éventuelle. Un client ne conclut à
l'application que sur une réponse terminale persistée.

La reprise suit la table suivante :

| Observation client | Action | Conclusion permise |
|---|---|---|
| tunnel absent avant écriture | erreur immédiate | aucun dépôt |
| `queued` ou `in_flight` | consulter avec le même `request_id` | dépôt possible seulement |
| réponse perdue | rejouer mêmes id, date et octets, puis consulter | aucune nouvelle intention |
| terminal `accepted` | vérifier charge exacte | effet appliqué |
| terminal `refused` | rendre motif exact | aucun effet demandé |
| expiration sans terminal métier | erreur explicite | issue inconnue, jamais succès |

## Contexte d'exécution central

La relève actuelle reçoit seulement store, socket et horloge. Elle sera
étendue par un contexte explicite construit dans le binaire maître :
configuration Maicie, catalogue validé, classes de durée, politiques et
candidats Bridget. Ce contexte n'est jamais sérialisé depuis l'appelant.

Le contexte reste une structure de données, pas un second service ni un
wrapper sans comportement. Les fonctions métier existantes (`delegate`,
`append_add`, clôture) restent les seules autorités d'application.

## Ordre d'implémentation et composition

1. Geler les témoins de fermeture et la présente spécification sur
   `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`.
2. Attendre l'admission de 021, rebaser et relire ses motifs engendrés depuis
   une source unique.
3. Vérifier la forme v17, appliquer le DDL v18, puis marquer v18 dans cet ordre.
   L'installation refuse toute source différente de v17 ; une base déjà v18
   reste un no-op et une version future échoue. Le bootstrap vide prouve
   séparément qu'il exécute tous les DDL intermédiaires.
4. Livrer le paquet A avec ses témoins CLI/guichet et son audit durable.
5. Livrer le paquet B par couches : protocole, store Bridget, greffe Maicie,
   contexte central, CLI/lookup.
6. Mesurer le parcours réel et les reprises, puis geler la tête.

## Fichiers prévus

- Contrat : `crates/bridget-transport/src/protocol.rs`.
- Dépôt, lookup et persistance transport :
  `crates/bridget-daemon/src/cli.rs`, `crates/bridget-daemon/src/daemon.rs`,
  `crates/bridget-daemon/src/store.rs`.
- Vocabulaire, validation et cas d'usage : `plugins/maicie/src/domain.rs`,
  `plugins/maicie/src/guichet.rs`, `plugins/maicie/src/app.rs`.
- Migration et reçus : `plugins/maicie/src/store.rs`.
- Contexte central : `plugins/maicie/src/reconcile.rs`,
  `plugins/maicie/src/main.rs` et, seulement si nécessaire,
  `plugins/maicie/src/bridget_client.rs`.
- Catalogue : réemploi de `plugins/maicie/src/catalogue.rs`; modification
  seulement si une primitive atomique manque réellement.
- Témoins : tests contractuels Maicie et intégration daemon existants, plus le
  contre-test autonome de la session.

## Validation

Avant tout comptage Rust :

```bash
cargo test --workspace --no-run
```

Puis, dans cet ordre :

1. témoins exacts des approbations interdites ;
2. corpus de contradiction sur CLI locale et claim guichet ;
3. tests v18 depuis v17, refus des versions non adjacentes, bootstrap vide et
   injection inconnue ;
4. tests ciblés Maicie et guichet daemon ;
5. parcours réel dépôt → relève → effet central → lookup terminal ;
6. `cargo fmt --all --check`, clippy ciblé, puis workspace complet.

Chaque compte annonce SHA mesuré, passés, rouges, ignorés et imputation de
chaque rouge. Le gate `--features test-support` n'est pas forcé sur Linux s'il
échoue sur `kqueue`/`kevent`; les suites non mesurées sont nommées.

## Constitution check

| Gate | État | Justification |
|---|---|---|
| Une vérité durable | PASS | un seul guichet et un seul store/catalogue centraux |
| Sécurité ADR 011 | PASS sous tests | aucune approbation dans l'enum ; deux contre-tests séparés |
| Idempotence | PASS sous tests | identité externe unique, rejeu octet-exact, résultat terminal durable |
| Minimalisme | PASS | réemploi des cas d'usage et du guichet ; aucun second service ou accès DB |
| Observabilité | PASS sous migration | refus et signaux persistés sans recopier le contenu libre |
| Maintenance future | PASS sous corpus | source Rust unique et bijection `ALL` ↔ fixtures ; coût SQL documenté |

## Non vérifié à ce jalon

Le code partagé, la forme finale des motifs issue de 021, l'implémentation v18
et les comptes workspace finaux ne sont pas mesurés tant que 021 n'est pas
admise. Les contre-tests initiaux sont volontairement rouges jusqu'au paquet B.
