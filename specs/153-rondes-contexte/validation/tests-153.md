# Tests153 — preuves distinctes, validation en cours

Aucun succès global153 n'est déclaré. Le principal exécute les tests ; cet
owner documentaire lit le code et archive ses preuves, sans lancer Cargo.

## Référence avant code153 — T001

Six filtres réels :194 PASS/0 FAIL/1 ignoré. Sept commandes historiques, dont
spec_103 vide remplacé par spec103 (17 tests). La référence et ses empreintes
restent dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline.md et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-results.json.
Ces compteurs ne sont pas additionnés aux contrôles ci-dessous.

## T002 — protocole, GREEN du principal

| Filtre/crate | PASS | FAIL | Ignorés | RC | Durée |
|---|---:|---:|---:|---:|---:|
| native153_protocole /bridget-transport --lib | 3 | 0 | 0 | 0 | 18,91s |

Commande exacte :

`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-transport native153_protocole --lib -- --test-threads=4 --nocapture`

Code relu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:2011.
keep_alive_secs est omis si absent. Le brief de Delegate et de Continue réutilise
deserialize_present_optional, maintenant à la ligne2738. Le null explicite
reste présent ; le refus métier brief_invalid sera prouvé en T005.
L'enveloppe fermée et les champs historiques sont conservés.

| Oracle exécuté | Preuve de code |
|---|---|
| native153_protocole_delegate_sans_champs_garde_octets150 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:7935 ; mêmes octets avec reply_timeout150 |
| native153_protocole_continue_aller_retour | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:7952 ; brief objet, absent et null traversent le vrai JSON |
| native153_protocole_brief_serialisation_deterministe | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:7965 ; deux objets de même contenu, octets identiques |

SHA du protocole avant/après et à cette lecture :
a65d6849698d24c230b816036895d4a69e718ae6ac83e35cb64bc56535361350.

JSON copié : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t002-protocol153-r1.json
SHA256 : 1384fb04f02cbb8ba2f0f2e91c1b17900de4356986bf4e6b96843f963023c195.
Journal copié : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t002-protocol153-r1.log
SHA256 : 996380aad46512f7aba4acd58c8e341a453070a21cd791f9c1f9aacef1046a7a.

Environnement de la commande : HOME conservé, namespace privé
/private/tmp/b.p1xqyp4_/ns, TMPDIR privé
/private/tmp/b.p1xqyp4_, CARGO_TARGET_DIR partagé
/Users/moi/Nextcloud/10.Scripts/64.bridget/target. BRIDGET_BUILD_CACHE_GIB=20 est temporaire
pour conserver la compilation pendant la session. Aucun changement de
configuration globale/source ; retour au nettoyage par défaut10 à la livraison.

T002 seule nouvellement cochée après cette preuve et la lecture. T003 et les
tests métier153 restent ouverts. T022/T023 attendent T018.

## Comparaison finale T021/T025 — préparation, sans résultat attribué

Le principal a confirmé par lecture que
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/build_identity.rs:27
surveille ROOT/plugins absent, ce qui réinvalide le script de compilation.
Aucun prune de baseline n'est constaté. La limite temporaire20GiB évite un
prune distinct ; elle ne prouve pas la résolution de cette invalidation.

Première adaptation envisagée, historique de méthode : regrouper les filtres
existants, support libtest [FILTERS...] vérifié par l'agent du principal :

`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support -- native148 native149 native150 delegation149 spec103 spec_087 --test-threads=4 --nocapture`

Cette union n'a pas été exécutée et ne constitue pas la méthode finale.
Le steering utilisateur autorise maintenant **un seul lancement workspace
final pour T021 et T025**, sur les mêmes sources finales gelées. La suite
doit réellement exécuter tous les noms de T001 avec les mêmes résultats :
194 PASS et un ignoré, vérifiés nom par nom et par cible. Un compteur global
seul ne suffit pas. Aucun test n'est retiré. Si cette présence complète est
prouvée, les six filtres ne sont pas rejoués séparément. T021 reste ouverte
jusqu'au lancement et à sa comparaison. Les reprises prévues de tests
sensibles à la charge restent conditionnées à une préoccupation réelle.

Avec `--nocapture`, les sous-processus peuvent écrire leurs propres résumés
libtest. Le compteur canonique prend seulement le dernier résumé de chaque
cible Cargo `Running` ou `Doc-tests`. La somme brute reste conservée séparément ;
elle ne devient pas le compteur workspace.

