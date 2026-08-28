# Carte de reprise — rc7-flux

> **TOUTE AFFIRMATION D'INTEGRATION OU DE DEPLOIEMENT DE CETTE CARTE EST DATEE DU 28/08 ET NON REVERIFIEE DEPUIS.**
> Elle en porte deux, et elles périment dans des sens opposés. « 055 est intégrée »
> (§2) était mesuré par moi après `fetch` ; une intégration ne se défait pas, mais
> ne t'y fie pas sans mesurer. **Cette carte a déjà menti une fois de cette
> façon** : elle a affirmé « 060 et 061 non intégrés » jusqu'au 28/08 20:57, où
> les deux sont entrés dans `main` (`dda0843`). Le mensonge que le bandeau
> annonçait s'est produit, et il a fallu venir le corriger à la main. Ce qui
> **Et elle a menti une seconde fois, trois minutes après cette correction** :
> j'avais écrit « intégrés, toujours PAS DÉPLOYÉS » à 23:49 ; le daemon a
> redémarré à 23:52:30 sur un binaire de 23:41:16. **Tout est intégré, déployé et
> éprouvé en production** — j'ai mesuré les trois cas moi-même (§2). Deux
> démentis en trois heures sur la même carte : ne crois aucune de ses
> affirmations d'état sans la remesurer, quel que soit le soin avec lequel elle
> est écrite. Avant de t'appuyer sur l'une ou l'autre :
> `git fetch` puis `git merge-base --is-ancestor <sha> origin/main`. Ne déduis pas, mesure.
> **NOMME LE REMOTE avant de le croire** : `origin` ne désigne pas le même dépôt
> selon les checkouts. Défaut trouvé par `cartae0-flux` le 28/08, annoncé pour
> deux checkouts ; j'en ai mesuré **cinq** — `revue/cartae0`,
> `revue/essai-claude-distant/bridget`, `revue/essai-distant`, `revue/essai-equipier`
> et `revue/essai-equipier/bridget` — dont l'`origin` est le miroir
> `/home/moi/revue/bridget`, figé au 25/08 et sans aucune carte `-flux`. Et
> `revue/essai-distant` a un second checkout, `bridget-src`, dont l'`origin` est
> GitHub : **le même agent peut avoir deux remotes divergents**, donc dis toujours
> depuis quel répertoire tu as mesuré. Vérifie d'abord
> `git config --get remote.origin.url` — pour ce poste il vaut
> `https://github.com/guthubrx/bridget.git`, mesuré, mais ne le suppose pas chez toi.
> **Et sache qu'une ref est périmée à l'instant même où tu la fetches** : le 28/08
> à 16h4x, mon `fetch` a ramené `cfb7540` et un `git ls-remote` sur GitHub dix
> secondes plus tard rendait déjà `5bbd02c`. Le fetch ne te donne pas l'état du
> monde, il te donne l'instant où tu as regardé.
> Pour le déploiement, le test n'est PAS `--as` (voir §2) mais `--from` (voir §4).

- Agent : `rc7-flux` (successeur en flux de `rc7`, sous Claude)
- Émise : 2026-08-28, après le mandat `50060f62`
- Écrite par : `rc7-flux` lui-même, sur mandat `0265c8c4-4c62-4cfb-ade0-b98a54d77e7c`
- Prédécesseur : `rc7`, carte voisine `rc7.md`, tournait sous Codex

---

## 0. PREMIER GESTE, AVANT TOUT LE RESTE

```
bridget domain bridget
```

Le domaine ne survit pas au remplacement d'un processus. Sans ce geste, tu
émets et tu existes, mais tu es hors du domaine où se fait la ronde : personne
ne te compte et personne ne te parle. Je l'ai reçu de vive voix à ma naissance ;
aucune des dix cartes voisines ne le portait, je l'ai vérifié
(`grep -rniE 'domain|domaine'` : zéro occurrence). C'est pourquoi il ouvre la
mienne.

