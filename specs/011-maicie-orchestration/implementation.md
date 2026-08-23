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

T015b est reportée à la phase 4 : avant toute évolution du protocole Bridget,
T017/T018 doivent établir si Subscribe 008 et `in_reply_to` corrèlent les
réponses et les timeouts de manière suffisamment fraîche, sans inférence
locale ni seconde source de vérité.

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
