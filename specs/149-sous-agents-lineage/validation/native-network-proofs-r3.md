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

- Binaire release r8 immuable : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`, SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`, égal au reçu `native149-release-receipt-r8.json` au début et à la fin de chaque recette (`native-network-recipes/results-r3/shacheck.log`) et à la fin de la ronde (19:51:47).
- Production Rust : empreinte `3943009ca82db59d...` (égale au reçu r8) au début. **À 19:19:03, `crates/bridget-daemon/src/wrapper.rs` a été modifié par un autre propriétaire** (digest `49f3a304d9504fe6`). L'empreinte devient `4fa56fcac7b37220f4d27cf15a8cdd94c8e434d92d86261b32a508a901f3adfa`. Aucun autre fichier. Mes recettes ne touchent aucun fichier de production. Toutes les preuves valent pour le binaire r8 seulement. Un correctif de wrapper (par exemple pour O4) demandera un binaire neuf et une nouvelle ronde : la preuve actuelle ne ferme pas ce code.
- Sources T3 : HEAD `33f6d04e11`, `git diff HEAD` `59a54267ce7c1d2a` avant et après (inchangé).
- Détail : `native-network-recipes/results-r3/fingerprints.json`.

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

Total de contrôles : 143 = 131 RÉEL PASS + 2 RÉEL FAIL + 2 SIMULÉ PASS + 1 SIMULÉ FAIL + 7 OBS (les OBS ne sont ni PASS ni FAIL au sens des trois colonnes ; ils sont comptés PASS dans les tableaux de recette parce qu'ils n'ont pas de critère). La recette UI (T040) est mesurée avec les outils de la preview T3 : voir son rapport, pas de compteur automatique.
Des échecs de passes antérieures (recovery passe 1 : 5, passe 2 : 2) étaient des défauts de mes prédicats, corrigés (détail dans `recovery149.md`).

## Ce qui est prouvé (RÉEL, binaire r8)

- T036 : sessions T3 scopées -> daemon réel, union v1/v2, rotation, révocation, identité étrangère, projet croisé, un seul serveur T3 qui émet le credential et sert Lineage.
- F1 (reprise sans doublon) : R9.2 a à f. 1 tour, 1 PID, lancements 12 -> 12, tâche `failed/unreachable` dès le démarrage, exécution durable `running` -> `unreachable/daemon_restart`, 1 notice, rejeu sans lancement.
- **F2 corrigé** (R9.4.a à i, sans aucune mutation de base) : parent hors ligne, la racine retenue sort de l'attente en 4,7 s et garde son résultat ; au retour du parent : UNE remise exacte ; 2e redémarrage : la remise accusée n'est pas rejouée ; exécutions avant/après cohérentes (16/16, aucune rouverte ni active).
- Annulation, retry, refus de fournisseur, parent extérieur, identité inconnue, droits réduits, panne de T3 indépendante (six scénarios US6) : PASS.
- O1/G-P-07(b) : le comportement est conforme au contrat clarifié (R8.4).
- O5/G8 : `journal --follow` de la CLI réelle (7/7). Panne T3 : zéro tour T3 (4 PASS).
- UI web native : journal réel en direct, nesting, modèles, statuts, arrêt qui termine le PID réel, reconnexion après redémarrage de T3, zéro champ de saisie.

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
