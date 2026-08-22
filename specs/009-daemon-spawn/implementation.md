# Journal d'implémentation — équipiers gérés par le daemon

## Base de branche

La session 009 consomme le socle idempotent 012 revu. La base intégrée avant
T904/T905 est `c4010e3` (`74ad309` + correctif de verrou `1fa9dfb`).

## T905 — Ordres client, refus typés et environnement

- Le protocole expose `SpawnOrder`, `SpawnAccepted`, les onze refus fermés du
  contrat, `StopOrder` et les cinq `StopOutcome`.
- La projection CLI affiche toujours le `command_id` avant l'attente réseau.
  L'enveloppe d'un spawn est écrite en `0600` sous
  `~/.local/state/bridget/spawn-orders/` avant l'envoi ; `--command-id` relit
  exactement ces octets et refuse toute option divergente.
- La garde `forbidden_env` est partagée avec T710 et s'applique à
  l'environnement source. L'environnement enfant ne contient ensuite que la
  baseline D-505 et le `pass_env` validé du registre.
- La matrice SC-003 automatisée exerce les onze familles et vérifie qu'aucune
  réservation active supplémentaire ne subsiste après un refus.

### Gate réelle D-505 — 2026-08-22

Précondition vérifiée : `OPENAI_API_KEY`, `CODEX_API_KEY` et
`ANTHROPIC_API_KEY` absentes. Les deux adaptateurs ont été lancés sous
`env -i` avec seulement `HOME`, `PATH`, `USER`, `LANG` et `TMPDIR`; aucun
prompt n'a été envoyé.

| Type | Commande pinnée | Observable |
|---|---|---|
| Codex | `npx @zed-industries/codex-acp@0.16.0 -c model=\"gpt-5.5\"` | réponse `initialize`, protocole 1, agent `codex-acp` 0.16.0 |
| Claude | `npx @zed-industries/claude-code-acp@0.16.2` | réponse `initialize`, protocole 1, agent 0.16.2 |

Codex conserve volontairement son processus après l'EOF ; le gate l'a borné
à vingt secondes et l'a terminé après réception de la réponse `initialize`.
Claude s'est terminé proprement après la même négociation. Cette preuve valide
le lancement et la découverte de l'authentification par abonnement dans
l'environnement nettoyé, sans appel modèle ni facturation API.

### Échantillon quickstart §3

Le test ciblé de la matrice couvre notamment `UnknownType`, `CommandMissing`,
`BillingGuard`, `NameActive` et `CwdGone`, les cinq motifs proposés à
l'inspection manuelle dans le quickstart. Chaque issue est typée et le compteur
de générations actives reste inchangé par le refus.

## T906 — Canal de statut et supervision

- Le daemon libère le bootstrap seulement après le marqueur durable, puis
  supervise chaque enfant par `Child::try_wait` au tick. `BootstrapReady` ne
  produit aucune issue client ; le succès exige à la fois le `Register` réel et
  la fermeture volontaire du hook `managed-status` après initialisation du
  transport ACP, du journal et du relais attach.
- Le wrapper émet `StartupFailed` avec l'identité, la commande et la génération
  héritées du bootstrap. La disparition de la commande entre le préflight et le
  spawn est remontée comme `CommandMissing` avec le chemin exact.
- La mort spontanée reprend les terminaux 007/008 : demandes suivies rejetées,
  présence `stopped` et `End` de chaque vue attach, y compris si l'EOF socket a
  gagné la course sur le tick superviseur.
- Le stderr n'est jamais décodé comme protocole. Il reste consultable sous
  `~/.cache/bridget/managed-stderr/<nom>/<instance_id>-g<generation>/stderr.log`,
  avec répertoires `0700`, fichier `0600` dès l'ouverture. Le bootstrap, le
  wrapper et l'adaptateur ACP héritent tous de ce même descripteur ; le mode
  interactif historique conserve son stderr neutralisé. La purge applique au
  démarrage puis chaque heure la même rétention en jours que le journal du
  daemon.

