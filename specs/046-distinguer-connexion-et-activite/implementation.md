# Implémentation 046 — Distinguer connexion et activité

## Objet mesuré

- Plateforme : Linux 6.8.0-94-generic x86_64.
- Chaîne Rust : rustc et cargo 1.92.0, imposés par le dépôt.
- Base gelée : `bc745335530985ce305e82fea4007071c752d5b0`.
- Branche : `session-046-distinguer-connexion-et-activite`, publiée vide avant
  toute écriture.
- Production modifiée : `crates/bridget-daemon/src/daemon.rs` uniquement.
- Aucun protocole, routeur, schéma SQLite, ledger ou transport de remise n'est
  modifié.

## Mesure réelle avant correction

Le tir réel `rc5` vers `bridget` a rendu :

- `last_seen_secs=3874` avant l'envoi ;
- `last_seen_secs=3874` immédiatement après ;
- `last_seen_secs=3877` après 2,5 secondes ;
- `last_seen_secs=4139` au contrôle tardif.

Le ledger contenait pourtant le message `mcp-3245475-6a905620-58`, avec le
delivery id `77871501-352c-4878-9bcf-b06834b346d8`. Ce contrôle positif prouve
que l'envoi a été enregistré : la valeur publique a ignoré une activité réelle.

## Inventaire avant modification

`Presence` conserve deux horloges :

- `capacity_seen`, exposée sous le nom `last_seen_secs` ;
- `link_seen`, utilisée pour la santé et la rétention du socket principal.

L'enregistrement initialise les deux. `TurnState`, le runtime, le modèle servi,
les limites et l'usage rafraîchissent la capacité. Le heartbeat ne rafraîchit
que le lien. Les deux écrivains manquants étaient :

- `SendIdempotent`, après préparation durable de la remise ;
- `Send`, après écriture durable du message historique.

Les consommateurs décisionnels sont `scripts/bridget-idle.py` et
`scripts/bridget-ronde.py`. Le protocole, la CLI JSON et MCP ne font qu'exposer
la valeur. L'UI calcule une autre date à partir des messages entrants et
sortants : recevoir un mandat ne prouve pas une activité du destinataire, donc
cette mesure ne remplace pas `last_seen_secs` pour la ronde.

L'annuaire ne porte aucun état de lot. L'outil de ronde joint séparément les
agents connectés et les participants actifs de Maicie ; une branche distante
ne prouve ni l'ouverture ni la fermeture d'un lot.

## Correction

La correction conserve le contrat filaire et ajoute un écrivain manquant à
l'horloge existante :

1. l'envoi idempotent rafraîchit l'expéditeur logique après
   `begin_send_delivery*` réussi ;
2. l'envoi historique rafraîchit l'expéditeur logique après
   `record_message` réussi ;
3. la résolution ne touche qu'une présence déjà enregistrée ;
4. le rafraîchissement spécialisé modifie `capacity_seen` sans modifier
   `link_seen`.

Un refus, une erreur de persistance, un expéditeur non enregistré, un heartbeat
ou la seule réception d'un mandat ne fabriquent donc aucune activité.

## Oracle rouge puis nominal

Le paquet daemon a été compilé avant inventaire. L'univers ciblé exact contient
trois tests : les deux voies d'envoi et le contrôle de refus.

Avant la production : **1 passé / 2 échoués / 0 ignoré**. Les deux voies
acceptées meurent à la même assertion avec `last_seen_secs=1900`; le refus reste
vert.

Après correction : **3 passés / 0 échec / 0 ignoré**. Dans les deux voies
acceptées, l'âge public est inférieur à deux secondes et l'âge du lien reste
supérieur à soixante secondes. Le refus conserve un âge supérieur à 1 800
secondes.

## Mutants et restauration

### Mutant 1 — retirer les deux écrivains

Le mutant supprime uniquement les appels placés après les deux acceptations
durables. Le chemin est vérifié muet : la fonction reste définie mais aucun site
productif ne l'appelle.

