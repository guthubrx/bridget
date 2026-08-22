# Journal d'implémentation — Vue attach

## T808 — Budget d'observation SC-005

- **Date** : 2026-08-22
- **Banc réel** : `sc005_attach_budget::sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`.
- **Adaptateur** : faux adaptateur ACP stdio déterministe, selon le mécanisme
  introduit par T704 ; chaque phase exécute 20 tours de chauffe puis 200 tours
  identiques mesurés. Les phases sont appariées dans le même daemon et le même
  wrapper ; la charge avance sur la frontière d'append, sans être cadencée par
  la réception des vues.
- **Métrique** : durée physique de `SessionJournal::append_entry`, relevée dans
  le thread écrivain du journal par une instrumentation limitée aux tests et
  aux événements `turn_start`/`turn_end`. Dans le cas observé, deux connexions
  Unix réelles négocient `RoleHandshake(Attach)`, obtiennent `Subscribed` et
  consomment chaque frame pendant les 200 tours.

Le correctif remplace les tails JSONL redondants par un flux mémoire unique,
publié sans blocage après le flush puis multiplexé par le daemon. Le journal
reste l'autorité du rejeu ; une saturation mémoire produit un `Gap` coalescé
et un rattrapage disque. Le fichier du jour reste ouvert jusqu'à sa rotation.

Résultat reproductible de deux exécutions complètes :

| Campagne | Vues attach réelles | Échantillons d'append | p95 |
| --- | ---: | ---: | ---: |
| 1 | 0 | 400 | 30,792 µs |
| 1 | 2 | 400 | 30,333 µs |
| 2 | 0 | 400 | 38,292 µs |
| 2 | 2 | 400 | 28,208 µs |

Le critère SC-005 (< 5 %) passe dans les deux exécutions : les variations
mesurées sont respectivement de -1,5 % et -26,3 %. Le timeout global de
30 secondes et le seuil contractuel restent inchangés.

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

Le banc distant SC-001 demeure non exécuté : aucun générateur déterministe de
600 événements (10 événements/s pendant 60 s) ne fait partie de T809, et une
génération par l'adaptateur Codex mesurerait sa latence de modèle plutôt que la
latence append→rendu. Aucun p95 artificiel n'est consigné ; T809 reste
décochée jusqu'à un banc distant adapté.
