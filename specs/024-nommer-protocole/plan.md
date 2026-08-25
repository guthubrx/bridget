# Plan 024 — Nommer le protocole réel sans perdre le canal

## D-2401 — Deux dimensions, deux colonnes

`AgentInfo.transport` conserve son contrat public et devient strictement le
protocole d'agent. Un champ additif `channel: Option<String>` porte le chemin
vers le daemon. `who` affiche `TRANSPORT` puis `CANAL`. Cette forme évite une
rupture des consommateurs du champ `transport` tout en préservant la distance.

## D-2402 — Le chemin d'exécution fait autorité

- géré : protocole issu de la définition figée (`acp`,
  `codex_app_server`, `claude_stream_json`) ;
- interactif : `PresenceMode::Tmux` implique le protocole `tmux`, car la
  remise passe réellement par `TmuxTransport` ;
- canal : annoncé par le wrapper (`unix` ou `ssh-unix`), jamais déduit du
  type `codex`, `claude` ou `cursor`.

Les douze agents Cartae ne seront donc pas étiquetés app-server avant leur
migration effective hors tmux.

## D-2403 — Compatibilité filaire progressive

`WrapperToDaemon::Register` reçoit un champ optionnel `channel`. L'ancien
champ `transport` reste accepté. Pour une ancienne trame tmux, sa valeur est
reclassée comme canal et le protocole devient `tmux`. Pour un géré, la
définition reste prioritaire. `AgentInfoWire.channel` a une valeur par défaut
absente afin qu'un client récent lise un ancien daemon.

## D-2404 — Corriger l'écrivain, garder les alias

Le nom interne devient `connection_channel`. Ordre de lecture :
`BRIDGET_CHANNEL`, alias `BRIDGET_TRANSPORT`, `channel=` dans
`federation.env`, alias `transport=`, puis `unix`. L'installateur écrit les
deux clés pendant la période de compatibilité afin qu'un ancien wrapper
continue de voir `ssh-unix`.

## D-2405 — Matrice de preuve

1. tmux local : `tmux` / `unix` ;
2. tmux fédéré historique : `tmux` / `ssh-unix` ;
3. ACP : `acp` / `unix` ;
4. Codex natif : `codex_app_server` / `unix` ;
5. Claude natif : `claude_stream_json` / `unix` ;
6. trame sans canal attesté : protocole conservé, canal absent.

Les tests couvrent aussi le rendu des deux colonnes et la préférence de la
nouvelle clé de configuration sur l'alias historique.

## Fichiers pressentis

- `crates/bridget-transport/src/protocol.rs`
- `crates/bridget-daemon/src/wrapper.rs`
- `crates/bridget-daemon/src/daemon.rs`
- `crates/bridget-daemon/src/cli.rs`
- `scripts/federate-ssh.sh` et son test
- `README.md` et `README.en.md`
