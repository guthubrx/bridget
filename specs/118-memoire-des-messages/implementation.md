# Journal 118 — Un grand fil adaptateur de bureau n'est recopié qu'une fois

- **Base** : main `f165f47c` — **Date** : 2026-09-25 — **Statut** : Implemented, livré le 2026-09-25 03:36

## Diagnostic (données réelles)
- Pont : `bridget-observation` remis à `sol_city_ai` (e86d89f9) toutes les ~6 s ; `journal du fil
  106521cc : journal ACP saturé` toutes les ~3 s depuis 00:13 UTC.
- Catalogue des sources : `ea69b65b` (opus-city-glmkrkr4) bascule entre deux événements et aucun.
- Journal `ea69b65b/2026-09-25.jsonl` : 146 644 lignes, 1 363 identifiants distincts, jusqu'à 131
  copies chacun ; 336 Mo avec la veille. État du fil : `seen` plein à SEEN_BOUND (1 000).
- Mécanisme : éviction des plus anciens sous le nombre de messages visibles → recopie à chaque
  lecture → file d'écriture (256) pleine → journal jamais rattrapé. `refresh` annonçait la source
  prête avant la projection, puis indisponible après : deux notifications par lecture.
- L'instrumentation 117 a bien nommé le motif (« journal non rattrapé »).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/118-memoire-des-messages/implementation.md#extrait-1).

## Écarté
- `glmwrk1`, `glmwrk2`, `opus_city_ai` sans événement déclaré : dernier tour sans origine prouvée
  (fils Claude, réveils d'arrière-plan), comportement voulu de la 116 ; journaux petits.
- Journal de 336 Mo de `ea69b65b` : laissé en place, suppression à la décision de l'utilisateur.

## Vérifications
- `spec118_oubli_limite_a_ce_que_t3_ne_montre_plus`, `spec118_un_grand_fil_n_est_recopie_qu_une_fois`,
  `spec118_journal_en_retard_une_seule_annonce_sans_alternance` ; les deux derniers échouent sur
  l'ancien code (le dernier rend exactement `[prête, vide, prête, vide]`).
- fmt OK ; clippy `-D warnings` OK ; recette complète : **1550 réussis, 0 échec, 52 ignorés**.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/118-memoire-des-messages/implementation.md#extrait-2).
