La revue est terminée. Voici le rapport R1.

---

# Revue R1 — Session 148 « Identité et délégation natives »

Périmètre relu : `spec.md`, `contracts/delegation.md`, `contracts/session.md`, `adr-001-moteur-natif.md`, `docs/delegation-native.md`, `skills/bridget/SKILL.md`, comparés au code (`delegation_mcp.rs`, `daemon/native_delegation.rs`, `delegation.rs`, `registry.rs`, `mcp_identity.rs`, `t3code_mcp.rs`, montage `mcp.rs`, CLI). Aucun test, build ni modification exécutés, comme demandé.

## Points vérifiés conformes au code

- **Sélection exacte** : `for_delegation` (registry.rs:314-365) refuse `model_unavailable`/`effort_unavailable`, retire tout argument modèle/effort hérité, puis réécrit la valeur demandée. Aucune substitution. Testé (native_delegation.rs:439-450).
- **Rejeu** : clef `(owner, request_id)` + enveloppe canonique ; 10 rejeux = une tâche, un spawn (test :408-422, e2e :305-344). `envelope_mismatch` prouvé (test :425-436). Le rejeu précède la vérification de grant — cohérent avec le doc.
- **Identité T3** : preuve vérifiée à chaque appel (mcp.rs:359, test « résout l'identité à chaque appel »), paire d'variables partielle = refus fermé sans repli PID (t3code_mcp.rs:22-42), borne 64 Kio / 3 s / zéro redirection (lignes 8, 92-93), identités conflictuelles refusées (mcp_identity.rs:324-345).
- **Autonomie sans T3** : aucun appel T3 dans le moteur (grep vide) ; la recette e2e tourne avec `t3_present:false`. Le tick natif est un timer réel d'une seconde (daemon.rs:4642-4652) : « Un timer natif pilote les phases acceptées » est prouvé.
- **Droits/postures** : grant humain seul via terminal interactif (cli.rs:2536-2541, refus agent prouvé test :465-473) ; la projection `discovery` retire `--dangerously-skip-permissions`/`bypassPermissions` du profil GLM réel (registry.rs:1312-1336) — point critique vérifié contre le registre utilisateur vivant.
- **Résultat** : capture uniquement depuis la connexion enfant corrélée, résultat durable avant ACK, `waiting_for_children` bloque la publication, fin de tour seule ne publie rien, réponse tardive divergente n'écrase pas le résultat publié (test :517-594).
- **Secrets** : variables T3 retirées de l'environnement enfant même si `pass_env` les demande (lifecycle.rs:463, test :638-647).
- **Parcours humain** : `delegate-grant` accepte un nom de fil, pas seulement un UUID (cli.rs:2542-2546). Aucun UUID demandé à l'humain dans le parcours nominal.

## Défauts prouvés (documentation)

| # | Gravité | Lieu | Défaut |
|---|---|---|---|
| 1 | **Moyenne** | `skills/bridget/SKILL.md:170,247,328` ; `skills/bridget/references/commandes.md:97` | « Seize outils » est périmé : le catalogue sert désormais 20 outils (16 de base + 4 outils148, mcp.rs:2236). `references/commandes.md` ne documente aucun des quatre outils ni `delegate-grant`, alors que SKILL.md:169-171 la présente comme l'inventaire complet. Un agent peut conclure que les outils148 sont étrangers au catalogue fermé. |
| 2 | Mineure | `docs/delegation-native.md:17` | L'exemple utilise `agent_type:"claude_glm"`. Ce type n'existe ni dans le registre par défaut (codex/claude/cursor/gemini) ni dans le registre réel (le profil GLM s'appelle `glm`, modèle `glm-5.3`). Copié tel quel, l'appel est refusé « type d'agent inconnu ». L'avertissement d'adaptation est présent mais la valeur réelle de la session était disponible. |
| 3 | Mineure | `docs/delegation-native.md:48` | « Bridget refuse cette posture avec `development_protocol_unavailable` » : ce littéral n'existe que dans le champ `development_refusal` du catalogue (native_delegation.rs:119). Un `bridget_delegate` réel en `development` sur GLM est refusu avec le message « posture développement réservée à Codex app-server » (registry.rs:217-219). Le doc confond l'annonce du catalogue et le code de refus de l'appel. |

Aucun défaut prouvé dans le code natif.

## Limites réelles (distinctes des défauts de doc)

- L'outil `bridget_session`, l'injection des variables par montage et l'écriture du credential de fil vivent côté connecteur T3, hors de ce dépôt. Le contrat `session.md` les spécifie et le dépôt les vérifie. C'est une limite de périmètre, pas une promesse non tenue.
- SC001 en conditions réelles (deux conversations vivantes partageant un processus fournisseur réel) n'est pas prouvable depuis ce dépôt. Les recettes synthétiques équivalentes existent (t3code_mcp.rs:255, 331 ; mcp_identity.rs:868).
- SC007 (tests verts) n'est pas constaté ici : l'exécution était interdite. La couverture existe par lecture.

## Verdict

**APPROVE** — avec les trois corrections de documentation ci-dessus à faire avant livraison. Ce sont des corrections de texte, pas des changements de spécification ni de code. Les promesses centrales de la session (délégation en un appel, sélection exacte, identité par session, autonomie sans T3, refus fermés) sont toutes adossées à du code lu et à des tests présents.
