# Session 043 — Refuser les incohérences de réponse CLI

**Statut** : En cours

**Base gelée** : `75dd315bc2e6389f9c206d179443701f03f94100`

**Objectif** : `747a0775-7b9c-4069-94a8-62f94f97fd07`

## Défaut mesuré

Un vrai binaire `bridget`, relié à une socket Unix jetable, sérialise :

- `send --timeout 30` sans `--reply` avec `reply: false` et
  `reply_timeout: 30` ;
- un `send --reply` de l'expéditeur implicite `human` avec `reply: false` ;
- la combinaison humaine `--reply --timeout 30` avec `reply: false` et
  `reply_timeout: 30`.

Le même dernier état est aussi mesuré sur `reply`. L'intention explicite de
l'humain est donc supprimée sans erreur, et le délai est présent alors que le
daemon ne suit aucune réponse.

## Contrat existant

`BridgetMessage::reply_timeout` est documenté comme valable seulement si
`reply=true`. Le daemon ne crée ni ne surveille de cycle de réponse lorsque
`reply=false`. Le canal MCP applique déjà cette règle : `reply_timeout` avec
`reply=false` est `invalid_params`.

## Propriété

Les commandes `send` et `reply` refusent avec le code 2, avant toute
connexion au daemon :

1. `--timeout` sans `--reply` ;
2. `--reply` quand l'expéditeur résolu est `human`.

Le cas combiné humain `--reply --timeout N` est refusé au titre de `--reply` :
aucune attente ne peut être créée pour un expéditeur non connecté. Un
expéditeur non humain conserve `--reply` et son délai, y compris le défaut de
60 secondes. Un message humain sans `--reply` ni délai reste inchangé.

## Hors périmètre

L'absorption des options inconnues par `send` appartient à un autre lot. Cette
session ne modifie ni le parseur de ces options ni les mots ajoutés au corps du
message.

## Oracles et mutants

Les oracles exécutent le vrai binaire contre une socket Unix jetable et
observent séparément la trame et `connection_accepted`. La fixture tente
d'abord `accept` avant de lire son signal d'arrêt : une connexion déjà dans la
file ne peut donc pas être masquée par la fin du client. Les refus exigent à la
fois l'absence de trame et `connection_accepted=false`. Ils couvrent les deux
commandes, les deux contrôles positifs et le cas humain combiné.

- Retirer la garde de délai doit laisser partir un message ayant
  `reply=false` : la construction ne sérialise désormais le délai que sous
  `reply=true`, mais l'intention explicite reste interdite avant connexion.
- Retirer le refus humain laisse partir un message humain malgré `--reply`.
- Insérer une connexion nue juste avant chaque garde fait mourir les deux cas
  de sa voie sur `connection_accepted=true`.
