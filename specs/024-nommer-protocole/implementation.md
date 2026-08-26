# Implémentation 024 — Nommer le protocole réel sans perdre le canal

## Résultat

La présence porte désormais deux faits indépendants :

| Chemin réel | `TRANSPORT` | `CANAL` |
|---|---|---|
| agent interactif local | `tmux` | `unix` |
| agent interactif fédéré | `tmux` | `ssh-unix` |
| Cursor/Gemini ACP | `acp` | canal de connexion attesté |
| Codex natif | `codex_app_server` | canal de connexion attesté |
| Claude natif | `claude_stream_json` | canal de connexion attesté |

La mesure du code de remise a réfuté l'hypothèse « Codex distant implique
app-server » : les agents Cartae observés sont des panes pilotés par
`TmuxTransport`. Aucun protocole n'est donc déduit du type de fournisseur.
ACP est bien le protocole exact exposé par `cursor-agent acp`.

## Modèle et compatibilité

- La clé JSON `WrapperToDaemon::Register.channel` et `AgentInfo.channel` sont
  additives et optionnelles. Le type interne `ChannelReport` distingue clé
  omise, `null` explicite et chaîne attestée.
- Une trame historique `mode=tmux, transport=ssh-unix` devient
  `transport=tmux, channel=ssh-unix` dans la présence.
- La définition figée reste l'autorité pour tout agent géré.
- Une valeur réseau historique n'est jamais reprojetée comme protocole ; en
  l'absence de preuve, le protocole vaut `unknown`.
- `BRIDGET_CHANNEL` et `channel=` sont prioritaires. Les alias historiques
  `BRIDGET_TRANSPORT` et `transport=` restent lus.
- L'installateur de fédération écrit les deux clés pendant la transition afin
  qu'un ancien wrapper conserve l'information distante.

## Affichage et documentation

`bridget who` et la sortie humaine `agents` montrent le protocole puis le
canal. La projection JSON conserve le champ public `transport` et ajoute
`channel`. Les README français et anglais documentent cette séparation et la
migration de variable.

## Oracles et contre-épreuves

Les tests `spec_024_*` couvrent le fil JSON, l'écrivain tmux, la priorité de
configuration, la normalisation d'une trame fédérée historique et
l'indépendance protocole/canal. Les oracles natifs existants ont été renforcés
pour exiger `claude_stream_json` ou `tmux` au lieu des anciennes valeurs
`stdio` ou `unix`.

Deux mutations discriminantes ont été exécutées puis restaurées :

1. conserver `ssh-unix` comme protocole d'une trame tmux historique fait
   rougir l'oracle avec `left="ssh-unix", right="tmux"` ;
2. faire annoncer `ssh-unix` par l'écrivain interactif dans `transport` fait
   rougir l'assertion qui inspecte la trame `Register` réelle.

## Portes mesurées

- `cargo fmt -p bridget-daemon -p bridget-transport -- --check` : vert ;
- Clippy des deux crates, tous targets, avertissements interdits : vert ;
- `cargo test --no-run` avant comptage : vert ;
- tests `spec_024_*` : 5 passés, 0 échoué ;
- test de fédération shell : vert ;
- suite workspace : **935 passés, 3 échoués, 16 ignorés**.

Les trois rouges sont les références hors lot :

- `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty`
  (`attach.rs:2338`, PTY `tcgetattr` EIO) ;
- `attach::tests::reconnexion_socket_reprend_exactement_a_last_seq_plus_un`
  (`attach.rs:3932`, course de reconnexion) ;
- `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`
  (`lifecycle.rs:709`, fixture de types connue).

Une passe intermédiaire a vu deux collisions de `sc005_attach_budget`. La cible
a rendu 2 passés / 0 échoué / 1 ignoré isolément, puis la passe workspace finale
l'a rendue verte. Elle n'est pas imputée à G11.

## Non mesuré

- Aucun redémarrage du daemon de production ni migration de la flotte vivante.
- Aucun comptage workspace macOS post-changement.
- Aucun croisement réel entre un ancien et un nouveau processus ; la
  compatibilité filaire inverse reste établie statiquement par les structures
  Serde sans `deny_unknown_fields`.
- `--all-features` reste non compilable sur Linux à cause du banc historique
  `test-support` qui appelle `kqueue`/`kevent`; le schéma G11 y a néanmoins été
  mis à jour avant cette frontière de plateforme.