Résultat sur l'univers exact de trois : **1 passé / 2 échoués / 0 ignoré**.
Meurent à l'assertion d'âge public :

- `session_046_envoi_idempotent_ancien_agent_actif_rajeunit_last_seen` ;
- `session_046_envoi_historique_ancien_agent_actif_rajeunit_last_seen`.

Le contrôle `session_046_envoi_refuse_ne_rajeunit_pas_last_seen` reste vert.

### Mutant 2 — rajeunir aussi le socket

Le mutant remplace `touch_message_activity()` par `touch_capacity()` au site
réel de résolution de l'expéditeur. Résultat : **1 passé / 2 échoués / 0
ignoré**. Les deux voies acceptées meurent à l'assertion « l'activité MCP/CLI
ne doit pas rajeunir le socket principal » ; le refus reste vert.

Une première tentative de restauration mécanique de ce second mutant a visé
une autre occurrence identique dans `TurnState`. Le tir post-restauration a été
entièrement exclu. Les deux sites ont ensuite été restaurés avec leur contexte,
et le condensat final de `daemon.rs` est revenu exactement à
`5bbfc2b678b10f2aa47d540f9c66eb47b72e07dbe7bfaf4c1efd71a98fde42f6`.
Le nominal final rend de nouveau **3/0/0**.

## Comparaison base/tête — paquet daemon

Les deux côtés ont leurs propres worktree, target Cargo et `TMPDIR` court en
mode 0700. `cargo test -p bridget-daemon --no-run` est vert avant inventaire.

Le test historique
`stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels` liste un
univers de un mais ne termine pas ; il est filtré explicitement et
symétriquement :

- base : univers 653, **630 passés / 11 échoués / 11 ignorés / 1 filtré** ;
- tête : univers 656, **634 passés / 10 échoués / 11 ignorés / 1 filtré**.

La différence des listes est exactement les trois témoins 046, sans disparition
de test. Dix rouges ont les mêmes noms des deux côtés. Le seul rouge propre à
la base,
`sc001_vingt_spawns_survivent_a_la_fermeture_du_client_et_repondent`, a ensuite
été relisté sur un univers de un et rend **1/0/0** sur la base comme sur la
tête. Il s'agit donc d'une panne sous charge préexistante, non d'un effet du
lot.

## Workspace final

`cargo test --workspace --no-run` est vert. L'univers listé vaut 1 287. Le
passage final, avec le même filtre non terminal, rend :

- **1 256 passés / 11 échoués / 19 ignorés / 1 filtré**.

Deux lignes produites par des sous-processus internes, chacune à `1 passed / 181
filtered out`, ont été exclues du total. Sans cette exclusion, l'arithmétique
ne retombe pas sur l'univers listé.

Les rouges appartiennent aux mêmes familles préexistantes de présence,
wrappers natifs, reprise MCP, UI 024 et attachement. Trois témoins dont le nom
alterne sous charge ont été relistés séparément et passent chacun **1/0/0** :

- `wrapper_interactif_avec_journal_actif_est_attachable` ;
- `spec_024_ui_sans_attestation_reste_inconnue_dans_agent_info` ;
- `spec_024_ui_locale_attestee_projette_unix_dans_agent_info`.

## Gates statiques

- `rustfmt --edition 2024 --check crates/bridget-daemon/src/daemon.rs` : vert ;
- `git diff --check` : vert ;
- `cargo fmt --all -- --check` : rouge hors diff sur plusieurs fichiers ;
- Clippy strict avec et sans dépendances : rouge, avec exactement la même suite
  de diagnostics sur la base et la tête ; aucun diagnostic n'est né du lot.

## Non mesuré

- macOS ;
- déploiement du binaire corrigé sur le daemon vivant ;
- âge de connexion stable, qui n'est pas conservé aujourd'hui ;
- issue fonctionnelle du test historique explicitement filtré.