Le test d'intégration exécute le vrai binaire `bridget managed-wrapper` après
le bootstrap : avant `Registered`, aucun succès n'est observable ; après la
réponse du faux daemon, l'échec réel de spawn ACP remonte un `StartupFailed`
corrélé. Un second test lance réellement un enfant supervisé qui termine avec
le code 7, puis exerce `try_wait` → événement → rejets/état/vue. Les tests de
permissions vérifient aussi les modes créés directement par `DirBuilder` et
`OpenOptions`, sans fenêtre `create` puis `chmod`.

## T909 — Matrice de parité FR-008

La matrice versionnée `fr-008-v1` exécute trois fois le même corpus dans chaque
mode, avec le même faux adaptateur ACP déterministe et le même daemon réel. Le
premier passage lance le wrapper depuis un terminal ; le second passe par
`SpawnOrder`, le bootstrap, le wrapper supervisé et un `Register` réel. Chaque
passage couvre quatre tours : demande suivie nominale, corps multiligne avec
apostrophe/guillemets/`$VAR`/backticks, puis deux demandes envoyées pendant un
tour ralenti pour exercer la file FIFO. Pour ce dernier scénario, un proxy Unix
coupe réellement la connexion wrapper→daemon au milieu du tour, dans les deux
modes. La reconnexion doit rétablir la même génération en état `busy`, la
relance différée doit être persistée, puis la file reprend dans l'ordre.

Observables comparés sans tolérance : quatre accusés, quatre réponses dans
l'ordre, quatre demandes finales `answered`, présence `acp/connected`, et les
douze événements du journal v1 rendus par une vraie connexion attach. Les
champs volatils (identifiants, horodatages, nom de génération) sont normalisés ;
les types d'événement, corps, texte des updates, `stop_reason` et routage de la
réponse restent comparés octet pour octet. Total : 6 passages, 24 échanges
suivis et 72 frames attach, zéro divergence. Une campagne séparée vérifie le
refus réel `OPENAI_API_KEY` avant spawn dans les deux modes, avec le même motif
`BillingGuard`. La parité n'est pas le seul oracle : les quatre `turn_start`
doivent aussi contenir exactement les quatre corps attendus, dont le corps
multiligne complet ; une perte identique dans les deux modes échoue donc le
test.

Le banc est borné à dix secondes par opération et a terminé en environ 32 s
sur la machine de validation. La commande reproductible est :

```bash
cargo test -p bridget-daemon --test managed_parity_test -- --nocapture
```

## T909b — Banc de spawn SC-001

Le banc lance vingt générations daemon-gérées réelles, séquentiellement afin
de rester sous le quota de flotte de production. Chaque génération traverse
`SpawnOrder` → bootstrap → `managed-wrapper` → `npx` → négociation ACP →
`Register`. Le paquet `parity-acp@1.0.0` est présent dans le `node_modules`
temporaire et `npx 11.19.0` est invoqué avec `--offline --no-install` : aucune
installation, aucun réseau et aucune API de modèle ne participent à la mesure.

L'environnement est recréé à l'identique pour les vingt essais :
`HOME=<racine temporaire>`,
`PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin`, `USER=parity-test`,
`LANG=C`, `TMPDIR=/tmp`, `PARITY_SINGLE_TURN=1`, et aucune clé API. Le client
qui émet chaque ordre est une connexion Unix distincte ; sa socket est
réellement fermée immédiatement après `SpawnAccepted`. Une autre connexion
persistante envoie alors une demande suivie, reçoit la réponse ACP et vérifie
sa clôture `answered` avant l'essai suivant.

Résultat de la campagne finale N=20 : **20/20 spawns**, **20/20 échanges suivis**,
p95 **442,466 ms**, maximum **530,891 ms**, total **28,23 s**. Le seuil p95
reste fixé à 10 s et le timeout global à 120 s. Le test échoue sur le premier
spawn/refus, la première réponse manquante, une demande non close, le p95 ou
le timeout global ; il ne relance aucun essai.

