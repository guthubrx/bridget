# Plan 099 — Fiabilité et identité des communications

Statut : Implemented. Branche : session-099-fiabilite-communications.
Spec : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/099-fiabilite-communications/spec.md.

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/plan.md#extrait-1).

## Choix et réutilisation de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/plan.md#extrait-2).

## Fichiers impactés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/plan.md#extrait-3).

## Modèle de menace et compatibilité

L'attaquant connaît agent/instance et peut ouvrir le socket mais ne possède pas la
preuve privée. Un processus hostile du même compte pouvant lire cette preuve
reste hors cloisonnement garanti. Le jeton ne sort ni dans who/status ni dans logs.
Un token ancien ne survit pas à un changement de connexion propriétaire.
Ancien propriétaire sans auxiliaire demeure compatible ; auxiliaire ancien :
refus nommé, mise à jour wrapper/client nécessaire. Tester les accès légitimes
locaux et la lecture d'identité privée côté distant sans supposer que le chemin
du daemon existe sur la machine du wrapper.

## Stratégie de validation

Tests avant corrections pour chaque famille. Combiner tests unitaires des états
et tests d'intégration aux frontières (vrais sockets, base temporaire, faux HTTP).
Réutiliser les fixtures existantes ; ne pas remplacer leur comportement métier
par des mocks internes. Feature Gherkin 099 décrivant les acceptations, tests Rust
nommés spec099 et commandes explicites dans quickstart.md.
Tests de régression + workspace selon compatibilité et ressources, fmt, clippy.
Ne pas interpréter le nombre de tests comme une preuve d'exhaustivité.

## Gates constitutionnels et Article XIX/XX

- PASS : demande explicite, nouvelle session, français, aucun commit/déploiement.
- PASS : source/état isolés ; aucun secret réel dans les essais.
- PASS : protections existantes réutilisées, zéro nouvelle dépendance/service/table.
- Extension de protocole justifiée par trois usages réels : mutations auxiliaires,
  Send classique et SendIdempotent ; identité déclarée insuffisante aujourd'hui.
- État supplémentaire de réponse nécessaire à la reprise, stocké dans l'état
  existant du fil. Pas de nouvelle file de jobs générale.
- Le code de compatibilité historique n'est pas supprimé sans preuve d'inutilité.
- Aucun test utilisateur payant, aucune modification de configuration fournisseur.

## Ordre d'exécution

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/plan.md#extrait-4).
