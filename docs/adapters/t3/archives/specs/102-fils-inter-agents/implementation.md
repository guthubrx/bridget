# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/implementation.md](../../../../../../specs/102-fils-inter-agents/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Constat sur la 101 (20:37 CEST) : `git worktree list` montre 19 worktrees ;
`.worktrees/101-abonnements-t3` et `.worktrees/105-arret-boucles` sont à 1738a072
avec respectivement 21 et 26 fichiers modifiés non commités, 0 commit d'avance.
Le daemon installé (build-id `1738a072ba22-dirty`, adoption
`~/.cache/bridget-adoptions/105-20260917.bWasBb`, relancé 06:43) provient de la
105, qui englobe la 101. Le binaire installé n'est pas une preuve d'intégration Git.

<a id="extrait-2"></a>

## Extrait 2

### T001 Base commune
- **Statut** : ✅ Complété à 21:13 CEST.
- **Arbitrage** : Bridget-enhance (propriétaire des worktrees 101/105/106) a donné
  à 21:03 son accord écrit explicite pour que j'assure commits et intégration
  « 101 → 105 → 106 dans main », en préservant l'arbre principal. Aucun agent
  concurrent, worktrees 102–104 non touchés par lui.
- **Commits créés par plumbing** (`write-tree` + `commit-tree` + `update-ref`,
  aucun checkout ni modification de fichier dans les worktrees d'autrui) :
  - `9a1329e1` feat(101): Attester les observations T3 et l'identite MCP (parent 1738a072, 32 fichiers) ;
  - `853f2903` fix(105): Arreter les reponses automatiques en boucle (parent 9a1329e1 ; delta pur 105 : t3code.rs, wrapper.rs, envelope.rs, acp.rs, claude_stream_json.rs, codex_app_server.rs, docs ; daemon.rs/observation.rs/store.rs identiques à la 101) ;
  - `a4244c42` build(106): Ajouter le script de build et l'entretien des caches (parent 1738a072 ; la section « Entretien des caches » d'AGENTS.md, présente seulement dans l'arbre principal, y est incluse).
- **Fusions dans main** : `105bbb04` merge(105) puis `f36804ea` merge(106), sans
  conflit. État sale antérieur de main comparé fichier par fichier au résultat :
  identique partout sauf README.md (+8 lignes de la 105) et AGENTS.md (en-tête de
  session 105) ; copie de sécurité `/tmp/main-dirty-RDd3` ; stash temporaire supprimé.
- **Base 102** : commit `2d39a2d7` docs(102) (artefacts de spec, ADR 038, feature.json,
  AGENTS.md) puis `a3e0786d` merge(102) de main ; conflits feature.json/AGENTS.md
  résolus en gardant la 102. `t3code_identity.rs` et `spec101_observation_test.rs`
  présents. Aucune copie manuelle du worktree 101.
- **Non fait** : aucun déploiement, aucun redémarrage, aucune recette complète ;
  vérification cargo ciblée (tests 100/101 intégration, lib spec101/spec105,
  harnais 102) lancée sur cette base, résultat consigné à T030 et ci-dessous.

<a id="extrait-3"></a>

## Extrait 3

### T015–T017 Adaptateurs et réponse implicite
- **Statut** : ✅ Complétés.
- **wrapper.rs** : `connect_and_register_at` annonce `ThreadNoticeCapability{[1]}` après
  `Registered` (chemin commun Codex/Claude gérés et interactifs et pont T3) ; le contexte de
  réponse reçoit le marqueur `{"kind":"thread_notice","thread_id":…}` pour une alerte
  (Deliver et DeliverIdempotent), l'ancien format DM reste écrit pour les vrais DM.
- **t3code.rs** : enveloppe dédiée 🧵 (lire/confirmer/publier par `bridget_thread`, aucun
  relais de réponse finale) ; aucune attente `pending` (reply=false, règle 105).
- **cli.rs** : `bridget reply` refuse avec `thread_notice_not_replyable` (JSON, exit 2) quand
  le contexte est un marqueur ; les anciennes lignes `expéditeur[TAB]id` restent lues.
- **Tests** : V28 (`spec102_v28_alerte_de_fil_injectee_sans_attente_ni_relais`), V29 CLI
  (`spec102_v29_marqueur_thread_notice_reconnu…`) et wrapper
  (`spec102_v29_alerte_de_fil_laisse_un_marqueur_type…`).
- **Limite** : la recette des wrappers Codex/Claude réels avec faux fournisseur (V29
  intégration) n'est pas écrite ; la capacité est annoncée par le chemin commun et la remise
  suit le tracker 099 existant.

<a id="extrait-4"></a>

## Extrait 4

### Écarts préexistants corrigés pour une recette propre (hors périmètre 102, tolérance zéro)
- `core_089_skill_test` : l'exemple de réponse de SKILL.md (session 105) dépassait les 60
  caractères affichés par `bridget ledger` → corps raccourci à 59 caractères.
- `claude_interactive_097_test` (6 tests) : le prompt système de la session 105 contenait
  « ; », refusé par le filtre d'arguments du wrapper (`;&|$`) → deux « ; » remplacés par « . »
  et « , » ; texte sinon inchangé.
- `claude_native_permissions_test::spec094_claude_recoit_la_liste_fermee…` : la liste attendue
  n'incluait pas `bridget_events`/`bridget_journal` (sessions 100/101) ; complétée avec
  `bridget_thread`.
- `t3code.rs:2780` : emprunt inutile (clippy) dans la 105.
- **Environnement** : `cli::hook_tests::spawn_roundtrip…` et `cli::spawn_executor_tests::…`
  échouent quand le processus de test descend d'un agent Bridget attesté (la résolution
  d'identité par filiation trouve mon identité T3 dans `~/.cache/bridget-core/agent-pids`
  puis exige une preuve auxiliaire absente du namespace de test) ; ils passent avec
  `BRIDGET_HOME` pointé sur un namespace privé vide. La recette T030 utilise ce réglage.

