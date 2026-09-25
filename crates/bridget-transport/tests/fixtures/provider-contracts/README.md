# Contrats publics minimaux de fournisseurs

Ces fixtures figent seulement les trames nécessaires aux frontières Bridget.
Elles utilisent des identifiants synthétiques et ne sont ni des journaux de
production ni des captures de conversations.

| Fixture | Binaire relevé | Frontière stabilisée |
|---|---|---|
| `codex-0.150.1.jsonl` | Codex CLI 0.150.1 | `thread/start`, `turn/start`, `turn/interrupt`, `item/started.userMessage.clientId` |
| `claude-2.1.221.jsonl` | Claude Code 2.1.221 | prompt stream-json, `control_request/interrupt`, accusé et terminal annulé |
| `acp-v1.jsonl` | ACP v1 | initialisation, session, prompt, autorisation, `session/cancel` |

## Règles de maintenance

1. Les fixtures ne portent ni clés, ni chemins personnels, ni texte utilisateur.
2. Une version ou une forme nouvelle est ajoutée à côté de celle-ci.
3. Un test de protocole consomme la fixture avant de dépendre de sa forme.
4. `provider_kind=cursor` reste distinct de `execution_path=acp` : ce contrat
   couvre Cursor, Gemini et tout fournisseur ACP compatible.

Les événements facultatifs et les champs libres sont volontairement absents.
Leur apparition réelle exige un relevé de T002 puis une nouvelle fixture.
