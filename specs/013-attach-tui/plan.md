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

## D-1304 — Bornes mémoire (obj. 5)
Tampon par bloc borné (64 Kio / 400 lignes) ; troncature VISIBLE
(`… tronqué, N lignes`) jamais silencieuse.

## D-1305 — Resize (obj. 6)
Politique : largeur recapturée avant chaque rendu ; si un bloc est ouvert au
moment d'un changement de largeur, clôture propre + redessin du bloc courant.
Tests pseudo-TTY avec changement de largeur en cours de tour.

## D-1306 — Concurrence du renderer (obj. 7)
Propriétaire unique du rendu : un seul thread écrit stdout. Ordre de verrous
documenté ; AUCUNE I/O stdout ni attente socket sous le verrou `screen`
(l'état se copie sous verrou, le rendu s'exécute hors verrou). Corrige le
point existant attach.rs:1017.

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
