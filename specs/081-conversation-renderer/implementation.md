# Journal d'implémentation - SPEC-081

## Correctif de présence UI - 2026-09-01

- Observation de production : les envois avec réponse demandée pouvaient être
  refusés après un redémarrage du daemon avec `reply_sender_unavailable`, alors
  que le marqueur local de présence UI était encore vivant.
- Correction : ce refus est désormais le signal d'autorité qui invalide la
  présence locale, force une réinscription, puis rejoue une seule fois la même
  demande. L'opérateur conserve donc « Attendre une réponse » sans choisir
  entre réception et envoi.
- Preuve ciblée : `cargo test -p bridget-daemon
  refus_reponse_sans_presence_declenche_la_reinscription --lib` : succès.

## Correctif de reprise de présence humaine - 2026-09-01

- Cause observée : après le redémarrage du relais UI, une ancienne socket
  pouvait encore retenir l'identité humaine. Le nouveau relais était alors
  refusé avec `agent_id déjà connecté`, puis les envois `reply=true` devenaient
  impossibles avec `émetteur humain non inscrit`.
- Correction : seule l'identité singleton du relais UI peut désormais reprendre
  sa route. Le daemon envoie `Disconnect` à l'ancienne connexion, libère sa
  route et inscrit la nouvelle génération. Aucun agent ordinaire ne peut
  utiliser ce mécanisme.
- Preuve ciblée : `cargo test -p bridget-daemon --lib
  relais_ui_redemarre_reprend_lidentite_humaine_sans_rester_non_inscrit -- --nocapture`.

Ce journal ne consigne que des opérations réellement observées. Il ne contient
ni secret, ni jeton de relais, ni contenu de conversation de production.

## Reprise US1 - validation manuelle infirmée - 2026-08-31

- Le daemon actif `426b9646ab04` contenait bien le renderer enrichi.
- La capture de conversation sur texte simple a néanmoins montré que les
  réponses agents, rendues sans bulle, manquaient d'un repère et d'un contour
  de tour perceptible. Les fonctionnalités Markdown, code et tableau ne se
  déclenchent pas sur ce contenu et ne corrigent donc pas cette lecture.
- La SPEC reste en cours : T040 à T042 reprennent exclusivement la
  composition visuelle de US1. Aucune preuve visuelle de fermeture n'est
  déclarée avant un nouveau contrôle manuel.
- T040 et T041 sont implantées : chaque tour expose ses marqueurs de demande,
  activité, travail et réponse ; la réponse agent reçoit un libellé et un
  filet de lecture, tandis que la bulle humaine reste à droite.
- `node crates/bridget-daemon/assets/ui/app.js` : succès, 112 tests, dont
  l'assertion de composition US1 ajoutée.
- `/Users/moi/.cargo/bin/cargo fmt --check` : succès.
- La suite Rust UI est compilée. Avec `TMPDIR=/tmp`, elle atteint 149 succès
  et deux échecs de harnais hors périmètre : comparaison `/tmp` contre
  `/private/tmp` dans SPEC-080, et écriture temporaire refusée dans un test de
  présence. Sans `TMPDIR` court, les sockets Unix temporaires dépassent la
  limite `SUN_LEN`. Ces limites ne sont pas masquées par la SPEC.
- `env TMPDIR=/tmp /Users/moi/.cargo/bin/cargo test -p bridget-daemon
  ui::tests::assets_statiques_annoncent_etag_et_revalidation` : succès. Le
  daemon embarque et revalide bien les assets modifiés.
- `/Users/moi/.cargo/bin/cargo build --release -p bridget-daemon` : succès.
  Le SHA-256 du binaire de validation est
  `4322e26aae5ca17d219dfb0044aa83ef427380f06da0f67f613d1833c2f1ecb3`.
  Le binaire a été installé dans `/Users/moi/.local/bin/bridget` après une
  sauvegarde récupérable dans
  `/Users/moi/.local/bin/bridget.before-conversation-composition-20260831-144000`.
  Le LaunchAgent `com.bridget.daemon` a été relancé et le status atteste le
  build-id `0abc763db94e`. T042 reste ouverte uniquement pour le contrôle
  visuel manuel.

## T004 - Point de départ - 2026-08-31

