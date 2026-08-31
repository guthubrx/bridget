# Recherche et décisions - SPEC-076

## Objet

Cette recherche prépare l'interface des espaces projets et le coordinateur
initial sans introduire de seconde autorité pour les projets, les profils ou
les secrets.

## Sources externes consultées

| Source | Apport retenu | Conséquence pour SPEC-076 |
|---|---|---|
| [Microsoft Research - Guidelines for Human-AI Interaction](https://www.microsoft.com/en-us/research/publication/guidelines-for-human-ai-interaction/) | Une interface agent doit expliquer capacités, limites, erreurs et changements. | Les états de coordinateur, les incompatibilités et la progression sont factuels et visibles. |
| [Microsoft Research - Human-AI interaction over time](https://www.microsoft.com/en-us/research/articles/how-to-build-effective-human-ai-interaction-considerations-for-machine-learning-and-software-engineering/) | Les attentes initiales et les changements exigent des contrats fiables, pas de simples éléments visuels. | Les étapes d'onboarding viennent de verdicts durables, jamais du navigateur seul. |
| [Anthropic - Trustworthy agents in practice](https://www.anthropic.com/research/trustworthy-agents) | Outils, données et permissions remis à un agent restent sous contrôle humain. | Découverte initiale lecture seule, profils hors UI selon SPEC-067 et créneaux bornés. |
| [NIST AI RMF Core](https://airc.nist.gov/airmf-resources/airmf/5-sec-core/) | Les rôles, l'oversight humain et les risques tiers sont documentés. | Outil agent, upstream, modèle, effort et permissions restent distincts. |

Ces sources ne servent pas à introduire un framework. Elles soutiennent le
choix minimal : transparence, contrôle humain, périmètre borné et contexte
durable.

## Réutilisation vérifiée sur la tête 20e50ac

| Besoin | Existant vérifié | Décision |
|---|---|---|
| Identité et liaison | plugins/maicie/src/app.rs:108 et crates/bridget-daemon/src/store.rs:267 | Réutiliser ProjectIdentity et ProjectBinding. Aucun registre UI. |
| Idempotence et audit | crates/bridget-daemon/src/store.rs:602 et :1051 | Réutiliser commandes et ProjectAuditEvent. |
| Politique de racines | crates/bridget-daemon/src/project_policy.rs:16-123 | Étendre la politique fermée, jamais la contourner. |
| Projection agents | crates/bridget-daemon/src/ui.rs:650-691 et :1479-1524 | Étendre le snapshot existant, sans second flux. |
| Relais UI local | crates/bridget-daemon/src/ui.rs:798-903 | Ajouter des routes UI typées et authentifiées. |
| Saga Maicie | plugins/maicie/src/main.rs:659-825 | Maicie reste propriétaire, sans accès direct UI à sa base. |
| Fournisseurs | crates/bridget-daemon/src/registry.rs:44-86 et SPEC-072 | Exposer seulement les configurations attestées. |
| Cycle de vie | fleet.rs, lifecycle.rs, ui.rs intégrés par SPEC-075 (7423464) | CONSOMMER le contrat livré, ne pas le refaire. |
| Runtime et profils | project_runtime.rs et profils/extensions/secrets intégrés par SPEC-066/067 | CONSOMMER ; ne pas cloner leurs autorités ni leurs approbations. |

## Constats

1. Les routes UI actuelles servent messagerie, arrêt, recherche, snapshot,
   journal et un diagnostic runtime de projet (GET /v1/projects/runtime).
   Aucune liste de projets, mutation projet ni explorateur de dossiers borné
   n existe encore.
2. La projection actuelle expose seulement project_id et project_state par
   agent. Elle ne liste pas les projets et ne connaît pas leur coordinateur.
3. La politique de racines est chargée une fois au démarrage. Le service
   installé dans /home/moi/.config/systemd/user/bridget-daemon.service ne passe
   pas de politique de racines. Les mutations sont donc correctement fermées.
4. SPEC-065 réservait sa tranche aux commandes Maicie locales. L'ouverture UI
   demandée est une évolution de surface explicite, non un court-circuit de
   Maicie.
5. SPEC-074 reste dans un worktree séparé et ne doit pas être modifiée. SPEC-075
   est désormais intégrée à main. SPEC-076 part de la tête propre 20e50ac.

## Décisions de planification

### D1 - Projection UI, pas de second registre

La liste assemble ProjectIdentity, ProjectBinding, ProjectAuditEvent et liens
agents. Elle ne persiste aucune copie indépendante dans l'interface.

### D2 - Commandes UI étroites

Chaque intention est finie : prévisualiser, créer, importer, rebind, retirer,
réactiver ou demander une action coordinateur. L'UI ne reçoit ni commande shell
libre ni accès direct à la base Maicie.

### D3 - Politique de racines fail-closed

L'interface locale authentifiée pourra modifier une politique seulement par
validation complète, écriture atomique, génération attendue et reload attesté.
Une politique absente ou invalide bloque toute mutation.

### D4 - Configuration durable

Le coordinateur conserve un instantané outil, fournisseur ou upstream, modèle,
effort, permissions et digest. Les défauts globaux ne changent que les futurs
projets. Une incompatibilité n'autorise aucune substitution.

### D5 - Découverte bornée

Le coordinateur écrit son rapport dans la conversation existante, sans créer
de fichier mémoire. Chaque créneau est lecture seule, borné et confirmé avant
poursuite.

## Alternatives écartées

| Alternative | Rejet |
|---|---|
| Fichier Bridget dans chaque dépôt | Mémoire concurrente et écriture inutile dans un import. |
| Domaine ou nom de dossier comme identité | Collisions et faux rattachements déjà exclus par SPEC-065. |
| UI vers la base Maicie | Viole la frontière de propriété des stores privés. |
| Shell fourni par UI | Escalade de privilège sans contrat ni audit borné. |
| Remplacement automatique de coordinateur | Change le pilotage sans décision et sans transmission fiable. |
| Analyse sans limite | Retire le point de contrôle humain. |

## Risques avant implémentation

- Le reload de politique est une évolution de frontière de sécurité et exige
  ADR, tests de concurrence et preuve atomique.
- La politique de racines n’est pas configurée en production : les mutations réelles restent fail-closed jusqu’à une décision d’exploitation explicite.
- L’entrée Maicie UI doit être prouvée sans accès direct au store ni shell.
- Le reuse-audit sera rejoué sur une main propre après intégration des branches
  concurrentes.
