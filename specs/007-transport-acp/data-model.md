# Data Model : Transport ACP

## Registre d'agents (`~/.config/bridget/agents.json`)

Une entrée par type d'agent, lue avec `serde_json` (déjà en workspace —
arbitrage du reuse-audit : pas de crate TOML). Un registre par défaut embarqué
couvre les trois types prioritaires ; le fichier utilisateur le complète ou le
surcharge. Les valeurs par défaut suivent research R-002.

```json
{
  "agents": {
    "codex": {
      "command": "npx",
      "args": ["@zed-industries/codex-acp@0.16.0"],
      "protocol": "acp",
      "forbidden_env": ["OPENAI_API_KEY"],
      "permissions": "allow"
    },
    "claude": {
      "command": "npx",
      "args": ["@zed-industries/claude-code-acp@0.16.2"],
      "protocol": "acp",
      "forbidden_env": ["ANTHROPIC_API_KEY"],
      "permissions": "allow"
    },
    "gemini": {
      "command": "gemini",
      "args": ["--acp"],
      "protocol": "acp",
      "forbidden_env": ["GEMINI_API_KEY", "GOOGLE_API_KEY"],
      "permissions": "allow"
    }
  }
}
```

**Constat du spike T701 (2026-08-22).** La commande par défaut ci-dessus est
validée avec un `CODEX_HOME` isolé qui ne contient que l'authentification
abonnement. Les surcharges `-c model=\"gpt-5.6-sol\"` et
`-c model_reasoning_effort=\"high\"` ne sont pas retenues : le coeur Codex
embarqué par l'adaptateur 0.16.0 les refuse comme trop récentes. L'isolation du
home est une condition de validation du spike, pas encore un champ du registre
à implémenter ; l'ajouter exige une décision et une tâche dédiées.

`protocol` vaut `acp` ou `tmux` ; `forbidden_env` porte la garde de facturation
(FR-011) ; `permissions` la réponse automatique aux `session/request_permission`
(R-005).

Clés optionnelles par entrée, avec défauts embarqués :

| Clé | Défaut | Rôle |
|---|---|---|
| `queue_capacity` | 32 | capacité de la file de messages (politique de file) |
| `notify_timeout_secs` | 600 | timeout de transport des tours de notification `reply=no` (D-206) |

Contraintes de validation (au chargement, erreurs en français) :

- `command` non vide ; `protocol` ∈ {`acp`, `tmux`} ;
- `permissions` ∈ {`allow`, `deny`} ;
- type inconnu au lancement → refus nommant le fichier et les types disponibles
  (remplace la liste blanche en dur, FR-013/FR-006) ;
- clé inconnue dans une entrée → avertissement (typo probable), pas un refus ;
- validation **atomique** : un fichier utilisateur invalide (JSON malformé,
  contrainte violée) refuse le lancement avec l'erreur précise — jamais de
  fusion partielle silencieuse ;
- garantie de version (arbitrage contre-revue, objection 6) : la version des
  adaptateurs npm est garantie par le **pin dans `args`** (`@paquet@x.y.z`) et
  la compatibilité protocolaire par la négociation `initialize` — il n'y a
  **pas** de sonde générique de version de paquet (promesse retirée de la spec ;
  pour Gemini natif, seule la négociation ACP fait foi).

## État d'un équipier (en mémoire, propriété exclusive d'`AcpTransport`)

Le wrapper ne tient aucune copie de cet état : il relaie les signaux de cycle
de vie (`CancelDelivery`) au transport et remonte ses événements
(`DeliveryRejected`, transitions de tour) au daemon.

| Champ | Type | Rôle |
|---|---|---|
| `session_id` | chaîne ACP | session unique de l'équipier (D-203) |
| `turn` | `Idle \| InProgress { message_id, depuis }` | état de tour — source des relances différées (FR-008) |
| `queue` | `VecDeque<BridgetMessage>`, **bornée** | messages en attente pendant un tour (FR-007), livrés en ordre FIFO |

L'état de tour est remonté au daemon à chaque transition (variante
wrapper→daemon), jamais sondé.

### Politique de file (contre-revue cxbridget, objection 4)

| Situation | Comportement |
|---|---|
| capacité (défaut : 32, configurable au registre) atteinte | échec **asynchrone terminal** motivé : `DeliveryRejected { id, reason }` remonté au daemon, qui notifie l'émetteur (D-209 — le daemon a déjà accusé réception au moment du push, un refus synchrone est impossible) ; jamais de perte silencieuse |
| échéance d'un message dépassée pendant l'attente | retiré de la file sur `CancelDelivery` du daemon (autorité unique, D-206), jamais livré en retard |
| annulation d'une demande (cycle de vie 003) | `CancelDelivery { id, reason }` daemon→wrapper : le message est retiré de la file par son id (l'annulation textuelle actuelle ne permet pas de purger, D-209) |
| mort du processus adaptateur | tous les `reply=yes` restants (file **et** tour actif) produisent un échec motivé vers leurs émetteurs ; les `reply=no` sont journalisés |

### Transitions d'état de l'équipier (annuaire)

Complément demandé en contre-revue (objection 8) — `stopped` est distinct
d'`unreachable` :

| Événement | État annuaire | Effet sur les relances |
|---|---|---|
| tour démarré | `busy` (tour en cours) | relances différées, événement consigné |
| tour terminé | `connected` (inactif) | relances normales |
| processus adaptateur mort | `stopped` | échec motivé immédiat des demandes en cours (pas de relance) |
| wrapper déconnecté du daemon (réseau) | `unreachable` | comportement fédération existant inchangé |
| reconnexion wrapper pendant un tour | l'état de tour est re-déclaré au `Register` (le wrapper connaît son `turn` local) | l'état `busy` n'est jamais perdu par une coupure |

## Journal de session (`~/.cache/bridget/sessions/<agent>/<date>.jsonl`)

Un objet JSON par ligne, append-only (FR-010) :

| Champ | Contenu |
|---|---|
| `ts` | horodatage ISO 8601 |
| `event` | `turn_start` \| `update` \| `permission` \| `turn_end` \| `error` |
| `message_id` | id du message Bridget à l'origine du tour (absent pour `update` internes) |
| `detail` | texte : extrait de réponse, nom d'outil appelé, `stopReason`, motif d'erreur |

Rotation : un fichier par jour et par agent ; pas de purge automatique dans
cette session (hors périmètre, noté pour la session 08 qui lit ces journaux).