- `node crates/bridget-daemon/assets/ui/app.js` : succès, 100 tests.
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon ui` : 136 succès et 10
  échecs de baseline, tous causés avant la feature par `path must be shorter
  than SUN_LEN` dans le worktree temporaire. L'incident est tracé dans le
  journal local non versionné ; aucune régression SPEC-081 ne lui est attribuée.
- `/Users/moi/.cargo/bin/cargo test --manifest-path
  apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` : succès,
  4 tests.
- `/Users/moi/.cargo/bin/cargo test --manifest-path
  apps/bridget-desktop/src-tauri/Cargo.toml --test secrets_and_diagnostics` :
  succès, 1 test.
- `git diff --check` : succès avant les modifications de cette feature.

## T002 - Coloration locale - 2026-08-31

- Choix : `@highlightjs/cdn-assets` 11.12.0, distribution locale
  `highlight.min.js`, licence BSD-3-Clause.
- Source vérifiée : registre npm, archive
  `https://registry.npmjs.org/@highlightjs/cdn-assets/-/cdn-assets-11.12.0.tgz`.
- Fichiers ajoutés : moteur et thèmes `github` clair/sombre dans
  `crates/bridget-daemon/assets/ui/vendor/`.
- Empreintes SHA-256 : consignées dans `vendor/SHA256SUMS`.
- Repli : si `globalThis.hljs` n'est pas disponible, le bloc conserve le texte
  brut et son contrôle de copie.

## T003 - Provenance T3 Code - 2026-08-31

- Licence MIT complète ajoutée sous `vendor/LICENSE.t3code.MIT.txt`.
- Référence inspectée : `/Users/moi/11.Repositories/t3code`, commit
  `b1670ac7d`, `apps/web/src/components/ChatMarkdown.tsx`.
- Adaptations directes prévues : uniquement l'analyse de métadonnées de fence
  `langage + titre`, avec commentaire de provenance à côté du helper final.
- Les principes de tours, documents agent et activités repliables inspirent
  l'interface mais ne reprennent pas de code React T3.

## Self-review Article XIX/XX - T001 à T004

- Pourquoi cette solution est nécessaire : les fixtures, l'intégrité des
  dépendances et la baseline empêchent de confondre une régression SPEC-081
  avec l'existant.
- Pourquoi elle reste simple : une fixture immuable, un seul moteur local et
  les fichiers de notices déjà présents remplacent un nouveau harnais ou une
  chaîne de construction JavaScript.
- Hypothèses prises : `highlight.js` reste disponible avant `app.js` dans le
  panneau ; son absence conserve du texte brut. La socket Unix du worktree
  temporaire explique les échecs Rust de baseline observés.
- Vérifications réalisées : 101 tests Node, cinq empreintes SHA-256 valides,
  cinq tests Tauri ciblés réussis et lecture de la licence T3 locale.
- Non vérifié : le rendu graphique final et la suite Rust complète depuis un
  chemin court seront rejoués dans les preuves finales.
- Code supprimé ou évité : aucune pile React, aucun CDN, aucun chargeur de
  dépendance à l'exécution.
- Complexité ajoutée et justification : environ 132 Ko de ressources locales
  pour une coloration hors ligne et déterministe, avec licences et hashes.

## Self-review Article XIX/XX - T005 à T007

- Pourquoi cette solution est nécessaire : le rendu, la sécurité et les tests
  ont besoin de décisions pures avant de toucher au DOM ou au relais.
- Pourquoi elle reste simple : `projectTimeline` demeure l'unique décodeur du
  journal ; `deriveConversationTurns` ne fait que regrouper sa projection.
- Hypothèses prises : un `messageId` est durable à l'échelle du fil, y compris
  pour l'entrée `work` qui ne porte pas forcément le nom de l'agent.
- Vérifications réalisées : 104 tests Node, dont défaut sûr, corruption,
  références interdites, URL canonique, segments et tour en erreur.
- Non vérifié : canonicalisation de fichier côté Rust et rendu dans le DOM,
  prévus par les lots suivants.
- Code supprimé ou évité : aucun store serveur, aucune liste de racines ou
  parseur de protocole parallèle.
- Complexité ajoutée et justification : trois fonctions pures courtes et un
  objet de tour, nécessaires pour rendre les invariants testables.

