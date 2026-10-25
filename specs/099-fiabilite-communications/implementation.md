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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-4).

Autres régressions sûres passées : stockage (filtre store::) 44 ; idempotency::
39 ; identité MCP 10 ; attribution CLI 7 ; pression annulation 3 ; matrice des
rôles 7 ; transport JSONL 2, journal 14, act_kind 2, refusals 5. Les filtres ne
doivent pas être additionnés à spec099 sans dédupliquer leurs tests communs.
T004–T015 terminées. T016 désormais terminée : preuves de convergence et d'audit ci-dessous.

## Vérification finale des lots

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-5).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-6).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-7).

Limite volontaire : cargo test --workspace SANS --no-run n'est pas exécuté.
Les harnais historiques utilisent SIGKILL/groupes (notamment
crates/bridget-daemon/tests/support/idempotent.rs:85), interdits par les règles
utilisateur. Leur compilation et analyse statique sont vertes ; pas de prétention
de réussite à l'exécution. Aucun fournisseur réel, vraie fédération SSH,
audit de dépendances/CVE ou benchmark de charge longue exécuté.

## Correspondance exigences → code et tests

Chemins de code ci-dessous relatifs au dépôt
/Users/moi/Nextcloud/10.Scripts/64.bridget ; références relues sur le diff final.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/implementation.md#extrait-8).

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
