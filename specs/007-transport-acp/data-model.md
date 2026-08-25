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
      "args": [
        "@zed-industries/codex-acp@0.16.0",
        "-c",
        "model=\"gpt-5.5\""
      ],
      "protocol": "acp",
      "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
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

**Constat du spike T701 (2026-08-22).** Le pin
`-c model=\"gpt-5.5\"` ci-dessus est validé avec le home Codex courant :
`session/prompt` retourne `stopReason: "end_turn"`. Les surcharges vers
`gpt-5.6-sol` et l'effort `high` ne sont pas retenues, car le coeur Codex
embarqué par l'adaptateur 0.16.0 les refuse comme trop récentes.

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

Un objet JSON par ligne, append-only (FR-010). **Schéma versionné v1** —
enrichi sur exigence de la contre-revue de la spec 008 (le journal est le
contrat de lecture de `bridget attach` ; il doit porter l'attribution, un
curseur stable et des payloads typés) :

| Champ | Présence | Contenu |
|---|---|---|
| `v` | toujours | version du schéma (`1`) |
| `seq` | toujours | **curseur stable** : entier strictement croissant par agent (jamais réutilisé, y compris à travers la rotation quotidienne) — c'est la clé de la jonction rejeu→suivi sans perte ni doublon |
| `ts` | toujours | horodatage ISO 8601 |
| `session_id` | toujours | session ACP concernée |
| `event` | toujours | `turn_start` \| `update` \| `permission` \| `turn_end` \| `error` |
| `message_id` | si tour lié à un message Bridget | id du message d'origine |
| `payload` | toujours | objet **typé par `event`** (voir ci-dessous) |

Payloads par type d'événement :

| `event` | Champs du `payload` |
|---|---|
| `turn_start` | `from` (expéditeur du message livré), `reply` (bool), `body` (corps **complet** du message — arbitré en contre-revue 008 : le journal contient déjà le texte intégral des réponses, tronquer l'entrant créerait une infidélité d'affichage ; la contrepartie est l'exigence de permissions ci-dessous) |
| `update` | `kind` = `text` \| `tool_call` ; `text` : `content` (fragment de réponse) ; `tool_call` : `tool`, `summary` |
| `permission` | payload typé (contre-revue T706) : `tool` (résumé de l'outil demandeur), `options` (les `optionId`+`kind` proposés), `decision` = **l'issue réellement émise** — `{ "outcome": "selected", "option_id": … }` ou `{ "outcome": "cancelled" }` — jamais la politique brute du registre (une politique `allow` peut aboutir à `cancelled` faute d'option compatible) |
| `turn_end` | `stop_reason`, `routed_to` (destinataire de la réponse si `reply=yes`, absent sinon) |
| `error` | `reason` (détail libre) ; `terminal_kind=turn_failed` uniquement lorsque le worker a constaté l'échec terminal du prompt. Une anomalie de protocole non terminale n'a pas ce champ et ne constitue jamais une borne de fin. |

Exigences associées (observables de T706) :

- une ligne est écrite **entière puis flush** — jamais de ligne partielle
  visible d'un lecteur comme état final ; un lecteur doit néanmoins tolérer une
  ligne incomplète en fin de fichier (écriture en cours) en attendant le
  newline ;
- l'écriture passe par un **propriétaire de journal dédié** (thread + canal
  borné, enqueue non bloquant) — jamais d'E/S synchrone depuis le lecteur
  stdout ni le worker de tours ; saturation ou erreur d'écriture = erreur
  terminale explicite, jamais une perte silencieuse ;
- à la reprise sur un fichier du jour existant dont le dernier octet n'est pas
  un newline, la queue partielle est **isolée** (newline d'isolement ou
  troncature au dernier newline) avant le premier append — sans quoi le nouvel
  événement se collerait au fragment et les deux seraient perdus ;
- les champs optionnels **absents** ne sont jamais sérialisés à `null`
  (`skip_serializing_if`) — la présence est contractuelle ;
- `seq` traverse la rotation : le premier événement d'un nouveau fichier
  quotidien continue la séquence de la veille ;
- `seq` **survit au redémarrage et au crash du wrapper** : au démarrage, le
  prochain `seq` est récupéré depuis le dernier événement valide des journaux
  existants — y compris quand le dernier fichier est vide ou se termine par
  une ligne partielle ou corrompue (elle est ignorée pour la récupération). Un
  compteur mémoire repartant à 1 casserait la jonction rejeu→suivi de l'attach ;
- **permissions** : répertoire `~/.cache/bridget/sessions/` en 0700, fichiers
  de journal en 0600 dès création (ils contiennent l'intégralité des échanges) ;
- **fixtures de compatibilité lecteur** versionnées : un jeu de fichiers JSONL
  de référence (tour complet, tour en erreur, permission, rotation, dernière
  ligne partielle) que les tests de la session 008 consommeront tels quels.

Rotation : un fichier par jour et par agent ; pas de purge automatique dans
cette session (hors périmètre, noté pour la session 08 qui lit ces journaux).
