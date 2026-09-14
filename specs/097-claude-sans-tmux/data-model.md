# Modèle 097

## Entités existantes réutilisées

- **Présence** (`Register` → `AgentInfo`) : `agent_type = claude`, `transport =
  claude_pty`, `mode = cli`, `location = None`, `channel = unix|ssh-unix`.
  Aucun champ ajouté.
- **Remise** (`send_deliveries`) : phases `dispatching → acked | indeterminate`
  inchangées ; l'accusé part après écriture PTY réussie, `indeterminate` sur
  échec d'écriture ou enfant mort.
- **Journal de session** (`sessions/<uuid>/<date>.jsonl`) : événements
  existants `turn_start` (Bridget) ; ajoutés depuis le transcript Claude avec
  le même vocabulaire : `turn_start {from:"human"}`, `update {kind:"text"}`
  (assistant), `turn_end` ; corps borné (même plafond que l'injection), sans
  octets de terminal.

## Objets internes nouveaux

- `PtySession` (daemon) : `master: OwnedFd`, `child: Child`, `saved_termios`,
  threads de relais, drapeau d'arrêt. Cycle : `open → spawn → relay → wait →
  restore`. Invariant : `restore` s'exécute exactement une fois, y compris sur
  panique ou signal.
- `PtyTransport` (transport) : `writer` = duplicata du maître, `child_pid`.
  `deliver` = validation → collage encadré → digestion bornée → `\r`.

## Transitions

Lancement : terminal vérifié → PTY ouvert → enfant lancé → Register (cli,
claude_pty) → JournalReady → relais. Un échec avant Register n'a produit aucune
présence. Un échec après Register termine l'enfant et publie la déconnexion.
