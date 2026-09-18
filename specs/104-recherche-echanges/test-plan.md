# Plan de tests 104

Statut : 31 scénarios PLANIFIÉS, zéro exécuté. Les assertions ci-dessous
sont des gates de l'implémentation, pas des succès annoncés.

## Harnais et méthode

Fixtures synthétiques, vraie baseSQLite et sockets temporaires aux frontières ; aucun
fournisseur réel ni API payante. L'identité doit suivre099 : RegisterAuxiliary/proof avant
ClientHello ou requête enregistrée ; pas d'UUID pris depuis un fichier sans preuve.
Exprimer chaque scénario sous forme Given/When/Then dans les tests Rust natifs :
Given fixture initiale, When appel réel, Then état/résultat observable.
Les tests de performance doivent rapporter matériel, buildmode, taillecorpus, p95 et mémoire.
Un test qui ne mesure que ses mocks n'est pas une preuve de transport ou d'accès.

Avant un test processus, inspecter ses helpers de nettoyage : SIGKILL et kill de groupe
non vérifié sont interdits. Home/socket/base/journaux isolés ; ne jamais lancer le daemon
sur /Users/moi/Nextcloud/10.Scripts/64.bridget ni sur le home de production /Users/moi/.cache/bridget-core.
Arrêt ciblé d'un PID propre vérifié (jamais Firefox), attente et collecte du code de sortie.

## Matrice

