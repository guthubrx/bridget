# Contre-revue adverse — cxbridget — session 008

**Croisement** : relecture par un moteur distinct de celui de l'auteur

## Round 1 (spec.md seule)

**Date** : 2026-08-22 · **Verdict** : `BLOCKED` — 8 objections, toutes retenues
(8/8, 0 rejetée). Le relecteur juge le bornage 007/008/009 sain ; le blocage
porte sur le contrat de journal et le chemin distant.

| # | Objection | Retenue | Correction appliquée |
|---|---|---|---|
| 1 | le journal 007 (ts/event/message_id/detail) ne porte ni expéditeur, ni curseur, ni payload typé — FR-002/005/006 intestables | oui — **urgente, impacte 007** | schéma **v1 versionné** poussé dans `007/data-model.md` (`v`, `seq` continu à travers la rotation, `session_id`, payloads typés, écriture ligne+flush, fixtures lecteur) ; T706 mis à jour ; coderBridget notifié avant d'implémenter T706 |
| 2 | « ré-attach sans doublon » contradictoire avec « rejouer l'historique » | oui | FR-004 redéfinie : garantie à la jonction rejeu→suivi d'une même invocation via `seq` ; nouvelle invocation rejoue sa fenêtre ; reprise inter-invocations explicitement exclue |
| 3 | « lecteur pur » contredit l'envoi de messages ; US2 ne testait que le chemin heureux | oui | FR-003 bornée au plan d'observation (pas d'écriture/verrou/contre-pression, pas de commande ACP) ; FR-005 : chemin `Send` existant + affichage des 4 issues non heureuses ; scénarios 4-5 ajoutés à US2 |
| 4 | le distant était architecturalement indéfini (le tunnel ne rend pas le fichier distant lisible) | oui | FR-007 réécrite : extension applicative sur la socket fédérée — wrapper distant lecteur de son journal, daemon multiplexeur, instantané/curseur/suivi/terminaison ; SC-006 exige des répertoires réellement distincts (boucle SSH locale insuffisante) |
| 5 | cas réels du tail JSONL absents | oui | 5 edge cases ajoutés en scénarios d'acceptation (ligne partielle, ligne invalide, création tardive, rotation minuit avec continuité `seq`, troncature/renommage) |
| 6 | « sans perdre » / « peut résumer » / « coût nul » incompatibles | oui | intégrité du journal distinguée de la fidélité du rendu ; tampon lecteur borné sans contre-pression ; retard signalé et rejouable ; SC-005 → budget mesurable (dégradation p95 < 5 % sur N=20 tours) |
| 7 | SC-001 sous-défini | oui | mesure de la fin d'append au rendu, cadence fixée 10 evt/s sur 60 s, p95 < 1 s et max < 3 s en local, budget distant séparé (p95 < 3 s) |
| 8 | injection ANSI/OSC depuis les données, saisie corrompue, terminal non restauré | oui | FR-010 créée (neutralisation des contrôles issus des données, préservation de la ligne de saisie, restauration du terminal sur tous les chemins de sortie, fixtures hostiles) + SC-007 |

## Round 2

**Verdict** : `APPROVE_WITH_CHANGES` — BLOCKED levé, le plan 008 peut démarrer
en intégrant 4 résidus (tous retenus, « aucune nouvelle contre-revue longue
requise ») :

| # | Résidu | Correction appliquée |
|---|---|---|
| 1 | continuité de `seq` non garantie après redémarrage/crash du wrapper | contrat 007 durci : récupération du prochain `seq` depuis le dernier événement valide (cas fichier vide / ligne partielle / corrompue), tests et fixtures ajoutés à T706 |
| 2 | « mêmes capacités » distantes contredites par la relecture d'un équipier distant arrêté | FR-007 bornée « tant que le wrapper est connecté » + message d'indisponibilité défini ; relais persistant explicitement hors périmètre (recommandation du relecteur suivie) |
| 3 | `body_preview` insuffisant pour FR-006 (« voir le message ») | arbitré : `turn_start.body` **complet** au journal (le journal contient déjà les réponses intégrales) + permissions 0700/0600 exigées sur `sessions/` — appliqué au data-model 007 et à T706 |
| 4 | ligne invalide sans `seq` connaissable ; SC-005 bruité (p95 sur 20 tours LLM) | signalement par numéro de ligne + offset, `seq` inconnu ; SC-005 redéfini sur faux adaptateur déterministe, N ≥ 200, métrique instrumentée (latence d'append) |

Le plan 008 intégrera ces points dans ses décisions et sa matrice de tests.

## Round 3 (plan + data-model + contrat)

**Verdict** : `BLOCKED` — D-301 jugé défendable et les résidus du round 2 bien
intégrés, mais 7 objections de niveau contrat (1, 4, 5, 6 motivant le blocage).
**Toutes retenues (7/7)** :

| # | Objection | Correction appliquée |
|---|---|---|
| 1 | aucune identité d'abonnement : rejeux croisés entre vues, fin d'ancienne génération fermant la nouvelle | `subscription_id` opaque sur tout le plan de contrôle ; générations obsolètes ignorées ; cas de test imposés (contrat) |
| 2 | `from_seq` seul rend `Today`/`--depuis date` impossibles (le client ne lit pas le fichier) ; rattrapage inobservable | sélecteur typé `Today`/`Seq(incl.)`/`Date` résolu par le wrapper + `SnapshotCaughtUp { through_seq }` |
| 3 | booléen « au moins un abonné » insuffisant ; rejeu d'historique dans le thread d'écoute = blocage de Deliver/heartbeat/cancel | worker de relais dédié à canal borné, curseur par abonnement, lecture par tranches bornées avec équité (D-302 réécrit) |
| 4 | 256 *événements* ne borne pas la mémoire (body complet) ; motif d'E/S sous verrou global du daemon (`daemon.rs:1334-1351`) → une vue lente fige le daemon | bornes en **octets** (1 Mio, frame 256 Kio), handler qui ne fait qu'enfiler, écrivain dédié par vue avec délai + fermeture motivée, `Gap` coalescé hors file inlâchable, relais jamais bloquant pour l'écrivain JSONL (D-305 réécrit) |
| 5 | FR-010 irréalisable en stdin canonique (écho des contrôles, ligne partielle inconnue) | raw mode minimal termios/libc, tampon applicatif, garde RAII (D-307 créé, R-104 révisée) |
| 6 | `DeliveryRejected` différé impossible sur connexion cli-send éphémère (`daemon.rs:1203-1207`) | envoi sur la connexion attach persistante, issue corrélée par `message_id`, cas « accusé puis rejet tardif » testé (D-308 créé) |
| 7 | « texte imprimable » vague (bidi/zero-width/format), newlines forgeant de faux préfixes | liste blanche Unicode précise (hors `Cc`/`Cf`), continuations indentées, longueur bornée, fixtures hostiles étendues |

## Round 4

**Verdict** : `BLOCKED` — les 7 objections du round 3 jugées matériellement
traitées ; 2 contradictions bloquantes + 4 précisions. **Toutes retenues
(6/6)** :

| # | Objection | Correction |
|---|---|---|
| 1 | frame max 256 Kio vs ligne v1 « telle quelle » non bornée : un événement volumineux n'avait aucun chemin, Gap+rejeu aurait bouclé | **fragmentation filaire** `{subscription_id, seq, offset, final}` (256 Kio par fragment), abandon du `seq` entier si un fragment est lâché, fixture > 256 Kio rejeu+live sans boucle |
| 2 | sémantique termios/signaux indéfinie : `ISIG` actif → SIGINT tue sans destructeurs (RAII décorative) | `ISIG` désactivé, Ctrl-C = `0x03` traité dans la boucle (restauration puis sortie), garde RAII en filet, tests pseudo-TTY + mode non-TTY dégradé |
| 3 | acceptation wrapper / transit de l'id / snapshot vide non définis | `Subscribed` émis après acceptation du wrapper, id propagé, `through_seq` optionnel (fenêtre vide) |
| 4 | canal de commandes wrapper sans politique de saturation (abonnement fantôme/fuite) | `try_send` plein → refus typé de Subscribe ; arrêt/désabonnement sur canal de contrôle séparé jamais refusé ; tests saturation |
| 5 | connexion attach multiplexée sans modèle de concurrence | lecteur socket unique + dispatch (`subscription_id`/`message_id`), writer sérialisé (motif D-204), unicité et nettoyage de `pending_send` à `End`/déconnexion |
| 6 | incohérence reprise « dernier reçu » vs `Seq(last_seq+1)` | aligné partout sur `Seq(last_seq + 1)` (`Seq` inclusif) |

Le relecteur confirme qu'après 1-2, le choix daemon local+distant et l'absence
de frontière snapshot/follow explicite sont acceptables (`subscription_id` +
`seq` + `SnapshotCaughtUp` suffisent).

## Round 5

**Verdict** : `APPROVE_WITH_CHANGES` — **BLOCKED levé**. 4 résidus à inscrire
avant tasks.md, tous retenus (4/4) :

| # | Résidu | Correction |
|---|---|---|
| 1 | nettoyage de `pending_send` à `End` incorrect (End ≠ fermeture de connexion ; rejet tardif après End possible) | retrait uniquement sur issue terminale / expiration bornée / fermeture réelle ; test End→réabonnement→rejet tardif corrélé |
| 2 | réassemblage d'une ligne arbitrairement grande non défini | borne de réassemblage distincte (4 Mio), dépassement → `Gap(event_too_large)` consommé jusqu'à `final` sans boucle ; `offset` en octets contigus depuis 0 ; un `Gap` invalide l'assemblage partiel du `seq` |
| 3 | `Today`/`Date` ambigus entre hôtes | référentiel = fuseau et frontière de jour de l'hôte du wrapper ; refus typés (invalide/future/hors rétention) |
| 4 | reprise `Seq(last_seq+1)` inapplicable après snapshot vide | sélecteur initial mémorisé, utilisé pour la reprise sans `last_seq` ; test snapshot vide→End→réabonnement→premier événement une seule fois |

**Bilan 008 à ce stade** : 5 rounds, 29 objections, 29 retenues, 0 rejetée.
**Suite actée par le relecteur** : reuse-audit puis tasks.md après livraison de
007-T706.

## Round 6 (tasks.md + reuse-audit)

**Verdict** : `APPROVE_WITH_CHANGES` — 7 points, tous retenus (7/7) :
(1) `reply=false` accepté et inscrit dans la spec (FR-005/SC-003) **+** table
daemon `pending_attach_sends` exigée pour router les rejets tardifs (sans elle
US2-5 irréalisable) → T804b ; (2) rôle attach à frontière de confiance fermée
(matrice de messages, identité forcée humain, anti-usurpation de `from`,
invisible dans `who`) → T801/T802/T804a, entité de spec corrigée ; (3)
T803-T806 scindées en a/b (Article XV) ; (4) banc SC-001 distant ajouté à
T809 ; (5) quatre scénarios contractuels ajoutés aux observables (deux vues à
flux identique + fermeture sans résidu, snapshot vide→End→réabonnement
exactement-une-fois, fragmentation >256 Kio et >4 Mio sans boucle, saisie
intacte à l'octet pendant événement hostile) ; (6) observable T803b rendu
falsifiable (barrières + sonde zéro-lecture) ; (7) base du reuse-audit
corrigée (`48c3bf7`) + gate T810 d'intégration de la 007 finale.

**Tasks 008 : 12 tâches (avec découpages), prêtes à déléguer** dès la fin de
l'implémentation 007. **Bilan conception 008 : 6 rounds, 36 objections, 36
retenues, 0 rejetée.**
