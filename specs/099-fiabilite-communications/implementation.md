# Exécution 099

Statut : Implemented — 16/16 tâches vérifiées. Début : 2026-09-16 06:28 CEST.
ETA initiale 58–108 min ; après tâches : 50–90 min restantes vers 06:38 CEST.
Aucun commit/déploiement ; état réel de l'installation hors périmètre.

## Baseline et oracles

- cargo build --offline --locked -p bridget-daemon : PASS.
- cargo test --offline --locked -p bridget-core : 39 PASS.
- cargo test --offline --locked -p bridget-daemon --lib spec098 : 13 PASS.
- reproduce_daemon.py : blocage global + emprunt MCP reproduits, enfant arrêté.
- reproduce_t3.py : annulation suivie de dispatch, réponse non confirmée oubliée,
  demande durable encore open ; trois enfants arrêtés.
- Scripts et logs de preuve : chemins absolus consignés dans spec.md.

## Outillage et dégradations

sync-project.py : exécuté, déjà à jour. Moteur local (templates/scripts/bash)
absent ; setup-plan, setup-tasks, check-prerequisites tentés : exit 127.
Skills specify/plan/audit-existing/tasks/analyze/implement lues par le principal,
protocoles appliqués directement. Aucun outil global réinstallé.
Mémoire projet au chemin conventionnel absente, constatée.
Analyse croisée réalisée dans ce tour, voir analysis.md.
Contre-revue Claude : envoi refusé identity_not_found, voir adversarial-review-claude.md.

## Revue de T001

Nécessaire : conserver des scénarios observables avant correction.
Réutilisation : scripts de preuve et fixtures existants, feature descriptive 099
sans nouveau framework de test. Hypothèse : processus et HTTP synthétiques isolés.
Vérifié : sorties réelles ci-dessus ; pas encore la correction.
Non vérifié : comptes fournisseurs, production, performance en charge longue.
Code évité : aucun banc de messagerie parallèle ajouté en production.

## Résultats par tâche

T001 terminée : preuves initiales et scénarios d'acceptation conservés.
T002–T003 terminées : tests spec099_classic_delivery_tests, 1 PASS/2 FAIL avant
correction, puis 3 PASS en 2,17 s. Tests à vrais socketpair ; dix consultations
et un échange témoin pendant une écriture saturée ou un verrou writer tenu.
Le suivi est créé avant la remise ; l'état terminal ne peut être rouvert après.
Écriture et acquisition writer bornées par la primitive existante, hors état
global ; Nack explicite si persistance ou écriture échoue. Aucune seconde
primitive writer. Diff relu intégralement pour cette tâche.

T009 terminée : la quatrième régression daemon passe, module 4/4 PASS en 1,15 s.
La notification est structurée pour mode ACP OU transport t3code ; aucun autre
transport CLI n'est modifié. La compilation initiale avait croisé des champs
de tests encore en migration, pas un échec de l'oracle.

Recettes après correction (vrai binaire debug, aucun compte fournisseur) :
- reproduce_daemon.py --expect-fixed : PASS, /tmp/bg-audit-behy875h. Identité
  MCP déclarative refusée ; auxiliaire avec preuve réelle accepté. Annuaire
  0,22 ms au repos et 0,78 ms pendant cible saturée ; Nack sur écriture échouée.
  Le processus enfant s'est terminé, code 0.
- reproduce_t3.py --expect-fixed : PASS, /tmp/bg-t3-audit-kwgrk5r3. Annulation
  confirmée, zéro démarrage du travail annulé ; réponse sauvegardée pendant
  déconnexion, remise après redémarrage et demande answered. Une seule
  exécution fournisseur. Les quatre processus possédés sont arrêtés et vérifiés.
- Protocole transport : 67/67 PASS ; cœur : 39/39 PASS après correction.