Les noms ne sont pas stockés en tableau dans le reçu T001 : le JSON contient
results[] avec filter, command, exit_code, seconds, log et counts. Les noms
et résultats exacts sont les lignes libtest des journaux :
`test <nom complet> ... ok` ou `test <nom complet> ... ignored, <motif>`.
Les journaux durables, qui conservent les cibles Cargo, font autorité :

| Filtre | PASS | Ignorés | Journal absolu immuable |
|---|---:|---:|---|
| native148 | 27 | 1 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-native148.log |
| native149 | 78 | 0 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-native149.log |
| native150 | 51 | 0 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-native150.log |
| delegation149 | 10 | 0 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-delegation149.log |
| spec103 | 17 | 0 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-spec103.log |
| spec_087 | 11 | 0 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-spec_087.log |

Ignoré unique :
`native148_parent_codex_fixture_delegates_once_to_real_glm`,
motif `GLM réel : opt-in explicite après validation de l’identité`.
L'ancien journal baseline153-spec_103.log (zéro test) reste intact ; son
filtre n'apporte aucune preuve. Le remplacement spec103 est bien compté.

Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/baseline153-results.json
SHA256 : b7b7aebf006a1b15e9b95b4803a8a8071573a05e5ab34ffe369a059c8bfa8549.
Aucun nouveau checker, helper, outil ou test. Cette adaptation ne coche
aucune tâche ; compteur courant12/29. Aucun résultat final encore attribué.

Référence historique des huit échecs workspace151/152, noms et causes identiques :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/152-reprise-missions/validation/workspace-152-r1-comparison.json.
Cette référence compare des traces antérieures ; elle ne dispense pas de
vérifier les causes sur la suite finale153 et n'est pas un succès global.


## Première passe groupée T003/T004/T005/T017 — RED conservé

19 tests unitaires exécutés : **11 PASS/8 FAIL/0 ignoré, RC101**.
Durée44,74s ; compilation43,89s et tests0,69s. Les199 empreintes Rust
avant/après sont identiques dans la capture ; le travail produit ultérieur
n'est pas déclaré validé par ce run.

| Groupe | PASS | FAIL | Statut exact |
|---|---:|---:|---|
| T003 fondations | 3 | 0 | GREEN, code et oracles relus, tâche cochée |
| T004 begin_round | 0 | 3 | RED compilé : méthode encore stub au moment du run ; tâche ouverte |
| T005 brief | 2 | 5 | RED métier compilé ; deux témoins déjà verts, tâche ouverte |
| T017 catalogue/permissions unitaires | 6 | 0 | Un nouveau cas MCP et cinq cas existants ; intégration ClaudeOwned attend la suite finale T025, tâche ouverte |

Les tests T003 sont :
native153_fondations_payloads149_150_et_predicats,
native153_fondations_index_legacy_courant_et_etranger,
native153_fondations_projection_ignore_sept_champs_et_garde_immuable.
Sources relues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:49,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:169,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:276,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage_tests.rs:1144,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage_tests.rs:1183,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage_tests.rs:1213.

Preuves : défauts de lecture149/150, prédicats et ID historiques/suffixés ;
index legacy migré sous Immediate, index courant conservé, définition étrangère
refusée avec rollback complet ; table/index missions passées et JOIN réel ;
projection ignorant les sept champs, guarded immutable dans save_projected.
T004 garde son propre passage RED avant correction, séparé de cette réalisation.

Commande exacte :

`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support --lib -- native153 native151_ack_facade_schema_ferme_bornes_et_annotations catalogue_mcp_expose_exactement_les_quatre_verbes_du_greffe spec091_politique_mcp_reste_octet_pour_octet_stable spec094_codex_autorise_exactement_les_outils_de_communication_surs native148_claude_allows_native_delegation_without_global_mcp_approval --test-threads=4 --nocapture`

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t003-t004-t005-t017-native153-red-r1.json
SHA256 : 0121744910b38c24313d6e0801fe315a96c683e613ba7c8edc18c8f212fb2b51.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t003-t004-t005-t017-native153-red-r1.log
SHA256 : 68be1db6be1828a2ff50475e3b0ae28ca0aefa6edef951a4aa29f5c3b351931f.

Aucune addition avec la référence194/0/1 ou le protocole3/0/0. Aucun succès
global. Compteur documentaire actuel : **3/29** (T001/T002/T003 seulement).

### Réemploi du montage de test150

