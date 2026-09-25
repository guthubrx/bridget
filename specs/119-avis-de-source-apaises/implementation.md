# Journal 119 - Les avis d'état de source ne réveillent plus pour rien

- **Base** : main `6b920ad4` - **Date** : 2026-09-25 - **Statut** : Implemented, livré le 2026-09-25 06:04

## Mesures
- Nuit du 25/09 : 222 avis d'observation vers `sol_city_ai`, 197 tours démarrés (GPT-6 astra,
  réflexion élevée), 37 réponses de simple commentaire. Base T3 lue en lecture seule.
- T3 propose `thread.activity.append` sans tour, mais l'activité n'entre pas dans le contexte de
  l'agent : piste écartée, l'avis doit lui parvenir.

## Correction
- `observation.rs` : `refresh_sources` met à jour l'état consultable sans émettre ; nouveaux champs
  `announced_count`, `changed_at`, `diverged_since`, `flips` ; `due_source_notices` émet après
  SOURCE_NOTICE_GRACE (30 s) de stabilité, reste muet si l'état stable est celui déjà annoncé, et
  signale « source instable » (avec `changes`) au plus tard SOURCE_NOTICE_MAX_DELAY (5 min) après
  le premier écart. `set_source` / `remove_source` ne rendent plus de notifications.
- `daemon.rs` : quatre sites ne transmettent plus d'avis immédiat ; la boucle d'une seconde des
  rappels appelle `due_source_notices`.
- Premier jet corrigé par son propre test : un retour à l'état annoncé effaçait l'écart, si bien
  qu'une source qui clignote n'était jamais signalée.

## Vérifications
- `spec119_aller_retour_bref_de_source_sans_avis`, `spec119_source_qui_clignote_avis_borne_par_periode`
  (10 min de bascules toutes les 3 s : un seul avis « source instable ») ; 45 tests d'observation.
- Recette complète à charge 62-65 : 1545 réussis, 7 échecs. `spec101_real_daemon_journal_share…`
  attendait l'avis immédiat (contrat changé, test adapté, vert). `managed_parity_test` 7/7 et
  `core_089_content_test` 7/7 relancés seuls. `search_104_test` s2x : 5 échecs sur 10 sur la 119
  ET 5 sur 10 sur main, en essais alternés : instabilité préexistante.

## Livraison
- 06:04 : fusion `955b0339`, construction, relance du daemon. 31 agents avant, 31 à t+15 s ;
  build-id `955b03399b21`, plus d'alerte « daemon périmé ».
- `sol_city_ai` : quatre avis « daemon redémarré », réabonnement autonome au worker 3 en 35 s ;
  les trois workers Claude restent sans événement d'observation (dernier tour sans origine
  prouvée), contournés par des demandes de bilan.
