# Journal d'implémentation — Session 047

## Métadonnées

- Branche : `session-047-verdict-tete-reecrite`
- Base contractuelle : `bc745335530985ce305e82fea4007071c752d5b0`
- Statut : implémentation terminée, prête à relire

## Tranche 1 — Continuité du verdict

- Verdict typé relu depuis les octets terminaux de `guichet_receptions`, sans
  nouvelle colonne ni migration.
- Observation distante par `ls-remote`, puis ancêtralité dans un dépôt nu
  temporaire utilisant les objets locaux en lecture seule.
- États fermés : cible absente, verdict absent, ancêtre, réécrit et
  inobservable.
- Projection ajoutée à `maicie status` et à la carte de reprise.
- Les lectures SQLite sont groupées et les cibles Git identiques mises en
  cache pendant une campagne de statut.
- Mesure du 27 août : 540 objectifs et 540 délégations dans le greffe actif.
  Le chemin scalaire `review_verdicts_for` aurait donc appelé SQLite 540 fois ;
  la campagne appelle `review_verdicts` une fois, puis indexe les 41 réceptions
  de verdict trouvées. Comparatif structurel : 540 lectures avant, 1 après.

## Preuves ciblées

- Empilement puis réécriture : 1 passé / 0 échec.
- Mutant égalité de SHA : 0 passé / 1 échec sur le cas empilé ; restauration
  attestée par SHA-256 identique
  `4705b6ef6c05b32fc4d05da1c5d816ee965338aca252fca8cc0a8589539f7d05`.
- Statut réel après réécriture : 1 passé / 0 échec.
- Carte de reprise, verdict absent puis alerte réécrite : 1 passé / 0 échec
  pour chacun des deux témoins.

## Tranche 2 — Contrat métier avant raccord filaire

- Maicie distingue désormais la délégation historique v1 de la délégation de
  revue v2 : v1 interdit une cible, v2 exige une cible valide, et aucune autre
  opération n'accepte v2.
- La canonisation conserve la version effectivement lue ; elle ne peut donc
  pas rabattre silencieusement une demande v2 sur v1.
- Le service de greffe transmet la cible au constructeur métier avant la
  transaction. Le témoin central relit ensuite la même cible depuis la
  délégation durable.
- La carte de reprise choisit le dépôt attesté par
  `review_project.repository_root`, jamais le worktree de l'agent comme source
  implicite du fait Git.

## Preuves ciblées de la tranche 2

- Matrice v1/v2 du parseur : 1 passé / 0 échec.
- Effet central avec politique chargée, faux daemon réel et relecture SQLite :
  1 passé / 0 échec.
- Mutant `review_target: None` au point d'effet : 0 passé / 1 échec sur
  l'assertion durable (`left: None`, cible exacte attendue), après mise en
  place réussie.
- Restauration du mutant : 1 passé / 0 échec et SHA-256 identique
  `afceb013b052e9c2275c32c2b1fccb8aa1c54e17ee991561a20f9b5dc7f1a414`.
- Carte de reprise existante après sélection de la racine configurée :
  1 passé / 0 échec.

## Compatibilité mesurée sur le binaire contractuel

- Binaire réel construit depuis
  `bc745335530985ce305e82fea4007071c752d5b0`, daemon lancé dans un `HOME`
  isolé.
- Trame v2 portant `review_target` : refus public
  `canonical_bytes_mismatch`, zéro ligne dans `guichet_requests` avant comme
  après.
- Contrôle v2 sans champ nouveau : refus public `unsupported_version`, zéro
  ligne avant comme après.
- Le contrôle d'octets canoniques de l'ancien daemon observe donc la perte du
  champ inconnu avant le contrôle de version. Exiger `unsupported_version`
  avec le fait nouveau est impossible sans modifier l'ancien binaire ou
  cacher la cible dans un champ historique. La propriété utile tient : refus
  explicite et fail-closed avant toute écriture SQLite.

## Tranche 3 — Raccord filaire versionné

- `maicie_delegate` accepte `review_ref` et `expected_head` comme une paire
  atomique, dans le CLI comme dans le serveur MCP. Un champ isolé est refusé
  avant l'ouverture de la connexion de service.
- Une délégation historique sans cible reste en v1. Une délégation portant une
  cible valide exige v2. Le daemon neuf refuse une cible en v1, une délégation
  sans cible en v2 et toute autre opération en v2.
- La canonisation de `Delegate` utilise le type public
  `ServiceRequestPayload` comme source unique. Un premier oracle a révélé un
  `EnvelopeMismatch` dû à deux ordres de champs distincts entre une structure
  privée et le protocole public ; la structure privée ne sert plus qu'au
  décodage strict des champs inconnus.
