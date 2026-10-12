# Validation158A en cours

## Checkpoint historique avant la FULL
20/22 tâches : T001–T020 prouvées ; T021 et T022 restent ouvertes.
T001 : 253 PASS, 0 FAIL, 2 ignored, 255 noms exacts, 217 sources stables.
RED1 : 9 PASS / 11 FAIL ; T010 nouvelle clé admise à tort, quatre vues indisponibles et table scopes vide constatés.
Le premier essai précédant RED1 était un échec de compilation (match non exhaustif), pas une preuve RED.
RED2 : 23 PASS / 10 FAIL ; huit handlers humains absents et façade discovery non gardée constatés. Un oracle de fixture Continue était encore à corriger.
Les deux défauts de fixture Continue ont été observés dans les vrais JSON : paused=false sans changement retourne NothingToChange ; suspended suit la pause, et save renseigne completed_at.
Ces fixtures utilisent désormais le bon état persistant sans changer le code métier.
GREEN final ciblé : 49 PASS, 0 FAIL, soit46 oracles daemon158 +3 régressionsguidance156.
Transport : 1 PASS, anciennes représentations Delegate identiques.
Total158 : 47 oracles prescrits (44 noms imposés +3 décrits sans nom).
Le plafond parent-alltools atteint les validateurs métier des30 outils avec paramètres volontairement invalides ; il ne prétend pas exécuter30 opérations.
Le helper RPC d'anciennes fixtures rétablit le mode bloquant d'une socket acceptée BSD ; aucune exemption cfg(test) côté production.
T021 : une FULL unique exécutera les255 noms de référence et les47 tests158. Comparaison exacte par cible/nom/statut ; pas de seconde exécution des mêmes tests pour fabriquer une baseline.
Aucun succès global ni activation158 revendiqué avant T021/livraison.


## T021 — FULL unique terminée, quatre échecs nouveaux résolus par replays ciblés

La FULL unique a terminé en 804,17 s, RC101, avec 218 sources stables.
95 cibles Cargo : **2440 PASS, 12 FAIL, 74 ignored**. Les résultats des
processus enfants intégrés ne sont pas additionnés aux cibles Cargo.
Les 255 noms de baseline ont les mêmes statuts : 253 PASS et 2 ignored.
Les 47 oracles158 prescrits passent dans cette FULL.

Preuves primaires :
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-FULL158-final.json
  SHA256 2532508491a1e34f575c3217e02bb4704c97d93dc48d203b757f437bfa170b88.
- Journal /Users/moi/.cache/bridget-plan158-20261012-r1/T021-FULL158-final.log
  SHA256 70483926b59f67a005733b5a37aba4104e86d5e1af9079cd52c117d0cb97e3e0.
- Comparaison /Users/moi/.cache/bridget-plan158-20261012-r1/T021-full-comparison158.json
  SHA256 50ad3e3462987ef9c8f83e57523eadc2a1afdb30243421ae1ebb61fadabb230d.

Les huit échecs historiques conservés sont :
- `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee`.
- `cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent`.
- `cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere`.
- `client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation`.
- `deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance`.
- `noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper`.
- `spec094_mcp_modifie_uniquement_sa_propre_presence`.
- `spec098_pont_expose_remet_repond_renouvelle_et_retire`.

Quatre échecs nouveaux sont distingués :
- `spec103_s02_apercu_sans_daemon_ni_socket_ni_identifiant` : le nouveau RPC
  précédait l’aperçu pur. Le parseur strict/rendu local existant est maintenant
  repris avant CallerScope pour ce seul preview, sans DB, identifiant ou transport.
- `mcp_binaire_huit_register_coexistants_neuvieme_busy_sans_neuvieme_socket`
  et `mcp_reel_garde_outcome_unknown_apres_reponse_invalide_sans_fuite_du_canari` :
  les lecteurs des fixtures de protocole ne répondaient pas au vrai CallerScope.
  Les réponses de fixture sont adaptées au protocole, sans suppression du plafond.
- `native152_rouge_panne_mission_working` : le montage doit demander une posture
  Complete pour son travail d’écriture. La fixture est corrigée ; le même
  oracle152 passe ensuite seul. Aucun scénario nouveau.

### Replays après correctifs appliqués

