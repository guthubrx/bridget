# Feature Specification: Activation gouvernée des outils de pilotage

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 018-activation-outils-pilotage
Titre: Activation gouvernée des outils de pilotage
Statut: Prête en contre-relecture
Priorité: P1
Tâches: 14/14 (100%)
Tests: 14/14 (100%)

Résumé:
- Contexte: `bridget-idle` a été activé par un lien vers un worktree avant son jury et son merge ; sa suppression aurait cassé l'outil actif.
- Objectif: rendre impossible et immédiatement lisible toute activation d'un outil de pilotage depuis un code non admis sur `origin/main`.
- Exécution: matérialiser une version explicitement activée et identifiable, indépendante de tout espace de travail.
- Risque principal: reproduire un « canary clandestin » propre et versionné mais encore non jugé.
- Mitigation: refuser l'activation avant tout effet si la provenance Git ou l'état local ne satisfait pas le contrat.
- Validation: refus discriminants, représentation et modes exacts, activation attestée, unités exactement idempotentes et survie sans source.
- Dépendances: SPEC-011

Fichiers:
- spec.md: ✓ (specs/018-activation-outils-pilotage/spec.md)
- tasks.md: ✓ (specs/018-activation-outils-pilotage/tasks.md)
- plan.md: ✓ (specs/018-activation-outils-pilotage/plan.md)
- implementation.md: ✓ (specs/018-activation-outils-pilotage/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-18-activation-outils-pilotage`
**Created**: 2026-08-25
**Status**: Prête en contre-relecture
**Priority**: P1
**Dependencies**: SPEC-011

---

## Contexte et Problème

Le 25 août 2026, `/Users/moi/.local/bin/bridget-idle` pointait vers
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/fix-bridget-idle-partition-complete/scripts/bridget-idle.py`.
Le fichier actif était propre, commité, poussé et identique au commit
`aa16dd36598ae0395014014ef8637c6d52e1d81f`. Le défaut n'était donc pas la
qualité ni la traçabilité du code : le lien l'avait mis en service avant le
verdict et avant le merge.

Trois évolutions — partition complète, catégorie des agents bloqués et raisons
d'indétermination — sont ainsi entrées en production sans revue préalable.
Supprimer le worktree aurait aussi supprimé la cible de l'outil actif.
`bridget-ronde`, qui rythme la vigilance et lit le registre, possède la même
frontière fragile.

Repointer les liens vers un checkout stable après merge répare l'instance mais
pas la classe d'incident : une modification locale, un changement de branche
ou un nouveau lien vers un worktree pourrait redevenir actif sans geste de
déploiement identifiable.

## Principes Directeurs

- **Merge avant activation** : l'activation ne peut jamais servir de canary clandestin avant jury.
- **Production indépendante du chantier** : supprimer n'importe quel worktree ne doit jamais casser ni modifier un outil actif.
- **Origine lisible** : un opérateur doit retrouver le SHA complet et la référence d'admission depuis la cible installée, sans enquête sur les checkouts.
- **Refus avant effet** : une provenance invalide ne doit créer ni commande, ni unité, ni archive annoncée comme prête.

## User Scenarios & Testing

### User Story 1 — Activer une version admise et durable (Priority: P1)

Après le jury et le merge, l'opérateur active `bridget-idle` ou
`bridget-ronde` depuis le checkout principal. La commande active désigne une
version explicite, dont l'origine est lisible, et continue de fonctionner si
le dépôt ou ses worktrees deviennent indisponibles.

**Why this priority**: les deux outils pilotent la détection et la ronde ; leur disparition ou leur mutation implicite fausse directement la coordination.

**Independent Test**: installer chaque outil depuis un `main` propre et admis, relever sa cible et son origine, supprimer la copie source de test, puis exécuter la commande installée.

**Acceptance Scenarios**:

1. **Given** un checkout principal propre sur `main` dont `HEAD` est ancêtre de `origin/main`, **When** l'opérateur installe un outil, **Then** la cible active porte le SHA complet et demeure hors de tout dépôt ou worktree.
2. **Given** un outil déjà activé au même SHA, **When** l'opérateur rejoue l'installation, **Then** l'opération réussit sans changer les octets ni la cible.
3. **Given** une installation sûre terminée, **When** la source de test est supprimée, **Then** la commande installée reste exécutable.

### User Story 2 — Refuser tout canary clandestin (Priority: P1)

Un auteur ou opérateur qui lance l'installateur depuis un état non admis reçoit
un refus explicite avant toute mutation. Le message nomme la condition qui
manque et la marche à suivre, au lieu de laisser croire que l'outil est prêt.

**Why this priority**: un code propre, commité et poussé peut encore être non jugé ; contrôler seulement la présence d'un commit reproduirait exactement l'incident.

**Independent Test**: exercer successivement un worktree lié, une branche autre que `main`, un arbre sale, une commande régulière préexistante sans autorisation de remplacement et un `main` local en avance sur `origin/main`.

**Acceptance Scenarios**:

1. **Given** chacun des cinq états interdits, **When** l'installation est lancée, **Then** elle échoue avec un motif distinct et aucun artefact actif n'est créé ou remplacé.
2. **Given** un `main` local en retard mais ancêtre de `origin/main`, **When** l'installation est lancée, **Then** ce rollback vers un commit déjà admis reste autorisé.
3. **Given** une référence `origin/main` absente, **When** l'installation est lancée, **Then** elle refuse et demande une synchronisation sans tenter le réseau implicitement.

### User Story 3 — Ne jamais annoncer une ronde non installée (Priority: P1)

Si une copie régulière de `bridget-ronde` existe et que le remplacement n'est
pas explicitement autorisé, l'installateur s'arrête avant d'écrire ou d'activer
une unité. Il ne peut plus conserver l'ancienne copie tout en annonçant la
ronde prête.

**Why this priority**: le faux succès actuel rend la provenance réelle indiscernable et peut lancer une unité sur un code différent de celui annoncé.

**Independent Test**: poser une copie sentinelle, lancer l'installateur sans autorisation de remplacement, puis vérifier le code de sortie, les octets de la sentinelle et l'absence d'unité.

**Acceptance Scenarios**:

1. **Given** une copie régulière préexistante sans autorisation de remplacement, **When** l'installation de la ronde est lancée, **Then** elle échoue, conserve la copie à l'identique et ne crée aucune unité.
2. **Given** la même copie avec autorisation explicite, **When** toutes les conditions Git sont satisfaites, **Then** l'activation remplace atomiquement la copie par la version admise.

### Edge Cases

- Une tête détachée est « hors main » et doit être refusée même si son commit est déjà ancêtre de `origin/main`.
- Une branche `main` divergente ou en avance doit être refusée ; une branche `main` strictement en retard reste un rollback admis.
- Une version déjà matérialisée dont les octets ne correspondent plus au commit annoncé doit être considérée corrompue et ne jamais être écrasée silencieusement.
- Un lien existant vers un checkout ou un autre SHA exige une autorisation explicite de remplacement.
- Une interruption pendant la préparation ne doit jamais laisser l'entrée de commande viser un artefact partiel.
- Une release aux bons octets mais représentée par un lien, ou portant un mode
  différent, n'est pas la release immuable attendue et doit être refusée.
- Des feuilles régulières exactes ne suffisent pas si leur répertoire, ou l'un
  de ses parents, se résout canoniquement dans le dépôt source.
- Avec `--force`, un lien actif vers un répertoire doit être remplacé comme une
  entrée exacte ; la cible obtenue doit être attestée avant toute annonce.
- Un rejeu de ronde au même SHA mais avec une configuration différente n'est
  pas idempotent : sans `--force`, il doit échouer sans annoncer la ronde prête.

## Requirements

### Functional Requirements

- **FR-001**: Les installateurs de `bridget-idle` et `bridget-ronde` DOIVENT appliquer le même contrat d'admission.
- **FR-002**: L'installation DOIT refuser un worktree lié avant toute mutation.
- **FR-003**: L'installation DOIT refuser toute branche autre que `main`, y compris une tête détachée.
- **FR-004**: L'installation DOIT refuser un arbre de travail sale.
- **FR-005**: L'installation DOIT refuser une entrée préexistante non gérée sans autorisation explicite de remplacement.
- **FR-006**: L'installation DOIT vérifier que `HEAD` est ancêtre de la référence locale `origin/main` et refuser une tête locale non poussée ou divergente.
- **FR-007**: L'installation NE DOIT PAS effectuer de synchronisation réseau implicite ; une référence distante absente ou périmée doit produire une instruction explicite.
- **FR-008**: La version active DOIT contenir exactement les octets du fichier suivi au SHA admis, indépendamment des octets présents dans le checkout au moment de l'activation.
- **FR-009**: L'entrée de commande DOIT viser une version stockée hors de tout dépôt et worktree, identifiée par le SHA complet.
- **FR-010**: L'origine de chaque commande active DOIT être lisible sous forme de remote, référence d'admission, SHA complet, nom et empreinte de l'artefact, sans recopier une URL susceptible de contenir un secret.
- **FR-011**: L'activation DOIT être atomique et son rejeu au même SHA idempotent.
- **FR-012**: Une version matérialisée sous un SHA donné NE DOIT PAS être écrasée si ses octets diffèrent de l'objet Git correspondant.
- **FR-013**: En cas de refus, l'installateur de ronde NE DOIT créer, modifier ni activer aucune unité de service.
- **FR-014**: L'autorisation explicite de remplacement DOIT permettre la migration contrôlée d'une copie ou d'un ancien lien uniquement après validation de toutes les conditions Git.
- **FR-015**: Une release préexistante DOIT être un fichier régulier qui n'est
  pas un lien, aux octets et au mode exacts attendus ; la preuve d'origine
  adjacente obéit au même invariant de représentation, d'octets et de mode. Le
  chemin canonique de leur répertoire DOIT rester hors du dépôt source.
- **FR-016**: Après une activation, l'installateur DOIT relire l'entrée active
  et attester qu'elle est un lien dont la cible textuelle est exactement la
  release attendue avant d'annoncer le succès.
- **FR-017**: Une unité de ronde préexistante n'est idempotente que si son
  contenu et son mode sont exactement ceux demandés. Toute divergence sans
  `--force` DOIT produire un refus non nul sans annonce finale de succès.

### Non-Functional Requirements

- **NFR-001**: Le mécanisme ne doit ajouter aucune dépendance externe au socle Git, Bash et aux outils système déjà requis.
- **NFR-002**: Les chemins et messages d'origine doivent être exploitables sur macOS et Linux.
- **NFR-003**: Tous les refus doivent être déterministes et testables sans accès à la machine de production.

### Key Entities

- **Version admise** : commit porté par la branche locale `main` et ancêtre de `origin/main`.
- **Artefact de pilotage** : copie immuable d'un outil, matérialisée hors du dépôt et nommée par le SHA complet.
- **Entrée active** : commande stable dans le chemin utilisateur qui sélectionne atomiquement un artefact.
- **Preuve d'origine** : métadonnées lisibles reliant l'artefact à sa référence, son dépôt, son SHA et son empreinte.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Les cinq états interdits produisent 5 refus sur 5, sans modification de l'entrée active.
- **SC-002**: Les deux outils installés restent exécutables après suppression complète de leur dépôt source de test.
- **SC-003**: Un opérateur retrouve le SHA complet et `origin/main` en au plus deux lectures locales (`readlink` et lecture de la preuve d'origine).
- **SC-004**: Deux installations consécutives du même outil au même SHA produisent une cible et une empreinte strictement identiques.
- **SC-005**: Le scénario de copie régulière de la ronde sans autorisation produit un échec non nul, zéro unité créée et zéro octet modifié.
- **SC-006**: Les harnais existants des outils et la batterie projet ne présentent aucun rouge imputable à la session.
- **SC-007**: Remplacer une release par un lien de mêmes octets, ou modifier son
  mode de `0555` à `0755`, produit deux refus discriminants.
- **SC-008**: Avec `--force`, une entrée active liée à un répertoire devient le
  lien exact vers la release ; aucun lien n'est déposé dans l'ancien répertoire.
- **SC-009**: Un rejeu de ronde avec une configuration différente échoue, garde
  les unités initiales octet pour octet et n'imprime jamais `ronde portable prête`.
- **SC-010**: Si le répertoire SHA est remplacé par un lien vers des feuilles
  exactes sous `.git`, le rejeu refuse ; retirer cette garde puis déplacer le
  dépôt casse réellement la commande avec un code non nul.

## Reprise après verdict STOP — 2026-08-26

La contre-épreuve a établi trois invariants manquants, pas un besoin de
refactor : identité de la release dans le système de fichiers, remplacement de
l'entrée active exacte et égalité réelle des unités lors d'un rejeu. La base de
reprise est `7024df31de5b23bfeca27eb5588a5465a872a8b8`.

Ne sont pas mesurés par cette reprise : macOS et launchd réels, activation
systemd sans `--skip-activate`, course entre deux installateurs et falsification
volontaire des références Git.

## Assumptions

- `origin/main` est la référence locale qui matérialise l'admission après jury et merge ; les installateurs ne tentent pas de prouver le verdict séparément.
- Un rollback vers un ancêtre de `origin/main` est volontairement autorisé, car ce commit a déjà franchi la frontière d'admission.
- Le modèle protège contre les erreurs et contournements accidentels de workflow, pas contre un utilisateur propriétaire qui falsifierait volontairement ses références Git locales.
