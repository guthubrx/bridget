# Recherche 024 — Mesures et arbitrages G11

## Flotte observée

La mesure initiale portait 36 agents : 12 `ssh-unix`, 3 `unix`, 9 `acp`,
8 `codex_app_server`, 3 `claude_stream_json`. Une relève ultérieure a
confirmé la structure et identifié le seul Claude `unix` : `bridget`,
`mode=tmux`, `location=Who:2.2`, vivant sur macOS.

Les douze présences Cartae ont toutes `mode=tmux` et une localisation de
pane. `wrapper::launch` construit alors `TmuxTransport`, qui livre par
`load-buffer`, `paste-buffer` et `send-keys`. La distance n'implique donc
pas app-server : ces agents parlent actuellement le chemin tmux interactif.

## Écrivain de la valeur fautive

`scripts/federate-ssh.sh` écrit sur l'hôte distant :

```text
transport=ssh-unix
```

`wrapper::transport_name` lit ensuite cette clé et la place dans
`Register.transport`. Le tunnel écrase donc le protocole dans un champ
unique ; le renderer ne fait que montrer l'amalgame produit en amont.

## Statut de ACP

`cursor-agent acp` expose nativement Agent Client Protocol et
`AcpTransport` implémente ce JSON-RPC. `acp` est un protocole exact ; aucune
valeur plus spécifique n'est attestée ni nécessaire pour les neuf Cursor.

## Arbitrage

- Deux colonnes, car protocole et canal sont simultanément utiles.
- `TRANSPORT` reste le protocole pour compatibilité du modèle public.
- `CANAL` devient le chemin de connexion au daemon.
- Aucun protocole n'est déduit du fournisseur : seul le chemin lancé fait foi.
