# Plan d'implémentation - SPEC-076 Espaces projets et coordinateur initial

**Branche documentaire**: session-076-interface-projets-coordinateur
**Spec**: specs/076-interface-projets-coordinateur/spec.md
**Date**: 2026-08-30
**Statut**: En cours - gates techniques et documentaires à rejouer sur main 20e50ac

## Résumé

SPEC-076 transforme le registre de SPEC-065 en parcours utilisateur : créer ou
importer un dossier autorisé, l'ouvrir comme projet et arriver dans la
conversation de son coordinateur. Le dossier reste l'identité visible.

Maicie garde ProjectIdentity et sa saga. Bridget garde liaison canonique,
politique de racines, faits runtime et identité agent. SPEC-066, SPEC-067 et
SPEC-075 sont intégrées sur main et fournissent runtime, permissions et cycle de vie de référence.

## Périmètre et frontières

| Périmètre | SPEC-076 | Réutilisation |
|---|---|---|
| Dossier visible et navigation | propriétaire | ProjectIdentity et ProjectBinding SPEC-065 |
| Create, import, rebind, retrait, réactivation UI | surface utilisateur | Saga Maicie et audit Bridget SPEC-065 |
| Réglages racines | UI et reload sûr | Validation canonique SPEC-065 |
| Coordinateur initial | parcours onboarding | Registre SPEC-072, cycle SPEC-075 |
| Profils, secrets, extensions | hors périmètre | SPEC-067 |
| Environnement projet | hors périmètre | SPEC-066 |
| Connexion Desktop et SSH | hors périmètre | SPEC-074 |
| Remplacement coordinateur | hors périmètre | SPEC future dédiée |

## Architecture retenue

Interface Bridget locale authentifiée
vers contrôleur UI Bridget
vers projection ProjectBinding et audit Bridget pour les lectures
et vers entrée Maicie spécialisée pour les mutations métier.
Le relais UI est lié à la boucle locale du serveur. Bridget Desktop ne le
consomme qu'au travers du tunnel SSH attesté par SPEC-074; ce plan n'ajoute
aucune écoute réseau.
Cette entrée Maicie réutilise la saga et le client ProjectRegistry existants.
Elle ne donne pas à Bridget un accès direct au store Maicie.

Réglages racines UI
vers gestionnaire de politique Bridget
vers validation, écriture atomique et reload attesté.

Principes :

1. Étendre UiSnapshotV1 et le flux consommé par l'application UI existante.
   Aucun second polling ou store navigateur global.
2. Ajouter routes UI internes authentifiées, schémas fermés, version,
   command_id et erreurs structurées.
3. Créer une entrée Maicie dédiée aux intentions UI, sans shell libre.
4. Faire évoluer ProjectRootPolicy en politique versionnée rechargée
   atomiquement. Erreur : ancienne génération reste active.
5. Construire CoordinatorConfiguration depuis définition résolue et digest,
   jamais depuis heuristique de nom.
6. N'autoriser découverte qu'avec mode lecture seule prouvé par SPEC-066 et 067.
7. Projeter les étapes dans la conversation en événements système atténués.

## Lots prévus

### Lot 1 - Contrats et politique de racines

- Définir lectures, écritures, prévisualisations et confirmations UI fermées.
- Ajouter génération, concurrence, écriture atomique et reload attesté.
- Conserver refus chemin relatif, manquant, large, permissions invalides et lien sortant.
- Bloquer retrait de racine si projet actif concerné.
- Mettre à jour ADR-019 et tests de non-escalade.

Gate : politique absente ou invalide reste fail-closed; mise à jour concurrente
sans génération attendue refusée.

### Lot 2 - Projection et navigateur borné

- Étendre snapshot par liste projets issue identité, liaison, audit et agents.
- Parcourir seulement dossiers enfants des racines après canonicalisation.
- Diagnostiquer Git minimal : absent, propre ou modifié, sans contenu.
- Construire barre projets repliable, Toute la flotte et filtre ProjectReference.
- Conserver agents historiques non enregistrés.

Gate : chemins hors racine et contenus de fichier n'apparaissent jamais.

### Lot 3 - Commandes projet sans double autorité

- Implémenter entrée Maicie typée pour register, status, rebind, disable et réactivation.
- Implémenter la transition durable idempotente disabled vers active et la
  liaison active validée, sans seconde ProjectIdentity ni second audit.
- Ajouter prévisualisation et confirmation idempotente UI.
- Créer le dossier seulement après validation; import ne modifie rien hors Git confirmé.
- Réutiliser ProjectAuditEvent, sans second journal.
- Garder état de reprise après panne, sans suppression compensatoire.

Gate : mêmes command_id et octets convergent; création ne fusionne ni n'écrase.

### Lot 4 - Configuration et cycle coordinateur

- Lister seulement configurations résolues, attestées et compatibles.
- Persister instantané sans secret et sans appliquer futurs défauts aux existants.
- Créer un seul coordinateur courant avec cycle SPEC-075.
- Afficher incompatibilité sans remplacement.
- Retirer après arrêt propre selon SPEC-075.

