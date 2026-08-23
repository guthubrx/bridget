# Plan 013 — Vue attach lisible (répond à la contre-revue cxbridget, 10 pts)

## D-1301 — Détection de mode (obj. 2)
Le rendu TTY dépend de **stdout** ; le raw mode 008 dépend de **stdin**. Les
deux sont indépendants : matrice complète stdin×stdout (TTY/pipe) = 4 cas
testés. Golden byte-à-byte sur stdout non-TTY quel que soit stdin.

## D-1302 — Capture du titre d'outil à la source (obj. 3)
Le titre réel ACP n'est lu aujourd'hui qu'au chemin permission
(acp.rs:1234) et le journal ne garde que content/name (acp.rs:866-874) : la
correction est au **journal** (mapping title→name→kind persistant, fixture
hostile), le renderer n'est que le consommateur. Étend le périmètre 013 à ce
mapping côté `bridget-transport` (petit, versionné journal v1 compatible :
champ additif optionnel).

## D-1303 — Clé de bloc et bascule replay→live (obj. 4)
Clé de bloc = `session_id+message_id`. Un tour ouvert en rattrapage et clos
en live reste UN bloc (ni doublon ni trou) ; test dédié de la bascule.
Évacuation des blocs sans `turn_end` : `Gap`, `End` et ligne corrompue
flushent TOUJOURS (« tour incomplet » visible, borne conservée) ;
`SnapshotCaughtUp` privilégie la CONTINUITÉ — marqueur inséré dans le bloc,
qui reste ouvert si sa clé est corrélable en live, flush incomplet seulement
sinon. Aucun bloc retenu indéfiniment ; une fixture par frontière.

## D-1304 — Bornes mémoire (obj. 5)
Tampon par bloc borné (64 Kio / 400 lignes) ; troncature VISIBLE
(`… tronqué, N lignes`) jamais silencieuse.

## D-1305 — Resize (obj. 6)
Politique : largeur recapturée avant chaque rendu ; si un bloc est ouvert au
moment d'un changement de largeur, l'ancienne représentation est EFFACÉE puis
le bloc TOUJOURS OUVERT est redessiné (la sémantique du tour ne change pas).
Tests pseudo-TTY avec changement de largeur en cours de tour.

## D-1306 — Concurrence du renderer (obj. 7)
Propriétaire unique du rendu : UN SEUL thread écrit stdout, alimenté par un
canal `RendererCommand` BORNÉ et COALESCÉ ; dès T1301, le thread de saisie
n'écrit plus jamais stdout directement (write_input_bytes passe par le
canal) ; les commandes Stop/restauration termios sont PRIORITAIRES et ne se
coalescent pas. Ordre de verrous documenté ; aucune I/O ni attente socket
sous le verrou `screen`. Corrige attach.rs:1017. Test : saturation du canal
sans perte de Ctrl-C ni de la restauration.

## D-1307 — Golden et exception versionnée (obj. 9)
Le golden non-TTY couvre le CHEMIN DE RENDU inchangé ; la correction du nom
d'outil (D-1302) modifie le payload transport : exception explicitement
versionnée dans le golden (délta documenté, pas de « inchangé » mensonger).

## D-1308 — Sécurité des titres (obj. 10)
title/name passent par le sanitizer existant (Cc/Cf/ANSI) avant tout rendu ;
test ESC/OSC/C1/bidi hostile. Le renderer reste consommateur d'AttachEvent :
zéro changement au fan-out/journal live ; bancs 008 rejoués tels quels.

## SC-001 précisé (obj. 8)
Géométrie 80×24, corpus déterministe de 3 tours (dont 1 appel d'outil),
lignes comptées avant/après, timeout global.
