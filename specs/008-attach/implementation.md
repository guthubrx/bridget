# Journal d'implémentation — Vue attach

## T808 — Budget d'observation SC-005

- **Date** : 2026-08-22
- **Banc réel** : `sc005_attach_budget::sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`.
- **Adaptateur** : faux adaptateur ACP stdio déterministe, selon le mécanisme
  introduit par T704. Deux harness réels et simultanés exécutent chacun 100
  tours de chauffe puis 1 000 tours mesurés : l'un sans vue, l'autre avec deux
  vues. Les tours sont entrelacés en ordre AB/BA afin que chaque condition
  subisse le même état de chauffe et la même charge machine.
- **Métrique** : durée physique de `SessionJournal::append_entry`, relevée dans
  le thread écrivain du journal par une instrumentation limitée aux tests et
  aux événements `turn_start`/`turn_end`. Dans le cas observé, deux connexions
  Unix réelles négocient `RoleHandshake(Attach)`, obtiennent `Subscribed` et
  consomment chaque frame pendant les 200 tours.

Le correctif remplace les tails JSONL redondants par un flux mémoire unique,
publié sans blocage après le flush puis multiplexé par le daemon. Le journal
reste l'autorité du rejeu ; une saturation mémoire produit un `Gap` coalescé
et un rattrapage disque. Le fichier du jour reste ouvert jusqu'à sa rotation.

Avant le correctif, le banc réel séquentiel avait exposé une régression
systématique : environ 97 µs contre 297 µs (+206 %) puis 90 µs contre
316 µs (+251 %). Le premier banc post-correctif restait sensible au
démarrage à froid parce qu'il mesurait ses deux conditions en blocs successifs.

Résultat final de six exécutions entrelacées consécutives :

| Campagne | Vues attach réelles | Échantillons d'append | p95 |
| --- | ---: | ---: | ---: |
| 1 | 0 | 2 000 | 24,958 µs |
| 1 | 2 | 2 000 | 24,250 µs |
| 2 | 0 | 2 000 | 22,958 µs |
| 2 | 2 | 2 000 | 22,417 µs |
| 3 | 0 | 2 000 | 23,375 µs |
| 3 | 2 | 2 000 | 23,958 µs |
| 4 | 0 | 2 000 | 23,959 µs |
| 4 | 2 | 2 000 | 23,291 µs |
| 5 | 0 | 2 000 | 21,542 µs |
| 5 | 2 | 2 000 | 21,875 µs |
| 6 | 0 | 2 000 | 22,292 µs |
| 6 | 2 | 2 000 | 23,375 µs |

Le critère SC-005 (< 5 %) passe dans les six exécutions. Les variations sont
respectivement de -2,8 %, -2,4 %, +2,5 %, -2,8 %, +1,5 % et +4,9 %. Le seuil
contractuel reste inchangé ; le timeout global de 60 secondes ne sert qu'à
faire échouer proprement un banc bloqué.

**Dérogation T806a** : cette tâche a été commitée tandis que l'ancien banc à
  vues simulées était invalidé. Le banc réel qui le remplace est maintenant
  vert après correction du chemin produit.

## T809 — Gate distant fédéré

- **Date** : 2026-08-22.
- **Isolement** : daemon de test lancé avec `HOME=/tmp/bg-attach` et socket
  `/tmp/bg-attach/.cache/bridget/bridget.sock`. Le tunnel SSH manuel publiait
  ce socket uniquement à `/home/moi/.cache/bridget/bridget.sock` sur
  `cartae.app:2222`; le daemon de production local est resté intact.
