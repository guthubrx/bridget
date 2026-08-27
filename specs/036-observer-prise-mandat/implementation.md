# Mise en œuvre 036 — Observer la prise d'un mandat

## Gel et portée

- base : `2078a59bcf43ae8d6b5b9fd7a8fcf14fc26ba88c` ;
- branche : `session-036-observer-prise-mandat` ;
- production modifiée : `scripts/bridget-idle.py` uniquement ;
- remise, protocole, registre, ledger et producteurs de journaux : inchangés ;
- portée de la sonde : traces natives locales uniquement.

## Mesures qui fondent le calcul

Le spécimen rc5 conserve les deux sources et leurs horodatages :

- `/home/moi/.cache/bridget/sessions/rc5/2026-08-27.jsonl:8` : injection
  `2026-08-27T09:49:24Z` ;
- `/home/moi/.codex/sessions/2026/08/25/rollout-2026-08-25T15-13-15-01a0390d-b893-7f82-b11a-3116dcfcdd21.jsonl:8444` :
  `task_started` à `2026-08-27T10:01:24.444Z` ;
- même fichier, ligne 8446 : entrée utilisateur acceptée à
  `2026-08-27T10:01:24.857Z`.

Le délai réel est de 720,857 secondes. Les démarrages Codex rapides mesurés
avaient un maximum de 5,421 secondes ; le maximum Claude observé, file incluse,
était de 51,083 secondes. Le seuil de 60 secondes reste réservé à la population
où le client était au repos à l'injection.

La relecture hostile du premier tir réel a trouvé un faux positif avant
commit :

- `cartae0` : `task_started` à `10:03:53.760Z`, puis injection à
  `10:04:24Z`, sans borne terminale intermédiaire ;
- `rc7` : `task_complete` à `10:39:46.868Z`, puis injection à
  `10:40:17Z`.

Le premier est désormais `REMISE_PENDANT_TOUR_ACTIF` tant qu'aucune
acceptation ne le clôt ; le second reste éligible à `MANDAT_NON_SOUMIS`.

## Univers de contrôles relisté

La base contient 23 contrôles nommés et un agrégat. Liste brute :

```text
controle_positif_details_dangereux_presents
lecture_bornes_tour_et_incertitudes
controle_positif_ancien_rouge
controle_positif_trois_fantomes_omis
partition_correctif
controle_positif_bloques_deux_sens
borne_terminale_independante_du_backlog
semantique_correctif
tour_ouvert_65min_reste_occupe
maicie_copie_sidecars_intacts
backlog_copie_ledger_join_seule
branche_non_fusionnee_presente
branche_fusionnee_absente_apres_presence
branche_en_conflit_visible
branche_base_perimee_visible
refs_locales_sans_fetch
depot_git_lecture_seule
backlog_branches_timeout_indisponible
backlog_branches_depot_invalide_indisponible
cli_texte_branches_et_limites
rendu_texte_details_inertes
cli_timeout_sans_liste_partielle
json_diagnostic_valide_et_inerte_et_backlog_branches
test-bridget-idle: checks
```

Résultat base : **23 passés / 0 échoué**, agrégat passé.

La tête contient 38 contrôles nommés et un agrégat. Liste brute :

```text
controle_positif_details_dangereux_presents
lecture_bornes_tour_et_incertitudes
prise_specimen_720s_non_soumise
prise_sans_marqueur_visuel_non_soumise
prise_saine_interdit_faux_positif
prise_claude_enqueue_remove_et_direct_discrimines
steering_cartae0_nomme_et_controle_idle_rc7_conserve
prise_inobservable_cardinal_non_muet
prise_inobservable_sources_invalides
contenu_trace_non_projete
decouverte_trace_codex_par_processus
decouverte_codex_refuse_sous_agent_et_ambiguite
decouverte_trace_claude_par_projet
controle_positif_ancien_rouge
controle_positif_trois_fantomes_omis
partition_correctif
prise_independante_de_la_copie_maicie_locale
categories_prise_partitionnees_deux_sens
prise_inobservable_distincte_de_tout_bloque
populations_idle_et_steering_disjointes
controle_positif_bloques_deux_sens
borne_terminale_independante_du_backlog
semantique_correctif
tour_ouvert_65min_reste_occupe
maicie_copie_sidecars_intacts
backlog_copie_ledger_join_seule
branche_non_fusionnee_presente
branche_fusionnee_absente_apres_presence
branche_en_conflit_visible
branche_base_perimee_visible
refs_locales_sans_fetch
depot_git_lecture_seule
backlog_branches_timeout_indisponible
backlog_branches_depot_invalide_indisponible
cli_texte_branches_et_limites
rendu_texte_details_inertes
cli_timeout_sans_liste_partielle
json_diagnostic_valide_et_inerte_et_backlog_branches
test-bridget-idle: checks
```

