# ADR038 — Historique partagé et sollicitations ciblées

Date : 2026-09-16. Statut : Accepté — implémenté le 2026-09-17 dans la session 102 (worktree, non fusionné, non déployé).
Session : 102-fils-inter-agents.

## Contexte

Les échanges directs Bridget ne fournissent pas un historique commun à plusieurs.
Diffuser chaque contribution à tous les participants ferait exécuter des tours
et lire du contexte inutilement. Les observations100/101 restent best effort et
ne constituent pas une file durable de messages collectifs.

## Décision

Créer un fil persistant dans la SQLite existante, avec membres fixes et entrées
ordonnées. Les cibles sont structurées et explicites ; sans cible, aucun réveil.
Les alertes ne portent pas l'historique et utilisent le transport dédupliqué099.
La lecture s'effectue par pages avec reçu puis confirmation, distincte de l'ACK
d'injection fournisseur. L'intention de notification est commise avec le dépôt.
Les adaptateurs distinguent ce type d'alerte du DM à réponse automatique.
Synthèse uniquement sur demande par un agent déjà présent.

## Conséquences

Positives : membres non sollicités silencieux, historique commun, reprises et
incertitudes vérifiables, pas de routeur LLM ni service supplémentaire.

Négatives assumées : état spécialisé de reçus/sollicitations ; appel de lecture
puis confirmation ; possible relecture d'une page non confirmée ; anciens clients
ne recevant pas d'alertes tant qu'ils n'exposent pas la capacité. Pas de promesse
de cache gratuit ni d'exactement-une-lecture.

Limites : membres fixes, autorité Bridget unique, pas de purge/rétention nouvelle,
pas de résumé automatique, pas de quota de tours métier ni contrôle de consensus.
Le plan, le modèle et les contrats de la session102 détaillent les invariants.

## Mise en œuvre (2026-09-17)

Réalisée telle que décrite, avec un ajustement : la capacité de réception des
alertes est annoncée par un fait de connexion `ThreadNoticeCapability` envoyé
après l'enregistrement, plutôt que par des champs ajoutés à `Register` et
`Registered` (même sémantique, ~105 constructions littérales évitées). Les
alertes empruntent la remise idempotente 099 avec une clé
`thread-wake:<fil>:<membre>:<génération>`, une échéance d'injection de 120 s et
une exclusion de la réaffectation entre instances. Preuves : spécification 102,
`implementation.md` et `analysis.md` (Converge).
