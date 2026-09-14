# Recherche 097 — faits et décisions

## Environnement constaté le 2026-09-13

Claude Code 2.1.270 installé en `/Users/moi/.local/bin/claude`, connecté par
`claude.ai` (fournisseur first-party). Session d'abonnement conservée dans le
trousseau macOS (service « Claude Code-credentials »), aucun fichier
`~/.claude/.credentials.json`. Rust 1.92, workspace à trois crates, `libc`
déjà dépendance des crates transport et daemon, `signal-hook` dans le daemon.
Aucun serveur tmux ne tourne ; l'humain travaille dans iTerm.

## Sonde 1 — héritage de la connexion au compte (volet A)

Commande minimale `claude -p 'Réponds uniquement le mot: pong' --output-format
json --model claude-haiku-4-5-20251001 --max-turns 1 --tools "" --setting-sources
"" --strict-mcp-config`, un appel par cas, aucun secret lu ni copié :

| Environnement | Résultat |
|---|---|
| `env -i HOME=/Users/moi` | échec « OAuth session expired and could not be refreshed » |
| `env -i HOME=/Users/moi USER=moi` | `pong` en 1,0 s |
| environnement hérité complet | `pong` en 1,1 à 1,6 s |
| HOME isolé + `CLAUDE_CONFIG_DIR=/Users/moi/.claude` | « Not logged in · Please run /login » |
| HOME isolé seul | « Not logged in · Please run /login » |

Conclusion : le CLI retrouve sa session d'abonnement seulement avec le HOME réel
ET la variable `USER` (lecture du trousseau par compte). Un HOME neuf n'est pas
authentifié même avec le répertoire de configuration réel. Le rouge de la
recette 089 (« Not logged in ») venait de l'isolation du HOME, pas d'un défaut
du pilote. Le daemon de production (PID 996) porte HOME, USER et LOGNAME : un
Claude géré lancé par lui hérite bien de la connexion.

## Sonde 2 — état actuel de la remise interactive

`bridget claude` s'enregistre avec `PresenceMode::Tmux` en dur et ne construit
un transport que si `tmux display-message` répond (wrapper.rs, `transport =
None` sinon). Sans pane : livraisons idempotentes → `DeliveryIndeterminate`,
rappels → `warn!` silencieux. Le journal reçoit pourtant `turn_start`. Vérifié
sur la base du 2026-09-13 (phases `indeterminate`, deux demandes `timed_out`).

## Décision

1. **Recette Claude géré** : rejouer le gate réel de 089 avec HOME réel et
   `USER` transmis, BRIDGET_HOME/socket privés, registre privé neutralisant la
   configuration utilisateur (`--setting-sources ""`, `--strict-mcp-config`,
   `--tools ""`), modèle et effort explicites. Aucune clé d'API, aucune lecture
   du trousseau par le test : le CLI officiel s'authentifie seul.
2. **Transport pseudo-terminal** : le wrapper ouvre un PTY (`libc::openpty`,
   motif déjà utilisé par les tests d'attach), lance Claude Code dans une
   nouvelle session avec ce PTY pour terminal de contrôle, met le terminal réel
   en mode brut, relaie octets et taille de fenêtre dans les deux sens, restaure
   le terminal à la sortie. La remise d'un message = écriture dans le maître du
   PTY en collage encadré (bracketed paste) puis `\r`, sous les mêmes
   validations de contenu que la voie tmux. Le succès d'écriture est
   l'observable de `PromptDispatched`, comme pour tmux (wrapper.rs, §747).
3. **Présence** : mode `cli` et transport `claude_pty`, sans nouvelle variante
   du protocole ni nouvelle trame ; c'est le même schéma que Codex interactif
   (mode `cli`, transport `codex_app_server`). Attach reste admis comme en 090.
4. **Garde-fou** : sans terminal, sans PTY ou sans pane pour les types encore
   tmux (gemini, agents personnalisés), le wrapper refuse de démarrer avec un
   diagnostic et l'alternative `bridget spawn`. Plus d'enregistrement joignable
   sans voie de remise.

## Alternatives écartées

- **Tail du transcript pour détecter la réception** : le transcript arrive
  après coup et ne prouve pas la remise dans le composer ; conservé seulement
  pour modèle/effort et pour le journal des tours.
- **Maintenir tmux en parallèle du PTY** : deux voies à tester et documenter
  pour le même usage ; le PTY fonctionne aussi dans un pane tmux.
- **Nouvelle variante `PresenceMode::Pty`** : casserait la désérialisation par
  un daemon ancien lors d'une fédération SSH ; le champ `transport` libre
  suffit à nommer le canal réel.
- **Dépendance `portable-pty`/`nix`** : `libc` couvre `openpty`, `ioctl`,
  `tcgetattr/tcsetattr`, `setsid` ; une dépendance de plus n'apporte que du
  volume (Article XIX).
- **Jeton OAuth passé en variable** : recette 089 rouge, jeton périmé possible,
  secret dans l'environnement d'un processus enfant ; refusé au profit de
  l'héritage natif.

## Risques nommés

- Collage encadré : si Claude Code ne l'active pas dans le PTY, les sauts de
  ligne du corps pourraient soumettre trop tôt. Vérifié par le harnais avant
  la recette réelle ; repli : remplacer `\n` par un espace insécable de ligne
  comme le fait déjà tmux ? Non : tmux envoie le corps brut ; le repli
  documenté est l'attente de digestion avant `\r`, identique à tmux.
- Terminal brut : une panique du wrapper laisserait le terminal illisible ;
  la restauration termios vit dans un garde `Drop` et sur les signaux TERM/INT/HUP.
- Recette réelle : consomme le quota d'abonnement de l'humain ; un seul tour,
  modèle économique, effort bas.

## Sources

- Sondes locales du 2026-09-13 (tableau ci-dessus), journal daemon
  `~/.cache/bridget-core/daemon-stderr.log` 17:12–17:19Z, base
  `send_deliveries` et `tracked_requests`.
- `crates/bridget-daemon/src/wrapper.rs` (1518-1535, 1712-1716, 1978-2010),
  `crates/bridget-transport/src/tmux.rs`, `crates/bridget-daemon/src/attach.rs`
  (5452-5500), `specs/089-communication-core/implementation.md` (recette Claude
  rouge), `specs/090-codex-interactif/plan.md` (schéma interactif de référence).