## T013 - Validation US1 - 2026-08-31

- `node crates/bridget-daemon/assets/ui/app.js` : succès, 104 tests.
- La projection couvre un tour terminé, échoué, ouvert, actif, inter-agent et
  une interruption, avec une seule demande humaine par identifiant durable.
- Mesure factuelle disponible : exécution Node complète entre 0,36 et 0,48 s
  sur cette machine. Une mesure de défilement avec layout réel n'est pas
  revendiquée ici ; elle reste explicitement dans T034 et T038.

## Self-review Article XIX/XX - T008 à T013

- Pourquoi cette solution est nécessaire : les entrées d'un même tour doivent
  rester ensemble sans transformer les traces d'activité en réponses agent.
- Pourquoi elle reste simple : une seule section DOM par tour, rendue depuis
  la projection pure existante, remplace l'enchaînement visuellement ambigu de
  bulles identiques.
- Hypothèses prises : les entrées sans `messageId` restent des tours isolés et
  ne peuvent pas être inventées dans une demande voisine.
- Vérifications réalisées : 104 tests Node, contrôle de diff et revue ciblée
  du code de regroupement, d'ancre de lecture, de rendu d'activité et CSS.
- Non vérifié : apparence sur une fenêtre macOS réelle et comportement de
  l'ancre après changement de hauteur, réservés à la validation intégrée.
- Code supprimé ou évité : aucun second journal, aucune mutation d'état de
  remise, aucun temporisateur de défilement arbitraire.
- Complexité ajoutée et justification : deux petites fonctions d'ancre DOM,
  nécessaires pour préserver une lecture hors du bas lors d'un re-rendu.

## T014 à T020 - Markdown technique passif - 2026-08-31

- `extractFenceLanguage` et `extractFenceTitle` adaptent uniquement
  l'extraction de métadonnées de fence observée dans T3 Code
  `ChatMarkdown.tsx`, avec provenance MIT à côté du code.
- `highlight.js` 11.12.0 et ses deux thèmes GitHub sont embarqués localement.
  Un langage absent ou le moteur indisponible conserve le code en texte brut,
  sans affecter la copie exacte.
- Les blocs de code ont un titre ou langage, une copie accessible et un retour
  à la ligne indépendant. Les tableaux gardent leur défilement horizontal et
  peuvent être copiés en Markdown ou CSV.
- `node crates/bridget-daemon/assets/ui/app.js` : succès, 110 tests dont
  assainissement hostile, copie, thème et contenu actif refusé.

## T021 à T030 - Préférences locales et aperçus bornés - 2026-08-31

- Le store Desktop est en format V2. Un document V1 valide de l'opérateur
  actuel migre avec liens, fichiers et images actifs comme demandé. Une
  première installation, une corruption ou un reset restent fermés.
- La coque Tauri injecte `__BRIDGET_CONTENT_SECURITY__` immuable avant les
  scripts relayés et recharge le panneau après sauvegarde native. Le panneau
  ne peut donc pas forger une mise à jour qui élargirait ses autorisations et
  ne gagne aucune capability Tauri.
- Les liens n'ouvrent une destination HTTPS qu'après un clic fiable. Les
  images restent différées et sans référent. Les fichiers passent par une
  route relayée tokenisée, canonique, lecture seule, plafonnée à 256 KiB et
  bornée par `ProjectRootPolicy`.
- Vérifications : test Rust `spec_081_apercu_fichier_reste_borne_canonique_et_sans_chemin_racine`, test de store Desktop et 7 tests Desktop d'isolation réussis.

## T031 à T033 - Lecture d'historique - 2026-08-31

- La fixture comprend vingt tours. Hors du bas, le rendu mémorise le premier
  tour visible, y compris partiellement, et restaure son décalage après mise à
  jour. Le brouillon, sa sélection et son focus ne sont pas modifiés.
- Le rappel de retour au direct, les repères de tour, les petits écrans et
  `prefers-reduced-motion` possèdent des styles dédiés.

## Reprise US4 - suivi du direct pendant le streaming - 2026-09-01

- La capture utilisateur a infirmé l'hypothèse d'un simple calcul géométrique
  du bas du fil : entre deux fragments, le re-rendu peut modifier la hauteur
  avant la mesure suivante. Le lecteur se retrouve alors artificiellement
  hors du bas et l'ancre de lecture le ramène vers le haut.
