# Contrat d'observabilité du plan de contrôle

## Questions opérationnelles obligatoires

Le système doit répondre sans analyse du journal brut :

1. L'agent est-il connecté ?
2. Le fournisseur est-il vivant ?
3. Quel travail est en file ou en cours ?
4. Attend-il une autorisation ou une réponse humaine ?
5. Quand le dernier progrès a-t-il été observé ?
6. Le dernier message humain est-il seulement reçu, accepté ou visible ?
7. Quelle action est autorisée maintenant ?
8. Quelle version et quelles capacités sont actives ?

## Attributs de corrélation

Les journaux et traces peuvent porter les identifiants complets. Les métriques
utilisent seulement des dimensions bornées : fournisseur, version supportée,
état, raison fermée, origine et intention.

Interdits comme labels de métriques :

- contenu du message ;
- raisonnement ;
- identifiant de message libre ;
- chemin utilisateur arbitraire ;
- texte d'erreur non normalisé.

## Métriques minimales

- `work_submissions_total{origin,intent,outcome}`
- `work_queue_depth{priority_class}`
- `work_queue_oldest_age_seconds{priority_class}`
- `delivery_phase_duration_seconds{provider,phase,outcome}`
- `executions{provider,state}`
- `execution_last_progress_age_seconds{provider,state}`
- `steer_duration_seconds{provider,outcome}`
- `interrupt_duration_seconds{provider,outcome}`
- `approval_requests_total{provider,outcome}`
- `approval_loops_total{provider}`
- `provider_capability_fallback_total{provider,capability}`
- `agent_children{role}` avec rôle borné par catalogue
- `execution_usage_total{provider,unit}`

## Alertes minimales

- message humain au-delà de sa borne sans issue ;
- livraison dans une phase non terminale au-delà de sa borne ;
- tour actif sans progrès ;
- boucle d'autorisation ;
- file saturée ;
- fournisseur vivant mais contrat incompatible ;
- projection Maicie ou Bridget périmée au-delà du seuil déclaré.

## Confidentialité

- Le contenu intégral reste dans les sources déjà autorisées, pas dans les
  métriques.
- Les erreurs sont normalisées avant agrégation.
- Les exports de trace peuvent être expurgés selon la politique locale.
- Les identifiants opaques ne sont affichés qu'aux vues qui en ont besoin.
