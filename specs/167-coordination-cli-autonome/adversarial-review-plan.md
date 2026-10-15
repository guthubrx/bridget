# Contre-revue adverse du plan — SPEC-167

- Date : 2026-10-15
- Étape : après plan, avant tâches
- Détection : `bridget who`
- Résultat : aucun agent connecté.

La contre-revue inter-fournisseur n’a pas pu être lancée. Cette étape n’est pas bloquante selon le pipeline `$my-specify-all`.

## Auto-contrôle compensatoire

| Objection | Vérification | Retenue | Décision |
|---|---|---|---|
| Un simple terminal pourrait devenir parent d’agent. | Le moteur exige `owner`, `owner_instance`, une présence et une route vivante. | Oui | Garder une identité d’agent obligatoire. |
| Ajouter la CLI pourrait dupliquer le moteur MCP. | `delegation_mcp::execute` accepte déjà toutes les opérations prévues. | Oui | Construire seulement les charges JSON fermées. |
| `task list` et `task status` n’ont pas la même portée. | `control delegations` est une projection humaine globale ; `status` vérifie le propriétaire. | Oui | Documenter cette différence. Ne pas présenter la liste comme une autorité parent. |
| Une clé générée peut être perdue avant l’affichage. | La liste humaine permet de retrouver la tâche ; `--request-id` permet un rejeu contrôlé. | Oui | Afficher la clé et le `task_id`. Documenter `--request-id` pour les scripts critiques. |
| La portée pourrait devenir une refonte du protocole. | Toutes les variantes nécessaires existent déjà. | Oui | Interdire tout changement de protocole sans test RED qui le démontre. |