| ID | Story | Given / When | Then observable | Couverture |
|---|---|---|---|---|
| S01 | US1 | Oracle corpus | 1000lignes participant +1000tiers ; après pagination, résultats exacts de l'oracle. | FR-001, FR-008, SC-001 |
| S02 | US1 | Texte littéral | Tous termes requis, café/cafe, majuscules, á/å/ý/Ÿ ; % _ \\ ' guillemets inertes. | FR-001 |
| S03 | US1 | Unicode documenté | Emoji intacts ; forme décomposée non assimilée au repli précomposé, limite explicitée. | FR-001, FR-004 |
| S04 | US1 | Filtres | Author/peer/dates inclusives, même seconde et filtre sans occurrence ; pas de nom résolu arbitrairement. | FR-002 |
| S05 | US1 | Validation | Vide/blanc,257octets,9termes, limit0/51, dateinverse, fauxUUID : erreur sansSQL. | FR-001, FR-002 |
| S06 | US1 | Confidentialité participant | AppelA exclutB↔C et compteurs ; identité absente/fichier-only usurpé refusée. | FR-008, FR-010 |
| S07 | US2 | Page vide reprenable | 128non-correspondances avant un hit : hits=[] ET curseur, suite trouve le hit. | FR-005, SC-001 |
| S08 | US2 | Ordre et clés | ID identique vers deux cibles, même seconde ; ordre(ts, id, target), aucune perte/double sur corpus fixe. | FR-006, SC-002 |
| S09 | US2 | Bornes travail | 128candidats, budget1Mio,1corps16Mio autorisé puis arrêt ; >16Mio ignoré et signalé sans chargement. | FR-005, FR-011 |
| S10 | US2 | Bornes sorties | 50hits/extraits512octets,60Kio total, UTF-8 et JSON échappé ; hit non inclus non consommé. | FR-004, FR-005, SC-003 |
| S11 | US2 | Curseur non autorisant | CurseurA chezB refusé ; filtres changés refusés ; borne forgée ne révèle aucun tiers. | FR-006, FR-008, SC-002 |
| S12 | US2 | Cursor malformé | Hex invalide, versionfuture, trop long, tupleincorrect : erreur nommée sans SQL large. | FR-005, FR-008 |
| S13 | US2 | Purge/insertion/remplacement | Corpus vivant changé entre pages : pas de faux snapshot ; purge→source introuvable honnête. | FR-006, FR-010 |
| S14 | US2 | Erreur SQLite | Busy/corruption/décodage ligne simulé : storage_unavailable, jamais hits=[]. | FR-010 |
| S15 | US2 | Métadonnées héritées | ID/sender/target>256octets : refus avant gros chargement, aucune clé tronquée. | FR-004, FR-010 |
| S16 | US3 | Lecture exacte | Même id deux destinataires : bon corps seulement ; inconnu et interdit même erreur ; zéro voisins. | FR-007, FR-008 |
| S17 | US3 | Fragments | Body64Kio, emoji sur limite ; fragments16Kio, digest commun, reconstruction exactoctets. | FR-007, SC-003 |
| S18 | US3 | Corps changé | Remplacement avant morceau2 : content_changed sans morceau ; mauvaisoffset refusé. | FR-007 |
| S19 | US3 | Fil autorisé | Membre trouveentrée102 et référencehistory ; non-membre/filabsent mêmes réponses. | FR-003, FR-014 |
| S20 | US3 | Fil snapshot | Entrées1..200 puis201 concurrente : upperseq conserve1..200 ; nouvelappel voit201. | FR-003, FR-006 |
| S21 | US3 | Recherche sans ACK | Avant/après tables102 et demandes identiques ; aucun wake/sent/ack/read-receipt. | FR-009, FR-014, SC-004 |
| S22 | US3 | Révocation pendant travail | Identité révoquée après le début : aucun résultat publié ; permis libéré. | FR-008, FR-011 |
| S23 | US4 | Index | EXPLAIN deux branches→indexparticipant,≤129clésbranche, fusion de 258 clés maximum ; migration deuxouvertures. | FR-011 |
| S24 | US4 | Concurrence | Deux appels lents, troisième busy ; DM témoin passe, pas de mutex pendant scan ; transaction de lecture fermée avant CPU, y compris SQLite sans WAL. | FR-011, SC-005 |
| S25 | US4 | Performance archive | 100 000 lignes de 1 Kio,200 pages et 200 messages directs : p95<1s chacun, aucune requêteLLM. | FR-011, SC-005 |
| S26 | US4 | Performance gros corps | Plusieurs candidats de16Mio : sélection des tailles avant corps, au plus un corps hors budget, lotSQL<17Mio ; mémoire additionnelle<128Mio, page<2s, messages directs témoins p95<1s. | FR-011, SC-006 |
| S27 | US4 | Parité et compatibilité | CLI/MCP mêmesrésultats/refus ; ledgerancien inchangé ; vieuxdaemon sansrepli vers le ledger global. | FR-012, SC-007 |
| S28 | US4 | Recette skill | Requête→extrait→sourceexacte→citation ; annoncer recherche partielle, pasviderarchiveautomatiquement. | FR-013, FR-015 |
| S29 | US4 | Injection et non-réseau | SQL/ANSI/prompt hostile inertes ; fichier/URL/artefact cités jamais lus ou exécutés. | FR-015, FR-010 |
| S30 | US3 | Passage utile Unicode | Terme à la fin après caractères dont le repli change la longueur : match_offset original exact, extrait pertinent, read ciblé avec digest sans relecture du préfixe. | FR-004, FR-007, SC-003 |
| S31 | US4 | Intégration102 absente | Contrat thread reconnu mais capacité absente : capability_unavailable, jamais invalid_params/succès vide ; remettre tous les tests fils après intégration. | FR-003, FR-012, SC-007 |

## Commandes futures

Depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges, APRÈS implémentation et audit du harnais :

```sh
cargo fmt --all -- --check
cargo test -p bridget-daemon --lib spec104
cargo test -p bridget-daemon --test search_104_test
cargo clippy --workspace --all-targets -- -D warnings
```

Ajouter cargo test -p bridget-transport spec104 pour les trames et les régressions102 ciblées.

Faire correspondre les noms de tests au filtre spec104, et vérifier qu'un filtre ne lance
pas zéro test. Régressions089/094/099/100 et autres surfaces touchées seulement après audit
de leur isolation ; ne pas lancer aveuglément tout workspace en release sur ce poste partagé.
Les tests fonctionnels ne nécessitent pas T3 vivant. Toute recette réelle ou installation
future requiert une nouvelle autorisation, pas un héritage de l'accord donné pour101.

## Négatifs indispensables

Erreurs avant effets, identité manquante/révoquée, source inconnue ou interdite, caractères
Unicode aux limites, interruption et issue inconnue, ancien client et ancien daemon.
Conserver erreurs/status sans reformulation en réussite métier. Aucun secret/corps dans logs.
