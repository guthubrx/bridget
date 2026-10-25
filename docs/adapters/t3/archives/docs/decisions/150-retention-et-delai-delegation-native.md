# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/150-retention-et-delai-delegation-native.md](../../../../../decisions/150-retention-et-delai-delegation-native.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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