Recherche nom/responsabilité : helper existant parent_en_cours_150 dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:7481,
neuf appels actuels relus par recherche ciblée. L'extension approuvée transmet
la requête pour les scénarios153 prévus ; les neuf usages anciens conservent
leurs enveloppes/oracles. Aucun nouveau helper ni scénario de confort.
Cette extension reste prévue tant que son code et son GREEN ne sont pas reçus.


## T003 rouverte — défaut de reconnaissance d'index trouvé après R1

Le principal constate que la normalisation de tous les espaces du SQL transforme
aussi le littéral JSON '$.sta te' en '$.state'. Un index étranger peut ainsi être
reconnu à tort puis remplacé. Les3 PASS T003 du premier run restent historiques ;
ils n'affirment pas ce cas de chemin contenant un espace.

Correction minimale posée par le propriétaire stockage : comparaison sql.trim()
avec les deux définitions exactes connues. Le test étranger déjà prévu est
renforcé par '$.sta te'. Aucun nouveau scénario, helper ou outil de migration.
La validation GREEN corrigée reste attendue du principal. T003 est rouverte,
compteur courant **2/29** ; T004/T005/T017 demeurent ouvertes.


## Passe ciblée suivante —14PASS et deux échecs149 en diagnostic

16 tests exécutés : **14 PASS/2 FAIL/0 ignoré, RC101**.
Durée14,07s, compilation13,31s, tests0,60s. Empreintes Rust stables pendant
ce run. Le nom du fichier comporte green ; son résultat complet reste RC101.

| Groupe | PASS | FAIL | Portée |
|---|---:|---:|---|
| T003 fondations corrigées | 3 | 0 | Chemin étranger '$.sta te' refusé, index intact et rollback complet ; T003 recochée |
| T004 transactions | 3 | 0 | Atomicité, CAS, heure fraîche, resets/ACK, projection et signaux après commit ; T004 cochée |
| T005 brief | 7 | 0 | Tous les cas prévus de validation, bornes, JSON null, corps envoyé, rejeu et omission ; T005 cochée |
| T017 MCP unitaire | 1 | 0 | Catalogue/champs fermés ; intégration ClaudeOwned finale toujours attendue |
| native149_catalogue | 0 | 2 | Régressions potentielles encore en diagnostic, aucune waiver |

T007 est cochée avec native153_brief_corps_envoye_a_l_enfant et lecture du tick :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1828,
round_body si présent, fallback initial request.task sinon ; brief placé avant
la consigne. ID idempotent et mission conservés. Montage parent_en_cours_150
réutilisé avec requête en paramètre, neuf appels anciens gardent leur requête
d'origine. Aucun helper nouveau.

Correction T003 relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:201,
comparaison sql.trim() aux deux définitions connues ; cas étranger renforcé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage_tests.rs:1205.

T004 relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:307.
Immediate, lecture puis CAS, historique/écriture/projection dans le même commit ;
horloge relue après encodage et avant toute écriture ; compteur2/checkpoint de
vie entière conservés ; epoch et notification seulement après commit réussi.
Les trois tests passés sont les mêmes que le RED précédent.

Deux échecs149 à garder stricts :
- native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain :
  claude-child, development=false au lieu de true, assertion225.
- native149_catalogue_parent_lecteur_et_confinement_refuses_par_nom :
  permission_source_unavailable au lieu de permission_not_inherited, assertion241.

Ils étaient verts dans la référence native14978PASS. Ils ne sont pas classés
hérités des huit échecs workspace151/152. Le principal a lancé un diagnostic
en lecture seule ; aucune correction aveugle ni changement d'oracle déclaré.
T006 reste ouverte pour sa partie catalogue/politique, malgré validation brief
passée. T017 reste ouverte jusqu'à son intégration prévue dans T025.

Commande exacte :
`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support --lib -- native153 native149_catalogue --test-threads=4 --nocapture`

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t003-t004-t005-t007-native153-green-r1.json
SHA256 : c9044157586d0e8f961d38b5d20b7743d3d25d20f642245a575297008b0c34cc.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t003-t004-t005-t007-native153-green-r1.log
SHA256 : dcad4e6c0f6ebdb9b8fedbd6958c9a66b37a5f2518f15d2b6750f501b8718958.

Sources de réalisation, empreintes du run (pas du travail futur) :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs — SHA du run 772da894af39b3e7ccb7bb5805eb30f688aa9916dbf13cf0fa4f7e4bf278c6ef
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs — SHA du run 209374d9c89b31616a6305bc3f088c11529f658470cf1fa0bffdf4b67225bf77
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs — SHA du run c72c392fd4bae6db759779aff1f6137e2658b41a68ca6531143398f6bd3acfb2

