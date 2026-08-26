# Feature Specification: Nommer le protocole réel sans perdre le canal

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 024-nommer-protocole
Titre: Nommer le protocole réel sans perdre le canal
Statut: Complete
Priorité: P1
Tâches: 11/11 (100%)
Tests: 20/20 (100%)

Résumé:
- Contexte: La projection `TRANSPORT` mélange protocole d'agent et canal réseau.
- Objectif: Afficher le protocole réellement utilisé et conserver séparément le canal de connexion.
- Exécution: Normaliser la présence à l'enregistrement, ajouter `channel`, puis afficher `TRANSPORT` et `CANAL`.
- Risque principal: Perdre l'information de fédération ou casser les wrappers déjà lancés.
- Mitigation: Champs fil additifs, lecture des anciennes trames et alias de configuration conservés.
- Validation: Matrice tmux local/distant, ACP, Codex natif, Claude natif et trame historique.
- Dépendances: SPEC-007, SPEC-014

Fichiers:
- spec.md: ✓ (specs/024-nommer-protocole/spec.md)
- tasks.md: ✓ (specs/024-nommer-protocole/tasks.md)
- plan.md: ✓ (specs/024-nommer-protocole/plan.md)
- implementation.md: ✓ (specs/024-nommer-protocole/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-024-nommer-protocole`
**Created**: 2026-08-25
**Status**: Complete
**Priority**: P1
**Dependencies**: SPEC-007, SPEC-014

---

## Contexte et problème

La flotte mesurée mélange dans `AgentInfo.transport` deux natures :
`codex_app_server`, `claude_stream_json` et `acp` nomment un protocole
d'agent, tandis que `unix` et `ssh-unix` nomment le chemin de connexion au
daemon. Le fichier `federation.env` écrit par `scripts/federate-ssh.sh`
remplace ainsi le protocole affiché des agents interactifs par le tunnel.

La distance ne détermine pas le protocole. Les agents Cartae mesurés sont
actuellement `mode=tmux` et reçoivent les enveloppes par `TmuxTransport` :
leur protocole actuel est donc `tmux`, pas `codex_app_server`. Le Claude
isolé affiché `unix` est lui aussi un agent interactif tmux stable.

## Principes directeurs

- **Deux faits, deux champs** : le protocole et le canal ne s'écrasent jamais.
- **Chemin réel, jamais type d'agent** : `codex` n'implique pas app-server.
- **Compatibilité progressive** : un wrapper historique reste lisible.
- **Inconnu honnête** : aucune valeur n'est inventée quand le canal n'est pas attesté.

## User Scenarios & Testing

### User Story 1 — Diagnostiquer le protocole actif (Priority: P1)

Un opérateur consulte `bridget who` et voit immédiatement quel protocole
porte chaque agent, indépendamment de sa machine.

**Independent Test**: enregistrer un tmux distant, un ACP et deux pilotes
natifs, puis vérifier la colonne `TRANSPORT`.

**Acceptance Scenarios**:

1. **Given** un Codex interactif fédéré, **When** `who` est rendu, **Then** `TRANSPORT=tmux` et `CANAL=ssh-unix`.
2. **Given** un Codex géré local, **When** `who` est rendu, **Then** `TRANSPORT=codex_app_server` et `CANAL=unix`.
3. **Given** un Cursor géré, **When** `who` est rendu, **Then** `TRANSPORT=acp` sans renommage générique.

### User Story 2 — Déployer sans perdre les anciens wrappers (Priority: P1)

Un daemon mis à jour accepte les trames d'enregistrement antérieures et
conserve l'indication de fédération déjà annoncée.

**Independent Test**: décoder et enregistrer une trame historique
`mode=tmux, transport=ssh-unix` sans champ `channel`.

**Acceptance Scenarios**:

1. **Given** une ancienne trame tmux distante, **When** elle est enregistrée, **Then** le protocole devient `tmux` et le canal reste `ssh-unix`.
2. **Given** un ancien client qui lit `AgentInfo`, **When** le daemon ajoute `channel`, **Then** les champs historiques restent inchangés.
3. **Given** une présence UI locale, fédérée ou sans attestation, **When** elle s'enregistre, **Then** son canal vaut respectivement `unix`, `ssh-unix` ou reste absent dans la trame et dans `AgentInfo`.
4. **Given** un canal déjà attesté, **When** un client moderne annonce explicitement l'inconnu, **Then** le canal est effacé ; une trame historique qui omet réellement le champ conserve en revanche la dernière attestation.

## Requirements

### Functional Requirements

- **FR-2401**: `AgentInfo.transport` DOIT toujours nommer le protocole d'agent réellement utilisé.
- **FR-2402**: `AgentInfo.channel` DOIT porter séparément le canal de connexion attesté, ou rester absent.
- **FR-2403**: Un enregistrement `PresenceMode::Tmux` DOIT produire le protocole `tmux`, quel que soit `unix` ou `ssh-unix`.
- **FR-2404**: Pour un agent géré, la définition figée DOIT rester l'autorité du protocole.
- **FR-2405**: `acp` DOIT rester le nom du protocole Cursor/Gemini réellement parlé.
- **FR-2406**: `who` DOIT afficher deux colonnes distinctes `TRANSPORT` et `CANAL`.
- **FR-2407**: Une trame historique sans `channel` DOIT rester décodable et conserver toute information déjà attestée.
- **FR-2408**: `BRIDGET_CHANNEL` et `channel=` DOIVENT devenir les noms préférés dans leur source ; `BRIDGET_TRANSPORT` et `transport=` restent des alias de compatibilité.
- **FR-2409**: Le protocole NE DOIT PAS être déduit du seul `agent_type`.
- **FR-2410**: Tout `Register` productif DOIT publier un canal attesté ou aucun canal ; le type local de la socket NE DOIT PAS devenir une preuve réseau.
- **FR-2411**: Deux sources d'attestation divergentes DOIVENT produire un canal inconnu, jamais une priorité accidentelle.
- **FR-2412**: Le fil DOIT distinguer une omission historique d'un inconnu explicitement mesuré ; le premier peut conserver une attestation antérieure, le second DOIT l'effacer.

### Non-Functional Requirements

- **NFR-2401**: Aucun nouveau format obligatoire ni nouvelle dépendance.
- **NFR-2402**: Le changement de schéma filaire reste additif et tolère un déploiement progressif.

## Success Criteria

### Measurable Outcomes

- **SC-2401**: La matrice de six cas prévue passe sans valeur réseau dans `TRANSPORT`.
- **SC-2402**: Le cas historique `ssh-unix` conserve cette valeur dans `CANAL`.
- **SC-2403**: Les sorties CLI et JSON distinguent protocole et canal sans supprimer de champ historique.
- **SC-2404**: Les suites workspace conservent uniquement leurs rouges de référence hors lot.
