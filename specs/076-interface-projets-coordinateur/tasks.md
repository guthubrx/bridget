# Tâches - SPEC-076 Espaces projets et coordinateur initial

**Statut**: In Progress - reprise autonome sur main 20e50ac
**Branche documentaire**: session-076-interface-projets-coordinateur
**Dépendances de code**: SPEC-066, SPEC-067 et SPEC-075 intégrées sur une même tête main.
**Preuve**: aucune case ne peut être cochée sans test, audit ou preuve
documentaire explicitement demandée.

## Phase 0 - Gates de reprise

- [x] T001 Créer un worktree d'implémentation frais depuis main propre et
  prouver les intégrations de SPEC-066, 067 et 075 dans implementation.md.
- [x] T002 Rejouer reuse-audit.md sur cette tête et analyser le relais UI actuel ainsi que le contrat SPEC-075 intégré.
- [ ] T003 Cartographier et tester la frontière UI existante : loopback, jeton, UID pair, délais, lecture Maicie publique et absence d accès direct à sa base ou de shell libre.
- [x] T004 Constater le fail-closed de ProjectRootPolicy en fixture et en production, puis écrire les tests qui guideront son reload atomique avant de l implémenter.
- [x] T005 Relire ADR-019, spec, plan, contrat et checklists. Traiter toute
  case ouverte avant le premier changement de code.

## US1 - Racines et valeurs par défaut

- [ ] T010 Écrire les tests de contrat de racines : version, génération,
  chemin relatif, racine large, duplication, UID et permissions.
- [ ] T011 Écrire les tests de lien sortant, retrait de racine contenant projet
  actif et reload concurrent.
- [ ] T012 Étendre ProjectRootPolicy avec génération, écriture atomique, reload
  attesté et audit non sensible, en préservant les refus SPEC-065.
- [ ] T013 Ajouter les routes UI locales de lecture et modification des racines
  avec version, corps fermé et command_id, puis prouver l'absence ou le refus
  de toute approbation de profil, extension ou secret.
- [ ] T014 Ajouter les défauts coordinateur en séparant outil, upstream, modèle,
  effort et permissions. Ne proposer que les combinaisons attestées.
- [ ] T015 Ajouter l'écran réglages accessible au clavier et prouver qu'un
  changement de défaut ne réécrit aucun coordinateur existant.

## US2 - Créer ou importer sans surprise

- [ ] T020 Écrire les tests de prévisualisation create et import : chemin
  final, conflit, Git, configuration absente et confirmation divergente.
- [ ] T021 Créer l'entrée Maicie versionnée pour les intentions UI project,
  en réutilisant saga register et client ProjectRegistry existants, avec
  `display_name` dérivé du nom de dossier sans alias UI.
- [ ] T022 Implémenter création ou import avec canonicalisation, Git explicite
  et interdiction de fusion ou écrasement.
- [ ] T023 Ajouter routes UI de prévisualisation et confirmation avec
  idempotence command_id et état de reprise après panne.
- [ ] T024 Construire l'assistant de création ou import : racine, nom ou
  dossier, Git, configuration, durée et récapitulatif.
- [ ] T025 Projeter les états dossier prêt, liaison en cours et coordinateur
  non démarré. Prouver que configuration absente ne crée rien.

## US3 - Retrouver, déplacer, réactiver

- [ ] T030 Écrire tests de diagnostic Git absent, propre, modifié et worktree,
  sans retour de contenu de fichier, ainsi que le scénario de réactivation
  idempotente conservant identité, liaison et audit.
- [ ] T031 Implémenter la mutation typée de réactivation dans Maicie et Bridget :
  disabled vers active, racine de nouveau validée, même identité, liaison active,
  command_id idempotent et exactement un audit appliqué.
- [ ] T032 Étendre projection avec identité, liaison, audit et liens agents
  sans créer de store UI.
- [ ] T033 Implémenter dossier connu : ouvrir actif, proposer réactivation du
  retiré en s'appuyant sur T031 et refuser tout double enregistrement.
- [ ] T034 Implémenter rebind UI explicite pour path_missing et afficher audit.
  Prouver absence de scan, déplacement ou reconnexion automatique.
- [ ] T035 Prouver que les agents historiques restent non enregistrés lors d'un
  import et que seuls les nouveaux lancements portent ProjectReference.

## US4 - Coordinateur et découverte

- [ ] T040 Écrire tests de CoordinatorConfiguration : attestation, digest
  divergent, modèle indisponible, upstream distinct et défaut changé.
- [ ] T041 Implémenter instantané coordinateur sans secret, lié aux autorités
  SPEC-066, 067 et 072.
- [ ] T042 Implémenter coordinateur courant unique et reprise avec contrat
  stop, relaunch et decommission de SPEC-075.
- [ ] T043 Ajouter états unavailable et incompatible avec motif lisible et
  sans substitution, relance ou remplacement automatique.
- [ ] T044 Écrire tests DiscoveryRun : lecture seule, 10, 30, 60, 120 minutes,
  refus illimité, confirmation de poursuite et arrêt.
- [ ] T045 Implémenter DiscoveryRun sous permissions lecture seule prouvées par
  SPEC-066 et 067. Vérifier empreintes dossier et Git avant et après.
- [ ] T046 Ajouter événements système atténués et rapport structuré dans la
  conversation, sans fichier mémoire, bouton ni création agent.

## US5 - Barre projets, retrait et historique

- [ ] T050 Écrire tests snapshot projets, filtre ProjectReference, Toute la
  flotte et agents historiques.
- [ ] T051 Étendre UiSnapshotV1 et watch avec la projection projets sans second
  polling ou store navigateur.
- [ ] T052 Construire barre repliable, Toute la flotte et ouverture directe de
  conversation coordinateur, puis couvrir le clavier et le focus.
- [ ] T053 Ajouter retrait confirmé, arrêt propre SPEC-075, disable SPEC-065 et
  réactivation sans suppression de dossier, Git, messages ni audit.

## Validation intégrale

- [ ] T060 Rejouer tests unitaires, contrats et intégrations des lots 1 à 5.
- [ ] T061 Rejouer non-régressions SPEC-063, 064, 065, 072 et 075.
- [ ] T062 Exécuter cargo fmt, cargo clippy --workspace --all-targets et cargo
  test --workspace sans erreur.
- [ ] T063 Mesurer le parcours quickstart sur fixtures hors production, puis recueillir la validation manuelle utilisateur distincte.
- [ ] T064 Rejouer reuse-audit après code et documenter chaque création ou
  extension décidée après recherche.
- [ ] T065 Exécuter Analyze, corriger les findings non ambigus et produire
  analysis-report.md sans finding CRITICAL.
- [ ] T066 Demander contre-revue adverse, vérifier chaque objection et
  consigner adversarial-review.
- [ ] T067 Exécuter Converge entre exigences, code et tests; boucler jusqu'à
  CONVERGED puis mettre à jour implementation.md avec statut réel.