- **US1 et US2 observés** : le client distant a affiché `t809-acp ... acp ...
  connected` dans `bridget who`. `bridget attach t809-acp` a négocié
  l'abonnement, rejoué le journal (`historique rattrapé jusqu'à 796` puis 852),
  reçu le flux Codex, et une ligne saisie à distance a été réécrite en
  `humain`, accusée (`envoi 039473eadabd4 accepté`) et suivie de la réponse ACP
  et de `turn_end`.
- **Indisponibilité contrôlée** : après l'arrêt du seul wrapper de test
  `t809-acp`, `bridget attach t809-acp` distant a retourné le message motivé
  `équipier « t809-acp » inconnu ; aucun équipier ACP n'est actuellement
  attachable`, sans erreur brute.
- **Résultat strict** : la resynchronisation d'une vue attach persistante après
  coupure du tunnel n'a pas été démontrée : le wrapper ACP de remplacement est
  sorti avant sa reconnexion et l'ancien wrapper est resté enregistré comme
  présence obsolète. Le banc SC-001 distant de 60 s à 10 événements/s n'a donc
  pas été lancé; son p95 < 3 s n'est pas établi. T809 reste décochée.
- **Nettoyage** : tunnel, daemon de test, wrappers de test et socket distant de
  gate ont été arrêtés/supprimés. Aucune configuration persistante de
  fédération n'a été supprimée.

### Reprise de tunnel

Une vue attach distante a d'abord reçu le tour `resync avant coupure` avec
`turn_end`. Après arrêt du seul processus SSH `-R`, un nouveau `bridget who`
à distance a correctement signalé l'absence de socket. Au redémarrage du même
tunnel vers le daemon isolé, la vue persistante a affiché `abonnement actif`,
puis a envoyé et reçu le tour `resynchronisation réussie` avec `turn_end`.
La reprise de la connexion attach après coupure est ainsi démontrée.

### Banc distant SC-001

- **Topologie isolée** : le daemon de gate et son équipier ACP déterministe
  tournaient avec `HOME=/tmp/bg-t809-local`; le tunnel SSH `-R` temporaire
  publiait seulement son socket vers
  `/tmp/bg-t809-remote/.cache/bridget/bridget.sock` sur `cartae.app`. La vue
  distante utilisait `HOME=/tmp/bg-t809-remote`; aucun socket, daemon ou
  répertoire de production n'a été utilisé.
- **Méthode** : 600 appels `bridget send` locaux, cadencés toutes les 100 ms,
  ont porté `T809C seq=NNNN emitted_at=<ns>`. La vue attach distante a écrit
  son horodatage de rendu pour chaque ligne `turn_start` correspondante. La
  campagne a duré 59,940889 s.
- **Résultat** : 600/600 séquences distinctes reçues, zéro doublon ; minimum
  14,906880 ms, p95 (rang 570) **60,302080 ms**, maximum 250,861312 ms. Le p95
  est donc inférieur au seuil distant de 3 s.
- **Réserve de mesure** : les différences utilisent les horloges NTP des deux
  hôtes. Leur biais attendu, inférieur à 100 ms, reste négligeable au regard
  du seuil de 3 s ; il est toutefois inclus dans les valeurs rapportées.
- **Nettoyage** : le daemon, l'équipier, le tunnel et les sockets temporaires
  de gate ont été arrêtés après la campagne; aucune configuration persistante
  de fédération n'a été supprimée.

## T807 — Bancs locaux SC-001 et SC-002

- **Date** : 2026-08-22.
- **SC-001** : le banc `sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux`
  utilise le faux adaptateur ACP déterministe de T704, un writer JSONL, le
  wrapper et le daemon réels, puis deux connexions Unix négociant le rôle
  `Attach` et consommant les frames. L'instrumentation de test prélève
  l'horodatage juste après le flush de chaque ligne `turn_start`/`turn_end` et
  le corrèle à son `seq` rendu par la première vue. Sur 600 tours (1 200
  événements) cadencés à 10 tours/s pendant 60,92 s, le p95 append→rendu est
  de **11,888459 ms** et le maximum de **13,178 ms** : les seuils locaux
  respectifs de 1 s et 3 s sont satisfaits. Le timeout global de 75 s fait
  échouer proprement le banc en cas de blocage.
- **SC-002** : `sc002_rejeu_vers_suivi_traverse_la_rotation_sans_perte_ni_doublon`
  injecte l'événement `seq=5` dans le fichier de la veille, ouvre la vue et
  attend `SnapshotCaughtUp`, puis démarre un tour actif qui crée le fichier du
  jour et produit `seq=6,7` via le flux live. La suite finale strictement
  observée est `[5, 6, 7]` : aucune perte ni doublon à la bascule
  catch-up→live introduite par le relais mémoire, y compris à la rotation de
  fichier simulée.
