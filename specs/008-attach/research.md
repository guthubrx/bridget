# Research : `bridget attach` (session 008)

**Date** : 2026-08-22 · Rédigé pendant l'implémentation de la 007 ; à confronter
au verdict de contre-revue de la spec avant le plan.

## R-101 — Suivre le journal en continu, sans dépendance nouvelle

Le journal 007 est un JSONL append-only par agent et par jour. Trois voies pour
le suivre :

| Voie | Verdict |
|---|---|
| notification filesystem (inotify/kqueue via crate `notify`) | écartée : dépendance nouvelle + différences macOS/Linux, pour un gain de latence sans exigence (SC-001 tolère 1 s) |
| `tail -f` externe | écartée : process externe à superviser, parsing de sortie |
| **polling incrémental** : `stat` de la mtime, relecture depuis l'offset mémorisé | **retenue** — c'est le motif déjà éprouvé de la sonde runtime Codex (`wrapper.rs`, `RuntimeProbe` : stat quelques µs, relecture seulement sur changement). Période 300 ms → SC-001 (< 1 s) tenu avec ~3 stats/s, coût nul pour l'équipier (FR-003 : le lecteur ne touche que le fichier) |

Position de lecture = offset d'octets ; le ré-attach sans doublon (FR-004)
découle du rejeu depuis le début du fichier du jour puis suivi à l'offset.

## R-102 — L'attach distant : abonnement via le canal existant, pas de nouveau réseau

Contrainte FR-007 : pas de nouveau canal réseau. État des lieux : en
fédération (002), le wrapper distant est connecté au daemon via le tunnel SSH
inverse — il existe donc déjà un chemin bidirectionnel daemon↔wrapper pour
chaque équipier, local ou distant.

**Décision proposée (à trancher au plan)** : l'attach est un **abonnement
médié par le daemon**, identique en local et en distant :

1. le client `attach` demande au daemon l'abonnement au flux d'un agent ;
2. le daemon relaie la demande au wrapper concerné (nouvelle paire de messages
   de protocole : abonnement/désabonnement, événement de journal) ;
3. le wrapper rejoue la fenêtre demandée (à partir d'un curseur `seq` du schéma
   v1) puis pousse chaque nouvel événement (c'est lui qui écrit le journal — il
   connaît l'offset sans polling) ; la reprise après coupure repart du dernier
   `seq` reçu par le client ;
4. le daemon fanne vers tous les clients attachés (multi-vues FR-006 gratuit).

Avantages : un seul mécanisme pour local et distant (minimalisme), fan-out
naturel, aucun accès disque croisé. Coût assumé : les événements transitent par
le daemon même en local — volume faible (texte de tours), acceptable. La voie
« lecture directe du fichier local » (R-101) reste l'implémentation *interne*
du wrapper, pas celle du client.

Conséquence FR-003 : « lecteur pur » se mesure côté équipier (le wrapper pousse
depuis son thread d'écoute existant, jamais depuis le worker de tours) — SC-005
le vérifie.

## R-103 — L'émetteur humain existe déjà

Les envois CLI apparaissent déjà sous identité éphémère `cli-send-<pid>` dans
le ledger, et le daemon les distingue des wrappers (contrôle `--reply`,
`daemon.rs:1212`). La vue attachée réutilise ce chemin d'envoi tel quel — le
marquage « humain » est une présentation à l'affichage (préfixe `cli-send-` ou
nom réservé), pas une identité nouvelle. Zéro mécanisme d'identité ajouté.

## R-104 — Rendu terminal : texte ANSI simple + raw mode minimal (révisé)

FR-009 exclut les TUI lourds. Première version de cette recherche : « stdin
ligne à ligne, pas de raw mode » — **révisée après contre-revue du plan**
(objection confirmée : en mode canonique, l'écho du terminal affiche les
contrôles hostiles avant toute neutralisation, et la ligne partielle est
inconnue au moment de redessiner — FR-010 serait irréalisable). Décision : raw
mode **minimal** via `termios` (`libc`, déjà dépendance du workspace ; pas de
`crossterm`), tampon de saisie applicatif, garde RAII de restauration. Le rendu
reste des lignes horodatées ANSI de base ; l'édition reste volontairement
pauvre. Détail : plan D-307, data-model « Saisie et terminal ».

## R-105 — Périmètre des nouvelles variantes de protocole

Trois ajouts pressentis (noms définitifs au plan) : abonnement, désabonnement
(implicite à la déconnexion du client), événement de flux. Ils suivent le
précédent D-209 de la 007 (variantes typées plutôt que texte). Le reuse-audit
008 devra confronter ces ajouts aux variantes 007 livrées d'ici là
(`CancelDelivery`/`DeliveryRejected`, état de tour) pour mutualiser ce qui peut
l'être.