- `ServiceHello` reste en v1. Seule l'enveloppe `ServiceRequest::Delegate`
  ciblée étend le contrat.

## Preuves ciblées de la tranche 3

- Contrat transport v1/v2 : 1 passé / 0 échec.
- Producteurs CLI et MCP réels : 2 passés / 0 échec.
- Matrice d'acceptation du daemon, y compris la réponse filaire : 1 passé /
  0 échec.
- Effet central et relecture de la cible durable : 1 passé / 0 échec.
- Mutant du producteur de version (`v2` remplacé par `v1`) : 0 passé / 1
  échec ; restauration au SHA-256
  `20ffbcf00fedfa8f6aa3786262626d4b87c31c3447814399d08c8ec8af589a34`.
- Mutant du transport MCP (cible remplacée par `None`) : 0 passé / 1 échec
  après enregistrement et lecture de la trame réelle ; restauration au
  SHA-256
  `e4d4ace5d0c8e966207b4cb4518ecc52ece92f191fdb02f0e5b6332cbd80d90a`.
- Mutant de la garde daemon (v1 ciblée autorisée) sur l'arbre rebasé : 0 passé
  / 1 échec sur l'assertion métier ; restauration au SHA-256
  `2cedad059249e6b78f62e82b96177ad12f0dfff3266c93ae9afed18731a9c481`.
- Mutant du point d'effet (cible remplacée par `None`) : 0 passé / 1 échec
  sur la relecture durable, après mise en place franchie ; restauration au
  SHA-256
  `afceb013b052e9c2275c32c2b1fccb8aa1c54e17ee991561a20f9b5dc7f1a414`.

## Mesures Git réelles du défaut

- Objet de tranche avant rebase
  `23fea840e2f938789fd94f16cb4c126a20915706` vers tête réécrite
  `6f0c00e745bf4bb456e11ed825835bbfe45fc6ed` : le premier n'est pas
  ancêtre du second, donc réécriture.
- Base `2f5fc7121f03ce91bafcd764a599ccedeb337b5c` vers tête empilée
  `74d51dc166981a22f3d14c2d3dd45b2893d3b326` : la base reste ancêtre,
  donc aucune alerte.
- Ces objets réels complètent la fixture ; aucun test permanent ne dépend de
  leur présence future dans le reflog.

## Gates finaux base / tête

Périmètre déclaré : `bridget-transport`, `maicie` et `bridget-daemon`.
`cargo test --no-run` a été exécuté avant le listage et le comptage, et est
vert des deux côtés.

Le banc
`daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`
ne termine pas sur cet hôte, sur la base comme sur la tête. Il est soustrait
par son nom exact ; aucun résultat partiel de sa passe suspendue n'est compté.

| Arbre | Univers brut | Univers mesuré | Passés | Échoués | Ignorés |
| --- | ---: | ---: | ---: | ---: | ---: |
| base `06d54831761107f5ccf3d501047eb10561e4c569` | 1257 | 1256 | 1229 | 8 | 19 |
| tête 047 | 1267 | 1266 | 1239 | 8 | 19 |

Les huit rouges sont identiques en nom et en message des deux côtés :

- `daemon::presence_tests::enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant` ;
- `daemon::presence_tests::sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` ;
- `daemon::presence_tests::tour_non_abouti_redevient_mandatable_et_le_mandat_parvient` ;
- `claude_gere_sans_bypass_reste_sans_outil` ;
- `wrapper_codex_natif_repond_et_reste_attachable` ;
- `wrapper_codex_sans_signal_laisse_effort_et_limite_inconnus` ;
- `prompt_reduit_rejoue_le_corpus_dans_la_meme_session` ;
- `reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees`.

Une première composition de la tête avait trois rouges supplémentaires. Ils
ont été instruits au lieu d'être imputés au hasard : diagnostic CLI fusionnant
deux erreurs, fixture de canonisation utilisant l'ordre d'une structure
privée, et fixture `status` dépourvue de l'attestation locale désormais
obligatoire. Après correction, leurs harnais complets rendent respectivement
6/0/0, 54/0/0 et 18/0/0 ; la passe globale ne les reproduit plus.

`cargo clippy --all-targets` analyse les deux arbres jusqu'au bout et rend 48
avertissements sur la base comme sur la tête : écart nul. `cargo fmt --check`
reste rouge sur une dette préexistante, avec 74 emplacements sur la base et 72
sur la tête. `git diff --check` est vert.

## Non mesuré

- macOS après raccord ;
- suite workspace complète hors du périmètre déclaré ;
- le banc suspendu ci-dessus, qui reste une non-mesure et non un rouge ;
- détection automatique d'un lot ouvert sans branche distante, explicitement
  hors périmètre ;
- persistance future des objets réels de rebase dans les reflogs.
