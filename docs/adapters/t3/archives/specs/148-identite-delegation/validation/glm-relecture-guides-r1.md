# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-relecture-guides-r1.md](../../../../../../../specs/148-identite-delegation/validation/glm-relecture-guides-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Périmètre relu : `spec.md`, `contracts/delegation.md`, `contracts/session.md`, `adr-001-moteur-natif.md`, `docs/delegation-native.md`, `skills/bridget/SKILL.md`, comparés au code (`delegation_mcp.rs`, `daemon/native_delegation.rs`, `delegation.rs`, `registry.rs`, `mcp_identity.rs`, `t3code_mcp.rs`, montage `mcp.rs`, CLI). Aucun test, build ni modification exécutés, comme demandé.

<a id="extrait-2"></a>

## Extrait 2

- **Sélection exacte** : `for_delegation` (registry.rs:314-365) refuse `model_unavailable`/`effort_unavailable`, retire tout argument modèle/effort hérité, puis réécrit la valeur demandée. Aucune substitution. Testé (native_delegation.rs:439-450).
- **Rejeu** : clef `(owner, request_id)` + enveloppe canonique ; 10 rejeux = une tâche, un spawn (test :408-422, e2e :305-344). `envelope_mismatch` prouvé (test :425-436). Le rejeu précède la vérification de grant — cohérent avec le doc.
- **Identité T3** : preuve vérifiée à chaque appel (mcp.rs:359, test « résout l'identité à chaque appel »), paire d'variables partielle = refus fermé sans repli PID (t3code_mcp.rs:22-42), borne 64 Kio / 3 s / zéro redirection (lignes 8, 92-93), identités conflictuelles refusées (mcp_identity.rs:324-345).
- **Autonomie par le parcours natif Bridget** : aucun appel T3 dans le moteur (grep vide) ; la recette e2e tourne avec `t3_present:false`. Le tick natif est un timer réel d'une seconde (daemon.rs:4642-4652) : « Un timer natif pilote les phases acceptées » est prouvé.
- **Droits/postures** : grant humain seul via terminal interactif (cli.rs:2536-2541, refus agent prouvé test :465-473) ; la projection `discovery` retire `--dangerously-skip-permissions`/`bypassPermissions` du profil GLM réel (registry.rs:1312-1336) — point critique vérifié contre le registre utilisateur vivant.
- **Résultat** : capture uniquement depuis la connexion enfant corrélée, résultat durable avant ACK, `waiting_for_children` bloque la publication, fin de tour seule ne publie rien, réponse tardive divergente n'écrase pas le résultat publié (test :517-594).
- **Secrets** : variables T3 retirées de l'environnement enfant même si `pass_env` les demande (lifecycle.rs:463, test :638-647).
- **Parcours humain** : `delegate-grant` accepte un nom de fil, pas seulement un UUID (cli.rs:2542-2546). Aucun UUID demandé à l'humain dans le parcours nominal.

<a id="extrait-3"></a>

## Extrait 3

- L'outil `bridget_session`, l'injection des variables par montage et l'écriture du credential de fil vivent côté connecteur T3, hors de ce dépôt. Le contrat `session.md` les spécifie et le dépôt les vérifie. C'est une limite de périmètre, pas une promesse non tenue.
- SC001 en conditions réelles (deux conversations vivantes partageant un processus fournisseur réel) n'est pas prouvable depuis ce dépôt. Les recettes synthétiques équivalentes existent (t3code_mcp.rs:255, 331 ; mcp_identity.rs:868).
- SC007 (tests verts) n'est pas constaté ici : l'exécution était interdite. La couverture existe par lecture.
