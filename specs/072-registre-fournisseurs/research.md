# Recherche technique - SPEC-072

## Faits observés

| Sujet | Observation | Conséquence |
|---|---|---|
| Cursor | /home/moi/.local/bin/cursor-agent est présent, authentifié et expose cursor-agent acp. | Réutiliser AcpTransport, ne pas créer d adaptateur Cursor. |
| Registre | native_cursor_definition() existe déjà dans crates/bridget-daemon/src/registry.rs. | Le problème est l activation d un agent Cursor, pas son support de protocole. |
| Claude Code | claude_stream_json est le transport géré actuel. | GLM et DeepSeek réutilisent ce transport. |
| Provenance | ClaudeStreamJsonTransport publie aujourd hui provider_kind = claude en dur. | Le contexte fournisseur doit recevoir le type explicite lancé. |
| Environnement | Le registre ne peut qu hériter des variables présentes dans le daemon. | Un profil statique doit fournir un CLAUDE_CONFIG_DIR distinct, sans persister de secret dans la définition résolue. |
| GLM local | La fonction glm() configure un endpoint, un jeton, des modèles par défaut et un timeout. | La configuration doit être transférée directement vers un fichier privé serveur, sans passer par Git ni le journal. |
| DeepSeek local | La fonction deepseek() configure déjà un endpoint, un jeton et le modèle. | DeepSeek peut recevoir le même contrat de profil que GLM et faire l objet d une preuve réelle. |
| SPEC-071 | Elle présente le runtime et refuse toute déduction du fournisseur du modèle. | SPEC-072 publie une provenance explicitement déclarée. SPEC-071 pourra la présenter plus tard, sans inférence et sans modifier ses fichiers en cours. |

## Sources externes vérifiées le 2026-08-30

- Cursor documente le CLI, l authentification locale ou par clé, et la sortie structurée en continu. Les événements texte et outils sont émis en NDJSON.
  https://docs.cursor.com/en/cli/overview
  https://docs.cursor.com/en/cli/reference/output-format
  https://docs.cursor.com/en/cli/reference/authentication
- Anthropic documente la configuration de Claude Code vers un endpoint compatible via ANTHROPIC_BASE_URL et ANTHROPIC_AUTH_TOKEN, y compris le placement de variables dans settings.json.
  https://docs.anthropic.com/en/docs/claude-code/llm-gateway
- Z.AI documente explicitement Claude Code, son endpoint Anthropic compatible, le jeton, les variables de modèles et le timeout.
  https://docs.z.ai/devpack/tool/claude
- DeepSeek documente son endpoint Anthropic compatible et son intégration Claude Code. Il signale aussi les éléments de compatibilité ignorés ou non supportés, ce qui interdit de conclure à une compatibilité totale sans essai.
  https://api-docs.deepseek.com/quick_start/agent_integrations/claude_code/
  https://api-docs.deepseek.com/guides/anthropic_api/

## Décisions de recherche

1. agent_type reste la provenance déclarée. Les types nouveaux sont anthropic, glm et deepseek. claude reste un alias historique.
2. Le runtime est indépendant de cette provenance : GLM et DeepSeek affichent Claude Code comme runtime, mais leurs contextes d exécution identifient GLM ou DeepSeek comme fournisseur sélectionné.
3. Un unique champ claude_config_dir est préféré à une carte arbitraire de variables d environnement. Il a trois usages réels maintenant : Anthropic, GLM et DeepSeek. Il évite de persister un jeton dans la définition ou son digest.
4. Les fichiers settings.json de profils restent hors dépôt, répertoires 0700 et fichiers 0600. Le registre n en conserve que le chemin absolu non secret.
5. Cursor ne reçoit ni clé API dans les arguments ni adaptation maison. Il réutilise son authentification locale déjà vérifiée et le transport ACP.

## Risques retenus

- Les flux Claude Code compatibles doivent être prouvés sur texte, outil, permission, erreur et interruption. Un endpoint compatible ne vaut pas une compatibilité métier démontrée.
- DeepSeek peut mapper des noms Claude vers ses modèles. Bridget ne doit donc pas promettre un modèle déduit : il journalise l identifiant déclaré puis complète seulement avec le modèle réellement observé.
- Les profils secrets risquent de contaminer Anthropic si le répertoire de configuration n est pas isolé. Le test parallèle GLM/Anthropic est un gate.
- Aucun changement ne doit écraser le worktree de SPEC-071, actuellement non fusionné et propriétaire de la présentation UI de runtime.

## Réutilisation imposée

- AcpTransport pour Cursor.
- ClaudeStreamJsonTransport pour Anthropic, GLM et DeepSeek.
- AgentRegistry, build_environment, les définitions résolues et les ManagedProviderIdentity existants.
- Aucune dépendance, crate, service HTTP ou abstraction multi-fournisseur supplémentaire.
