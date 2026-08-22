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