Relecture indépendante de même fournisseur : deux cas limites supplémentaires
retenus avant clôture US4/US5 : conserver le besoin de publication après refus
enqueue sur fil inchangé ; ne pas effacer Pending sur réponse HTTP 2xx malformée
ou erreur ambiguë après acceptation possible. Deux tests rouges avant correction,
puis verts ; module t3code 25/25 PASS dont 13 nouvelles régressions.
Le refus enqueue est simulé par un writer arrêté, pas une saturation réelle ;
la branche d'erreur et sa reprise à clé inchangée sont vérifiées. Le journal
arrête le lot au premier refus pour préserver l'ordre.

Recette enrichie /tmp/bg-t3-audit-1dewkaac : demande expirée et perte du daemon
pendant une vraie attente busy n'ouvrent pas le travail original. Cependant,
l'oracle initial était trop faible : dispatch.jsonl contient un tour de rappel
créé par le daemon après expiration (commandId 1ae8b0e562684). Le PASS de ce
premier script enrichi ne vaut donc PAS validation complète SC03. Oracle renforcé
à zéro tour de rappel ; la correction daemon ciblée et le nouveau test rouge/vert
ont ensuite résolu ce défaut, sans changer les rappels des autres transports.

Autres régressions sûres passées : stockage (filtre store::) 44 ; idempotency::
39 ; identité MCP 10 ; attribution CLI 7 ; pression annulation 3 ; matrice des
rôles 7 ; transport JSONL 2, journal 14, act_kind 2, refusals 5. Les filtres ne
doivent pas être additionnés à spec099 sans dédupliquer leurs tests communs.
T004–T015 terminées. T016 désormais terminée : preuves de convergence et d'audit ci-dessous.

## Vérification finale des lots

T004–T006 : les trois voies d'emprunt d'identité sont refusées, la preuve valide
fonctionne et la rotation invalide les connexions déjà admises. Réutilisation des
contrôles de connexion, du stockage privé et des clients ; seul le type secret et
la trame d'attestation sont nouveaux. Tests supplémentaires : rôles Attach/Service,
preuve d'une autre instance, secret absent de Debug/annuaire, fichier 0600 borné et
sans symlink. Les mocks de protocole ont une preuve synthétique vérifiée ; tout
test parlant au vrai daemon obtient sa preuve par Registered. Aucun bypass de
production. Les tests ont été écrits avant correction, mais leur première
compilation a croisé le travail t3 ; la preuve rouge initiale est aussi la recette
réelle d'usurpation. Non vérifié : tunnel SSH réel, compte OS hostile.

T007–T009 : lecteur socket indépendant, contrôles partagés revérifiés à la
frontière HTTP et file existante étendue ; pas de second moteur de jobs.
T010–T011 : response optionnelle dans Pending et sauvegarde atomique existante ;
aucun retrait sur Nack, absence de liste bornée ou HTTP ambigu. Test de deux attentes,
rechargement, ACK perdu et anciennes données. Pas de nouvelle table/outbox.
T012–T013 : texte complet sous borne JSON de 4 Mio ou gap explicite, confirmation
par lecteur incrémental existant et failure sink. Refus retenté, ordre préservé,
aucun curseur avancé sur simple enqueue. Coût assumé : lecture du journal et
persistance nécessaires à cette preuve ; pas de mesure en charge longue.

T014 : README français/anglais, ADR035, contrat, plan et quickstart cohérents ;
mise à jour conjointe des clients explicitée, aucune migration destructive.

Emplacement des recettes T007/T010 ajusté avant convergence : unités dans
t3code.rs et vrais processus dans reproduce_t3.py du dossier de preuves, au lieu
du harnais t3code_098_test.rs qui réutilise les arrêts SIGKILL. Même faux serveur
HTTP et mêmes scénarios fonctionnels, aucune réduction de périmètre. Ces recettes
sont conservées comme artefacts d'audit locaux ; les régressions Rust font partie
du diff versionnable et ne dépendent pas du dossier d'audit.