| Reçu | Résultat | Temps | SHA256 du journal |
|---|---|---:|---|
| /Users/moi/.cache/bridget-plan158-20261012-r1/T021-review-fixes158-lib.json | 22 PASS | 16,63 s | cbdb3bc3ba4a4801a8540edada180445cd3664d96ef9caa10aaff82f85c5e630 |
| /Users/moi/.cache/bridget-plan158-20261012-r1/T021-preview103-replay.json | 1 PASS | 9,24 s | 04aad2ef396a1c5805f165b8caab5f24c2dceb679aef80a1e5cabaacf828e303 |
| /Users/moi/.cache/bridget-plan158-20261012-r1/T021-concurrency089-replay.json | 1 PASS | 7,26 s | a41722b2c1e1fd9d2d190dc2295253573efc45bfaef244f0243fa7d8af87645a |
| /Users/moi/.cache/bridget-plan158-20261012-r1/T021-security089-replay.json | 1 PASS | 7,65 s | 3f9e193452124a7c62837477206d9fee56ad0766305d30fffb6b01b67bf045ba |
| /Users/moi/.cache/bridget-plan158-20261012-r1/T021-native152-replay.json | 1 PASS | 9,29 s | 5a45fa4e827c9d28e0752808384f3a6ca8f653424fd79131dffc73137904f3e7 |

Chaque run ci-dessus a 218 sources avant/après identiques et RC0. Les 22 tests
lib incluent l’oracle156 `mcp::tests::guidance156_mcp_queue_schema`, restauré
avec son listener non bloquant et l’absence de connexion dès le premier appel.
TEST-001 est corrigé par la garde ChildProof préconnexion. Seule la section
ChildProof avant `let queue = tools()` retrouve ses octets historiques : aucun
credential factice ni serveur CallerScope ne masque le refus. La section
positive conserve une ligne `reply_caller_scope` adaptée au protocole ; le
corps entier de la fonction ne doit pas être présenté comme byte-identique. Les autres tests de ce lot couvrent le plafond, la table et la CLI.
Le succès du lot ne prouve pas un scénario forcé de panne de stockage après
commit ; le message d’incertitude F1 est relu dans le bras CLI réel.

Ces replays sont des preuves distinctes. Ils ne sont pas une seconde FULL,
ni un nouveau total de tests uniques, ni un GREENglobal. La FULL historique
reste RC101/12FAIL sur ses sources antérieures aux correctifs. Les quatre nouveaux échecs sont désormais résolus par quatre replays
ciblés distincts. Les huit échecs historiques restent documentés. Les
sources diffèrent de celles de la FULL par les correctifs CLI/MCP et trois
fixtures ; aucune seconde FULL n’a été exécutée sur ce dernier ensemble.
T021/T022 sont ensuite prouvées et cochées par ROOT : 22/22. Ce résultat
n’efface ni les échecs historiques ni la distinction FULL/replays ciblés.
La livraison et l’activation restent hors preuve de ce rapport.

## Précisions de contre-revue Haiku R2

R2 est APPROVE_WITH_CHANGES, sans défaut bloquant. C1 est traité par un texte
CLI neutre, valable aussi pour resume : « l’état du contrôle des délégations
a pu changer ». L’annulation ciblée est proposée seulement si nécessaire.
Le code de sortie1 et le refus StoreUnavailable sont inchangés.

C2 : la section ChildProof, depuis la ligne `fn guidance156_mcp_queue_schema()`
et avant `let queue = tools()`, compte1710octets et a pour SHA256
c107269454f8b8d3923eaf36a546972137760630a74a86fd6f61f3a010010d69.
Seule cette section est byte-identique. La fonction entière diffère de la base
par une ligne positive `reply_caller_scope`, nécessaire au nouveau protocole.

La provenance des tests repose sur les octets, pas sur la date de modification.
Le lot lib22PASS et les quatre replays ont, avant ET après chaque run :
- mcp.rs :0c525f7c5b5f6d78d929570f0f18e0920902a30ef6f7b00c22760bcecdd914ed ;
- cli.rs :3c4286217e5b2269c10b2f9c2cda227885e85214ddd6f69657faa1569ce5008f.
Les218sources de chaque run sont strictement stables. Ces reçus attestent
les correctifs appliqués malgré une écriture et un lancement dans la même
seconde. Le dernier texte C1 a ensuite changé le seul littéral CLI : empreinte
cedf1abb594c5720e37bc94d94b15b6bbe39eca1f5128de7263a99ca413dc543.
Ce dernier texte est relu ; il n’a pas de test de panne forcée (C3).
La compilation de livraison apportera sa preuve séparée, sans être anticipée.

L’utilisateur demande les tests prévus et évite les répétitions inutiles. La stratégie retenue est une FULL unique puis des replays ciblés.
L’oracle de panne supplémentaire proposé par R1/R2 n’est donc pas ajouté.
La réserve C3 reste explicite. La demande R2 de refaire une FULL n’est pas
retenue : les quatre nouveaux échecs ont leurs replays PASS, les corrections
sont corrélées à leurs empreintes et les huit échecs historiques sont conservés.
Aucun GREENglobal ou succès sur une seconde FULL n’est fabriqué.
