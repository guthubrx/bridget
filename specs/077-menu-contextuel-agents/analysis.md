# Analyse croisée - SPEC-077

Date: 2026-08-30
Passe 1: FAIL avec corrections non ambiguës
Passe 2: PASS

## Périmètre analysé

- spec.md
- plan.md
- research.md
- data-model.md
- contracts/agent-context-menu-v1.md
- reuse-audit.md
- tasks.md
- checklists/requirements.md

## Comptage

- 20 exigences fonctionnelles FR-7701 à FR-7720.
- 8 critères de succès SC-7701 à SC-7708.
- 19 tâches T001 à T019.
- checklist de spécification entièrement cochée.

## Constats de passe 1

### A001 - MAJEUR - Accessibilité des actions indisponibles

Le plan demandait que les commandes indisponibles restent visibles avec une
raison, mais T009 demandait de les ignorer dans la navigation clavier. Un
utilisateur clavier n'aurait donc pas pu atteindre leur explication.

Correction appliquée: tous les menuitems, y compris ceux portant
`aria-disabled=true`, restent parcourables et annoncent leur raison.
L'activation d'un item indisponible reste bloquée. Le focus initial va au
premier item disponible.

### A002 - MINEUR - Rendu non invalidé par une préférence locale

Le plan décrivait la projection de la barre sans exiger explicitement que la
signature de rendu inclue les préférences. L'optimisation existante aurait pu
considérer le rendu inchangé après épinglage ou masquage.

Correction appliquée: le plan exige que la signature de rendu couvre les
préférences normalisées.

### A003 - MINEUR - Unicité du chemin d'exécution

Le modèle d'actions unique était spécifié, mais l'invocation pouvait encore
être recâblée séparément dans chaque déclencheur.

Correction appliquée: le plan précise qu'un seul répartiteur d'actions reçoit
les commandes, indépendamment de la voie d'ouverture.

## Couverture

| Domaine | Exigences | Tâches |
|---|---|---|
| Menu unique et déclencheurs | FR-7701 à FR-7705, FR-7718, FR-7719 | T006 à T009 |
| Identité compacte | FR-7706 | T016, T017 |
| Organisation locale | FR-7707 à FR-7712 | T002 à T005, T010 à T013 |
| Cycle de vie | FR-7713 à FR-7715 | T014, T015 |
| Stabilité de rendu | FR-7716, FR-7717 | T011 à T013 |
| Frugalité | FR-7720 | T018, T019 |

## Passe 2

- Aucun NEEDS CLARIFICATION.
- Aucune exigence orpheline.
- Aucune tâche hors périmètre.
- Aucun endpoint ou paquet nouveau.
- Aucune duplication du menu ou du moteur lifecycle.
- Aucune contradiction critique ou majeure restante.

Verdict: PASS.
