# Noyau SPEC-138 — preuves réelles

## Première RED avant code de production

Commande réelle : `cargo test -p bridget-daemon spec138_ -- --nocapture`.
Cargo : `/Users/moi/.cargo/bin/cargo`.
Target : `/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG`.
TMPDIR, BRIDGET_HOME et socket privés sous `/tmp/b138c.*`, umask 077.
Session d'exécution 66481 ; résultat final du transport d'outil : 6569fa.
Sortie réelle : exit 101 ; 0 passed, 5 failed, 1021 filtered out.
Cette fiche retranscrit les résultats de l'exécution. Elle ne constitue pas
une capture exhaustive du flux stdout/stderr.

Tests réellement RED :

- `communication::tests::spec138_reason_is_part_of_canonical_envelope` :
  assertion `left != right` ; le champ inconnu était perdu avant le canon.
- `communication::tests::spec138_explicit_null_reason_is_not_legacy_absence` :
  null explicite était accepté comme omission.
- `cli::hook_tests::spec138_directory_accepts_explicit_global_and_project_root` :
  les nouvelles options étaient refusées.
- `mcp::tests::spec138_valid_send_reason_reaches_transport_without_rewriting_body` :
  `InvalidParams("argument inconnu : cross_project_reason")`.
- `handoff::tests::spec138_handoff_accepts_structured_reason_without_body_rewrite` :
  `request.cross_project_reason`, `champ inconnu`.

## Réutilisation avant création

- `prepare_dispatch`, `live_connection_identity` et `register_auxiliary` restent
  les points de contrôle des envois et de l'identité.
- Le domaine mutable et le registre runtime retiré ne participent pas au fait.
- `send_deliveries.message_bytes` garde déjà l'enveloppe durable. Les warnings
  figés sont des métadonnées de ce JSON existant. Ils ne sont pas livrés au
  destinataire par la désérialisation du message.
- `idx_send_deliveries_kind_key` existe déjà dans idempotency.rs ; le lookup
  corrélé réutilise cet index et lit au maximum deux lignes. Aucun index ni
  table de consentement n'a été créé.
- Une demande OPEN avec participants inversés hérite seulement du motif d'une
  enveloppe autorisée et durable, en phase dispatching ou acked. Dispatching
  suffit pour répondre immédiatement : le mandat est admis avant la remise,
  même si son accusé de réception technique n'est pas encore revenu.
- Les faits inconnus restent compatibles avec un warning. Une absence initiale
  peut être renseignée ; deux racines connues contradictoires restent inconnues
  jusqu'à la reconnexion. Les connexions auxiliaires héritent du parent.

## Contrôles intermédiaires

- Première compilation bibliothèque : quatre erreurs de mécanique/types,
  dont deux noyau et deux champs de façades encore en cours. Aucun verdict PASS.
- Suite bibliothèque suivante : 14 PASS et un échec de fixture MCP ancien
  serveur. Les deux tests RED noyau étaient GREEN.
- Après ajouts de preuves : une fixture auxiliaire échouait car le scope du
  test avait 17 octets, contre 22 minimum dans le contrat existant. La fixture
  a été corrigée ; aucune garde de production n'a été assouplie.

Les tests ajoutés ultérieurement ne sont pas présentés comme RED antérieurs.
Les validations finales restent à consigner après leur exécution.

## Complément T019 : RED réellement exécutée

Le filtre `--lib spec138_steer_guard_refuses_crossproject_before_provider`
a échoué avant code de production : exit 101, zéro PASS, un FAIL.
Le résultat observé était `ControlExecutionResult / OutcomeUnknown`, au lieu
du refus `cross_project_reason_required` attendu. Capture complète :
`specs/138-priorite-projet/evidence/core-control-red.log`.

Deux essais de préparation ne sont pas des preuves RED du défaut : le premier
ne compilait pas ; le second employait un rôle Wrapper dans la fixture. Ces
erreurs de test ont été corrigées avant la capture ci-dessus.

