# ADR 015 - Séparer mission, contrôle d'exécution et fournisseurs

**Date** : 2026-08-29
**Statut** : Proposé
**Portée** : programme SPEC-064

## Contexte

Bridget garantit le transport durable, l'identité des instances, les
générations et le parc. le service compagnon garantit les objectifs, délégations, décisions
et outboxes métier. Les adaptateurs exposent les événements de Codex, Claude et
ACP. Cursor est déjà un fournisseur déclaré utilisant l'ACP maintenu par Cursor
et ne nécessite aucun adaptateur séparé.

L'incident de la session 063 a montré qu'une commande fournisseur acceptée ne
prouve pas qu'un message est devenu visible dans le tour du modèle. Plus
globalement, le système ne possède pas encore d'entité durable représentant le
travail logique, son exécution, son propriétaire et sa relation avec un thread
ou un tour fournisseur.

Le code officiel Codex distingue explicitement thread, tour, item, mailbox,
steering, interruption, reprise et graphe d'agents. Ces concepts sont utiles,
mais le goal Codex ne remplace pas la coordination durable service compagnon et ses
garanties multi-fournisseurs.

## Décision proposée

L'architecture comporte trois plans :

1. **Plan de mission le service compagnon** : objectifs, délégations, décisions, évaluations,
   clôtures et politiques métier.
2. **Plan de contrôle Bridget** : soumissions, livraisons, files, exécutions,
   agents, propriété, interruptions, reprise et état runtime.
3. **Plan des adaptateurs fournisseurs** : traduction des commandes neutres,
   capacités, identifiants et événements bruts propres au fournisseur. Codex
   utilise app-server, Claude utilise stream-json et Cursor utilise le transport
   ACP commun.

Les bases Bridget et le service compagnon restent privées et séparées. Les échanges passent
par des contrats publics versionnés. Une projection n'est jamais une source de
vérité supplémentaire.

La décision SPEC-034 reste applicable : le graphe porte les engagements et
liens de propriété nécessaires, sans devenir un moteur de workflow ou une
description de la pensée des agents.

## Conséquences positives

- Une livraison réseau, une consommation et une exécution deviennent des faits
  distincts et vérifiables.
- L'état d'un agent devient observable sans inférence depuis sa seule connexion.
- Les agents enfants, leurs mandats et leurs résultats ont un propriétaire
  durable.
- La reprise native peut être utilisée sans imposer Codex aux autres
  fournisseurs.
- le service compagnon conserve sa remplaçabilité et son autorité métier.
- Les budgets et continuations automatiques peuvent être ajoutés après preuve
  fiable d'inactivité et d'usage.

## Conséquences négatives

- Deux machines d'état supplémentaires doivent être persistées et migrées.
- Les contrats publics et leurs versions augmentent la discipline de release.
- Les adaptateurs doivent publier des capacités et corrélations plus riches.
- La suppression de la dépendance daemon vers le service compagnon demande une migration
  progressive des projections existantes.
- La période de compatibilité maintient temporairement des projections anciennes
  et nouvelles.

## Alternatives rejetées

- **Fusionner le service compagnon et Bridget** : double autorité, migrations couplées et
  perte de remplaçabilité.
- **Remplacer le service compagnon par le goal Codex** : modèle lié à un thread et
  insuffisant pour les délégations et décisions multi-fournisseurs.
- **Adopter un moteur générique de workflow** : charge cognitive et surface de
  panne sans besoin prouvé.
- **Conserver seulement `busy` et les journaux** : insuffisant pour distinguer
  travail, attente, blocage et absence de progrès.
- **Copier le code Codex** : verrou fournisseur et dette de synchronisation.

## Conditions d'acceptation

- La session 063 corrèle la consommation sur le contrat Codex réel.
- `reuse-audit.md` confirme la réutilisation des seams existants.
- Les contrats d'identifiants et d'autorité ne laissent aucune double vérité.
- Chaque lot est activable, testable et réversible indépendamment.
- L'ADR passe de Proposé à Accepté uniquement après validation du plan SPEC-064.