Résultat tête : **38 passés / 0 échoué**, agrégat passé. La partition de
fixture porte 28 agents, sans omission ni recouvrement.

## Mutants opposés

Les mutants ont été appliqués après le correctif complet, puis restaurés avant
le tir nominal :

1. remplacer `MANDAT_NON_SOUMIS` par `PRISE_ACCEPTEE` dans la branche échue :
   mort dans `prise_specimen_720s_non_soumise`, après lecture de 3 lignes,
   zéro acceptation corrélée ;
2. remplacer `PRISE_ACCEPTEE` par `MANDAT_NON_SOUMIS` dans la branche corrélée :
   mort dans `prise_saine_interdit_faux_positif`, après lecture de 3 lignes,
   une acceptation corrélée ;
3. neutraliser la condition `client_state_at_injection == "active"` : mort
   dans `steering_cartae0_nomme_et_controle_idle_rc7_conserve`, avec
   `client_state_at_injection=active`, 2 lignes lues et aucune acceptation.

La restauration finale est attestée par :

```text
f70d1c408954710773bd619a29cf2e78b46068fcab153c89246860455a6055e3  scripts/bridget-idle.py
00cbca0616da6ea738ed6a35dfa77feb6dd0a8d96edbbc84dbc9061ad65cd625  scripts/test-bridget-idle.sh
```

## Tir réel final sur Cartae

Commande productive :

```text
python3 scripts/bridget-idle.py --json
```

Projection brute bornée du dernier tir :

```json
{"daemon_count":14,"intake":{"scope":"local-only","eligible_count":10,"available_count":9,"records_read":59696},"maicie":{"state":"available","occupied_count":0},"partition":"partition ok (14 agents)","mandats_non_soumis":[{"name":"rc7","state":"MANDAT_NON_SOUMIS","client_state_at_injection":"idle","records_read":258,"matching_acceptances":0,"age_secs":248}],"remises_pendant_tour_actif":[],"prises_inobservables":[{"name":"rc1","state":"PRISE_INOBSERVABLE","source_state":"unavailable","records_read":0,"reason":"trace-codex-active-absente"}],"elapsed_seconds":1.318}
```

Le tir prouve que l'observateur n'est pas conditionné par la copie Maicie
locale vide. La projection ne contient aucun corps de conversation.

## Gates

- `python3 -m py_compile scripts/bridget-idle.py` : vert, cache hors dépôt ;
- `bash -n scripts/test-bridget-idle.sh` : vert ;
- `ruff check scripts/bridget-idle.py` : vert ;
- `git diff --check` : vert ;
- harnais base : 23 passés / 0 échoué, agrégat passé ;
- harnais tête : 38 passés / 0 échoué, agrégat passé.

## Revue hostile : 4 problèmes trouvés, 4 corrigés

| # | Problème | Sévérité | Correction |
|---|---|---|---|
| 1 | L'inventaire des tests ne comptait qu'une forme de `print` | Moyenne | Univers relisté par toute ligne de résultat `OK`, avec liste brute |
| 2 | La copie Maicie locale vide rendait l'observation entièrement muette | Haute | Observation de toutes les injections interactives ouvertes vues par le daemon ; contrôle sans mission locale |
| 3 | Le seuil idle accusait une remise pendant un tour actif (`cartae0`) | Haute | État natif à l'injection, catégorie `REMISE_PENDANT_TOUR_ACTIF`, contrôle opposé `rc7` |
| 4 | Une trace principale valide pouvait masquer un second descripteur de rôle inconnu | Moyenne | Tout rôle Codex inconnu et toute session Claude indéterminable rendent la découverte inobservable |

## Non mesuré et limite durable

- aucun tir macOS ;
- aucun relais de trace entre Cartae et la ronde centrale ;
- aucun redémarrage ni pilotage automatique d'un agent bloqué ;
- aucune calibration d'un état fournisseur postérieur à l'acceptation client ;
- une trace native distante, absente, vide, invalide ou ambiguë reste
  `PRISE_INOBSERVABLE`, jamais saine ni bloquée par supposition.