Compteur courant **6/29** : T001/T002/T003/T004/T005/T007. Aucune addition
entre runs, aucune recette de modèle ou production153 validée. T022/T023
attendent T018. Les rouges et l'ancien GREEN T003 sont conservés intégralement.


## T006 — rejeu causal des seuls deux catalogue149, GREEN

**2 PASS/0 FAIL/0 ignoré, RC0,8,71s** (tests8,66s).
Même binaire de bibliothèque déjà compilé que la passe précédente, sans Cargo,
recompilation, patch de fixture, configuration de production ou modification
de code par ce rejeu. Le principal corrige seulement le prérequis de commande
omis dans son pilote : `umask 077` (capture `0077`).

Commande exacte, après ce masque privé :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/bridget_daemon-0321bef9fccb32ef native149_catalogue --test-threads=4 --nocapture`

SHA du binaire avant/après :
95c8033934d5cc38ff8f929e3e403260b644d3f820996eb72ec7e510223e0a93.
Lien explicite compiled_source_receipt, empreintes de compilation conservées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t003-t004-t005-t007-native153-green-r1.json
SHA256 : c9044157586d0e8f961d38b5d20b7743d3d25d20f642245a575297008b0c34cc.
Les sources courantes évoluent pendant les RED US2 d'autres owners ; ce rejeu
binaire ne prétend pas qu'elles sont immuables ou recompilées à ce moment.

Les deux oracles stricts sont inchangés :
native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain et
native149_catalogue_parent_lecteur_et_confinement_refuses_par_nom.
Ils sont PASS avec le masque privé obligatoire. Les deux FAIL précédents
restent bruts, ne sont ni effacés ni rangés parmi les huit échecs hérités.

Diagnostic : le défaut de masque a été inféré à partir de résidus0755/0644,
puis confirmé expérimentalement par ce seul changement. Le répertoire exact
config-149 de la première panne avait déjà été supprimé ; ses permissions
n'ont pas été observées directement. Selon la lecture du principal, les
sources de politique ROOT/WT sont identiques et les chemins privés uniques.

T006 cochée : validation/rendu déjà verts dans les sept cas de brief, ajout
brief_max_bytes au catalogue relu :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:708.
Le contrôle global T017 et son intégration finale restent distincts et ouverts.

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t006-catalogue149-umask077-replay-r1.json
SHA256 : 3300c58288e06bc857703c27aaaf14a06d6e5794d32b6f0f128fae88c2bbcaa1.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t006-catalogue149-umask077-replay-r1.log
SHA256 : d26bb65f0601bd671f1faaebe0e9fee0fe862d9c549955345f800657c003ac87.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-1).


## T008/T012/T029 — premier RED garde/purge, avant production

**1 PASS/8 FAIL/0 ignoré, RC101**, neuf tests compilés puis exécutés.
Durée11,12s ; compilation10,40s, tests0,53s. umask0077 et empreintes Rust
avant/après identiques dans la capture. Aucun cochage sur cette preuve RED.

| Groupe | PASS | FAIL | Oracle avant correction |
|---|---:|---:|---|
| Façade : six native153_garde | 1 | 5 | Témoin sans garde conserve nettoyage150 ; bornes, résultat/expiration, annulations, reprise et nettoyage parent restent RED |
| Magasin : trois native153_limites | 0 | 3 | Purge refusée avec garde ; anciennes rondes non accusées même cancelled ; historique de mission supprimé avec l'arbre |

Témoin PASS : native153_garde_absente_nettoie_comme_150.
Les huit noms précis et assertions restent dans le JSON/journal bruts.
T029 de recopie des anciennes phases ACK dans le moteur n'est pas encore
réalisée à ce passage. Quotas/EXPLAIN T012 restent à vérifier. Le principal
autorise ensuite la production T009–T011 et magasin, sans nouvelle tâche
ou scénario. T022/T023 restent après T018.

Commande exacte :
`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support --lib -- native153_garde native153_limites --test-threads=4 --nocapture`

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-red-r1.json
SHA256 : e7a4387ae633faa85b6af5b521b52bb7605871ece5279eb0d8752c7d257f3dbc.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-red-r1.log
SHA256 : f921955bb7a2ab1e1c4830da483cf43e00d6c2ea3df3f7ddb8761ab1864ad2af.

Compteur inchangé **7/29** ; tasks/spec byte-identiques pendant cet archivage.
Aucune addition aux autres runs, aucun succès global ou recette réelle déduit.


## T008/T012/T029 — R1 arrêtée à la compilation

RC101 en 5,49s. Zéro test exécuté : results, passed_names et failed_names sont
vides. Les 200 empreintes de sources avant/après sont identiques.
La macro de log refusait le caractère `%` à la ligne1646 de
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs.
Le propriétaire corrige ce seul log avant R2. Ce passage ne prouve ni un
RED comportemental ni un GREEN. Son nom brut « green-r1 » reste historique.
Compteur au moment de cette tentative : 7/29.

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-green-r1.json
SHA256 : 5e62d813c14966116031d67ddf541c85fe61ff085f448de03544056cd4a954df.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-green-r1.log
SHA256 : 581a654e865c71a2a97ebdcfd7af14ee8adf22dd797d576911b461b547b9c8e6.

## T008–T011 — R2, garde validée ; T012 et T029 ouvertes

**14 PASS/1 FAIL/0 ignoré, RC101**, 16,16s ; compilation14,27s, tests1,72s.
Les 200 empreintes de sources avant/après sont identiques. Masque0077.
Les six `native153_garde_*` passent. Les trois cas purge/historique,
les trois autres cas quotas/arrêts et les deux catalogue149 passent aussi.
L'unique FAIL est `native153_limites_enfant_inactif_compte` : EXPLAIN
choisit l'autoindex owner_instance au lieu des deux recherches attendues
sur l'index étendu native_delegations_state. L'oracle reste strict.

Relecture documentaire de réalisation et d'oracles, sans exécution locale :
- T009 : bornes60..7200 et catalogue min/max/8 à
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:708 et736 ; bornes, rejeu et divergence de durée testés à2702.
- T010 : résultat et délai sauvegardés ensemble à2000 ; expiration à1637 ;
  nettoyage conditionné à l'absence de délai à1772. Le test2750 reçoit la
  vraie trame résultat, vérifie la connexion et le délai, puis confirme
  l'arrêt avec résultat/status conservés. Le témoin sans garde est à2821.
- T011 : Cancel à979 ; prepare_restart à1525 ; cancel_tree lève les gardes
  avant les arrêts à2288. Annulations agent/humain avant/après remise :
  quatre sous-cas à2857 ; reprise active/terminale à2938 ; descendant à2970.
  Fin humaine transactionnelle dans
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:455 ; libération monotone par save_projected à289.

Self-review façade reçue par le principal : contrat nécessaire de garde et
de fin durable ; admission/tick/Cancel/cancel_tree/prepare_restart et
préflight/arrêts152 réutilisés. Aucun nouveau helper, type, module ou
dépendance pour cette tranche. Les deux champs de libération restent
locaux dans quatre contextes. Verrou, sauvegarde atomique de résultat/garde
et confirmation StopOutcome conservés. Continue, E2E et workspace ne sont
pas vérifiés par ce lot ; T017 et ses intégrations restent distincts.

Le diagnostic SQLite T012 est ensuite confirmé sur
/private/tmp/b.p1xqyp4_/bridget-n148-idl-1e57648b-62170.db.
Deux clauses `INDEXED BY native_delegations_state` suffisent ; comptes
observés global4/owner4/foreign0/total6, deux SEARCH state par requête,
aucun SCAN table. SHA magasin communiqué :
2f8242259f5eee83b72203680a832060cf125b8f4ef0ef1c82d9f78dcd850f0a.
Cette observation SQLite ne prouve ni PASS Rust ni performance mesurée.
T012 attend le rejeu ROOT ; T029 attend aussi la recopie des anciens ACK.

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-green-r2.json
SHA256 : b19b09266914f3d70ef11c71718fd0317917744e5c7379d48bf18c9cd8833576.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t008-t012-t029-native153-green-r2.log
SHA256 : 9f11700a446a35f7996ad462e6c5e84e5332a3d0699abaa4a6df785876157ac5.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-2).


## T012 GREEN et T013/T014/T016/T018/T028/T029 RED compilés

**5 PASS/11 FAIL/0 ignoré, RC101**, 16,42s ; compilation15,34s, tests0,84s.
Les 200 empreintes de sources avant/après sont identiques ; masque0077.

| Preuve de ce lot | PASS | FAIL |
|---|---:|---:|
| T012 : enfant inactif compté et plan SQL corrigé | 1 | 0 |
| Transactions begin_round déjà existantes | 3 | 0 |
| Continue : admissions et recontrôles de politique | 0 | 6 |
| Vue de ronde et identifiant de résultat | 0 | 1 |
| Recopie de l'accusé ancien | 0 | 1 |
| Réponses tardives, dont témoin tiers authentifié | 1 | 3 |

T012 passe après les deux seules clauses INDEXED BY et le commentaire,
sur l'index existant. EXPLAIN affiche MULTI-INDEX OR et deux SEARCH
native_delegations_state, aucune SCAN table. Oracle inchangé, test relu à
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:3036.
Les comptes vérifiés sont (4,4,6) pour le propriétaire et (4,0,6) pour
l'instance étrangère. Le comptage de places source est à
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:415.
SHA magasin du run : 2f8242259f5eee83b72203680a832060cf125b8f4ef0ef1c82d9f78dcd850f0a.
Les autres bornes/arrêts et purge de T012 étaient PASS au lot R2 antérieur ;
aucun de ces résultats n'est additionné au compteur de ce lot.

Les six refus Continue sont encore continue_unsupported ; la vue de ronde
manque, l'ancien ACK n'est pas recopié. Trois réponses tardives passent
encore par la remise normale. Le tiers réellement authentifié suit cette
remise normale comme prévu : son témoin PASS n'est pas un GREEN T028.
Le montage T028 et ses quatre usages réels sont désormais référencés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/reuse-audit.md.
SHA D du run : bf224cd5600f9f7a29339253e59b9ae753fa28e79097978ca8efcebdc7c333b3.

Self-review stockage T012/T028, six réponses XX :
1. Nécessité : coût indexé du contrat et aucune remise parasite.
2. Réemploi : même index, handlers capture et trois primitives D existantes.
3. Structure : une fixture cfg(test) autorisée, quatre usages, aucun hook/module de production.
4. Vérifié : EQP réel et RED métier compilé ; montages locaux authentifiés.
5. Limites : capture pure N encore RED, aucun modèle réel ou benchmark de production.
6. Simplicité : INDEXED BY suffit ; aucun patch D de production ni extraction de confort.

JSON durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t012-t013-t014-t016-t018-t028-t029-native153-red-r1.json
SHA256 : 42756ec288d7389d20a208625208e834910cab29f107cfa72b16e691a122afd7.
Journal durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t012-t013-t014-t016-t018-t028-t029-native153-red-r1.log
SHA256 : 4e4848bb7ec8937ee44a92c506f2387fc96a2e757b3eba239c5fbd21e359d93e.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-3).


## T016/T018/T020/T028/T029 — preuves ciblées, 39 PASS/1 FAIL

Lot de40 tests : **39 PASS/1 FAIL/0 ignoré, RC101**,15,79s ;
compilation12,28s, tests3,38s. Les200 empreintes de sources sont stables.
L'échec est native153_ronde_refus_ordonnes_et_fin_durable, avant l'appel
Continue étranger : la fixture n'obtient pas Registered. Aucun défaut
comportemental Continue n'est démontré par cet échec de montage.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-4).

JSON : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-t014-t015-t016-t018-t020-t028-t029-native153-green-r1.json
SHA256 : cdfed4c2908f2f9104e34fe1602472edff6b4cfa289bd2609dd50de5648ff560.
Journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-t014-t015-t016-t018-t020-t028-t029-native153-green-r1.log
SHA256 : e858b5c6cd852ccb9687ad320ebad1b52b32ac5d37af93ff670dc3fd63f51302.
Ces cinq tâches sont cochées après GO et relecture : checkpoint17/29.

## Diagnostic du montage étranger et observateur FR-031

**10 PASS/1 FAIL/0 ignoré, RC101**,11,42s ; compilation8,92s, tests2,35s.
Sources stables. Le vrai retour Register est Nack : agent_id invalide
autre-parent153. La correction concerne seulement la donnée fixture.
Les39 PASS inchangés ne sont pas rejoués. Les dix tests observer149 existants
passent, dont trois ajustés pour Continue. Matcher, fait courant corrélé
et ACK réutilisent149. T017 demeure ouverte jusqu'à l'intégration workspace.

JSON : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-fixture-diagnostic-et-observer153-r1.json
SHA256 : ea312bc92e1b21407e1d7076c3d2abf5016f9015cc9e00854b2ce96297a74313.
Journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-fixture-diagnostic-et-observer153-r1.log
SHA256 : 3620cc72c0319617e9a1cdbbf860b8fc72f6710a27c808db09bcfc4f79508ac8.

## Montage étranger corrigé — GREEN ciblé, T013/T014/T015

**1 PASS/0 FAIL/0 ignoré, RC0**,10,40s ; compilation7,42s, tests1,12s.
Sources stables pendant le run. Fixture UUIDv4 et socket primaire, aucun
changement de production ou de garde. Le préfixe production N reste
53460218850e03d812a03efed8468fe5b184831b114fab6c3144817de3092741.
SHA N du run : d3ea623489cf99a329b8f91774b0dc9e5da9de4c5b5491ddb0601f1924c131b9.
Avec les39 PASS antérieurs et les dix observer PASS, les preuves prévues de
T013/T014/T015 sont acquises et ces seules trois tâches sont aussi cochées.
La matrice de politique couvre douze combinaisons et sources/réduction/
absence ; égalité complète conservée. Compatibilité native148 finale reste
à comparer en T021. Aucun total de40 ou50 sur une invocation unique inventé.

JSON : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-fixture-corrigee153-green-r2.json
SHA256 : 91daea7cbd2e206fc655624da42771a5b1f0a6a0d795f1a1e5c6915669c93020.
Journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t013-fixture-corrigee153-green-r2.log
SHA256 : 9854dbd2d78006b5ad8af312b08ebccd9a5819b32fa018ec2e01bdd2549e04d7.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-5).


## Finding T026 en cours — FR-019, propriété avant refus

Lecture du principal :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1055.
recover_owner peut sauvegarder owner_instance avant un refus ou un rejeu
Continue, si le même owner utilise une nouvelle instance et l'ancienne est
absente. FR-019 interdit cette mutation hors fin durable de garde admise.
Le constat est confirmé par lecture ; correctif et preuve ciblée restent
à produire après le workspace gelé en cours. Ne pas transformer cette limite
en exception au contrat. T015 rouverte en phase Implement : checkpoint19/29.
T017 reste aussi ouverte : oracle MCP de catalogue27 au lieu de28 en diagnostic.
Aucun statut Implemented ou Converged ; aucun second workspace demandé.


## Documents T022/T023 et consignation T024

T022 relue contre les31 FR, le contrat rounds153 et les vues/handlers actuels :
garde60..7200, huit rondes, clé par ronde, brief handoff16 384 octets, sept refus,
fin durable, droits complets, résultat courant et limites mémoire/biais/restart.
Les trois documents existants sont étendus. Le catalogue fermé contient24 noms
uniques, dont Continue. Aucune configuration installée ou activation déduite.
T023 : seul ADR153 prévu créé après le gate de réemploi ; D1–D12 présentes.
D12 explicite I1/I5/I7/T004, sans nouveau contrat. Statut architectural Accepté,
sans annoncer fin d'implémentation ou livraison. T015 reste rouverte sur FR-019.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-6).

Six réponses XIX/XX documentaires : nécessité = tâches T022–T024 explicitement
prévues ; simplicité = trois documents réutilisés et seul ADR prévu ; hypothèses =
catalogue réellement chargé et droits complets toujours valables ; vérifié =
relecture exacte des nouveaux blocs, catalogue24 et douze décisions raccordées ;
non vérifié = modèle réel, activation, E2E final et correction FR-019 ; conservé =
limites, gardes, preuves de remise et contre-revue indépendante, sans outil neuf.

## Workspace R1 — preuve finale exécutée, corrections ciblées encore ouvertes

**94 cibles,2145 PASS/10 FAIL/72 ignorés, RC101,550,82s**. Les200 empreintes
Rust/lock avant et après sont identiques. Le résumé brut2148 PASS inclut trois
sous-tests de sous-processus ; il reste conservé, sans remplacer le compteur
canonique. L'analyse V2 prend le dernier résultat de chacune des94 cibles.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-7).

Deux nouveaux FAIL : matrice_fr009_couvre_les_quinze_cas attend27 au lieu28 ;
native153_e2e_trois_rondes s'arrête sur round durable0 contre vue1, avant rondes2/3.
Les corrections sont en cours. T017/T019/T025 restent ouvertes. Le défaut statique
FR-019 impose aussi correction et test ciblé ; aucune autre suite workspace n'est
requise si les deltas finaux et leurs effets sont délimités. Aucun GREEN global.

Copies byte-exactes dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/ :
workspace-153-r1-results.json SHA0c892ce027088334a65a2b2a8d17009af6e7d75fcc5a104362bd07fe851927b1 ;
workspace-153-r1.log SHAb49eb3d27121d6e6acb92b0c8c297e97c02d2ba6d2195900edbd7d9c8c004b9a ;
workspace-153-r1-analysis-v2.json SHAf8d8485ac0e7ae80a5a643c6f0a55c29e44e7fdb146192c9d47f8d27545116b9.
La première analyse est aussi conservée, avec son statut supersédé.

Précision T026 de minimalisme : holds_slot possède neuf assertions dans quatre
tests (cinq lineage, quatre N). L'annotation cfg(test) ajoutée par son propriétaire
est le seul delta de17 octets ; son retrait restitue exactement le SHA antérieur.
Source magasin SHA5f6233f81a185aa713b2a1bac9beebcf030e375640aa355c726933c8576f68d4.
Cette preuve d'identité des octets ne prétend pas une compilation production.

## Clôture finale153 — 29/29

FR-019 R1 :E0382 du test,RC101,3,79s, zéro test exécuté.
R2 :1 PASS MCP/2 FAIL natifs,RC101,8,81s ; vrais refus/rejeux avaient modifié
owner_instance/epoch. Correction : can_own_task pur, WaitEpoch sous verrou,
tick après nouvelle admission committée seulement, constructeur round1.
GREEN final40 lib +1 E2E PASS,0 FAIL/ignoré,RC0,69,14s, sources stables.
E2E57,56s : trois rondes/missions/résultats, même processus/instance,
16 enfants inactifs/17e refus et fin explicite ; pas preuve de mémoire réelle.

T017 fermé : schéma/catalogue, matrice15 PASS R2, dix observer149 PASS et
wrapper::reconnect_tests::native148_claude_allows_native_delegation_without_global_mcp_approval
PASS workspace. Gardes de droits/sources/nonce/identité conservées.

T021 final :179 lib PASS/0 FAIL/0 ignoré,179/179 baseline mêmes noms/résultats,
RC0,65,33s. Binaire frais du GREEN40+1, SHA avant/après
f9a158390e3138926504eaa253f1f63b4ca6c6b2ce032e40fead40e15e0165dc.
16 autres noms (15 PASS/1 ignoré) maintenus par workspace V2 nom+cible et impact :
200 sources/lock,197 identiques, seuls N/store/MCP changent. Trois edits produit N :
can_own_task pur, epoch/Continue et round initial1. Store cfg(test)17 octets ;
MCP oracle27→28. Intégrations génériques sans Continue inchangées, variantes150
round_no/ID1 inchangés. Aucun second workspace ni baisse de couverture.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/153-rondes-contexte/validation/tests-153.md#extrait-8).

Reçus byte-exacts :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t015-t019-native153-corrections-green-r1.json
SHA256 : 45d6ab537328caaf8af27d63cbe2c5c2734d5ac6245116566590d175cc4976f9
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t015-t019-native153-corrections-green-r1.log
SHA256 : c1e33da9afdc4fb6101b3a9915909fda961ac63222ff01a094bd17f6e7cb24d8
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t021-final-lib-regressions-r1.json
SHA256 : 39b6f84fcade50294b7fd471558a4e81f887f3c94284cdc7ffab4d4c84b8d4be
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/workspace-153-final-impact.json
SHA256 : 4de6a52049f59346cab3d5c7e31aa597c78050d6284af5bd2fbecd55028c1444

## Converge — passage1 acquis

Verdict indépendant CONVERGED :31 FR/17 SC/5 US confrontés, zéro tâche manquante
et zéro ajout. Tâches29/29 byte-identiques, SHA256 a406fc926f3d0ddaa6896d2fdf72ced204aa14d50fcdd0e6b7a0d76090d7f5e2.
Reçu byte-exact : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/converge153-pass1.json
SHA256 : 6564ff059bc498cf8be99d5aff872ee990794c0999672ec28930fb92d31d5dd3.
La matrice et les six réponses XX restent dans ce reçu ; elles ne sont pas
dupliquées ici. Les limites du reçu restent applicables : horloges/cadence sans
mesure murale, fournisseur synthétique sans mémoire de vrai modèle, preuves
workspace/ciblées séparées et aucun GREEN global. Contre-revue code R1 en attente ;
audit et livraison non déclarés exécutés.
