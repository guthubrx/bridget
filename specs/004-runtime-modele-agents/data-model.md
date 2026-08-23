# Phase 1 — Modèle de données

**Feature** : 004-runtime-modele-agents

Aucune table, aucune migration : le runtime est un état volatil attaché à la
présence d'un agent, au même titre que l'hôte, l'OS et le transport
(arbitrage utilisateur du 2026-08-17).

## Entité — Runtime d'agent

Portée par la structure `Presence` existante (`crates/bridget-daemon/src/daemon.rs:65`).

| Champ | Type | Valeur d'inconnu | Origine |
|---|---|---|---|
| `model` | `Option<String>` | `None` → affiché `—` | dernière observation ou déclaration |
| `effort` | `Option<String>` | `None` → affiché `—` | idem ; légitimement absent sur les modèles sans réglage d'effort (research.md D-001) |

**Cycle de vie** : créé à `None` lors de `Register`, mis à jour par chaque
message `Runtime` reçu, conservé tel quel lors d'un passage à `unreachable`
(FR-010), détruit avec la présence à l'expiration de `PRESENCE_RETENTION`
(300 s, `daemon.rs:59`) ou sur `Unregister`.

**Invariants** :

1. **Une observation est atomique** : un message `Runtime` porte le résultat
   complet d'une observation réussie et remplace les deux champs d'un bloc.
   Un `effort: None` reçu signifie « observé absent » et efface donc une valeur
   antérieure.
   *Justification* : les deux parseurs lisent le modèle et l'effort au même
   endroit du même fichier ; le cas « je connais l'un sans l'autre » n'existe
   dans aucune source. Traiter `None` comme « pas d'information » figerait un
   effort périmé lors d'un passage d'un modèle qui expose l'effort vers un
   modèle qui ne l'expose pas — cas nominal Opus → Haiku établi par
   research.md D-001, et défaut soulevé par la contre-revue `agent-1`.
2. La dernière valeur reçue gagne, quelle que soit sa source (FR-008, scénario 3
   de la User Story 3). Aucune hiérarchie entre détection et déclaration.
3. Le runtime survit au renommage : il est indexé par `instance_id`, pas par nom.
4. Le runtime survit à une reconnexion sous la même identité, puisque la
   présence elle-même y survit (spec `002-federation-ssh`).

## Entité — Source d'observation

Non persistée. Elle existe uniquement dans les journaux (`log::debug`) pour le
diagnostic : `codex-rollout`, `claude-hook`, `declared`. Un champ de plus dans
l'annuaire ne servirait aucun scénario utilisateur décrit par la spec, il n'est
donc pas ajouté (Article XIX).

## Vue projetée — AgentInfo

`bridget_transport::protocol::AgentInfo` (`protocol.rs:83`) gagne deux champs
sérialisés, avec valeur par défaut pour rester compatible avec un client d'une
version antérieure :

```rust
#[serde(default)]
pub model: Option<String>,
#[serde(default)]
pub effort: Option<String>,
```

## Transitions

```text
Register                  → model=None, effort=None
Runtime{Some(m), Some(e)} → model=m,    effort=e
Runtime{Some(m), None}    → model=m,    effort=None      (effort observé absent)
Runtime{None, _}          → rejeté : une observation sans modèle n'est pas
                            une observation, le producteur ne doit pas l'émettre
mark_unreachable          → valeurs conservées, state="unreachable"
Unregister / expiration   → présence supprimée, valeurs perdues
```

## État côté producteur

Chaque producteur conserve en mémoire la dernière paire `(model, effort)`
transmise, afin de n'émettre que sur changement (FR-007, SC-003) :

- sonde Codex : variable locale du fil d'écoute du wrapper ;
- hook Claude : sans état, une exécution par tour ; le filtrage sur changement
  est fait par le daemon, qui ignore un message identique à l'état courant.
