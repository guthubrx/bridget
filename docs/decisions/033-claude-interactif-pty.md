# ADR 033 — Claude Code interactif dans un pseudo-terminal possédé par le wrapper

Date : 2026-09-13. Statut : proposé, validé par les gates 097 (harnais sous PTY et recette réelle du Claude géré).

## Contexte

Le 2026-09-13, deux envois d'un agent Codex vers un agent Claude lancé par
`bridget claude --resume` dans iTerm sont restés en phase indéterminée : le
wrapper interactif ne savait remettre un message que par `tmux paste-buffer`,
et s'enregistrait pourtant `connected | tmux` sans pane. L'utilisateur refuse
tmux. Codex a déjà une voie interactive sans tmux (ADR 030, app-server) ;
Claude Code n'expose pas de protocole équivalent pour sa TUI. La recette du
Claude géré était restée rouge depuis 089 parce que le CLI officiel ne retrouve
sa session d'abonnement qu'avec le HOME réel et `USER`.

## Décision

1. `bridget claude` ouvre un pseudo-terminal (`libc::openpty`) que le wrapper
   possède : Claude Code y démarre comme chef de session avec ce PTY pour
   terminal de contrôle ; le terminal de l'humain passe en mode brut complet et
   est restauré exactement une fois (`Drop`, signaux, fin normale) ; frappe,
   affichage et `SIGWINCH` sont relayés octet pour octet.
2. `PtyTransport` implémente le trait `Transport` existant : validation de
   contenu partagée avec tmux, `wrap_envelope`, collage encadré
   (`ESC[200~ … ESC[201~`), délai de digestion borné, retour chariot, écriture
   bornée par `poll`. Le succès d'écriture est l'observable de remise, comme
   pour tmux ; aucune lecture d'écran.
3. Présence `mode = cli`, `transport = claude_pty` : aucune variante de
   protocole ni trame nouvelle, même schéma que Codex interactif. Attach reste
   admis ; le journal relaie les tours humain et assistant depuis le transcript
   déjà localisé par la sonde modèle/effort, dans le vocabulaire rendu par attach.
4. Garde-fous : refus sans terminal (alternative `bridget spawn claude`) ;
   les alias restés tmux (`gemini`, agents personnalisés) sont refusés au
   lancement sans pane attesté au lieu de s'enregistrer joignables ; le bypass
   de permissions implicite pour Claude interactif est retiré (l'humain décide).
5. Claude géré : la recette réelle passe HOME réel et `USER` au CLI officiel,
   l'état Bridget restant privé ; la configuration utilisateur est neutralisée
   par `--setting-sources ""`, `--strict-mcp-config`, `--tools ""`. Aucune clé
   d'API, aucune lecture de trousseau par les tests.

## Conséquences

Positives : une session Claude Code lancée dans n'importe quel terminal
reçoit les messages ; aucune dépendance à un serveur tmux partagé ; aucune
dépendance Cargo nouvelle ; un fournisseur tué par signal sort `128 + n`.

Négatives : la remise ne vérifie pas l'état du composer (tmux le sondait par
capture d'écran) ; le collage encadré suppose que Claude Code l'active, ce que
le harnais vérifie sur les octets mais que seule la recette réelle prouve ; le
transcript reste la source des tours assistant, avec le retard de sa sonde.

Écarté : maintenir tmux pour Claude en parallèle ; nouvelle variante
`PresenceMode::Pty` ; dépendances `portable-pty`/`nix` ; jeton OAuth en variable
d'environnement.