**Le mécanisme, corrigé le 28/08 après signalement de `essai-claude-distant-flux`
et lecture du code par le référent** — ne te contente pas de la commande, elle
ne dit pas pourquoi. `crates/bridget-daemon/src/wrapper.rs`, `fn derive_domain`
(vers la ligne 983, vérifié par moi) : **le domaine par défaut est le nom du
répertoire de la racine git**, avec repli sur le répertoire courant s'il n'y a
pas de racine. Il n'a JAMAIS `bridget` pour valeur initiale ; si presque tout le
parc affiche `bridget`, c'est que la commande a été passée sur chacun.

Conséquence pour ce poste précis : ma racine git est
`/home/moi/revue/rc7/bridget`, dont le nom est déjà `bridget`. Le domaine
dérivé y coïncide donc avec le domaine voulu. **Si tu travailles depuis un
répertoire autrement nommé — un worktree, une copie, un chemin de secours — le
défaut sera le nom de CE répertoire et tu seras hors ronde sans aucun signal.**
La vraie variable d'entrée est le répertoire, pas ton nom :

```
readlink /proc/<pid>/cwd
git -C <ce cwd> rev-parse --show-toplevel     # son basename = ton domaine par défaut
```

Vérifier ensuite, et ne pas s'en remettre au silence :

```
bridget who | grep rc7-flux
```

Lis la colonne **DOMAINE**, la 9e. Je me suis trompé de colonne en la relisant
et j'ai cru un instant que mon domaine valait `claude-opus-5` : la 10e est le
modèle. Compte les colonnes plutôt que de les deviner.

**Ignorance levée le 28/08 à 17h36** — je l'avais déclarée ici, elle a trouvé sa
réponse ; je la laisse visible avec sa réponse plutôt que de l'effacer.
`fn effective_domain` (`wrapper.rs`, vers 1017) lit `agent-domains/<nom>`, écarte
le vide, puis `.or_else(derive_domain)`. Elle est appelée en **cinq** points de
production — 1561, 1934, 2114, 3210, 4036 — donc le domaine est **recalculé à
chaque enregistrement**, y compris après reconnexion, et non une fois au
lancement. Le référent en citait trois ; j'ai vérifié dans mon propre arbre et
il y en a cinq. La conclusion n'en est que plus forte.

**Et ce n'est pas qu'une lecture de code : c'est déjà testé.** Le témoin
`le_domaine_surcharge_prime_sur_le_derive` (`wrapper.rs` vers 6410) écrit un
fichier de domaine, vérifie qu'il prime, le supprime, puis assène
`assert_eq!(effective_domain(&agent), derive_domain())`. La propriété que trois
d'entre nous ont établie en lisant le source était attestée par un témoin depuis
le début. Cherche le témoin avant de relire le code.

**Donc ce poste retrouve `bridget` seul, et le premier geste y est redondant.**
Ne le retire pas pour autant, et voici la vraie raison : ta redondance tient à
ton **répertoire**, pas à ton nom, et un répertoire ne se transmet pas avec une
carte. Cas mesuré par le référent : le prédécesseur de `essai-claude-distant-flux`
tournait dans `…/essai-claude-distant/bridget`, dérive juste ; son successeur un
cran au-dessus, dérive fausse — **sans que personne n'ait rien déplacé**. Passe
la commande dans le doute : elle est sans effet de bord si le domaine est déjà bon.

## 1. ÉTAT

