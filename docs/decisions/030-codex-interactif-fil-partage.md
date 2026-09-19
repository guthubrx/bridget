# ADR 030 — Interface Codex native sur le même fil que Bridget

Date : 2026-09-05. Statut : validé par les gates 090 ; adopté sur demande le 2026-09-06.

## Contexte

Le wrapper interactif historique injecte par tmux. L'utilisateur ne veut plus
de tmux mais veut garder la vraie interface Codex. Le mode managed+attach ne
remplace pas cette interface. La sonde 090 établit le partage d'un fil app-server
nommé entre deux clients et la diffusion des permissions aux deux clients.

## Décision

Étendre le pilote natif et le wrapper existants. App-server local dédié avec
socket Unix privée, TUI remote sur le fil attesté, client Bridget observateur et
émetteur. Adaptation WS/JSONL sans protocole métier supplémentaire. Pas de tmux,
pas de TUI réécrite, pas de serveur réseau public. Les permissions sont décidées
par la TUI ; pas d'application de la politique headless allow à l'interactif.

## Conséquences

Le code de communication demeure unique ; nouveaux coûts : framing WebSocket,
cycle de vie du terminal et observation de tours non initiés par Bridget.
Compatibilité expérimentale vérifiée par recette, non présumée à partir du nom
du fournisseur. `app-server proxy` ne remplace pas la socket WS (sonde négative).