Consolidation US1 : une notification système pouvait encore attendre son writer
sans borne. Nouveau test rouge puis vert, réemploi de push_control_message_until
dans deliver_to_agent. Suppression du type DeliveryError et de ses branches
redondantes ; notifications Timeout et DeliveryRejected hors verrou global.
Les notices orphelines historiques gardent leur chemin existant, mais chaque
écriture est désormais bornée. Cela n'établit pas un SLA global tous chemins.
Le test writer occupé atteste aussi le suivi durable AVANT libération du writer.
Deux fixtures 046 ont été corrigées pour envoyer leur identité attestée : elles
ne doivent plus attendre qu'un champ from arbitraire soit remplacé implicitement.

## Résultats finaux reproductibles

Préfixe employé pour les unités daemon du principal :
env BRIDGET_HOME=/tmp/bg099-validation.ZaTKHv BRIDGET_SOCKET=/tmp/bg099-validation.ZaTKHv/b.sock TMPDIR=/tmp
Le HOME utilisateur n'est pas modifié. Les recettes processus définissent leur
propre HOME enfant privé, sans profil fournisseur.

| Commande (cargo = /Users/moi/.cargo/bin/cargo) | Résultat |
|---|---|
| cargo build --offline --locked -p bridget-daemon | PASS |
| cargo test --offline --locked -p bridget-core | 39 PASS |
| cargo test --offline --locked -p bridget-daemon --lib spec099 | 25 PASS |
| cargo test --offline --locked -p bridget-transport --lib protocol::tests::spec099_credential_optionnel_compatible_et_masque_dans_debug | 1 PASS |
| cargo test --offline --locked -p bridget-daemon --lib spec098 | 13 PASS |
| cargo test --offline --locked -p bridget-daemon --lib mcp::tests | 43 PASS |
| cargo test --offline --locked -p bridget-daemon --lib communication::client::security_tests | 13 PASS |
| cargo test --offline --locked -p bridget-daemon --lib wrapper::reconnect_tests::spec094_reconnexion | 2 PASS |
| cargo test --offline --locked -p bridget-daemon --lib session_046_ | 4 PASS |
| cargo test --offline --locked --workspace --no-run | PASS, 19,04 s ; compilation de toutes les cibles de tests |
| cargo check --offline --locked --workspace --tests --features test-support | PASS |
| cargo clippy --offline --locked --workspace --all-targets -- -D warnings | PASS |
| cargo clippy --offline --locked --workspace --tests --features test-support -- -D warnings | PASS |
| cargo fmt --all -- --check ; git diff --check | PASS |

Autres filtres sûrs et résultats consignés plus haut. Les essais intermédiaires
de compilation/fmt ont échoué sur la migration simultanée des fixtures ; ces
erreurs ont été corrigées et les commandes finales ont été relancées. Aucun
échec final masqué. L'espace disponible a été surveillé, sans nettoyage ; le
link global a finalement pu être exécuté, après retour de capacité disque.

Recettes finales sur le dernier binaire :
- reproduce_daemon.py --expect-fixed : PASS, /tmp/bg-audit-4j93yaxz ; annuaire
  0,24 ms au repos et 1,97 ms sous pression, identité illégitime refusée,
  rattachement valide accepté, Nack sur remise échouée ; enfant terminé code 0.
- reproduce_t3.py --expect-fixed : PASS, /tmp/bg-t3-audit-fz7xnjmr ; zéro tour
  après annulation, expiration et perte du daemon durant une attente busy.
  Aucun tour de rappel parasite. Une réponse durable remise après redémarrage,
  demande audit-lost-79ab2f37 answered, exactement un dispatch fournisseur.
  Les quatre enfants possédés ont tous été arrêtés et leur sortie vérifiée.

Limite volontaire : cargo test --workspace SANS --no-run n'est pas exécuté.
Les harnais historiques utilisent SIGKILL/groupes (notamment
crates/bridget-daemon/tests/support/idempotent.rs:85), interdits par les règles
utilisateur. Leur compilation et analyse statique sont vertes ; pas de prétention
de réussite à l'exécution. Aucun fournisseur réel, vraie fédération SSH,
audit de dépendances/CVE ou benchmark de charge longue exécuté.

