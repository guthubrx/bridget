# ADR 007 — Socle d’idempotence commun pour les clients Bridget

**Date** : 2026-08-22  
**Statut** : Accepté

## Contexte

Bridget transporte déjà des messages et conserve des demandes suivies, mais un
client externe ne peut pas reprendre un envoi après une coupure sans risque de
doublon : l’identifiant est créé par la voie historique et ses gardes de
déduplication sont temporaires et en mémoire.

le service compagnon doit pouvoir persister une délégation avant toute E/S, puis consulter
l’issue ou rejouer exactement la même opération après un redémarrage. Les
ordres de lancement de la session 009 et les projections MCP de la session 010
ont le même besoin structurel. Trois mécanismes indépendants rendraient leurs
durées de conservation, leurs reprises et leurs preuves de crash incompatibles.

## Décision

Le daemon possède un unique socle transactionnel d’idempotence, identifié par
`(issuer_scope, operation_kind, idempotency_key)`. Il est le seul propriétaire
du canon d’enveloppe, de `issued_at`, de `expires_at` et du résultat public.
Le canon est stocké comme octets déterministes et comparé exactement ; un hash
éventuel ne peut être qu’une optimisation, jamais une preuve d’égalité.

Une nouvelle opération réserve durablement sa clé avant les gardes mutables,
puis suit la machine monotone `Prepared → Dispatching → Terminal`. Un rejeu
avec le même canon retourne le résultat déjà connu ; un canon différent retourne
`EnvelopeMismatch`. Une opération non terminale produit `OutcomeUnknown`, qui
ne donne jamais l’autorisation de réémettre aveuglément. Au-delà de
`expires_at`, le résultat est `IdempotencyExpired`, y compris pour une première
opération dont `issued_at` est déjà hors de l’horizon négocié.

La frontière est un protocole local public et versionné. La CLI, le service compagnon et un
outil MCP futur en sont des projections : aucun ne dépend des crates internes
ni de la base SQLite du daemon. La voie historique `Send` reste inchangée et ne
traverse pas ce socle sans négociation explicite.

La négociation suit l'ordre strict D-604 : `RoleHello(Client)`, puis
`RoleAccepted`, puis seulement `ClientHello`. La matrice des rôles est fermée :
les opérations idempotentes sont réservées au rôle client, les accusés de
remise au wrapper, et aucune de ces variantes n'est admise sur une connexion
attach.

Pour les remises aux wrappers, le socle sera complété par une remise accusée
distincte, identifiée par `delivery_id`. La portée promise est explicitement
limitée : exactement une injection à travers les crashs client et daemon tant
que le wrapper reste vivant ; un wrapper arrêté après une marque `Seen` donne
une issue indéterminée, pas une seconde injection.

Le store de reçus D-608 est détenu par l'`instance_id` stable du wrapper sous
un répertoire d'état durable, non par son nom affiché. Toute corruption ou
disparition après son initialisation met la génération en quarantaine
fail-closed : la réponse devient indéterminée et le store n'est jamais recréé
vide.

## Conséquences positives

- le service compagnon peut reprendre son outbox avec des identifiants et une rétention
  contractuels.
- Les futures opérations idempotentes réutilisent les mêmes états, erreurs et
  règles de purge.
- Les clients héritent d’une sémantique observable, testable et indépendante de
  la CLI.
- La compatibilité de la voie historique est protégée par une variante de
  protocole séparée.

## Conséquences négatives

- Le daemon ajoute des tables durables et un rôle de connexion client.
- L’`issuer_scope` doit être généré et conservé par chaque client appelant ; il
  exprime une portée d’idempotence, pas une authentification.
- Les consommateurs 009 et 010 devront se raccorder au socle au lieu de créer
  leurs propres registres d’issues.
- le service compagnon devra compléter son outbox par `issuer_scope` et `issued_at` avant son
  implémentation effective.
