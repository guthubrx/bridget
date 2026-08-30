# Rapport Analyze: SPEC-066

**Date**: 2026-08-30
**Passage**: 6 après relecture indépendante et intégration de SPEC-068 sur main
**Verdict**: PASS documentaire après corrections RC8; aucune implémentation
autorisée avant preuve de SPEC-065 et nouveau worktree propre au minimum basé
sur `d589b24`

## Findings traités

| ID | Catégorie | Sévérité initiale | Correction appliquée |
|---|---|---:|---|
| S1 | Sécurité | CRITICAL | Le montage du cache Bridget complet est remplacé par un runtime ingress dédié. |
| A1 | Ambiguïté | HIGH | La promesse réseau est bornée: bridge sans ports, egress non filtré en v1. |
| C1 | Couverture | HIGH | Redémarrage Bridget avec socket recréée et agents vivants ajouté à FR-021/T017. |
| I1 | Incohérence | HIGH | Le fallback host est explicitement interdit en panne Docker. |
| M1 | Minimalisme | MEDIUM | SDK Docker, Compose, K8s et secrets sont retirés du scope. |
| C2 | Couverture | MEDIUM | Worktrees externes liés au common dir et layouts refusés sont couverts. |
| S2 | Sécurité | MEDIUM | Modèle de confiance intra-projet explicité comme coopératif. |
| RC7-C2 | Sécurité | CRITICAL | Endpoint Unix privé par projet/génération, handshake attesté et oracles trans-projet ajoutés. |
| RC7-H1 | Concurrence | HIGH | Réservation transactionnelle et environment_epoch ferment la course spawn/lifecycle. |
| RC7-R2-L1 | Cycle de vie | LOW | Rebind place recreate_required, bloque les nouvelles admissions et laisse finir les exécutions existantes. |
| RC8-R1 | ABI runtime | HIGH | UID/GID, HOME/XDG, state root et socket explicite sont figés et testés sans dépendance au HOME hôte. |
| RC8-R2 | Autorité | HIGH | La politique runtime possède un document hôte fermé, une source explicite, des permissions et un fail-closed. |
| RC8-I1 | Reproductibilité | MEDIUM | Les images distinguent digest de registre et identifiant local issu de `--iidfile`. |
| RC8-I2 | Cohérence aval | MEDIUM | Tout changement backend/politique/image/UID/GID produit `runtime_policy_changed` et invalide les profils 067. |
| IR-F02 | Traçabilité | LOW | Le reuse-audit utilise des chemins et symboles stables, rejoués contre `main` à `d589b24`. |
| IR-F03 | Processus SpecKit | LOW | Décision utilisateur: l'absence de `.specify` est non bloquante et ne déclenche ni installation ni mise à jour. |
| IR-F04 | Parité SPEC-068 | HIGH | Le backend Docker réutilise sans duplication la persistance, le rejeu, l'ordre et l'acquittement des incidents runtime délégués avec leur `ProjectReference`. |

## Couverture

| Inventaire | Total | Couverts par tâches |
|---|---:|---:|
| Exigences fonctionnelles | 42 | 42 |
| Exigences non fonctionnelles | 8 | 8 |
| Critères nécessitant du travail | 13 | 13 |
| User stories | 3 | 3 |
| Tâches | 36 | 36 mappées |

Couverture: 100 %.

## Alignement constitutionnel

- Aucun conflit MUST persistant.
- L'infrastructure est déclarée par image digest et politique versionnée.
- Le backend reste opt-in et réversible.
- Potentiel minimalisme: environ un SDK Docker, un Compose et un proxy réseau
  évités; aucun fichier supplémentaire à supprimer dans le plan courant.

## Risques non bloquants assumés

- Docker partage le noyau hôte.
- Les agents d'un projet partagent la confiance et les montages.
- L'egress n'est pas filtré par destination.
- Les credentials réels sont volontairement absents jusqu'à SPEC-067.

## Métriques Analyze

- Ambiguïtés restantes: 0
- Duplications restantes: 0
- Findings CRITICAL: 0
- Findings HIGH: 0
- Findings MEDIUM: 0
- Tâches non mappées: 0

La revue RC8 a fermé les ambiguïtés que RC7 ne pouvait pas voir sans confronter
les chemins HOME/socket du wrapper et l'UID 1002 de la cible. Le verdict reste
documentaire: l'ABI et la parité SPEC-068 doivent encore être prouvées sur
fixtures après SPEC-065. Le démarrage exige aussi un worktree propre au minimum
basé sur `d589b24`. L'absence de `.specify` ne déclenche ni installation ni
mise à jour.


## Passage 7 - Reprise pipeline sur la tête courante

| ID | Catégorie | Sévérité | Vérification | Correction |
|---|---|---:|---|---|
| P7-01 | Concurrence | HIGH | SPEC-075 modifie encore les mêmes entrées de lifecycle, daemon, fleet, protocol et UI que 066. | SPEC-075 devient dépendance explicite de la spec, du plan, des tâches et de la checklist. |
| P7-02 | Fraîcheur d'audit | MEDIUM | Le dernier audit portait sur d589b24; main est maintenant aba60f0. | Le reuse-audit conserve son historique et ajoute une revalidation factuelle, avec rejeu complet obligatoire après intégration 075. |
| P7-03 | Processus | LOW | La formule présent run ne reflète plus le lancement explicite de ce pipeline. | Le plan bloque désormais sur les gates objectifs, non sur la session documentaire passée. |

## Passage 8 - Analyse avant implémentation sur la tête 74234641

