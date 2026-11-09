# Recherche et décisions
## Sources primaires consultées le 9 novembre 2026
- T3 local, commit75c7dafe : apps/server/src/orchestration-v2/CommandPolicy.ts:145-171. Sans tour actif, start_immediately ; guider, interrompre et mettre en file sont distincts. Ce parcours ne suffit pas pour une information sans inférence.
- https://code.claude.com/docs/en/agent-teams : agent inactif/masqué reste joignable ; contexte indépendant et coordination ont un coût. Retenir disponibilité distincte d'une sollicitation, sans inventer un filtre de notification.
- https://openai.github.io/openai-agents-python/human_in_the_loop/ : état durable RunState et reprise explicite. Retenir persistance distincte de reprise, pas assimiler ce SDK à une messagerie.
- https://docs.langchain.com/oss/javascript/langgraph/thinking-in-langgraph : interrupt persiste l'état avant reprise explicite. https://docs.langchain.com/langsmith/control-plane : stockage et signaux de réveil distincts. Pas de Redis/service nouveau pour Bridget.
## Décisions
1. Ajouter StoreOnly au contrat MessageIntent, exposé information. Aucun nouveau champ, ni dépendance. Un ancien lecteur refuse une variante inconnue ; négociation supplémentaire avant envoi.
2. Réutiliser ledger et idempotency_records dans une transaction terminale sans send_deliveries. La persistance n'est pas un ACK fournisseur. Réponse façade : stored, effect=stored_without_wakeup, model_woken=false. Le Lookup historique Accepted indique acceptation durable seulement ; l'intention d'origine reste dans le canon.
3. Réutiliser prepare_dispatch pour identité/projets/routage/limites ; seule la garde de dérangement (dormance/DND) est inapplicable au dépôt silencieux. Aucun changement d'état de l'agent. Refuser toute corrélation, reply et autorité de résultat AVANT capture.
4. Réutiliser bridget_ledger search/read. Aucune récupération automatique dans les prompts. Pour des bulletins collectifs regroupés : fil existant, notify:[], supersedes_seq sur entrée de type approprié ; history reste immuable.
5. Pas de filtre heuristique/LLM de maintenance. Corriger le contrat et les consignes du producteur : bulletin général silencieux, action ciblée uniquement sur travaux réellement affectés. Aucun tri temporel arbitraire.
6. Vérifier le signal dormant dans observations : save_observation_batch précède la tentative ; conserver FailedBeforeWrite si dormant. Ne pas supprimer un fait ni réinjecter automatiquement.
## Inconnues résolues / limites
Les files159 et QueueOnly ne sont pas une boîte silencieuse : ne pas les utiliser. Les observations sont typées et ne doivent pas stocker de prose libre. Un fil est partagé par ses membres : ne pas créer un fil privé par avis ni élargir ses membres automatiquement. Aucun coût monétaire réel disponible dans les traces.
## Primitive SpecKit
Scripts/templates du runtime absents (ls .specify/scripts .specify/templates : No such file). Protocoles des skills globales appliqués manuellement ; pas de réinstallation du runtime.

Disposition R2 de revue : StoreOnly conserve identité/projet/routage/corps mais ne consomme ni ne subit les gardes de boucles de remise (disjoncteur, contenu, quarantaine, sauts). Une collision ledger constatée dans la transaction produit un refus durable information_id_conflict ; panne SQL reste une issue non confirmée sans repli. Tests renforcés T017/T018, sans table ni dépendance nouvelle.
