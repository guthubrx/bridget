# Revue222 R2 : partielle, sans APPROVE global
Tâche d51b1c2c-d469-4bd2-b5c4-c02f4c768265, Claude/Haiku SYS. Résultat lu, ACK et clôture demandés une fois. Le complément n'a pas été remis : enfant déjà indisponible, aucun rejeu. L'avis a lu les trois tronçons RequestApproval/RequestTemporary/RequestStatus et pas les autres priorités. Aucun test ni build par le relecteur.

## Objections et vérification
1. Rejeu applied : hypothèse de doublon non vérifiée par l'auteur. La relecture de AgentProfileStore::specialist_apply montre le reçu (issuer,key,canonical SHA) retourné avant toute mutation. RequestRole passe par specialist_requests+work_submissions avec la même règle. RequestApproval n'accepte que ces opérations. Assertion ciblée ajoutée au test d'approbation : nombre de mutations et référence d'approbation identiques après le même applied.
2. Répondant humain : daemon/user_input.rs exige Client nu, UserInputV1+ControlStateV1, control_client_actor natif ; un agent ne passe pas respond. La voie mobile181 authentifie l'appareil et sa portée courante. decision_projects recontrôle l'acteur stocké et sa portée, jamais une chaîne fournie par le demandeur. L'agent propriétaire conserve son issuer pour la corrélation, pas pour se donner une décision.
3. Non lu : budget, RELEASE, MemoryVersions, retour humain, Android. Ces limites restent déclarées. Relecture du responsable et preuves ciblées ; aucune prétention de seconde lecture indépendante complète.

Verdict du responsable : les deux hypothèses ont leurs protections dans les appelants. Ne pas déclencher une troisième revue générale sans défaut nouveau ; terminer les contrôles ciblés du lot et déclarer la portée partielle de l'avis.
