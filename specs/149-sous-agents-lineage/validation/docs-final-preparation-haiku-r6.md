# Préparation documentaire finale - session 149 - Haiku 5.5 - ronde r6

Date : 2026-10-10. Auteur : agent documentaire Claude Haiku 5.5 (medium).
Périmètre : mise à jour documentaire seule. Aucun test, build, service, configuration, Git, sous-agent ni revue globale n'a été lancé.
Statut : **PARTIEL, non livré.** Aucune validation globale n'est prononcée. Aucune case de `tasks.md` n'est cochée (0 cochée, 45 vides).

## 1. Fichiers touchés (absolus, worktree 149)

| Fichier | Nature de la modification |
|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/quickstart.md` | Statut, runtime, UI, grants, reprise (phrases et tableaux obsolètes seulement). |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/implementation.md` | Ce qui est vrai, phases, rubriques de preuves, corrections, six scénarios, SC006, compteurs, routage, reste à faire. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md` | En-tête « Gates » marqué historique ; rôles de test et de revue passés à Claude (Sonnet, Haiku medium, Haiku high). Cases inchangées. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/plan.md` | En-tête « Statut du plan » marqué historique ; répartition des agents (tableaux, phases, gates). |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/docs-final-preparation-haiku-r6.md` | Créé : ce rapport (le fichier n'existait pas). |

Non touchés : `spec.md`, `contracts/`, `AGENTS.md`, `final.md`, `proof-map149.md`, `review-final149.md`, `docs-proof-sync-haiku-r5.md`, `guides-native149-haiku-r1.md`, les rapports de preuve, les reçus et les recettes.

## 2. Répartition des agents (plan.md et tasks.md)

Répartition approuvée, appliquée aux rôles de test et de revue :

- Sol high : développement complexe (inchangé).
- Claude Haiku 5.5 medium : docs et tests simples.
- Claude Haiku 5.5 high : revues ciblées.
- Claude Sonnet 5.5 high : permissions, reprise et tests complexes.

Les mentions GLM qui désignent le fournisseur produit ont été gardées : `glm-5.3-flash`, « Claude/GLM » comme famille de fournisseur, « enfant GLM », « recettes réelles avec GLM et Codex », registre `glm`, et le titre des rapports GLM datés. Rôles remplacés : T001 à T003, T015, T023, T035 à T042, T045, la ligne « Tests et relectures », la ligne GLM 5.3 Flash du tableau des rôles, les gates « APPROVE GLM » devenues « APPROVE de relecture ».

## 3. En-têtes historisés (sans prétendre lire ni approuver les 45 tâches)

- G-L Lineage : « APPROVE GLM r3 » du 2026-10-10 : historique.
- G-P permissions : « REQUEST_CHANGES ciblé r3 (G-P-07, G-P-08) » du 2026-10-10 : historique.
- État actuel cité : revue des sources `native-restart-final-source-review-sonnet-r2.md` = SOURCE_ONLY_APPROVE (lecture seule), et preuves runtime `native-tests-sonnet-r9.md`, `native-real-smoke-sonnet-r6.md`, `native-network-proofs-r3.md`.
- GO et cochage du principal : en attente des essais finaux ciblés r4 et r7.

## 4. Changements d'état documentés (quickstart et implementation)

| Sujet | Avant | Après |
|---|---|---|
| Binaires de preuve | r5 `abfb346e23cc`, debug `ec6b18b5d468` | Candidat r9 `abc850858975`, debug `0fb06ae920e6`. Preuves réelles r3 et r6 sur release r8 `0a29ad9b2cdb`. |
| Tests natifs | 1806 (r7 et r6) | 1840 PASS agrégés (r9), 0 FAIL final, SC005 non conclu, aucun claim global GREEN. Recompte r8 : 1822. |
| F2 (racine bloquée) | Ouvert | Corrigé au runtime (r3, R9.4.a à i). |
| O4 (fournisseur orphelin) | Ouvert | Partiellement fermé : arrêt au SIGTERM prouvé (r6). Relance rapide O4.6 à prouver (r4). |
| E2 | Non documenté au runtime | FAIL simulé r3 ; garde de filiation et 3 tests r9 ; runtime à prouver (r4). |
| F3 | Non documenté au runtime | FAIL E3.c r3 ; fermeture CAS et 5 tests r9 ; câblage final à prouver. |
| Alias Codex | Dans `BRIDGET_HOME` | Sous `/private/tmp/bridget-codex-*` ; 4 recettes réelles avec faux fournisseur HTTP (pas un modèle). Recette TUI 0.153.4 hors périmètre. |
| Relance avec parent Codex vivant | Non traité | Refus par lien d'état du daemon (`symlink d'état interdit`) ; contourné en r6. Pas un PASS propre. Traité par r7. |
| Revue sources | Antérieure aux preuves | SOURCE_ONLY_APPROVE r2, production 111 fichiers `b2b87458`. |

Ajouts : tableau des documents (section 1 d'`implementation.md`) avec les rapports r9, r3, r1 UI, r6, journal UI r1 ; mention historique de `docs-proof-sync-haiku-r5.md` et `guides-native149-haiku-r1.md`.

## 5. Pending (non fait ici, à décider par le principal)

1. Essais finaux r4 (fermeture rapide O4.6 et E2 en réseau) et r7 (redémarrage réel, parent Codex TUI, alias hors home).
2. Décision de cochage des tâches et GO principal (aucun cochage fait).
3. Fermeture des gaps G1, G2, G4, G6 à G9 ; trancher O1 et O5.
4. Relecture de `proof-map149.md` et `review-final149.md` (antérieurs aux rondes r2 à r9, non relus).
5. `validation/recipes/README.md` affiche encore « préparé, non exécuté » (non modifié, hors périmètre).
6. SC005 p95 : test non conclu sous charge (r9), à relancer hors charge si le principal le demande.
7. Limite connue : E4 (fournisseur ignorant SIGTERM après SIGKILL externe du wrapper) ; pas une promesse globale.
8. Préparation de livraison : commits, fusion, push, installation, reçu final et nettoyage restent au principal (T043 à T045).

## 6. Vérifications faites (lecture et comptage, sans exécution)

- Cases : `grep -c '- \[x\]' tasks.md` = 0 ; `grep -c '- \[ \]' tasks.md` = 45.
- Remplacements de rôles : script de remplacement à assertions (chaque motif doit apparaître le nombre attendu, sinon aucune écriture). Application après dry-run réussi.
- Mentions GLM restantes relues : seules des mentions de fournisseur produit ou de titres historiques subsistent, plus une correction manuelle de la ligne « lire la stratégie GLM » du tableau de phases.
- Chemins : absolus dans les tableaux et rapports.

## 7. Limites

- Aucun test, aucune build, aucune revue globale, aucune relecture de code. Les chiffres cités viennent des rapports r2 à r9 cités.
- Les formulations « historique » et « à prouver » reflètent l'état des rapports à la date du 2026-10-10. Elles ne remplacent pas les essais r4 et r7.
- Le script temporaire d'édition (`/tmp/b149-docs/sync_agents.py`) est hors du worktree et a été supprimé après usage.