## Correspondance exigences → code et tests

Chemins de code ci-dessous relatifs au dépôt
/Users/moi/Nextcloud/10.Scripts/64.bridget ; références relues sur le diff final.

| Exigence | Réalisation | Preuve |
|---|---|---|
| FR01 | crates/bridget-daemon/src/daemon.rs:1518 et :11856 | socket saturée/writer occupé :8882/:8887 ; recette daemon |
| FR02 | crates/bridget-daemon/src/daemon.rs:12108 | Nack attesté sur vraie écriture échouée |
| FR03 | crates/bridget-daemon/src/daemon.rs:7419, appel avant push | test writer occupé vérifie tracked_requests avant remise ; réponse liée :8927 |
| FR04 | crates/bridget-daemon/src/daemon.rs:7232/:7244/:7465 | cinq tests d'autorisation :25003 et suivants ; recette auxiliaire réelle |
| FR05 | crates/bridget-daemon/src/daemon.rs:2957 ; mcp_identity.rs:79/:113 | rotation/même instance :25047 ; fichier :618 ; protocole :3880 |
| FR06 | crates/bridget-daemon/src/communication/client.rs:773 ; wrapper.rs:1632 | MCP43, client13, reconnexion2, canon CLI réel ; README bilingue |
| FR07 | crates/bridget-daemon/src/t3code.rs:1452/:1575 ; daemon.rs:6389 | contrôles :2283/:2315/:2483 ; rappel :20963 ; recette busy complète |
| FR08 | crates/bridget-daemon/src/t3code.rs:1892/:1936 | reprise :2437 ; HTTP ambigu :2643 ; recette redémarrage |
| FR09 | crates/bridget-daemon/src/t3code.rs:186/:220 | deux attentes :2192 ; ancien état :2473 |
| FR10 | crates/bridget-daemon/src/t3code.rs:1719/:1814 | Unicode :2237 ; refus :2268 ; panne :2364 ; reprise :2397/:2602 ; gap :2511 |
| FR11 | reuse-audit.md et diff des manifests vide | aucune dépendance, table ou service ; types/maps existants étendus |
| FR12 | fixtures socketpair et recettes Python dans audits/2026-09-16/session-2026-09-16-bridget-global-01 | états privés, faux HTTP, SIGTERM individuel, aucun déploiement |

SC01 : deux tests de pression avec dix consultations et échange témoin <1 s ;
SC02 : négatifs/positif/révocation ; SC03 : recette busy sans dispatch indu ;
SC04 : réponse reçue après redémarrage sans second travail ; SC05 : Unicode/gap ;
SC06 : tableau des commandes réussi et limites d'exécution nommées.

## Convergence et audit final

CONVERGED en un passage manuel : 12 exigences FR et 6 critères SC confrontés
au code réel et à leurs tests ; aucun manque de réalisation dans le périmètre.
La phase Converge n'a modifié ni le code ni tasks.md : SHA256 identique avant
et après cette phase, bb58ed0e17e999ed1dbb7e591212ff7196d90fe36a59a0866d3e5a7514b54512.
La clôture de T016 et la mise à jour de statut ont lieu ensuite, après l'audit.

Audit v14 mode fix, puis cycle-scoring en lecture seule : zéro défaut résiduel
confirmé sur le diff099, zéro patch supplémentaire pendant l'audit. Les notes
A/100 sont mécaniques et bornées à ce diff ; elles ne certifient pas tout Bridget.
Rapport :
/Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-09-16/session-2026-09-16-spec-099-01/scoring.md
Sorties structurées : grade.json, manifest.json, fix-report.md, cycles.log,
findings des sept modules et agrégats cycle-1/cycle-scoring dans ce même dossier.
fingerprint.py --batch exécuté sur chaque agrégat : exit0 ; validate_session.py :
exit0, zéro erreur, zéro warning, le 2026-09-16 à 07:27 CEST.
Le hash du diff Rust reste identique pendant le cycle de notation :
4a50419973732aefd0f42ff0b345db392f2102359089f6f2c707a9063129249c.
Aucune baseline présente ou modifiée ; aucun finding supprimé.

