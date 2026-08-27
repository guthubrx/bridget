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

Résultat de la base d'origine `2078a59` : **23 passés / 0 échoué**, agrégat
passé. Ce nombre ne décrit pas le parent immédiat du second amendement.

La tête amendée contient 43 contrôles nommés et un agrégat. Liste brute :

```text
controle_positif_details_dangereux_presents
lecture_bornes_tour_et_incertitudes
prise_specimen_720s_non_soumise
prise_sans_marqueur_visuel_non_soumise
prise_saine_interdit_faux_positif
correlation_identifiant_exact_positive
correlation_surensemble_negative
correlation_mention_non_correlee_negative
prise_claude_enqueue_remove_et_direct_discrimines
steering_cartae0_nomme_et_controle_idle_rc7_conserve
prise_inobservable_cardinal_non_muet
prise_inobservable_sources_invalides
contenu_trace_non_projete
decouverte_trace_codex_par_processus
decouverte_codex_ligne_partielle_fermee
chaine_prise_codex_ligne_partielle_inobservable
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

Résultat du parent immédiat `d2a720d` : **43 passés / 0 échoué**, agrégat
passé. Le relisting porte sur toutes les lignes de résultat, indépendamment de
la forme de l'instruction qui les imprime ; la liste brute ci-dessus contient
bien 43 témoins, et non 38.

La tête après contre-revue ajoute quatre témoins :

```text
decouverte_codex_permission_refusee_indisponible
maicie_participant_invalide_ferme_la_source
maicie_invalide_ferme_la_partition_complete
annuaire_invalide_refuse
```

Résultat final : **47 passés / 0 échoué**, agrégat passé. La partition de
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
4. remplacer l'égalité de l'identifiant canonique par l'ancienne appartenance
   de sous-chaîne : mort dans `correlation_surensemble_negative`, après le
   passage de `correlation_identifiant_exact_positive` ; `mcp-abc999` produit
   à tort une acceptation de `mcp-abc`.
5. rendre de nouveau `False` pour une première ligne Codex partielle : mort
   dans `decouverte_codex_ligne_partielle_fermee` ; la trace valide voisine
   serait sinon choisie avec `error=None`.
6. neutraliser la fermeture quand une délégation active est invalide : mort
   dans `maicie_participant_invalide_ferme_la_source`, après le contrôle sain
   `maicie_copie_sidecars_intacts` ; la charge partielle redevient `{'bob'}` ;
7. neutraliser la fermeture de l'annuaire : mort dans
   `annuaire_invalide_refuse`, après `cli_texte_branches_et_limites` vert ; la
   CLI rend à tort `partition ok (1 agents)` avec trois entrées reçues ;
8. ignorer de nouveau toute erreur d'accès aux descripteurs : mort dans
   `decouverte_codex_permission_refusee_indisponible`, après
   `decouverte_trace_codex_par_processus` vert ; la permission refusée devient
   à tort `trace-codex-active-absente`.

La restauration finale est attestée par :

```text
f134231956b5bd6645966c84fce205b86c34ea1550ff325ad08b906b8478dbad  scripts/bridget-idle.py
b7859ed57fe24e03336a08f610fe5c3c2571a1c52ff587481bf3e024207aae90  scripts/test-bridget-idle.sh
```

## Tir réel final sur Cartae

Commande productive :

```text
python3 scripts/bridget-idle.py --json
```

Projection bornée du tir d'amendement :

```json
{"daemon_count":15,"intake":{"scope":"local-only","eligible_count":10,"available_count":9,"records_read":63532},"maicie":{"state":"available","occupied_count":0},"partition":"partition ok (15 agents)","mandats_non_soumis":[],"remises_pendant_tour_actif":[{"name":"rc5","state":"REMISE_PENDANT_TOUR_ACTIF","client_state_at_injection":"active","records_read":10854,"matching_acceptances":0}],"prises_inobservables":[{"name":"rc1","state":"PRISE_INOBSERVABLE","source_state":"unavailable","records_read":0,"reason":"trace-codex-active-absente"}]}
```

Le tir prouve que l'observateur n'est pas conditionné par la copie Maicie
locale vide. Huit prises réelles sont corrélées exactement parmi les neuf
sources disponibles ; le neuvième cas est le steering `rc5` encore actif. La
projection ne contient aucun corps de conversation.

## Gates

- `python3 -m py_compile scripts/bridget-idle.py` : vert, cache hors dépôt ;
- `bash -n scripts/test-bridget-idle.sh` : vert ;
- `ruff check scripts/bridget-idle.py` : vert ;
- `git diff --check` : vert ;
- harnais base d'origine : 23 passés / 0 échoué, agrégat passé ;
- harnais parent immédiat : 43 passés / 0 échoué, agrégat passé ;
- harnais tête finale : 47 passés / 0 échoué, agrégat passé.

## Revue hostile : 9 problèmes trouvés, 9 corrigés

| # | Problème | Sévérité | Correction |
|---|---|---|---|
| 1 | L'inventaire des tests ne comptait qu'une forme de `print` | Moyenne | Univers relisté par toute ligne de résultat `OK`, avec liste brute |
| 2 | La copie Maicie locale vide rendait l'observation entièrement muette | Haute | Observation de toutes les injections interactives ouvertes vues par le daemon ; contrôle sans mission locale |
| 3 | Le seuil idle accusait une remise pendant un tour actif (`cartae0`) | Haute | État natif à l'injection, catégorie `REMISE_PENDANT_TOUR_ACTIF`, contrôle opposé `rc7` |
| 4 | Une trace principale valide pouvait masquer un second descripteur de rôle inconnu | Moyenne | Tout rôle Codex inconnu et toute session Claude indéterminable rendent la découverte inobservable |
| 5 | Une sous-chaîne ou une mention de l'identifiant fabriquait une prise inexistante | Haute | Extraction de l'identifiant de l'enveloppe canonique et égalité exacte, avec deux témoins négatifs |
| 6 | Une première ligne partielle disparaissait si une autre trace valide existait | Haute | État de rôle inconnu fermé, éprouvé à la découverte et dans la chaîne complète |
| 7 | Une délégation active au participant non textuel disparaissait de la charge | Bloquante | La source Maicie devient indisponible avec cardinal reçus/valides ; aucune absence de mission n'est ensuite déclarée |
| 8 | Des entrées d'annuaire invalides étaient filtrées avant une partition déclarée complète | Bloquante | Validation atomique de toutes les entrées et refus de toute vue partielle |
| 9 | Toute erreur d'accès à un descripteur était assimilée à une disparition | Haute | Seuls `ENOENT`, `ENOTDIR` et `ESRCH` sont tolérés ; les autres ferment la découverte |

## Non mesuré et limite durable

- aucun tir macOS ;
- aucun relais de trace entre Cartae et la ronde centrale ;
- aucun redémarrage ni pilotage automatique d'un agent bloqué ;
- aucune calibration d'un état fournisseur postérieur à l'acceptation client ;
- une trace native distante, absente, vide, invalide ou ambiguë reste
  `PRISE_INOBSERVABLE`, jamais saine ni bloquée par supposition.
