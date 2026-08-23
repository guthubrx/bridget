# Implémentation — Maicie v3

- Dérogation T002 : `sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill` est rouge hors périmètre Maicie, à cause d'un temporaire `fleet.json.0.tmp` non unique entre processus 009 ; le correctif est assigné séparément.
- Observation T006 : une seconde passe workspace a reproduit le flake hors
  périmètre `managed_parity_test::matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`
  (`Socket is not connected`, ligne 554). La première passe workspace et la
  relance isolée ont réussi ; aucun chemin T006 n'est présent dans ce test.
- Dérogation correctif domaine : les rouges T006 (#1 poison) et client (#4)
  proviennent des WIP légitimes respectifs de cxbridget et prospective ; les
  tests de contrat du domaine et son formatage sont verts, puis chaque lot
  revalide le workspace complet à son commit.
- T008 constate et corrèle les issues Bridget à l'objectif sans changer son
  état ; T015 portera la décision explicite et, si nécessaire, son écriture
  atomique dans le store.
- Cosmétique connue : `send_idempotent` sérialise l'enveloppe deux fois
  (validation puis émission) ; coût local négligeable, à considérer seulement
  lors d'une optimisation mesurée.

## Constat recette no 11 (2026-08-23, observation utilisateur)
La vue attach affiche « [outil] inconnu appel demandé » pour tout appel
d'outil ACP dont le kind n'est pas mappé, au lieu d'extraire le nom/titre
réel porté par l'événement (Read, Grep…). Sévérité mineure (observabilité) —
correctif : renderer attach = afficher title/name du tool_call ACP même pour
un kind inconnu. À assigner à la prochaine fenêtre libre.

## T015a — issues terminales attestées

Les refus, annulations et échecs locaux terminalisés par Bridget font passer
l'objectif à `a_evaluer` dans la transaction qui fige l'issue, avec une
décision `constater_issue` et la transition de délégation correspondante. Une
livraison `accepted` ne vaut ni réponse ni clôture.

T015b a été instruite par T017 : **Subscribe 008 seul ne permet pas de
corréler une réponse ni un timeout sans inférence locale**. Le journal public
atteste le `message_id` du prompt traité, mais pas l'`in_reply_to` de la
réponse émise ; le timeout est une issue de demande Bridget, non un fait ACP.
La boucle complète exige donc une identité Maicie joignable et une surface
publique Bridget de statut ou d'événements de demande corrélés. T018 affiche
les faits disponibles sans combler ces deux absences.

## T019 — délais passifs

La classe configurée est projetée vers le timeout Bridget et l'échéance
contractuelle immuable est affichée dans la sortie de délégation, y compris
pour un rejeu idempotent. Cette échéance ne produit ni minuterie ni transition
locale : seule une issue Bridget ou une consultation peut faire évoluer la
vue. T017/Subscribe confirme qu'elle ne peut pas être interprétée comme un
timeout : elle reste une valeur de contrat passivement affichée.

## T018 — sources affichées sans runtime caché

Chaque `maicie status` peut ouvrir, si `status_capture_budget_ms` est configuré,
une capture Attach strictement éphémère et bornée. La même échéance absolue
couvre négociation client, annuaire, souscription, fragments et
`SnapshotCaughtUp`; aucun fait runtime n'est écrit dans SQLite. Sans budget ou
à son épuisement, disponibilité, runtime et fraîcheur sont explicitement
`unavailable` avec un motif. Le statut sépare l'annuaire Bridget, les faits
ACP corrélés (`subscription_id`/`seq`), le flux `gap`/`ended`, les permissions
déjà auto-décidées, la remise locale durable, et le snapshot de demande qui
reste `unknown` faute de surface publique de statut corrélé. Cette séparation
met en œuvre la conclusion T015b sans simuler une réponse ou un timeout.

## T016 — gate MVP réel (2026-08-23)

Gate exécuté avec un daemon Bridget réel, un équipier ACP lancé par
`bridget spawn`, puis `maicie delegate`, `status` et `close` réels. La remise
est attestée dans le registre local Maicie (`accepted` + issue durable), tandis
que le snapshot transport reste honnêtement `unknown`. Mesure : 1 527 ms de la
délégation au statut accepté, 1 532 ms jusqu'à la clôture explicite.

Le premier `status` peut honnêtement constater `outcome_unknown` avec un
`delivery_id`, avant que l'ACK aval ne rende la même remise `accepted` ; le
gate attend cette convergence bornée. L'arrêt géré conserve ensuite la fiche
en `stopped` (historique de présence) et prouve l'extinction du PGID ACP.

Le premier passage a refusé `reply=true` avec `reply_sender_unavailable` :
Maicie est un client public durable, non un wrapper Bridget joignable. Le MVP
émet donc sans demande de réponse ; T015b devra fournir une identité Maicie
connectée avant d'activer une corrélation de réponse.

## Dérogation T017-2 levée

Le harnais `reprise_lente_sur_toutes_les_phases_reste_dans_le_budget_global`
frôlait volontairement l'échéance et le serveur pouvait observer un
`BrokenPipe` lorsque Maicie fermait légitimement la socket à l'expiration. Le
correctif de suivi maintient désormais la socket ouverte après le replay : la
mutation `connect_with_limits_until` vers `connect_with_limits` dépasse alors
la borne du test, tandis que l'échéance absolue correcte expire avant le
replay. Le protocole de mutation est consigné dans le test, qui redevient une
preuve discriminante.

## T020 — banc SC-008 : coût de `maicie status`

Le banc reproductible exécute la vraie sous-commande `maicie status --json`
sur une projection SQLite de 100 objectifs délégués, puis une capture Attach
publique éphémère. Les 100 objectifs partagent le même équipier : la
déduplication des abonnements est volontairement exercée, sans masquer le
coût de projection des 100 coordinations. Une chauffe précède 21 échantillons
mesurés ; le p95 utilise le rang le plus proche.

Quatre campagnes locales ont donné un p95 de 29,390 ms, 23,534 ms, 22,969 ms
et 23,419 ms (plage 22,969–29,390 ms), toutes sous le budget SC-008 de
250 ms. À cette charge, la capture éphémère ne justifie donc pas un runtime
résident v2 ; toute hausse future du nombre d'équipiers ou de la charge devra
être mesurée par ce même banc avant d'étendre la surface.

La performance ne justifie pas de runtime résident : le runtime v2 ne pourra
être justifié que par la boucle de réponse et une identité Maicie joignable,
conclusion de T015b, jamais par SC-008.

## T025 — recette et non-régression totale (2026-08-23)

Validation exécutée depuis un worktree isolé à la tête `0fa7f0c` :
`/tmp/bridget-t025-0fa7`. Le gate réel a utilisé
`/tmp/bridget-t025-0fa7/target/debug/bridget` et a réussi en 1,77 s ; la
remise attestée, le statut puis la clôture CLI ont été obtenus en 1 538 ms et
1 543 ms respectivement.

- Scénario 1 : `cargo test -p maicie --test mvp_gate -- --ignored --nocapture`
  avec `BRIDGET_MVP_GATE_BIN` a exercé le daemon, l'équipier ACP et les
  commandes `maicie delegate`, `status` et `objective close`. Les contrôles
  complémentaires `direct_message_isolation` (1/1),
  `delegation_outcomes_integration` (5/5), `cli_delegate_integration` (3/3)
  et `cli_objective_integration` (1/1) ont confirmé l'isolation du message
  direct, l'état `à_évaluer` sur issue terminale et les projections CLI.
- Scénario 2 : `ack_lost_recovery_integration` 11/11 vert en 0,21 s, dont les
  trois crashs réels, le lookup/replay exact et l'absence de double envoi.
- Scénario 3 : `status_sources_integration` 3/3 vert en 0,43 s : source,
  séquence, permission constatée et `Gap` restent factuels.
- Scénario 4 : `duration_timeout_contract` 1/1 vert en 0,01 s : les trois
  timeouts sont persistés sans timer ou relance locale.
- Le scénario 5 reste API/test-only comme l'indique le quickstart ; il n'a pas
  été présenté comme une recette CLI.

La non-régression complète a passé : `cargo test --workspace -q` (code de
sortie 0, environ 159 s ; les gates manuels documentés restent ignorés) puis
`cargo clippy --all-targets -- -D warnings` (code de sortie 0, 6,71 s).