Contre-revue d'un autre fournisseur : Claude, agent bdget
127bccff-8490-453a-8182-884b749ec41e, sollicité après le plan puis retenté après
implémentation. Les deux envois ont échoué avec identity_not_found avant
livraison. Aucun agent d'un autre fournisseur joignable depuis cette session
non enregistrée ; aucun verdict Claude et aucune objection de sa part inventés.
Les objections retenues de la revue de même fournisseur sont consignées plus haut.

Phases1–7 réalisées : synchronisation, spécification/plan, gate de réutilisation
PASS puis tâches, analyse en deux passes, implémentation, convergence, audit.
Les scripts locaux absents et la primitive converge absente ont été remplacés
par les protocoles manuels documentés, sans réduire US1–US5. Aucune phase omise.

Créations justifiées : preuve privée, trame RegisterAuxiliary et champs de
réponse/contrôle dans l'état existant ; ADR035, artefacts099 et régressions.
Réutilisation : primitive d'écriture bornée, maps d'identité, stockage atomique,
Pending, tracker idempotent, lecteur de journal. Arbitrages dans reuse-audit.md.
Aucune dépendance, table, service ou framework ajouté.

Temps au point de clôture : environ 61 minutes (06:28–07:29 CEST).
ETA initiale58–108min (centre83), recalibrée après tâches50–90min restantes
à06:38 (centre70 restant, soit80min total). Écart au centre initial : environ
−27 % ; au centre recalibré : environ −24 %. Cause principale : lots identité
et pont traités en parallèle, réemploi des primitives et compilation incrémentale.
L'exécution globale interdite n'est pas comptée comme une vérification réussie.

Première tâche non cochée : aucune. Aucun commit ni déploiement ; diff à relire
par l'utilisateur. Prochaine action proposée : revue humaine, puis validation
séparée de la mise à jour coordonnée des clients et essais SSH/fournisseurs réels.

## Reprise du 2026-09-16 (bdget, Claude) — recette complète et corrections

La recette complète n'avait pas été exécutée (modules ciblés seulement). Lancée
avec racine temporaire courte (`/tmp/b.XXXX`), `umask 077`, `--no-fail-fast` :
1333 réussites, 11 échecs, 49 ignorés sur 75 cibles. Causes et corrections :

| Échecs | Cause | Correction |
|---|---|---|
| 2 × presence_tests | anciens tests envoyant en SendIdempotent au nom de « human » sans identité | migrés vers un expéditeur attesté (T017) |
| 6 × claude_interactive_097 | la CLI exigeait une identité d'agent pour tout envoi idempotent : humain refusé | humain d'un client nu admis, côté CLI et daemon (T018) |
| core_089_isolation | `bridget_who` exigeait une preuve auxiliaire | annuaire public sans preuve (T019) |
| core_089_security | faux daemon attendant ClientHello avant RegisterAuxiliary | ordre des étapes aligné (T020) |
| managed_parity matrice_fr008 | état busy lu juste après l'envoi ; remise asynchrone depuis la 099 | attente d'état + diagnostic (T021) |

Vérifications : parité rejouée 3 fois à vide (verte) ; référence avant 099
(worktree 9739fc84) verte ; recette complète après corrections : 1344 réussites,
1 échec (fixture de concurrence 089 attendant encore un enregistrement pour
`who`, alignée ensuite : 3/3), puis recette complète finale : **1345 réussites, 0 échec, 49 ignorés sur 75 cibles** ; `cargo fmt --check` et `cargo clippy --workspace --all-targets` sans remarque.
Un tour métier en trop a été observé une fois dans la parité pendant une
compilation concurrente ; non reproduit à vide ; piste consignée (ADR035).

