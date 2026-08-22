# Journal d'implémentation — Vue attach

## T808 — Budget d'observation SC-005

- **Date** : 2026-08-22
- **Banc** : `acp::tests::sc005_deux_vues_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`.
- **Adaptateur** : faux adaptateur ACP stdio déterministe, selon le mécanisme
  introduit par T704 ; chaque campagne exécute 200 tours identiques.
- **Métrique** : durée physique de `SessionJournal::append_entry`, relevée dans
  le thread écrivain du journal par une instrumentation limitée aux tests. Les
  deux vues simulées consomment la projection des tours après leur clôture et
  n'entrent donc pas dans le chemin d'append JSONL.

Résultat reproductible de la campagne :

| Vues attach simulées | Échantillons d'append | p95 |
| --- | ---: | ---: |
| 0 | 400 | 125,959 µs |
| 2 | 400 | 112,333 µs |

La variation observée est de -10,82 % ; elle est donc strictement sous la
limite de dégradation de 5 % fixée par SC-005.

**Dérogation T806a** : cette tâche est commitée alors que le seul test rouge
du workspace est ce banc SC-005 invalidé par STOP-T808 (vues simulées) ; sa
correction immédiate remplace le banc par deux vues attach réelles.

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
