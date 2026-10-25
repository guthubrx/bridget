# Preuves natives réseau et récupération 149 - r3 (binaire release r8)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun redémarrage de service réel, aucun modèle, aucun Cargo, aucun code de production édité.
Archive r2 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md`, `interop149-r2.md`, `recovery149-r2.md`, résultats `native-network-recipes/results-r2/` (index `results-r2/INDEX.md`).

## Verdict global : PARTIAL

| Tâche | Verdict | Rapport |
|---|---|---|
| T036 interop réseau | **APPROVE** : 65/65 PASS, deux exécutions | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| T039 scénarios dégradés | **PARTIAL** : 43/43 PASS, mais deux écarts nommés ci-dessous (O4, E2) | `.../validation/recovery149.md` |
| T040 / SC001 UI web réelle native | **APPROVE** sur le périmètre exécuté (daemon réel r8, enfants fermés) | `.../validation/ui-native-recipe149-sonnet-r1.md` |

## Binaire, sources, dérive

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r3.md#extrait-1).

## Compteurs précis (dernière exécution de chaque recette)

| Recette | PASS | FAIL | RÉEL | SIMULÉ | OBS |
|---|---|---|---|---|---|
| `interop149.ts` | 43 | 0 | 43 | 0 | 0 |
| `server149.ts` | 15 | 0 | 15 | 0 | 0 |
| `unified149.ts` | 7 | 0 | 7 | 0 | 0 |
| `recovery149.ts` | 43 | 0 | 41 | 0 | 2 |
| `o4-149.ts` | 8 | **1** (O4.6) | 9 | 0 | 0 |
| `e2e3-149.ts` | 8 | **2** (E3.c, E2.b) | 6 (5 PASS, 1 FAIL) | 3 (2 PASS, 1 FAIL) | 1 |
| `exec-lag149.ts` | 4 | 0 | 1 | 0 | 3 |
| `follow149.ts` | 7 | 0 | 7 | 0 | 0 |
| `outage149.ts` | 5 | 0 | 4 | 0 | 1 |
| **Total** | **140** | **3** | **133** | **3** | **7** |

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r3.md#extrait-2).

## Ce qui est prouvé (RÉEL, binaire r8)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r3.md#extrait-3).

## Écarts à trancher par le principal (cause minimale, pas de correctif proposé)

1. **BLOCK-O4** (RÉEL, O4.6 FAIL). Si le daemon est relancé aussitôt après son SIGTERM, la réconciliation du nouveau daemon envoie SIGKILL au groupe du wrapper après 0,5 s (`daemon.rs:4549`, `managed_process.rs:1221-1254`). Le fournisseur natif dirige un autre groupe : il reste orphelin jusqu'à la fin naturelle de son tour (18,3 s mesurées, aucun SIGTERM reçu). Un fournisseur qui répond à `turn/interrupt` est arrêté en environ 1,0 s ; un bloquant en 4,05 s si rien ne le tue avant. Fixture `/Users/moi/.cache/bridget149-native-interop.GyH22l`.
2. **BLOCK-E2** (SIMULÉ, E2.b FAIL). Tâche `queued` dont le propriétaire est un enfant natif mort, avec une racine déjà `waiting_for_children` : `tick` saute la tâche, la racine attend sans fin, seul `cancel` la libère et le résultat retenu est perdu. Niveau : état durable rejoué (transaction exacte de l'admission) ; non atteint par SIGTERM. La variante réaliste E2c ne bloque pas. Fixture `/Users/moi/.cache/bridget149-native-interop.3znVhG`.
3. **F3 hygiène** (RÉEL, E3.c FAIL, faible). Après une coupure pendant l'annulation d'un descendant, son exécution durable reste active (`starting`) alors que la tâche est `cancelled`. La racine se termine quand même (E3.b PASS). Hypothèse E3 de blocage : réfutée sur ce chemin.
4. Écart non expliqué : une fois (première passe de `recovery149.ts`), l'exécution d'une racine au résultat capturé est restée `running` sur deux redémarrages. Non reproduit en 12 + 6 essais contrôlés (décalage 0 ms). Données dans `results-r3/recovery149-premiere-passe.json`.

## Observations

- La remise non accusée est rejouée une fois après redémarrage avec le même identifiant de message (livraison au moins une fois).
- Le suivi de journal d'un enfant arrêté finit par `journal_unavailable` ; l'UI écrit « Bridget indisponible ».
- L'arrêt d'un enfant depuis l'UI envoie au parent une notice « Échec de livraison de la demande ... ».
- Fournisseur qui ignore SIGTERM : le wrapper attend la fin de son tour, pas d'escalade (limite connue, E4).

## Non prouvé

Arrêt brutal (SIGKILL/OOM), launchd, vrai Codex, modèle réel (T037), voie standalone Claude/GLM et observer PTY (T038), `can_use_tool`, coque desktop, mobile, iOS, nouveau `wrapper.rs`. Les compteurs Cargo r8 (daemon 1492, transport 333, 1825 agrégés) viennent du reçu ; je n'ai lancé aucun Cargo.

## Fichiers et nettoyage

- Recettes, outils et résultats : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (`README.md`, `results-r3/*.json`, `results-r3/*.stdout.txt`).
- Captures UI : `.../validation/ui149-native/` (9 images).
- Fixtures conservées pour preuve (preuves de credentials supprimées) : `/Users/moi/.cache/bridget149-native-interop.yXDJxF` (T039), `.GyH22l` (O4), `.3znVhG` (E2/E3), `.w9UAwP` (UI). Les autres fixtures et les dossiers de projet sous `/tmp/b149n.*` sont supprimés.
- Aucun processus résiduel (vérifié). Un listener Vite orphelin (PID 48442, mon port 15778) a été arrêté seul par SIGTERM vérifié. Onglet preview fermé. Aucun job différé.