Le filtre stockage `--test execution_store_test spec_138_control_warnings`
a produit exit 101, zéro PASS, deux FAIL, 14 filtered out. Les deux assertions
observaient dix colonnes, au lieu des onze requises. Capture complète :
`specs/138-priorite-projet/evidence/core-control-store-red.log`.

La production T019 est postérieure à ces deux captures. Elle réutilise le
registre `execution_control_commands`. Le warning JSON est figé par la
transition atomique prepared vers dispatched. Les refus fermés sont conservés
dans le champ de refus existant. Aucun second registre n'a été créé.

## Réutilisation du test de contrôle réel

Recherche par nom et responsabilité avant création : `ControlExecution`,
`ExecutionStore`, `spec138_`, `Client` et `spawn_daemon` dans les tests existants.
Le contrôle de daemon.rs utilisait une socketpair et un état en mémoire ; ce
n'était pas une preuve de redémarrage d'un daemon réel. Le test de fils102
possédait déjà ses propres scénarios, sous un autre propriétaire.

Le fichier `crates/bridget-daemon/tests/spec138_project_test.rs` n'introduit
donc aucun harnais. Il réutilise `support/idempotent.rs` : socket privée,
processus daemon réel, clients, isolation, suivi du processus et arrêt SIGTERM.
Il assemble ces composants existants pour la responsabilité distincte du
pilotage SteerCurrent, de son accusé durable et de son rejeu après redémarrage.
Il n'est pas présenté comme une RED antérieure : son exécution GREEN reste
à consigner ci-dessous.

## GREEN final exécuté par le noyau

- `cargo test -p bridget-daemon --test spec138_project_test -- --nocapture` :
  exit 0, un PASS, un helper de performance ignoré. Un vrai daemon a été
  redémarré sur socket privée. Refus sans motif et motif invalide stables,
  corps exact, motif normalisé, warning durable, ACK accepté, changement de
  faits après reprise, reçu accepté et Lookup identiques, aucune seconde remise.
  Capture : `evidence/core-control-real-green.log`.
- `cargo test -p bridget-daemon --lib spec138_ -- --nocapture` : exit 0,
  25 PASS, un test Agent Loop opt-in ignoré. Capture :
  `evidence/core-green-final.log`. Ce passage inclut les nouvelles assertions
  de tiers sans mandat, demande clôturée et expéditeur de contrôle falsifié.
  La falsification conserve le même refus InvalidCommand au rejeu et ne
  produit aucun push vers le fournisseur.
- `cargo test -p bridget-daemon --lib
  temoin_commande_controle_est_recue_puis_resolue_par_le_wrapper_cible
  -- --nocapture` : exit 0, un PASS. Capture :
  `evidence/core-control-legacy-green.log`.
- Binaire de test issu de cette dernière compilation, filtre
  `spec_087_pause_interrompt_le_tour_en_cours --nocapture` : exit 0, un PASS.
  Le chemin Interrupt interne sans message reste valide. Capture :
  `evidence/core-control-interrupt-green.log`. Un premier filtre exact avec
  mauvais nom de module sélectionnait zéro test ; il n'est pas compté PASS.
- rustfmt ciblé et `git diff --check` : exit 0.

Les suites globales, la recette Agent Loop opt-in, le lint et la revue finale
restent sous la responsabilité du principal et des autres propriétaires.
Le noyau n'a fait ni commit, ni installation, ni déploiement.

## Complément Analyze SC13805 — homonymes réels

Le test manquant `communication::tests::
spec138_same_basename_repositories_remain_other_projects` a été ajouté
dans communication.rs. Deux parents privés distincts portent chacun un
dépôt Git initialisé avec le même basename `same-name`. Le helper de
résolution existant renvoie deux racines complètes distinctes, une relation
Other et un refus sans motif volontaire.

Commande ciblée avec `--exact --nocapture` : exit 0, un PASS, zéro FAIL,
1046 filtered out. Capture complète : `evidence/core-homonyms-green.log`.
Il s'agit d'une preuve supplémentaire, pas d'une correction de production.
Aucune RED artificielle n'est revendiquée. Source noyau gelée à nouveau.