| ID | Catégorie | Sévérité | Vérification | Correction |
|---|---|---:|---|---|
| P8-01 | Traçabilité | LOW | Les artefacts pointaient vers une branche et une tête anciennes. | Branche, gate et journal alignés sur `session-066-environnement-partage-projet` et `74234641`. |
| P8-02 | Couverture | MEDIUM | FR-033, FR-036 et le lancement Docker impliquent aussi protocole, config daemon/CLI et superviseur daemon. | T012, T021 et T023 nomment explicitement ces surfaces. |
| P8-03 | Isolation | LOW | Le worktree principal contient un document non suivi hors SPEC-066. | Le worktree 066 provient du commit propre `74234641`; le document est préservé hors branche. |

Résultat : aucun finding CRITICAL ou HIGH. Les 42 exigences fonctionnelles, les
8 exigences non fonctionnelles, les 13 critères de succès et les 3 user stories
restent couverts par les 36 tâches. Les corrections sont strictement
documentaires et ne modifient pas la portée fonctionnelle.

Verdict : **PASS pour l'implémentation**. Le cycle de tests part de T001 et
T002, sans activation de projet réel ni livraison production.

Résultat du second passage : aucune exigence sans tâche, aucun doublon
évident, aucun finding CRITICAL documentaire. L'implémentation reste
**BLOCKED** par l'état externe de SPEC-075 et de main, non par une ambiguïté
de SPEC-066.

## Passage 9 - Analyse pendant implémentation

| ID | Catégorie | Sévérité | Constat | Conséquence |
|---|---|---:|---|---|
| P9-01 | Contrat d'exécution | HIGH | `ProjectRuntimePolicy` fixe image/UID/limites mais ne définit pas la commande interne autorisée pour Bridget et les providers. Les définitions actuelles désignent des chemins de l'hôte. | T023 à T032 ne peuvent pas être réalisés sans inventer une autorité ou contourner la politique fermée. |

Le runtime US1 est prouvé et l'ingress est fail-closed, mais l'analyse ne peut
plus conserver un verdict d'implémentation complet. Une décision de produit est
requise: ajouter un manifeste fermé d'exécutables internes à SPEC-066, ou
porter cette autorité par le catalogue de ressources prévu dans SPEC-067 puis
faire dépendre explicitement T023. Aucune commande host n'est transmise à
`docker exec` dans l'intervalle.

## Passage 10 - autorité execution Docker (2026-08-30)

P9-01 est traité et ne bloque plus les travaux prouvables sans docker exec.
La politique v1 porte maintenant un lanceur Bridget interne optionnel et un
mapping fermé type agent vers commande interne. Ces champs sont validés,
inclus dans le digest, et une absence de résolution produit la raison
runtime_executable_unavailable. Le daemon refuse explicitement tout spawn,
relaunch ou recovery du backend Docker tant que docker exec reste non raccordé.
Aucun chemin de définition hôte ne traverse cette frontière.

L ingress compare désormais un handshake à une réservation à usage unique,
avec UID kernel, conteneur, génération et epoch, avant Register. Le finding
restant est de réalisation: T017, T019, T021 et T023 à T032 restent ouverts
car seul docker exec peut créer puis revérifier cette réservation en condition
réelle et rattacher les agents au cycle de vie partagé.

## Passage 11 - convergence manuelle intermédiaire (2026-08-30)

Confrontation de la spec, des tâches et du code produit: FR-006 et FR-042 ont
une preuve dans le refus Docker structuré et la policy fermée. FR-020 et FR-038
ont une preuve partielle dans ingress privé, socket explicite, UID kernel et test
fixture. Les montages de racine et worktrees sont prouvés par le test Git dédié.
Aucun finding CRITICAL nouveau.

Il reste 13 tâches ouvertes: T017, T019, T021, T023 à T029, T031, T032 et T035.
Les écarts principaux sont docker exec, création de réservation avant exec,
corrélation flotte, reconciliation et rollback avec agents. Le statut reste In
Progress. Archive SpecKit non créée: aucun dossier .specify n est présent et
la consigne utilisateur interdit sa mise à jour.

## Passage 12 - limite de réalisation Docker (2026-08-30)

Observation vérifiée contre infra/project-runtime/Dockerfile et le superviseur
managed: la fixture ne contient que les répertoires ABI, sans lanceur Bridget ni
provider. Le superviseur host repose sur les descripteurs RELEASE et STATUS du
bootstrap local, que docker exec ne transporte pas. T023 ne peut donc pas être
marquée terminée en lançant artificiellement un processus host ou une commande
fixture. La suite requiert une supervision Docker dédiée, puis les preuves T017,
T019 et T023 à T032. Aucun fallback n a été ajouté.

## Passage 13 - convergence finale (2026-08-30)

Les constats intermédiaires des passages 9 à 12 sont clos par le runtime exec attesté, sa réservation et son ingress privé. Les tests Docker réels prouvent l'exécution partagée, l'isolation inter-projets, les montages, la relance et le rollback.

| Axe | Verdict | Preuve |
|---|---|---|
| Contrats et persistance | PASS | migrations additives, reason runtime durable, tests store et transport |
| Cycle de vie | PASS | stop/remove/switch refusent les agents actifs et invalident les réservations |
| Incidents et reprise | PASS | tests SPEC-068, ingress après restart, raison fermée OOM/PID/exec/restart |
| Validation | PASS | fmt, Clippy workspace et tests workspace verts |

Findings CRITICAL : 0. Findings HIGH : 0. Tâches ouvertes : 0.
Verdict final : **PASS - SPEC-066 terminée**. Aucune activation réelle n'est requise pour cette conclusion.