Gate : retour projet sans doublon; configuration indisponible bloque avant écriture évitable.

### Lot 5 - Découverte et conversation

- Ajouter état onboarding et événements système conversationnels.
- Démarrer sous permissions lecture seule SPEC-066 et 067.
- Borner à 10, 30, 60 ou 120 minutes avec confirmation de poursuite.
- Produire compte rendu : constats, risques, questions, propositions sans agent.
- Ouvrir directement conversation et la conserver au retour.

Gate : fixture inchangée, aucun agent secondaire créé.

### Lot 6 - Validation et exploitation

- Parcours clavier, confirmations, erreurs, projets retirés.
- Refus chemins, liens, politiques et configurations incompatibles.
- Courses register, import, rebind et retrait avec audits.
- Absence secrets, contenus fichiers et shell dans UI, logs et diagnostics.
- Non-régressions SPEC-063, 064, 065, 072 et 075.

## Stratégie de tests

1. Unitaires : politique, transitions, compatibilité, limites durée.
2. Contrat : version, champs inconnus, command_id divergent, refus de surface.
3. SQLite : projections, audits, retrait, unicité coordinateur.
4. Intégration Maicie-Bridget : saga, rebind, disable et reprise.
5. Intégration UI : navigateur fixture, Git propre, Git modifié, non-Git, lien et déplacement.
6. Cycle coordinateur : contrat SPEC-075.
7. Découverte lecture seule : empreinte dossier et Git avant/après.
8. Accessibilité : barre, confirmations et clavier.
9. Validation manuelle hors production avant déploiement.

## Sécurité et observabilité

- Aucun shell ni chemin non validé comme autorité.
- Aucun secret, contenu de fichier, environnement ou clé dans réponses, audit ou capture.
- Mutations par relais UI local authentifié seulement, jamais MCP, agent ou API générique.
- Le relais UI est lié à loopback; tout accès Desktop passe par le tunnel SSH
  attesté de SPEC-074 et aucune écoute réseau directe n'est autorisée.
- Audit : command_id, projet, génération, résultat, prochaine action et durée.
- Politique : ancienne et nouvelle génération auditées; pas de chemin en label métrique.
- Aucun repli automatique fournisseur, modèle, permission ou backend.

## Concurrence

- Worktree documentaire : /home/moi/bridget-referent/.worktrees/session-076-interface-projets-coordinateur
- SPEC-074 reste isolée dans son worktree ; SPEC-076 ne modifie ni son application Desktop ni son tunnel SSH.
- SPEC-075 est intégrée à main par 7423464 ; SPEC-076 consomme son contrat sans le redéfinir.
- Aucun code de ces worktrees n'est modifié.
- Avant code, worktree frais depuis main propre et rejeu complet du reuse-audit.

## Gates avant implémentation

- [x] SPEC-066 est intégrée à main (commit dda4ec2) avec ses preuves versionnées.
- [x] SPEC-067 est intégrée à main (commit 2ada91f) avec ses preuves versionnées.
- [x] SPEC-075 est intégrée à main (commit 7423464) et son contrat de cycle de vie est disponible.
- [ ] Frontière UI actuelle prouvée : loopback, jeton, UID pair, lecture Maicie publique et aucune entrée shell libre.
- [ ] Fail-closed actuel de ProjectRootPolicy prouvé ; reload atomique et réversible couvert par tests avant son implémentation.
- [ ] État de politique production constaté avant toute mutation. L’installation d’une racine réelle reste une décision d’exploitation, jamais un prérequis à l’écriture du code.
- [ ] Reuse-audit frais sans NEEDS_ARBITRATION ni BLOCKED.
- [ ] Analyze sans finding CRITICAL.
- [x] Autorisation utilisateur d’implémenter après revue par commande explicite.

## Constitution Check

| Règle | Verdict | Décision |
|---|---|---|
| Français | PASS | Artefacts en français. |
| Cycle SpecKit | PASS avec dérogation | Synchronisation .specify interdite explicitement; artefacts versionnés font foi. |
| Worktree | PASS | Worktree 076 isolé, 074 et 075 intacts. |
| Réutilisation | PASS provisoire | Audit devra être rejoué sur tête propre. |
| Minimalisme | PASS | Aucun second registre, framework, base commune ou shell libre. |
| Responsabilité future | PASS | 066, 067, Desktop et remplacement restent séparés. |
| Sécurité | PASS avec gate | ADR-019 et preuve reload requises avant code. |
| Déploiement | PASS | Aucun déploiement ni redémarrage dans cette phase. |

## Livrables

- spec.md
- research.md
- data-model.md
- contracts/project-ui-v1.md
- quickstart.md
- reuse-audit.md
- tasks.md
- analysis-report.md
- implementation.md
- audit-report.md
- adversarial-review-<agent>.md
- docs/decisions/019-administration-locale-racines-projets-ui.md

## Statut d'implémentation

Aucun code SPEC-076 n’est encore commencé. Les dépendances intégrées sont prouvées ; la première étape est le rejeu des gates T001 à T005 sur main 20e50ac.
