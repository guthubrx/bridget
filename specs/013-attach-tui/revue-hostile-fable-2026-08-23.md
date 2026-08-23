# Revue hostile du lot 013 — fable-reviewer, 2026-08-23 (mission Maicie 7cc14c8f)

Commits revus : 6931879, 508405b, dd695e7, 91643b3, 2d24ccd contre D-1301..D-1308.
Verdict : CHANGES REQUESTED. Fichiers dans crates/bridget-daemon/src sauf mention.

1. [HIGH — régression inter-commits] Écho de saisie non-TTY sans garde
   raw_terminal. attach.rs:1448,1476,1484 : handle_input_byte appelle
   renderer.input_changed inconditionnellement ; attach.rs:852-862 + 988-991 :
   en tty_output=false les octets legacy sont TOUJOURS écrits sur stdout.
   Avant le lot : `if raw_terminal { write_input_bytes }`. Cas stdin=pipe +
   stdout=pipe : stdout reçoit l'écho (\r\n, octets, \x08 \x08) — violation
   FR-001/SC-002. Introduit par 2d24ccd (écho déplacé depuis 6931879, garde
   perdue). Le test saisie pipée ne le voit pas : test_renderer_sender droppe
   le récepteur (attach.rs:~1993), sends silencieux. Fix : propager
   raw_terminal dans RendererSender ou garder la garde à l'appel.

2. [HIGH] Hauteur du terminal ignorée. clear() attach.rs:1132 remonte
   rendered_rows via ESC[1A, curseur buté en haut d'écran ; terminal_columns
   (attach.rs:1161) ne lit que ws_col. Bloc live > hauteur (D-1304 = 400
   lignes vs 24 !) → effacement partiel + réécriture intégrale à chaque
   token → duplication massive dans le scrollback. Contradiction D-1304 vs
   D-1305. Arbitrage référent : rendu en place borné à la FENÊTRE VISIBLE
   (queue du bloc, min(contenu, hauteur-2)) ; ce qui défile au-dessus s'écrit
   une fois, jamais ré-effacé ; lire ws_row.

3. [MEDIUM] Redraw O(n) par token → O(n²) par tour (Art. XVIII, hot path).
   block.lines() (attach.rs:770) clone header+response+details, et
   render_prefixed (attach.rs:1832) re-sanitise jusqu'à 64 Kio à chaque
   update. Aucune coalescence des RendererCommand::Event (D-1306 promettait
   borné ET coalescé — seul InputChanged l'est). 64 events en file = 64
   redraws complets.

4. [MEDIUM] Saturation partiellement tenue. RendererSender::event
   (attach.rs:848) est un send BLOQUANT appelé aussi par le thread principal
   via render_expired_sends (attach.rs:1491, boucle drive_interactive) :
   canal plein → lecture stdin (dont Ctrl-C 0x03) suspendue. Si le renderer
   bloque en write stdout (XOFF), renderer.stop() joint indéfiniment →
   termios jamais restauré. Le test saturation (attach.rs:~2560) construit un
   canal artisanal sans renderer_loop actif — partiellement décoratif.

5. [MEDIUM] FR-003 incomplet : statut accordé/refusé absent de la ligne
   d'outil. render_journal_event (attach.rs:1632-1651) n'affiche que
   titre+summary ; le statut vit dans un événement permission séparé non
   corrélé. tool_call_journal_payload (transport/acp.rs:1209) ne capture
   aucun champ status des tool_call_update.

6. [MEDIUM] Troncatures incohérentes : TurnBlock 64 Kio/400 lignes
   (attach.rs:30-31) mais lines() rend via render_prefixed borné 16 Kio
   (attach.rs:22,1838) → 3/4 du tampon jamais affichés, deux marqueurs
   différents, le marqueur D-1304 peut être coupé par la borne 16 Kio.

7. [LOW] append_bounded (attach.rs:749) rejette la valeur ENTIÈRE au premier
   dépassement (même avec 63 Kio libres) et sous-compte les lignes des
   appends sans \n.

8. [LOW] Bloc démarré sans turn_start (reconnexion en cours de tour) :
   header « ??:?? {agent} → » (attach.rs:1040-1046) même pour un tour humain,
   heure réelle du premier événement ignorée.

9. [LOW] D-1301 « golden quel que soit stdin » faux tel qu'écrit : la sortie
   non-TTY dépend de stdin. Test matrice (attach.rs:~2148) tautologique
   (asserte que le constructeur stocke ses arguments). Reformuler : « chemin
   de rendu inchangé ».

10. [LOW] Resize : refresh_geometry (attach.rs:1122) efface avec les
    rendered_rows de l'ANCIENNE largeur après rewrap terminal → rangées
    résiduelles possibles au rétrécissement, non testé.

11. [INFO — hors lot, PRÉEXISTANT sur main] managed_parity_test::
    prompt_reduit_rejoue_le_corpus_dans_la_meme_session ROUGE (assertion
    in_reply_to vide, tests/managed_parity_test.rs:826), reproduit 2/2 sur
    branche ET main — vraisemblablement retombée de l'ajout in_reply_to
    (d00d4f3+). À traiter avant tout /log, tâche séparée.

POINTS TENUS : D-1302 capture source + additif v1 + fixture hostile ✓ ;
D-1308 sanitisation au rendu ✓ ; D-1303 clés/flushs/continuité ✓ ; un seul
écrivain stdout, verrou screen supprimé ✓ ; FR-002 fermeture sur terminal
explicite seul ✓ ; FR-005 zéro re-streaming aux octets ✓ ; visual_rows
Unicode ✓ ; 263/263 daemon + transport verts.