- `renderThread` conserve désormais l'intention `followLatest` séparément de
  la position instantanée. Elle reste active tant que l'opérateur suit le
  direct, est désactivée par un défilement vers le haut, et l'ancre existante
  reste seule responsable de cette lecture historique.
- `node crates/bridget-daemon/assets/ui/app.js` : succès, avec le test de
  non-régression `suivi_du_flux_reste_actif_pendant_une_generation`.
- La vérification visuelle complète reste T034 : elle doit couvrir un vrai
  flux de fragments dans Bridget Desktop, sans prétendre qu'un test Node
  reproduit le layout WebView.

## Correctif - activité interrompue sans `turn_end` - 2026-09-01

- Observation réelle : après le redémarrage du relais, le journal de Jim
  contenait des fragments `update` mais aucun `turn_end`, tandis que le
  snapshot attestait `state: alive` sans `turn_state`. L'ancien rendu en
  déduisait à tort une rédaction permanente.
- La vue ne fabrique aucune fin de tour : sans exécution attestée, un
  indicateur vivant expire après 30 secondes. Il reste affiché sans limite
  seulement lorsque le snapshot atteste un tour `running` ou une attente
  d'autorisation.
- Le rafraîchissement de flotte réévalue uniquement ces indicateurs. Il ne
  reconstruit pas le fil et ne peut donc ni déplacer la position de lecture
  ni perturber la saisie.
- Vérification : `node crates/bridget-daemon/assets/ui/app.js` réussit avec
  le témoin `activite_sans_fin_ne_reste_pas_vivante_sans_execution_attestee`.

## Correctif - réponse attendue de l'opérateur - 2026-09-01

- Observation réelle : le relais UI était bien inscrit sous son UUID canonique,
  mais les messages de la conversation utilisent le libellé métier `humain`.
  Le routeur cherchait ce libellé comme un second agent, puis refusait
  `--reply` avec `reply_sender_unavailable`.
- La traduction vers l'UUID n'existe désormais qu'à la frontière de routage,
  dans les deux sens. Le ledger, le fil et les demandes suivies conservent le
  libellé `humain`.
- Vérification : test Rust
  `envoi_ui_avec_reponse_route_lhumain_canonique_sans_perdre_son_libelle`.

## Correctif - identité affichée au fournisseur - 2026-09-01

- La même observation a révélé un second défaut : le profil historique du
  libellé `humain` pouvait projeter « Agent (870) » dans le prompt Codex.
  L'agent répondait alors à cet agent fictif, au lieu de répondre à
  l'opérateur.
- Le message conserve `humain` pour le ledger et le routage. Seule la
  projection destinée au fournisseur devient désormais `Utilisateur`, valeur
  fixe qui ne peut pas être contaminée par un profil ancien.

## Self-review Article XIX/XX - T014 à T037

- Pourquoi cette solution est nécessaire : un fil riche sans séparation de
  tours ni politique locale de contenu rend les agents difficiles à piloter et
  les références difficiles à consulter en sécurité.
- Pourquoi elle reste simple : la projection reste au-dessus de
  `projectTimeline`, le panneau ne reçoit qu'un snapshot et l'aperçu réutilise
  `ProjectRootPolicy` plutôt qu'un explorateur de fichiers.
- Hypothèses prises : le navigateur système est l'unique destination des
  liens externes dans Desktop et 256 KiB suffit à une consultation de contexte
  sans se transformer en lecteur de médias.
- Vérifications réalisées : 110 tests Node, format Rust, test Rust ciblé,
  7 tests Desktop ciblés, sommes SHA-256 et contrôle de diff réussis.
- Non vérifié : le quickstart sur une fenêtre Desktop et un serveur approuvé,
  en thèmes clair et sombre. Cette preuve reste ouverte, elle n'est pas
  remplacée par les tests automatisés.
- Code supprimé ou évité : aucune capability de panneau, aucun CDN, aucune
  navigation ou chargement automatique, aucune seconde allowlist de chemins.
- Complexité ajoutée et justification : une route de preview et un moteur de
  coloration local sont nécessaires pour respecter la borne de sécurité et la
  lisibilité recherchée.
