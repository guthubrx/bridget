# Journal d'implémentation - Session150 socle de la délégation native

## Métadonnées

- **Spec** : 150-socle-delegation
- **Branche** : session-150-socle-delegation (worktree `.worktrees/150-socle-delegation`)
- **Démarré** : 2026-10-11
- **Terminé** : 2026-10-11 (code sur disque, non commité, aucun service relancé)

## Tâches

| Tâches | Contenu | Tests |
|---|---|---|
| T001 | Référence 126/0/1 | `validation/baseline.md` |
| T002-T005 | `reply_timeout` protocole, champs de tâche, `AutonomousEffect::NativeDelegation`, lecture de phase de remise | unitaires verts |
| T006-T011 | Purge par arbre, pierres tombales, vivacité + décommission sous verrou, recopie des accusés, entretien | 22 tests `native150` |
| T012-T014 | Admission par comptages, purge de borne, `retention_saturated`, ouverture sans plafond | rouge vu (`task_limit`), puis vert |
| T015 | Refus `referent_paused` après rejeu | rouge vu, puis vert |
| T016-T018 | Délai par tâche, bornes, contrôle parent explicite, catalogue, schéma MCP | verts |
| T019 | Non-régression 148/149/087 : 126/0/1 inchangé | `validation/tests-150.md` |
| T020 | Doc, skill, commandes | relu |
| T021-T022 | ADR 150 et feuille de route | - |
| T023 | Suite complète : 1969/8/72 ; 8 échecs préexistants prouvés sur la base | `validation/tests-150.md` |
| T024 | Ce journal, diff enregistré | `validation/diff-150.patch` |
| T025-T035 | Convergence 1 : amendement FR-009, pause avant droits, annulation humaine en pause, refus `control_state_unavailable` documenté, recopie des accusés atomique, décommission par lot, `tick` après admission seulement, purge de borne à l'entretien, suivi aligné sur l'admission | tests `native150` |
| T036 | Quota de 128 arbres partagé entre purge d'âge et purge de borne dans `maintain_retention` | rouge vu, puis vert |
| T037 | `control_state_unavailable` au rang de la pause (contrat, doc) ; commentaire de `purge_retained` corrigé | relu |
| T038 | Vivacité : agent routé et décommission en cours conservent l'arbre | 2 tests, verts |
| T039 | Réalignement du suivi à chaque tick (`align_request_deadline`) après un échec d'écriture | rouge vu, puis vert |

Total `native150` : 50 tests daemon (`cargo test -p bridget-daemon native150`) + 1 transport.

## Écarts au plan, acceptés

1. `purge_trees(now, bound: Option<usize>)` : borne paramétrable (tests à
   petite échelle).
2. Vivacité étendue à `decommissioning_names`.
3. Une pierre tombale expirée reste refusée jusqu'au prochain entretien (≤ 1 h).
4. Purge de borne à l'admission sans limite de 128 arbres ; arrêt dès que le
   total repasse sous 4096 ; accusés recopiés juste avant.
5. `control_state_unavailable` : refus fermé si l'état de pause est illisible.
6. Bornes de `reply_timeout` jugées par le daemon seul ; le MCP vérifie la forme.
7. `COUNT(*)` du total en O(n), n ≤ 4096 ; actives par l'index d'état.

## Relecture (Articles XIX et XX)

- Aucun module, aucune dépendance. Deux gardes de plafond retirées, le
  chargement complet de l'admission remplacé par des comptages.
- Limite connue : si la décommission échoue au milieu d'un arbre, les enfants
  déjà décommissionnés le restent alors que l'arbre est conservé. Sans danger :
  ils étaient déjà arrêtés.
- Une tâche terminée avant le déploiement dont la remise a déjà été effacée
  n'aura jamais de preuve d'accusé : son arbre reste conservé (choix de la spec).

## Non vérifié

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/implementation.md#extrait-1).

## Convergence

| Passage | Résultat |
|---|---|
| 1 | 4 manques partiels → T025-T030 ; contre-revue de l'implémentation → T031-T035 |
| 2 | 1 manque (FR-009, introduit par T034) + écarts mineurs → T036-T039 |
| 3 | CONVERGED (relecture de `maintain_retention` et `align_request_deadline` ; coût d'alignement borné à 128 lectures indexées par tick) |

## Audit v14 (mode fix, 2026-10-11)

Session : `audits/2026-10-11/session-2026-10-11-spec-150-01/`. Aucun commit (consigne).

| Finding | Correction | Tests |
|---|---|---|
| PERF-001 | `purge_retained` : présences et lancements gérés indexés une fois (`HashSet`), plus de recherche linéaire par enfant | native150 51/0 |
| TEST-001 | test `native150_pause_etat_illisible_refus_ferme` (`control_state_unavailable`) | vert |
| QUAL-001/002 | commentaires de complexité (`purge_expired_tombstones`) et du refus sans `tick` corrigés | - |
| QUAL-003/004 | doc « Rétention des tâches » reformulée ; ADR150 : admission en comptages O(n), pas O(log n) | relu |
| QUAL-005 | fiche synthèse de `spec.md` à jour | - |

Non-régression rejouée : native148 27 (1 ignoré), native149 78, delegation149 10,
spec_087 11 : 126/0/1, identique. Le témoin `managed_parity_test::TEMOIN_abandon_apres_N_tentatives_est_nomme`,
rouge dans la suite complète `/tmp/s150-full3.log` (1981/9/72), passe seul : sensible à la charge.