## Amendement de composition sur main 2330dfde

La composition avec la GUI a révélé une propriété distincte de la
compatibilité filaire : un champ Serde optionnel reste obligatoire dans les
constructeurs Rust exhaustifs. Après rebase, le premier `--no-run` a reproduit
`E0063` dans l'initialiseur productif de la présence UI, puis dans trois
initialiseurs de `guichet_integration_test` ajoutés sur la nouvelle base.

Le premier amendement avait attribué `channel=unix` à la présence UI parce que
`open_human_presence` appelle `UnixStream::connect`. La relecture a invalidé
ce raisonnement : sur une machine fédérée, le tunnel SSH publie lui aussi le
daemon maître sous forme de socket Unix locale. L'API décrit le dernier saut,
pas le fait réseau.

L'amendement final partage donc une seule résolution d'attestation entre les
wrappers et l'UI. `BRIDGET_CHANNEL` et `channel=` gouvernent leurs alias dans
leur propre source. Entre environnement et `federation.env`, une valeur
unique ou concordante est publiée ; une divergence ou l'absence des deux reste
inconnue. La présence UI n'envoie plus `transport=unix`, car ce champ historique
permettait au daemon de recréer artificiellement `channel=unix` quand le champ
additif était absent.

Trois tests lisent la trame `Register` réelle et quatre tests de couture
lancent le vrai daemon puis le vrai sous-processus `bridget ui`. Ils attestent
`unix` en configuration locale explicite, `ssh-unix` depuis
`federation.env`, et l'absence en l'absence de source ou en cas de divergence,
jusque dans `AgentInfo`.

Quatre mutations discriminantes ont été exécutées puis restaurées :

1. reclasser `ssh-unix` en `unix` tue les oracles fédérés de trame et de
   projection ;
2. remplacer un canal absent par le défaut `unix` tue les oracles d'inconnu ;
3. remettre `transport=unix` dans la trame UI tue l'oracle `AgentInfo` en
   réactivant le repli historique ;
4. faire gagner arbitrairement l'environnement sur un fichier divergent tue
   l'oracle de résolution et l'oracle de projection.

Mesures de composition, dans l'ordre demandé :

- base nue `2330dfde` : `--no-run` vert, 962 tests listés,
  **941 passés / 3 échoués / 18 ignorés** ;
- composition avant l'amendement d'attestation : `--no-run` vert, 968 tests
  listés, **947 passés / 3 échoués / 18 ignorés** ;
- `ui_relay_test` avant l'amendement d'attestation : **11 passés / 0 échoué /
  0 ignoré**.

Comptes finaux post-amendement, après `--no-run` vert des deux côtés puis
inventaire et exécution séquentielle de la même closure
`bridget-transport + maicie + bridget-daemon` :

- base nue `2330dfde` : 962 tests listés,
  **941 passés / 3 échoués / 18 ignorés** ;
- tête amendée : 978 tests listés,
  **957 passés / 3 échoués / 18 ignorés** ;
- `ui_relay_test` exact : **15 passés / 0 échoué / 0 ignoré** ;
- famille `spec_024_*` : **15 passés / 0 échoué**, plus le test shell de
  fédération vert, soit 16 oracles G11.

Une première exécution simultanée base/tête a produit deux rouges
supplémentaires uniquement sur la base et trois rouges sur la tête dans des
harness qui lançaient un wrapper local avec un environnement volontairement
vidé. Cette passe n'a pas été retenue comme soustraction. Les harness locaux
déclarent désormais explicitement `BRIDGET_CHANNEL=unix`; leurs trois cibles
ont passé isolément, puis la répétition séquentielle a retrouvé exactement les
trois rouges de référence des deux côtés.

L'inventaire des consommateurs a aussi fermé la compatibilité du retrait de
`Register.transport=unix` pour l'UI : le daemon de `1fc67ef` filtrait déjà
cette valeur réseau et publiait `AgentInfo.transport=cli`. CLI, MCP et Maicie
continuent donc de lire et publier `cli`; le snapshot UI et Attach ne lisent
pas ce champ, et aucun chemin ledger ne le persiste.

