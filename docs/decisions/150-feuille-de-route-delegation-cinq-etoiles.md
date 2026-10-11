# Décision150b - feuille de route « cinq étoiles » communication et délégation

Date : 2026-10-11. Statut : accepté par l'utilisateur (« go », puis pipeline
`my.specify-all 150-158`).

## Contexte

Une comparaison avec T3 Code, limitée au métier de Bridget (communication et
délégation entre agents), donne Bridget devant sur l'annuaire, les fils
partagés, les agents hors outil et les garde-fous de droits. T3 est devant sur
l'attente d'un résultat, la reprise après panne, le guidage en cours de tour,
les réveils sur événement, la remontée des demandes d'un enfant et l'isolation
de son espace de travail. Bridget ne doit pas devenir un environnement de
développement : ni interface graphique, ni gestion des PR.

## Décision

| Session | Sujet | Critères visés |
|---|---|---|
| 150 | Socle : rétention, admission, refus pendant la pause, délai par tâche | prérequis |
| 151 | Attente synchrone, boîte de fin de tâche, résultat structuré | attente, livraison, suivi |
| 152 | Reprise après panne et après pause, nouvel essai borné, gel des délais | résilience |
| 153 | Enfants réutilisables pour plusieurs rondes, brief de contexte | contexte et rondes |
| 154 | Isolation optionnelle de l'enfant dans un worktree Git | isolation |
| 155 | Remontée des demandes de l'enfant au parent ou à l'humain | demandes de l'enfant |
| 156 | Guidage, interruption et file d'attente exposés aux agents | guidage, messages |
| 157 | Catalogue fiable (disponibilité, quotas), reprise sur quota, remplacement explicite | lancement, résilience |
| 158 | Réveils programmés, vue globale, budget par tâche, arrêt global, portées MCP | réveil, contrôle, sécurité |

Mécanismes repris de T3 : attente pilotée par événements, boîte de fin à états,
résultat structuré, reçus qui gardent les refus, annulation en cascade vérifiée
d'abord, modes file/guidage/interruption, reprise sur quota, catalogue qui dit
ce qui marche. Points où Bridget vise mieux que T3 : remontée des approbations
au parent, remplacement explicite de fournisseur, enfant réutilisable, budgets,
tout cela entre outils et entre machines.

## Conséquences

- La suspension des missions pendant une pause, prévue en 150, passe en 152 :
  elle partage le mécanisme de reprise d'une mission avec son identifiant
  d'origine et le gel des horloges (contre-revue 150, objections 1 à 4 et 7).
- Constat pour 157 : `bridget_capabilities` annonce `discovery: true` pour
  `codex-pro`, mais `bridget_delegate` refuse `permission_source_unavailable`
  depuis un fil T3 Claude (2026-10-11).
- Les sessions modifient les mêmes fichiers (`native_delegation.rs`) : exécution
  séquentielle dans un seul worktree, sans commit automatique, diff enregistré
  par session.

## Révision du 2026-10-11 (après deux rondes de contre-revue par session)

Les huit plans 151-158 ont chacun reçu deux rondes de contre-revue Codex.
Plusieurs périmètres ont été réduits pour rester sûrs et simples. Notes
visées réalistes :

| Session | Tâches | Critère | Note visée | Ce qui manque pour 5 |
|---|---:|---|:-:|---|
| 151 | 26 | Attente, livraison, suivi | 5 | - (doublon de résultat accepté) |
| 152 | 50 | Résilience (panne, pause) | 4 | 157b (reprise sur quota) |
| 153 | 29 | Contexte et rondes | 5 | - |
| 154 | 20-34 | Isolation | 3 à 5 | arbitrage utilisateur (option 1, 2 ou 3) |
| 155 | 41 | Demandes de l'enfant | 4 | approbation par le parent (question 2) |
| 156 | 31 | Guidage | 4 | édition de file (cancel/reorder/promote), session dédiée |
| 157 | 26 | Lancement | 5 | - |
| 158 | 22 | Contrôle humain / sécurité | 4 / 5 | budget de tours fiable, session dédiée |

Sessions futures nées de cette révision : 157b (reprise sur quota, après
152 livrée), édition de file (après 156), budget de tours (quand les tours
seront attestés pour Claude et Codex).

Correctifs de sécurité préexistants trouvés, à traiter hors feuille de route :
- repli `acceptForSession` en mode `deny` (`codex_app_server.rs:2209-2213`),
  tâche T000 de la spec 155, livrable seule ;
- interruption d'un agent par un autre sans contrôle de droits (`Send` /
  `SendIdempotent` acceptent `origin=Human` du client, `ControlExecution`),
  couvert par la 156.
