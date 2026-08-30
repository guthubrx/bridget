# Modèle de données - SPEC-072

## Concepts

| Concept | Source de vérité | Rôle |
|---|---|---|
| Type d agent | clé du registre, par exemple glm | fournisseur sélectionné explicitement |
| Runtime | protocole et commande résolus | programme qui exécute le tour, par exemple Claude Code ou Cursor |
| Transport | acp, claude_stream_json, codex_app_server | protocole Bridget, jamais une étiquette fournisseur déduite |
| Modèle déclaré | capacités ou arguments du profil | intention de lancement, éventuellement absente |
| Modèle observé | trame fournisseur | information affichable seulement après observation |
| Profil Claude | répertoire CLAUDE_CONFIG_DIR privé | endpoint, token et variables spécifiques à un upstream |

## Extension de définition

AgentDefinition
  command
  args
  protocol
  claude_config_dir: chemin absolu non secret et optionnel
  autres champs existants

ResolvedAgentDefinition
  mêmes champs non secrets
  digest incluant claude_config_dir

Invariants :

- claude_config_dir est absent ou absolu.
- Il n est admis que pour claude_stream_json.
- Il ne peut pas coexister avec un pass_env de même nom.
- Il ne contient jamais de valeur de jeton dans le registre ou la définition résolue.
- Le chemin est contrôlé au lancement ; un profil manquant produit un refus explicite et n entraîne pas de repli silencieux vers Anthropic.

## Profils opérationnels

| Type | Runtime | Transport | Profil privé | État attendu |
|---|---|---|---|---|
| codex | Codex | codex_app_server | aucun | existant |
| cursor | Cursor | acp | authentification Cursor locale | à activer et prouver |
| claude | Claude Code | claude_stream_json | historique | compatibilité maintenue |
| anthropic | Claude Code | claude_stream_json | Anthropic isolé ou historique vérifié | nouveau libellé explicite |
| glm | Claude Code | claude_stream_json | GLM isolé | actif après preuve |
| deepseek | Claude Code | claude_stream_json | DeepSeek isolé | actif après preuve |

## Provenance publiée

ManagedProviderIdentity.provider_kind = type d agent explicitement lancé
ManagedExecutionBinding.provider_kind = même valeur
execution_path = protocole résolu

Cette provenance est déclarative. Elle ne tente pas de conclure qui fournit le modèle derrière Cursor ou de reconstruire un fournisseur depuis le texte.
