# Contre-revue adverse de l’implémentation — SPEC-167

- Date : 2026-10-15
- Étape : après implémentation et tests ciblés
- Détection : `bridget who`
- Résultat : aucun agent connecté.

La contre-revue inter-fournisseur n’a pas pu être lancée. Le daemon observé est aussi plus ancien que le client courant. Le pipeline autorise cette absence quand aucun autre fournisseur n’est disponible.

## Relecture compensatoire

| Objection | Preuve | Verdict |
|---|---|---|
| La façade pourrait fabriquer un parent humain. | `run_native_cli` appelle `resolve_current_identity()` avant `delegation_mcp::execute`. | Refutée. |
| La sortie humaine pourrait masquer l’agent choisi. | Le moteur rend `child_agent_id`; le formateur utilise ce champ et un test le vérifie. | Refutée après correction. |
| Une erreur native pourrait perdre son code réel. | `ClientError::Technical { code, message }` traverse maintenant la façade. | Refutée après correction. |
| Les nouvelles racines pourraient échapper à l’inventaire de sécurité. | `skills/bridget/references/commandes.md` documente `auto-delegate`, `delegate` et `task`; le test d’inventaire passe. | Refutée après correction. |
| La CLI pourrait dupliquer les règles métier. | Toutes les actions appellent `delegation_mcp::execute`; aucun protocole, magasin ou daemon n’est modifié. | Refutée. |
| Une liste pourrait accorder les droits du propriétaire. | `task list` reste l’alias de la projection globale bornée; les autres actions résolvent l’identité propriétaire. | Refutée. |

## Verdict

APPROVE. Aucun défaut bloquant ne reste dans le diff.
