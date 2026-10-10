# Dégradations 149 - couche T3 (T039 partiel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), r2. Aucun edit de production, aucun commit, aucune case cochée.

**Statut : PARTIEL.** Les dégradations de la couche T3 sont prouvées contre le serveur réel. Les scénarios qui dépendent du daemon natif (panne T3 après admission, retry après changement de droits, résultat retenu natif, reprise sans relance native, doubles lancements natifs) ne sont PAS prouvés : pas de binaire natif prouvé frais. T039 ne peut pas être coché.

Couches : serveur T3, auth, WS, lecteur, projection SQL privée et UI web RÉELS. Daemon SIMULÉ par `bridget_fixture.mjs` (magasin JSON privé). Données synthétiques `[recette149]`. Détail des couches et isolation : voir `network149.md`.

## Compteurs

| Scénario | Résultat | Preuve |
|---|---|---|
| D1 CLI absent (fixture non exécutable) | PASS : `unavailable` | RPC `lineage.read list`, puis retour `OK` après `chmod 755` |
| D2 exit 3 (verrou magasin) | PASS : `store_unavailable` | idem, retour `OK` après retrait du verrou |
| D3 CLI qui plante (magasin JSON corrompu, exit 1, sauvegarde/restauration) | PASS : `invalid_output`, aucun crash serveur | idem, retour `OK` après restauration |
| D4 timeout (daemon lent 8000 ms) | PASS : `timeout` après 7 s | `scenario_step.mjs hang 8000`, puis `hang 0` |
| D5 génération changée | PASS : `list` rend la nouvelle génération (`…999`), fils virtuels 7 avant et 7 après, marqueur `bridgetTaskRef.generation=…999` et `seq=14` sur les 4 fils | requête SQL sur `payload_json` |
| D6 lacune de journal | PASS : le serveur transmet `gap {from_seq:3,to_seq:5}` ; l'UI affiche « Journal incomplet : séquences 3–5 (…) » avec seq 1-7 sans doublon | RPC + capture 10 |
| D7 coupure du flux journal puis « Reconnecter » | PASS : état dégradé avec contenu conservé, puis reprise sans message ni doublon (seq 1-5) | captures 03 et 04 |
| D8 journal absent sur une tâche échouée | PASS : « Bridget indisponible (journal_unavailable). Le contenu déjà lu est conservé. » et « Dernier état connu » | capture 08 |
| D9 reprise UI après D1-D4 | PASS : plus d'« indisponible » sur le fil hôte, sans rechargement manuel du serveur | capture 09 |
| D10 annulation : rejeu identique, enveloppe différente, tâche terminale | PASS (C1-C3 de `matrix149.mjs`) | `network149-matrix-run.txt` |
| D11 100 `show` + 100 `list` sans changement | PASS : 0 nouvel événement d'orchestration (207 avant, 207 après), 0 run, 0 tour, magasin inchangé (seq 9, 4 tâches) | `rpc149.ts --repeat 100`, SQL |
| D12 pagination 134 tâches avec mutation entre pages | PASS : relance depuis la page 1 puis succès ; refus `snapshot_changed` après trois tentatives | voir `network149.md` |
| D13 annulation racine puis imbriquée depuis l'UI | PASS : deux `request_id` UUID distincts, statut « Annulation en cours », 0 run/tour/session | reçus dans `store.json` |
| Scénarios natifs (panne T3 après admission, retry après changement de droits, résultat retenu natif, reprise sans relance native, double lancement natif) | SKIP | binaire natif absent |

Total : 13 PASS, 0 FAIL, 1 SKIP (groupe natif).

## Comptage des exécutions et des processus

- Tables privées après toute la recette : `runs=0`, `run_attempts=0`, `provider_turns=0`, `provider_sessions=0`, `provider_threads=0`, `provider_session_bindings=0`, `runtime_requests=0`, `effect_outbox=0`, `thread_launch_workflows=0`, `thread_messages=0`. Aucune remise.
- Processus : le seul processus fixture vivant pendant la recette était le `lineage watch` du navigateur (enfant du serveur, un seul, jamais deux). Après fermeture de l'onglet et arrêt du serveur : 0 processus fixture, 0 port en écoute.
- Aucun appel de lancement n'existe dans la fixture : « absence de double lancement » est prouvée au niveau T3 uniquement (voir limites de `network149.md`).

## Constats pour l'owner de production (aucun patch de ma part)

1. **Réécriture O(n) à chaque mutation.** Un seul ajout de journal sur une tâche (4 tâches) produit 4 `subagent.updated` + 5 `thread.metadata-updated` ; les 3 autres tâches n'ont pas changé. Avec 134 tâches, une mutation ajoute environ 270 événements. Sur la recette, le journal d'événements totalise 2082 lignes (1147 `thread.metadata-updated`, 778 `subagent.updated`, 137 `thread.created`). À l'inverse, 100 lectures sur snapshot inchangé n'ajoutent rien. Reproduction : `scenario_step.mjs <store> journal`, puis comparer `select event_type,count(*) from orchestration_events where sequence > <max avant>`.
2. **Fils orphelins.** Après `scenario_step.mjs unseed` (130 tâches retirées du magasin, même génération), les 130 fils virtuels restent en base avec leur ancien marqueur (`seq 28`, statut `cancelled`), ni supprimés ni archivés. Le panneau Lineage affiche « Previous agents (133) » pour 4 tâches natives (capture 11). À confirmer : intentionnel (historique) ou fil à retirer sur snapshot complet vérifié.
3. **Indisponibilité d'une seule tâche = tout le lineage.** `journal_unavailable` sur T4 fait afficher « Lineage · Bridget indisponible » et « Dernier état connu » pour tout le fil racine, tâches saines comprises, jusqu'au prochain `list` réussi. Le contrat dit seulement « garde l'historique vérifié sans le présenter comme état courant » ; la portée n'est pas précisée.
4. Voir aussi `ui149.md` (bouton « Arrêter » masqué à 1280 px).

## Reprise

Les scénarios natifs se rejouent avec le binaire natif (reçu `native149-debug-receipt.json` frais). Les outils sont prêts : `scenario_step.mjs` (étapes `journal`, `nested-status`, `result`, `cancelled`, `generation`, `gap`, `journal-off`, `store-corrupt`, `hang`, `seed`, `unseed`, `cursor-bump`), `matrix149.mjs`, `db_counts.sh`. Étape `reset` : non implémentée (réexécuter `fixture_store_init.mjs`).