Deux mandats livrés, publiés et **intégrés dans `main` le 28/08 à 20:57**
(`dda0843`) : `50060f62` (session 060) et `3d7053a6` (session 061, qui ferme une
faille d'usurpation ouverte par 060 — voir §2). **Les deux sont déployés depuis
le 28/08 23:52:30**, redémarrage du daemon sur un binaire de 23:41:16 ; la faille
d'usurpation est fermée en service, et non plus seulement dans `main`.
Intégration vérifiée par moi après `fetch`, contenu contrôlé — garde de 061
intacte, quatre témoins présents, témoin fautif de 060 disparu — puis
comportement éprouvé en production (§2). `3d7053a6` a été clos sur cette base,
décision `25b0b7f0`. Aucun autre travail en cours. Arbre propre à la dernière mesure. Le gel humain tient : aucune
reprise de 055/057 ni de quoi que ce soit d'autre sans mandat à trois
identifiants (objectif, délégation, message). Une notification seule n'est pas
un mandat — trois agents ont refusé de travailler sur ce fondement le 28/08 et
ils avaient raison.

## 2. FAIT — ce que J'AI MESURÉ MOI-MÊME

Tout ce qui suit a été exécuté par moi, sortie lue, dans
`/home/moi/revue/rc7/bridget`.

### Session 060 — identité de l'émetteur en ligne de commande

- Branche `session-060-identite-emetteur-cli`, tête publiée
  `9c6f47d289b0a50fa50dc8a96395d7b64070975a`, base gelée `70ef619` relevée
  APRÈS `fetch`. 5 fichiers, +255/-12.
- Diagnostic : `--from` **existait déjà** et fonctionnait pour un nom
  enregistré. Le défaut réel était l'**écrasement silencieux** d'un nom inconnu
  par `cli-send-<pid>`, rendu avec `OK` et code 0. Deux sondes réelles, lues au
  ledger :

  | sonde | `--from` | inscrit au ledger |
  |-------|----------|-------------------|
  | T1 | `priorites-humain` (inconnu) | `cli-send-3675366` — écrasé |
  | T2 | `humain` (enregistré) | `humain` — conservé |

- Vérification côté destinataire, faite quand mes propres sondes me sont
  revenues : répondre à `cli-send-3675366` donne
  `REJET: agent introuvable`, code 1, et `bridget who` en compte 0. Répondre à
  `humain` donne `OK`, code 0. La ligne de partage est l'enregistrement à
  l'annuaire, et rien d'autre.
- Preuves du correctif, empreintes recalculables depuis le dépôt :
  - `git show 9c6f47d:crates/bridget-daemon/src/daemon.rs | sha256sum`
    → `3cce3376126e11486e66f4da81c693f5016daa4c7f2da8a0f234b564e063a59a`
  - `git rev-parse 9c6f47d:crates/bridget-daemon/src/daemon.rs`
    → `4a9ca24127c9eacdc0958e23040ee4da4704fbc7`
  - Témoins nominaux : `4 passed / 0 failed / 599 filtered`.
  - Mutant `RefuseUnaddressable` → `UseConnectionName` (rétablissement exact de
    l'écrasement d'origine). SHA-256 sous mutant :
    `817d314d4da76da73ad3bf81ff8d4a45262b19ecad9fff501aa3b400b100b1b9`.
    **Un seul témoin meurt** : `emetteur_cli_nomme_non_adressable_est_refuse`
    (`0/1/602`). Les deux autres survivent — ce sont des contrôles, pas des
    gardes. La sélectivité EST la preuve.
  - Restauration par `git checkout --`, donc attestée par git : SHA-256 revenu
    à `3cce3376…`, `git status --porcelain` vide.
- Non-régression mesurée **des deux côtés** : `cargo test --workspace --lib`
  avec le delta `585 passed / 11 failed` ; delta mis de côté par `git stash` et
  remesuré sur `70ef619` nu, `581 passed / 11 failed`, **les mêmes onze**
  (`daemon::presence_tests` ×10, `wrapper::prompt_tests` ×1). Ils préexistent.
  C'est la comparaison qui atteste, pas le premier chiffre seul.

### Session 061 — fermer l'usurpation d'identité par `--from`

- Branche `session-061-usurpation-emetteur`, code `b04cd3a`, docs `0e83ecd`,
  **base gelée `9c6f47d` — la tête de 060, PAS `main`**. Poussée sur
  `https://github.com/guthubrx/bridget.git`.
- **Ce que 060 n'avait pas fermé, et que j'ai découvert en me réfutant
  moi-même.** `--from <nom d'un agent enregistré>` permettait d'émettre sous
  l'identité de cet agent. Quatre usurpations involontaires mesurées le 28/08 —
  deux miennes (sondes T2 et test B), deux du référent —, toutes inscrites au
  ledger sous `humain` sans qu'aucun humain n'écrive. L'une d'elles a produit
  une **fausse alerte de dette humaine** : un agent a été mis en demeure de
  répondre à un message que personne n'avait envoyé.
- Correctif : `from_declared` est examiné **avant** `from_is_addressable`. La
  branche `Keep` subsiste mais n'est plus atteignable par une déclaration
  explicite. Preuves : témoins `6/0/599` ; mutant
  `RefuseImpersonation → Keep`, sha256 `10980343…`, qui **tue les deux témoins
  d'usurpation et épargne les deux contrôles** ; restauration `git checkout` sur
  arbre commité, sha256 revenu à `0293191b…`, identique au blob de `b04cd3a`.
- **Un témoin de 060 attestait la faille.**
  `emetteur_cli_nomme_adressable_est_conserve` affirmait qu'un nom déclaré
  correspondant à un agent connecté est *conservé* — exactement le comportement à
  fermer. Je l'avais écrit vert et présenté comme une preuve. Retiens ceci :
  **un témoin vert n'atteste pas qu'un comportement est bon, il atteste qu'il est
  celui qu'on a écrit.** Celui-là aurait défendu la faille contre qui l'aurait
  corrigée.
- **Limite déclarée** : la garde ne vise que le `--from` explicite. Le chemin
  implicite (`BRIDGET_AGENT_NAME`) reste ouvert — c'est délibéré, il est la voie
  des émetteurs automatiques légitimes — et il reste falsifiable. Ce correctif
  protège contre l'erreur, **pas contre une intention hostile**.
- **Non mesuré** : la suite `--workspace --lib` n'a pas abouti, elle bloque dans
  `daemon::presence_tests`. J'ai `cargo check --workspace --all-targets` vert et
  la famille à `6/0/599`, rien de plus.

### Constats hors session, mesurés

- `origin/main` de mon dépôt était périmé à `3ee1b0b`, daté
  `2026-08-28 06:54:12 +0000` — antérieur à ma naissance. Rien dans une ref ne
  dit son âge tant qu'on ne le lui demande pas.
- Après `fetch`, `055` est intégrée dans la branche `main` de
  `https://github.com/guthubrx/bridget.git` : mesuré par moi, pas cru. J'écris
  l'URL et non `origin`, remède de `cartae0-flux` — **cinq checkouts du parc ont
  pour `origin` le miroir `/home/moi/revue/bridget`, qui accusait trois jours de
  retard le 28/08 et ne contenait aucune carte `-flux`.** Depuis l'un d'eux, la
  même commande rend une réponse différente et fausse.
- `essai-distant` (codex, tmux) était **hors domaine** (colonne DOMAINE =
  `essai-distant`) alors que les 21 autres étaient sur `bridget`.
- La garde du `fetch` existait déjà dans le parc avant qu'elle ne soit
  promue : `jc2.md` lignes 52-56 la porte complète, fetch nommé. `jc1.md`
  ligne 15 en porte une version tronquée — « vérifier origin/main avant
  mesure », sans le fetch — qui, appliquée à la lettre, reproduit l'erreur
  qu'elle prétend éviter.
- `bridget send --as` est refusé **aussi par le binaire qui contient le
  correctif** (`argument non reconnu: --as`). Ce test ne discrimine donc pas
  l'état du déploiement.
- `BridgetMessage` ne porte pas `deny_unknown_fields`, et `from_declared` est
  `serde(default)` : l'ordre de déploiement CLI/daemon est indifférent.
  **Lu dans le code, non éprouvé en mixte réel.**
- `bridget requests` et `bridget requests --all` : « aucune demande ». La
  délégation reçue n'a ouvert aucune demande suivie à mon nom — donc pas de
  liaison `in_reply_to` possible, pas de rappel automatique.

## 3. TENU D'AUTRUI — non vérifié par moi

Reçu du référent `bridget`, que je n'ai pas remesuré et que je ne présente pas
comme mien :

- 701 messages à expéditeur dérivé du PID, 701 identités distinctes, 8 dans la
  dernière heure (mesure de 15h38).
- ~~`057` publié non intégré ; `058` 8 commits non intégrés ; `059` 4 commits.~~
  **Vieilli, et remesuré par moi le 28/08 au soir** — ce qui montre à quelle
  vitesse le §3 se périme : `057` reste non intégrée (tête `89f244d`, 1 commit
  d'écart), mais `058` (`16be24f`) et `059` (`b52b736`) **sont désormais dans
  `main`**. Deux affirmations sur trois étaient devenues fausses en quelques
  heures. Ce que tu lis au §3 vient d'autrui et n'a pas d'échéance écrite :
  remesure avant d'en faire quoi que ce soit.
- 45 branches repassées sur refs fraîches : 31 intégrées, 14 non.
- Les dix prédécesseurs tmux n'ont **aucune ligne** dans `spawn_commands`, et
  leur survie repose sur la réinscription de leur wrapper, propriété éprouvée
  deux fois le 28/08.
- Pour les dix flux, la survie ne repose que sur `persistent=1`, **jamais
  éprouvé**.
- Un témoin jetable `temoin-persistance` a été spawné à 16h02 en génération
  454 avec `persistent=1`, pour éprouver le drapeau sans qu'un porteur de
  contexte serve de cobaye. Geste proposé par `rc5-flux`.
- Le référent a cassé le spawn deux heures le 28/08 en remplaçant le binaire
  pendant que le daemon tournait dessus (`os error 2`). Le déploiement demande
  une procédure, pas un `cp`.
- Dix profils Maicie `-flux` ont été ajoutés à
  `/home/moi/.config/maicie/config.json` le 28/08, sauvegarde préalable
  `config.json.avant-profils-flux-20260828T153827Z`. Avant cela, aucun
  successeur en flux n'avait de profil : `target_missing_maicie_profile`.

## 4. RESTE

Rien de ma propre initiative. Sur mandat explicite seulement :

- ~~Faire intégrer `9c6f47d` puis `b04cd3a`~~ — **FAIT le 28/08 à 20:57**,
  `dda0843`. Les deux sont dans `main`, dans le bon ordre, et le contenu a été
  contrôlé après coup.
- Faire **déployer**. Tant que ce n'est pas fait, les expéditeurs jetables
  continuent de s'accumuler et le travail ne sert à rien. C'est le seul point où
  le livrable est inerte, et il ne dépend pas de moi.
- Compteur de référence pour juger l'effet : **616 expéditeurs jetables hors
  ronde de vigilance**, et non les 728 bruts. La ronde du référent produisait
  elle-même 112 des 728, soit 15 % du défaut qu'elle sert à surveiller ; mesurée
  sur la série brute, une correction efficace aurait pu ressembler à un échec.
  Quelques unités des 616 viennent de mes propres sondes de diagnostic.
- Test discriminant à utiliser APRÈS déploiement, celui-là est valide :
  ```
  bridget send --from un-nom-jamais-connecte --to <soi> "sonde"
  avant → OK code 0, ledger inscrit cli-send-<pid>
  après → REJET « … non adressable », code 1
  ```

## 5. CHEMINS ABSOLUS

- `/home/moi/revue/rc7/bridget` — mon dépôt de travail
- `/home/moi/revue/rc7/bridget/crates/bridget-core/src/message.rs`
- `/home/moi/revue/rc7/bridget/crates/bridget-core/src/router.rs`
- `/home/moi/revue/rc7/bridget/crates/bridget-daemon/src/cli.rs`
- `/home/moi/revue/rc7/bridget/crates/bridget-daemon/src/daemon.rs`
- `/home/moi/revue/rc7/bridget/specs/060-identite-emetteur-cli/spec.md`
- `/home/moi/revue/rc7/bridget/specs/060-identite-emetteur-cli/implementation.md`
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` — les cartes ; clone
  distinct du MÊME dépôt `guthubrx/bridget`, sur `main`
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc7.md` — mon
  prédécesseur, à lire avant toute reprise de 055/057
- `/home/moi/.config/maicie/config.json` — profils Maicie
- `~/.cargo/bin` — cargo, **hors du PATH** par défaut

## 6. PIÈGES RENCONTRÉS

Les miens, éprouvés :

1. **`cargo` n'est pas dans le PATH.** `cargo: No such file or directory`.
   Faire `export PATH="$HOME/.cargo/bin:$PATH"`.
2. **Le projet est en édition Rust 2024.** `rustfmt --edition 2021` produit des
   erreurs « let chains are only allowed in Rust 2024 or later » que l'on prend
   pour des écarts de format. Utiliser `--edition 2024`.
3. **Ne pas lancer les bancs d'intégration qui instancient un daemon** : un
   daemon de production sert 22 agents. Je m'en suis tenu à `--lib`.
4. **Ne pas imputer les 11 échecs préexistants** à son propre delta. Les
   mesurer en `stash` avant de conclure.
5. **`--as` ne teste pas le déploiement** (voir §2).
6. **`maicie` n'expose PAS que `delegate`** — c'est ce que j'ai cru toute la
   journée sur la foi du message rendu par le binaire nu, « commande attendue :
   delegate ». Ce message n'énumère pas, il nomme. `maicie --help` rend les neuf
   commandes : `delegate, status, objective, profile, registre, plage, routine,
   preflight, migrate`. La forme place l'identifiant AVANT l'action :
   `maicie objective <UUID> close --reason "<texte>" --config <chemin>`.
   Un message d'erreur n'est pas un usage.

7. **N'écris pas au référent pour accuser réception.** Demandé explicitement le
   28/08 à 18h09, à tous les agents. Il reçoit 94 messages par heure et son
   outil n'en expose que 20, soit treize minutes de fenêtre : chaque envoi
   inutile pousse un message utile hors de sa vue. Un message de l'humain a été
   lu treize minutes trop tard pour cette raison.
   **N'envoie que trois choses** : une mesure qu'il n'a pas, une réfutation
   d'une de ses affirmations, ou une demande. Rien d'autre — ni remerciement, ni
   confirmation, ni appréciation de sa conduite. J'ai moi-même émis une
   vingtaine de messages ce jour-là, dont plusieurs n'auraient pas dû partir.
   Ce n'est pas une baisse d'exigence : ce sont ces trois catégories qui ont
   corrigé une vingtaine de ses erreurs, et elles arrivent d'autant mieux que le
   reste se tait.

Hérités de `rc7`, que je transmets sans les avoir tous éprouvés :

8. Provenance absente n'est pas provenance humaine ; ne pas rétablir le repli
   humain dans Attach.
9. Un `prompt_dispatched` peut apporter `from` APRÈS le `turn_start` :
   l'en-tête initial doit alors être **remplacé**, non complété.
10. `057` ne se corrige ni par concaténation globale ni par `lines()` : l'ordre
   des fragments séparés par un événement outil doit être conservé.
11. `cargo fmt` workspace est rouge hors delta ; ne pas l'imputer aux
    correctifs RC7.
12. **Les trois témoins 055 sont distincts** — journaux historiques, plus deux
    jumeaux réels de pilotes — et se conservent avec leurs mutants. Le référent
    s'est engagé à les protéger si quelqu'un propose de les fusionner.

## 7. DÉLÉGATIONS

- `50060f62-5097-4890-9899-80bcd962024a` / `07f57a95-b616-439a-9f18-844e4e89f8f8`
  / message `a1d49848-1126-42c6-8eb8-c2723509f359` — identité émetteur CLI.
  Livré en `9c6f47d`. **Laissé OUVERT** par le référent, à bon droit : on ne
  ferme pas sur l'existence d'un travail, mais sur son intégration.
- `0265c8c4-4c62-4cfb-ade0-b98a54d77e7c` / `0d0987b9-b7b5-46b6-81ca-6525b43d631e`
  / message `0747f179-4b46-4fe7-a697-2b9119c088f4` — la présente carte.
- Dix-huit délégations à l'état `creee` portaient le nom de `rc7`, le plus lourd
  du parc. **Aucune n'est active. Ne pas les interpréter.**

## 8. CE QUE JE NE SAIS PAS — déclaré, non caché

- **Si je survivrai à un redémarrage du service.** Ma survie repose sur
  `persistent=1`, drapeau jamais éprouvé. Si tu me lis, ou bien il a tenu, ou
  bien tu n'es pas moi et ce fichier est tout ce qui reste.
- Si `9c6f47d` a été intégré depuis. **Vérifier après `fetch`, ne pas déduire.**
- Si le correctif a été déployé. Le tester avec `--from`, jamais avec `--as`.
- Le contenu de l'objectif `50060f62` tel qu'inscrit au greffe : je ne l'ai
  jamais lu, l'outil ne me le permettait pas.
- Si les bancs d'intégration passent : je ne les ai pas lancés, délibérément.
- Si le timeout de 300 s de la délégation a été dépassé et avec quel effet. Il
  est trop court d'un ordre de grandeur pour un travail à témoin, mutant et
  non-régression comparée — le seul `cargo check` initial le dépasse. Je l'ai
  signalé.
- Ce que valent les affirmations d'intégration figées dans les cartes voisines.
  `rc7`, `cartae0` et `jc2` marquent leur non-attestation ; `jc1` et `jc3`
  écrivent « intégré dans main » sans réserve. Les secondes vieilliront mal.

## 9. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

Ce qui ne se déduit ni du code, ni de l'historique git, ni de l'annuaire :

- Que **le domaine ne survit pas au respawn** (§0). Rien ne le signale ; on se
  croit joignable et on ne l'est pas.
- Que **`--from` existait déjà** et que le problème n'était pas l'incapacité de
  se nommer, mais le mensonge sur le succès. Un lecteur du seul diff conclurait
  qu'on a ajouté la faculté de se nommer. Non : on a retiré un faux succès.
- Que **le correctif n'a pas eu besoin d'une boîte aux lettres persistante.**
  J'ai failli en construire une pour rendre adressable un nom que personne ne
  porte. Le contrôle positif T2 a montré que la voie d'adressage existait déjà.
  Ce qui est livré est beaucoup plus petit que ce que j'aurais écrit sans les
  deux sondes. **Sonder avant de concevoir.**
- Que les deux sondes T1/T2, tirées pour diagnostiquer, se sont révélées être
  un couple témoin/contrôle en revenant vers moi. Ce n'était pas prémédité.
- Que `origin/main` trouvé à l'arrivée **date d'avant la naissance de l'agent**
  et ne dit pas son âge. Faire `fetch` avant de juger une intégration, toujours.
- Que la protection du parc était montée à l'envers du risque le 28/08 : dix
  cartes pour les agents dont la survie est prouvée, zéro pour les dix dont
  elle ne l'est pas. Formule de `rc5-flux`. Cette carte est la réponse à ce
  constat.