<a id="extrait-5"></a>

## Extrait 5

**Ce qui est livré dans le worktree** (diff non commité, base `a3e0786d` = main `f36804ea`) :
- Contrat : `ThreadRequest`/`ThreadAction`/`ThreadNotify`/`ThreadResult`,
  `WrapperToDaemon::ThreadRequest`, `WrapperToDaemon::ThreadNoticeCapability`,
  `DaemonToWrapper::ThreadResult`, `BridgetMessage.thread_notice` (`ThreadNotice`).
- Daemon : `threads.rs` (règles), `store/threads.rs` (six tables, transactions, sollicitations),
  dispatcher de sollicitations dans `daemon.rs` (tick 1 s existant, 5 départs/s, lot 16,
  échéance 120 s, reprise même clé), exclusion des notices de la réaffectation 099,
  neutralisation des notices forgées, préflight du schéma.
- Façades : `bridget thread …` (CLI) et `bridget_thread` (MCP, 15e outil des allowlists
  Codex/Claude) ; `bridget reply` refuse après une alerte.
- Adaptateurs : annonce de capacité après enregistrement (chemin commun), enveloppe T3 dédiée
  sans attente ni relais, marqueur de contexte de réponse.
- Docs : SKILL.md, commandes.md (recettes + JSON), README, ADR 038, contrat amendé.

<a id="extrait-6"></a>

## Extrait 6

**Versions et capacités nécessaires à la recette réelle** : daemon, pont T3 et wrappers
issus du même build (le fait `ThreadNoticeCapability` n'est annoncé que par les nouveaux
wrappers ; un serveur MCP déjà vivant garde son ancien catalogue sans `bridget_thread`).
Un membre dont le client est ancien publie et lit mais n'est pas sollicité
(`capability_unavailable`) ; rien n'est dégradé en message direct.

<a id="extrait-7"></a>

## Extrait 7

**Adoption éventuelle (non exécutée, à autoriser)** : même procédure que les adoptions
100/101/105 (`~/.cache/bridget-adoptions/<session>-<date>/` : binaire avant, `.backup` de la
base, plists, skill), build isolé en release depuis le worktree, remplacement du binaire
installé, puis `launchctl kickstart -k gui/$UID/com.bridget.daemon` et `com.bridget.t3`.
La migration additive crée six tables au premier démarrage ; retour arrière = binaire
précédent (les tables ignorées ne gênent pas l'ancien daemon). Les sessions T3 déjà
ouvertes gardent leur serveur MCP ancien : `bridget_thread` n'y apparaît qu'après réouverture.

<a id="extrait-8"></a>

## Extrait 8

### T030 Recette complète (état définitif du code, 2026-09-17 22:12–22:18 CEST)
- **Statut** : ✅ Complété.
- **Environnement** : `umask 077`, `env -i HOME TMPDIR=/tmp/b.XXXX BRIDGET_HOME=/tmp/b102/ns
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`, `perl -e 'alarm 1800'`.
- `cargo fmt --all -- --check` : rc 0.
- `cargo clippy --workspace --all-targets -- -D warnings` : rc 0 (aucun avertissement).
- `cargo test --workspace --release --no-fail-fast` : **1460 réussis, 0 échec, 50 ignorés**
  (ignorés : recettes réelles Codex/Claude/T3 conditionnées par variables d'environnement, et
  le worker de performance privé du harnais). Journal : /tmp/b102/recette2-release.log.
- Régressions 089/094/097/098/099/100/101/105 incluses dans le workspace ; les quatre
  écarts préexistants corrigés (skill, prompt 105, allowlist Claude, clippy t3code) font
  partie du vert.
- Une première passe complète (22:04–22:12) avait donné 1458/0/50 avant les deux derniers
  tests de convergence.
- **Passage 3 (22:31–22:40, après la correction issue de la contre-revue)** : fmt rc 0,
  clippy rc 0, `cargo test --workspace --release --no-fail-fast` **1460 réussis, 0 échec,
  50 ignorés** (/tmp/b102/recette3-release.log). État du code figé à ce passage.
