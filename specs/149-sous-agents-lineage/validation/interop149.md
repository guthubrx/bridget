# Interop réseau natif 149 - T036, O1, O5/G8, panne adaptateur de bureau - ronde r3 (binaire release r8)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle, aucun Cargo.
Rapport r2 archivé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149-r2.md` (binaire debug r5, 65/65). Rapport r1 : `.../interop149-r1.md`.

## Verdict : APPROVE sur le périmètre exécuté (T036 rejoué : 65/65 PASS, deux fois)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-2).

## Binaire et sources

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-3).

## Couches : réel ou simulé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-5).

## Ports, fixtures, processus

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-6).

## Reprise de G-P-07(b) : O1 (R8.4 PASS)

Le contrat clarifié `permissions.md` (G-P-07, lignes 441-462) est vérifié sur le binaire r8 par `recovery149.ts` R8.4 :

| Voie après retrait du fait + rotation | Rejeu `bridget_delegate` | `bridget_task_status` | `bridget_who` |
|---|---|---|---|
| Montage MCP privé de l'ancien credential | `t3_session_unavailable` | `t3_session_unavailable` | `t3_session_unavailable` |
| Credential neuf qui n'a JAMAIS eu de fait (v1) | `ok` (même tâche, aucun lancement) | `ok` | `ok` |
| Lecture humaine Lineage native (CLI, gardes 147) | sans objet | résultat intact | sans objet |

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-7).

## O5/G8 : suivi `journal --follow` de la CLI réelle (7/7 PASS)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-8).

| Contrôle | Observé |
|---|---|
| F1 séquence | 16 événements, `seq` 1 à 16 strictement croissants, contigus, sans doublon ; `next_seq` ne régresse jamais ; aucune lacune ; 13 frontières `caught_up` ; 16 lignes arrivées sur 8,07 s (appends en DIRECT) |
| F2 égalité | les 16 `seq` suivis sont exactement ceux de la lecture ponctuelle |
| F3 reprise | `--after-seq 9` : 7 événements, `seq` 10 à 16, aucun doublon, aucun trou |
| F4 humains silencieux | le parent ne reçoit AUCUNE remise pendant les deux suivis (~20 s) ; tâche `working` ; 1 seul lancement |
| F5 fermeture | descripteurs du daemon 30 -> 40 pendant l'ouverture, retour à 30 ; threads 8 -> 12 -> 8 ; processus 2 -> 2 -> 2 (SIGTERM individuel des deux suivis) |
| F6/F7 fin de vie | après `cancel`, un suivi ouvert reçoit 16 événements contigus et se termine (exit 2, ligne `journal_unavailable`) ; en fin : 27 descripteurs, 7 threads, 0 processus |

Observation : à l'annulation de l'enfant, le suivi se termine par `{"status":"error","code":"journal_unavailable"}`. L'interface affiche alors « Bridget indisponible (journal_unavailable). Le contenu déjà lu est conservé. » alors que Bridget est disponible (vu dans la recette UI). Texte trompeur, pas de perte de données.

## Panne de adaptateur de bureau après admission (5 contrôles : 4 PASS RÉEL, 1 OBS)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-9).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149.md#extrait-10).

Résultat brut : `.../results-r3/outage149.json`.

## Limites et non prouvé

- Modèle réel (T037), voie standalone Claude/GLM et observer PTY (T038), `can_use_tool` (G-P-01), coque desktop : hors de cette ronde.
- Le nouveau `wrapper.rs` (19:19) : non couvert.
- Pas de launchd, pas de vrai Codex.
- Les résultats r2 sont archivés (`results-r2/`, index `INDEX.md`) pour que les références du rapport r2 restent lisibles.
