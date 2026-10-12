# Annexe de preuve — fondations 159B

État : implémentation B en cours. Aucun verdict de clôture, audit ou livraison.
T014/T015 : 7 PASS/0 FAIL, RC0, 20,99 s, 222 sources AV/AP identiques.
T016 : 2 PASS/0 FAIL, RC0, 16,19 s, source stable dans ce run distinct.
Les cinq oracles de reçus et les deux de droits ne prouvent pas encore cancel,
reorder, promote, les façades T022 ni les E2E T023.

Réemploi : queue_edit_once privé commun à trois mutations, même magasin et
init_schema v7, reçu plus effet dans la transaction existante. Identité et
Steer156, clôture de retenu, notice et rangs existants restent réemployés.
Aucun magasin, protocole d’injection ou framework supplémentaire.

Arbitrage déterministe ROOT : le message public peut correspondre à plusieurs
scopes. Résolution held de la cible bornée à deux lignes ; exactement une
correspondance, sinon refused/not_held avec delivery_id NULL. Cette règle est
normative ; les chemins de mutations restent à vérifier par leurs tests prévus.

T024/T025 : hunks documentaires préparés, aucune application WT ni tâche cochée.
Syntaxe fondée sur les candidats CLI/MCP T022, pas sur un runtime validé.
Le reçu de promotion atteste le commit ; effect unconfirmed, repli FIFO,
rejet si pleine, doublon Codex possible après erreur ambiguë. Aucun avis
universel neuf pour reply=false. held_queue reste fermé ; T3, edit et159c différés.

Preuve GREEN :
/Users/moi/.cache/bridget-tests159B-20261012/T014-T015-foundations-GREEN.json
SHA256 ed99d0e4706e5462e869ddacb1a9bf1d84b864b0d04e36b1884abdffc59542a9
Log SHA256 02feb04a3f1024527dbc1511cb616cb87e36e27333651ef294f2745cc038b979
Préparation des façades :
/Users/moi/.cache/bridget-code159B-T022-preparation-20261012/README.md
/Users/moi/.cache/bridget-code159B-T022-preparation-20261012/cli.GREEN.candidate.rs (4722–4810)
/Users/moi/.cache/bridget-code159B-T022-preparation-20261012/mcp.GREEN.candidate.rs (1274–1355, 2592–2610)
