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
