# Modèle de données - SPEC-071

La SPEC ne crée ni table, ni événement, ni champ dans le protocole de présence.

## Projection serveur étendue

`UiAgentRowV1` expose quatre faits déjà présents dans `AgentInfo` :

| Champ UI | Type | Source | Absence |
|---|---|---|---|
| `transport` | chaîne | `AgentInfo.transport` | chaîne vide normalisée comme inconnue |
| `mode` | `tmux`, `acp`, `cli` ou absent | `AgentInfo.mode` | inconnu |
| `model` | chaîne optionnelle | `AgentInfo.model` | non affiché |
| `effort` | chaîne optionnelle | `AgentInfo.effort` | non affiché |

Le champ filaire `type` existant continue de porter `AgentInfo.agent_type`.

## Identité runtime dérivée

Le client applique une fonction pure au seul champ `type` normalisé :

| Types explicites reconnus | Produit | Éditeur | Asset |
|---|---|---|---|
| `codex`, préfixe attesté `codex-` | Codex | OpenAI | `/providers/openai.svg` |
| `claude`, `claude-native` | Claude Code | Anthropic | `/providers/claude.svg` |
| `cursor` | Cursor | Anysphere | `/providers/cursor.svg` |
| `gemini`, `gemini-cli` | Gemini CLI | Google | `/providers/gemini.svg` |
| autre ou absent | Inconnu | Inconnu | aucun asset de marque |

Le nom de l'agent, son avatar, son modèle et son message ne participent jamais
à cette fonction.

## Mode visible dérivé

| `mode` | `transport` | Valeur visible |
|---|---|---|
| `tmux` | toute valeur | `TMUX` |
| `acp` | toute valeur | `FLUX` |
| `cli` | `codex_app_server` ou `claude_stream_json` | `FLUX` |
| absent, `cli` avec autre transport, valeur future | toute valeur | `MODE INCONNU` |

Cette table est volontairement fermée. Une évolution future du protocole doit
être ajoutée explicitement avant d'être affichée comme FLUX.

## État client éphémère

| Nom | Contenu | Durée de vie |
|---|---|---|
| `identityCardAgent` | agent dont la fiche est ouverte | tant que survol/focus actif |
| `identityCardTrigger` | bouton décrit par la fiche | tant que la fiche est ouverte |
| `identityCardCloseTimer` | fermeture différée annulable | quelques millisecondes |

Aucun de ces états n'est persisté et aucun ne modifie `selectedAgent`.

## Invariants

- Une donnée absente reste absente ou inconnue.
- Un runtime inconnu ne reçoit aucun logo de marque.
- Le catalogue identifie l'éditeur du runtime, pas le fournisseur du modèle.
- Une seule fiche peut être visible à la fois.
- Fermer la fiche ne change ni le focus, ni la sélection, ni le fil.