Les trois rouges de composition sont les mêmes références hors lot que sur la
base. Un passage intermédiaire a aussi produit trois timeouts simultanés dans
`guichet_integration_test`, tous à la lecture bornée commune. La cible isolée a
ensuite rendu 6/0/0 sur la composition et 6/0/0 sur la base ; le passage complet
final a retrouvé les seuls trois rouges de référence. Ce dernier tirage ne
requalifie pas les timeouts en stabilité.

Enfin, `sc005_attach_budget` a été listé à trois tests puis rejoué cinq fois sur
la tête pré-amendement `cf26d1f` : cinq harnais sur cinq verts, chacun à
2 passés / 0 échoué / 1 ignoré, soit 0 rouge observé sur 5 tirages. Le test
ignoré est `sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux`,
par attribut source explicite de mesure locale ; le cas SC-005 litigieux a bien
été exécuté et a passé cinq fois. Ce N ne suffit pas à qualifier le banc de
stable.

## Amendement de transition sur main 2c5271f

La publication correcte ne suffisait pas lors d'une reconnexion. Une présence
connue sous `channel=unix`, puis réinscrite avec `channel=null`, récupérait
encore `previous.channel`. La cause était le modèle `Option<String>` : il
confondait la clé absente d'une ancienne trame avec l'inconnu explicitement
mesuré par un client récent.

`ChannelReport` porte désormais trois états sans modifier leur représentation
JSON :

- `Omitted` omet la clé et conserve le fait précédent ;
- `Unknown` écrit `channel:null` et efface le fait précédent ;
- `Known(value)` écrit la chaîne et remplace le fait précédent.

Tous les constructeurs productifs modernes déclarent explicitement
`Unknown` ou `Known`. Le repli depuis l'ancien champ réseau `transport` ne
s'applique plus qu'à `Omitted`.

Les oracles exécutés sur le vrai daemon attestent :

1. `unix` puis `null` donne un `AgentInfo` sans canal ;
2. `unix` puis une clé réellement omise conserve `unix` ;
3. un vrai sous-processus `bridget ui` recevant des attestations environnement
   et fichier divergentes efface le canal `unix` antérieur ;
4. le codec distingue et reproduit omission, `null` et chaîne.

Le mutant qui traite de nouveau `Unknown` comme `Omitted` meurt dans
`spec_024_reconnexion_inconnue_explicite_efface_le_canal_precedent`, sur
l'assertion finale `AgentInfo` avec `Some("unix")` observé contre `None`
attendu. La mise en place reste verte ; le fichier productif est restauré au
même SHA-256 avant et après le mutant.

Dette explicitement laissée hors périmètre : la présence UI est projetée avec
`AgentInfo.transport=cli`. Cette approximation est préexistante et visible
dans `who` et `agents --json`; la modifier ici changerait le contrat public.
Le chemin de normalisation concerné est `crates/bridget-daemon/src/daemon.rs`
autour de la branche `PresenceMode::Cli`.

Fermeture finale sur la même closure
`bridget-transport + maicie + bridget-daemon`, avec targets séparés et
`--no-run` vert des deux côtés avant inventaire :

- base `2c5271f` : 963 tests listés,
  **942 passés / 3 échoués / 18 ignorés** ;
- tête amendée : 982 tests listés,
  **961 passés / 3 échoués / 18 ignorés** ;
- `ui_relay_test` exact : **17 passés / 0 échoué / 0 ignoré** ;
- famille Rust `spec_024_*` : **19 passés / 0 échoué**, plus le test shell de
  fédération vert, soit 20 oracles G11.

Les deux côtés rendent uniquement les trois références Linux : `attach`
raw-mode (`tcgetattr` EIO), `attach` reconnexion (course `BrokenPipe`) et
`lifecycle::matrice_sc003` (fixture `known_types`). Le rouge supplémentaire
`wrapper_interactif_avec_journal_actif_est_attachable`, vu sous charge par un
relecteur sur la base puis vert isolément des deux côtés, ne réapparaît pas
dans cette exécution séquentielle et reste imputé hors lot.

Restent non mesurés après cet amendement : un vrai tunnel SSH inter-hôtes,
un croisement effectif ancien/nouveau processus, macOS et le comportement
d'un ancien daemon recevant `channel:null` (il le traite encore comme
l'ancienne absence). Aucun redémarrage ni migration de la flotte vivante n'a
été effectué.
