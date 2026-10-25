# Décision156 — guidage, interruption et lecture de file

Date :2026-10-12. Statut : **Accepté**.

Cette décision fixe le contrat156. Elle ne prouve pas que l’implémentation,
les tests ou le runtime sont terminés.

## Contexte et cadre de menace

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/156-guidage-et-lecture-de-file.md#extrait-1).

Le modèle couvre les agents utilisant MCP, CLI, pont et wrapper. Un processus
hostile du même compte Unix qui lit les preuves privées, forge une connexion
Attach brute ou pilote son propre pseudo-terminal reste hors modèle ADR011/027.
La sélection du rôle est déclarative ; ce contrat ne promet pas une isolation
entre processus de ce compte. Les contrôles de trames brutes sont une défense
supplémentaire. T002 établit la provenance et les limites par lectures ciblées155.

## Décision

Ajouter intent queue/steer/interrupt à bridget_send et --intent à bridget send.
Absent ou queue garde les octets ordinaires. Poser l’intention avant le canon,
sans la réécrire selon la cible. Refuser auto et steer+reply=true avant connexion.
Le reçu contrôle indique l’intention demandée et effect=unconfirmed. La FIFO
reste : le prochain tour peut porter un message antérieur. Un avis différé de
refus n’est pas garanti pour steer sans demande de réponse. Le rejeu conserve
l’admission mémorisée ; autre intention sous la clé donne envelope_mismatch.

La garde commune couvre Send historique, SendIdempotent, ControlExecution et
l’interruption active par cancel. Seuls Attach attesté et parent prouvé contrôlent.
Un client nu from=human ne reçoit aucune autorité. L’identité vient de la connexion.
L’origine UI ordinaire exige credential, instance primaire vivante et route ;
le rôle Client reste. La provenance système vient du producteur interne.
ChildProof interne ferme contrôle et QueueList aux frontières CLI/MCP avant
connexion. La preuve privée d’agent continue de suivre ses accès auxiliaires.

Agent pendant pause : refus. Interrupt agent vers mission native active : refus ;
la reprise152 concerne pause/panne, pas une interruption utilisateur terminale.
Rythme agent :1 interruption/30 s par couple,3/600 s par cible. Les timestamps
LAST du couple et THIRD_LATEST de la cible donnent retry_after exact ; leurs
échéances <=now autorisent la reprise. Un refus ne consomme pas le rythme.
Pour SendIdempotent, l’admission guidance_events appartient à la transaction
existante de remise/suivi/lien/journal. Sur historique/control/cancel, l’audit
accepté est durable avant émission physique. Un INSERT échoué interdit le
contrôle. Un crash entre admission durable et émission peut consommer le rythme
sans effet physique : choix prudent, sans promesse exactement une fois physique.
Attach est exempt de pause/mission/rythme ; l’interruption interne de pause ne
consomme pas le rythme. Aucun steer agent non consommé ne replie en interrupt.

Le bundle149 prouve activeSteering pour Claude seul. Cursor steer est refusé ;
sa remise pendant busy attend même mid_turn_ok=true. Claude implicite exige
mid_turn_ok calculé par le daemon sur Deliver et DeliverIdempotent, envoi/reprise,
et un message déjà groupable de Attach/UI attestée/système interne/parent/enfant.
Les pairs, clients nus et valeurs absentes attendent. REST/V1 ferme les
intentions explicites avant émission ; None reste. Le trajet installé149 est V2.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/156-guidage-et-lecture-de-file.md#extrait-2).

Cancel d’une demande retire sa remise en attente. Il atteint le tour actif
seulement après la garde commune ; sinon le tour continue. Un ancien wrapper
garde l’ancien comportement jusqu’à sa relance.

## Alternatives rejetées et conséquences

- Un pair du même projet qui contrôle une cible : refusé ; le lien parent reste requis.
- Mode auto ou repli silencieux steer vers interrupt : refusés ; aucune promesse d’effet.
- Croire from/origin ou tester seulement un terminal CLI : insuffisant pour la provenance.
- Éditer la file avec cancel/reorder/promote : reporté. Une session dédiée devra
  réserver durablement avant émission, enregistrer l’accusé durable, réconcilier
  avant relivraison et prévoir une phase d’annulation ainsi qu’une saga de promotion.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/156-guidage-et-lecture-de-file.md#extrait-3).

## Réutilisation du montage T013

Le fichier guidance156_e2e.rs prévu reprend une copie légère du Fixture et du
compilateur de fournisseur152 de native_delegation_e2e.rs. Les trois scénarios
prévus le justifient : Codex actif, Claude actif, Codex inactif. Leurs extensions
d’oracles couvrent les délais et la FIFO. Le montage reste dans ce seul fichier
de test prévu ; aucune API de production, couche commune, nouveau framework
ou scénario de confort. Ces trois usages sont prévus, pas exécutés.

## Références

- Contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file/contracts/guidance156.md
- Preuves T002 : /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/research-T002.md
- ADR011 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/docs/decisions/011-approbation-exposee-puis-second-facteur.md
- ADR027 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md
