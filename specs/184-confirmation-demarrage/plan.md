# Plan
1. Reproduire le rejet de Released après retrait de l'attente corrélée.
2. Porter l'attente de Release à trente secondes, comme Start ; ne modifier aucun permis.
3. Tolérer l'accusé tardif authentifié d'une commande connue. La réconciliation existante reste propriétaire de la progression.
4. Tester un accusé retardé de six secondes, les identités et les régressions pertinentes.
5. Faire relire indépendamment, intégrer, pousser et livrer via la maintenance native.
Complexité inchangée : lookup des attentes O(1), validation de la commande existante ; aucune file ou reprise nouvelle.

## Complément de guidage
- Utiliser seulement les observations déjà reçues ; aucun protocole ni critère d'acceptation supplémentaire.
- Faire porter recovery et human_action par les refus de reprise connus.
- Fournir task_id, child_agent_id, machine_id et execution_cwd seulement après can_own_task et vérification du parent.
- Classer les preuves manquantes sans transformer le refus en succès. Ne pas migrer une exécution ni recréer son dossier.
- Conserver l'incident initial dans detail.initial_start avec current_failure=false lorsque la progression le rend historique.
Complexité inchangée : O(1) pour la classification et le contexte, hors contrôles existants.