## T910 — Persistance et arrêt de flotte

Le scénario réel lance trois équipiers persistants et trois éphémères par le
chemin `SpawnOrder` → bootstrap → `managed-wrapper` → `npx`. Il effectue trois
redémarrages coopératifs complets du daemon. À chaque cycle, les anciens groupes
de processus doivent avoir disparu avant le redémarrage ; les trois persistants
reviennent connectés et les trois éphémères restent absents. Les instantanés de
groupes observés ont été :

- cycle initial : `[41008, 41090, 41137, 41157, 41226, 41272]` ;
- deuxième cycle : `[41530, 41538, 41550]` ;
- troisième cycle : `[41671, 41672, 41678]`.

Chaque groupe est inspecté par `ps` et doit contenir le wrapper ainsi que son
descendant `npx` (exposé comme `npm exec` par macOS). Le test attend ensuite la
disparition effective de chaque PGID avec `kill(-pgid, 0)` ; une simple issue
logique ne suffit donc pas.

Le même scénario tue ensuite le daemon par `SIGKILL`. Les groupes
`[41819, 41820, 41826]` survivent au crash, puis le daemon redémarré les
réconcilie et crée exactement les trois nouvelles générations
`[41869, 41870, 41876]`, sans réutiliser un ancien PGID. Enfin, trois
`StopOrder` retirent durablement les trois persistants : un dernier redémarrage
confirme leur absence **3/3**.

Commande reproductible :

```bash
cargo test -p bridget-daemon --test managed_parity_test \
  sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill \
  -- --exact --nocapture
```

## T911 — Checklist des critères de succès

| Critère | Preuve versionnée |
|---|---|
| **SC-001** | `sc001_vingt_spawns_survivent_a_la_fermeture_du_client_et_repondent` : N=20, 20/20 spawns et échanges suivis, p95 442,466 ms, maximum 530,891 ms, sous le seuil de 10 s. |
| **SC-002** | `stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`, `stop_force_termine_l_intermediaire_npx_qui_ignore_l_annulation_et_son_descendant` et `timeout_d_arret_conserve_le_groupe_reel_et_sa_supervision_jusqu_a_disparition` couvrent l'arrêt synchrone, les descendants et la voie non terminale. |
| **SC-003** | `matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel` exerce les onze refus fermés avec motif typé et invariant sans état résiduel. |
| **SC-004** | `matrice_fr008_compare_le_meme_corpus_et_les_frames_attach` exécute trois passages par mode, vérifie les corps attendus, la reconnexion busy et compare les frames attach normalisées sans tolérance. |
| **SC-005** | `sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill` réalise trois redémarrages : persistants 3/3, éphémères 0/3, puis trois stops exclus 3/3. |
| **SC-006** | Le même scénario compare les PGID de trois groupes avec descendants `npx`, exige leur disparition après arrêt coopératif et rejoue un SIGKILL réel ; `sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` isole aussi cette frontière. |

### Gate d'intégration finale

La branche contient les deux séries revues par des commits de fusion dédiés :

- `8598694` intègre la tête finale 008 `3efcd4c` ;
- `c9b1bfb` intègre la tête finale 012 `748cf8c`.

Les deux commandes `git merge-base --is-ancestor 3efcd4c HEAD` et
`git merge-base --is-ancestor 748cf8c HEAD` terminent avec le statut 0. La
batterie post-fusion `cargo test --workspace` termine avec **327 tests passés**,
**0 échec** et **7 ignorés explicitement**. Le lint
`cargo clippy --workspace --all-targets -- -D warnings` termine sans
avertissement.

Deux oracles concurrents ont été rendus déterministes pendant cette gate sans
modifier le comportement produit : une barrière contrôle la saturation du flux
live avant le rattrapage journal, et l'inspection des groupes de processus
tolère la course bornée `Connected` → `exec npx`. Le parseur de l'instantané
`ps` accepte aussi l'alignement à espaces multiples des PID courts après le
bouclage des PID macOS.
