# Décision150 - rétention, admission, pause et délai de la délégation native

Date : 2026-10-11. Statut : accepté (plan validé après trois rondes de
contre-revue Codex, voir `specs/150-socle-delegation/adversarial-review-codex.md`).

## Contexte

La délégation native (148-149) garde toutes ses tâches. L'admission compte
l'historique cumulé : après 4096 délégations, elle refuse pour toujours
(`task_limit`). La pause du référent n'est pas consultée à l'admission. Le
délai de réponse vaut toujours 3600 s.

## Décision

1. **Unité de purge = arbre** : une tâche sans parent et ses descendants.
2. **Purge seulement sans perte** : toutes les tâches terminales et nettoyées ;
   chaque résultat ou avis d'échec remis et accusé, avec une preuve durable
   dans la tâche (`delivery_acked`) ; aucun sous-agent connecté, en cours de
   lancement ou désiré `Running`. Chaque sous-agent est décommissionné avant la
   suppression. Contrôles refaits sous verrou.
3. **Borne totale de 4096 tâches**, valeur déjà publiée aux lecteurs T3.
   Purge normale après 7 jours ; purge de borne, la plus ancienne d'abord ;
   sinon refus `retention_saturated`.
4. **Pierre tombale de 30 jours** par clé purgée : rejeu → `request_expired`.
   `bridget_delegate` n'a pas d'`issued_at`, donc l'expiration de l'ADR 007 ne
   s'applique pas.
5. **Purge à l'entretien seulement**, jamais à la création de l'état du daemon,
   car la réconciliation des anciens processus vient après.
6. **Nouvelle génération de projection** à chaque purge effective, puis signal
   aux propriétaires touchés : les lecteurs T3 refont un snapshot complet.
7. **Pause** : `AutonomousEffect::NativeDelegation`, refus `referent_paused` à
   l'admission d'une nouvelle clé, après le contrôle de rejeu.
8. **Délai** : `reply_timeout` optionnel, 60 à 86 400 s, défaut 3600. Un délai
   explicite d'enfant ne dépasse pas le temps restant du parent. Sans délai
   explicite, le comportement 149 reste intact.

## Conséquences

Positives : la délégation reste disponible sans limite de durée ; aucun
résultat non accusé n'est perdu ; l'admission ne lit et ne décode plus chaque
tâche : elle fait des comptages SQL (actives par l'index d'état, total en O(n)
avec n ≤ 4096) ; aucun contrat existant ne change de forme.

Négatives : une remise indéterminée bloque son arbre ; si ces arbres
remplissent la borne, la délégation est refusée. Les tables d'exécution,
d'idempotence et de flotte liées aux tâches purgées ne sont pas purgées (dette
notée). La suspension des missions en cours pendant une pause est transmise à
la session152.
