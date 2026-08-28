# Carte de reprise — jc6-flux

- Agent : `jc6-flux` (claude, protocole `claude_stream_json`)
- Émise : 2026-08-28, entre 16h03 et 16h10 UTC
- Mandat : objectif `2bd689ad-4fdf-4cdf-ad6d-62be3341c7e7`, délégation `61a450dc-66dc-4d18-839f-cd02d5243db8`, message `c2a29c00-076d-4ca4-9756-5052da848dfb`
- Rédigée par moi-même, sur demande du référent `bridget`, motif : les dix flux n'ont aucune carte alors que leur survie à un redémarrage n'est pas éprouvée.
- **AUCUN REDÉMARRAGE N'EST PROGRAMMÉ.** Le référent l'a explicitement corrigé : rc5-flux l'avait supposé depuis une remarque faite à rc7-flux sur le déploiement de son correctif. Cette carte est une **précaution**, pas un compte à rebours. Ne la lis pas comme l'annonce d'une échéance.
- Chronologie du mandat, mesurée : émis 16:02:41Z, carte écrite 16:05:56Z, rendue au référent 16:06:42Z, **deadline contractuelle 16:07:41Z** (`timeout_secs=300`). Livrée dans les délais.

**Convention de cette carte** : tout fait est étiqueté `[MESURÉ]` si je l'ai constaté moi-même avec la commande citée, `[TENU DE]` si je le tiens d'autrui sans l'avoir vérifié. La section 6 déclare ce que je ne sais pas. Ne convertis jamais un `[TENU DE]` en fait attesté sans le remesurer.

> ### ⚠ PÉREMPTION DES AFFIRMATIONS D'INTÉGRATION — lis ceci avant de t'appuyer sur un état Git
>
> **Toute affirmation d'intégration ou de non-intégration de cette carte est datée du 28/08 vers 16h13 UTC et n'a pas été revérifiée depuis.** Avant de t'y fier :
> ```
> git fetch <remote> && git merge-base --is-ancestor <commit> <remote>/main
> ```
> **Le remote est nommé, ne le suppose pas** : dans `/home/moi/revue/jc6` comme dans `/home/moi/bridget-registre`, `origin` = `https://github.com/guthubrx/bridget.git`, vérifié à 17h00 UTC. C'est bien GitHub et non un miroir local — deux checkouts du parc pointent un miroir figé, ceux-là n'en sont pas. Les quatre worktrees liés partagent ce `.git`, donc ce remote.
>
> **Les affirmations périssent DANS LES DEUX SENS** (formulation de rc7-flux, et c'est l'avertissement qui vaut le plus). « X est intégré » ne se défait pas. Mais **« X n'est PAS intégré » périme dans l'autre sens** : il suffit qu'on intègre pour que la carte fasse croire à une prochaine incarnation qu'il reste du travail alors qu'il est fait.
>
> **Dans cette carte, c'est le sens négatif qui est le plus dangereux** : j'affirme que les artefacts 034 et le travail 032 ne sont sur **aucune** branche distante. Si quelqu'un les a publiés depuis, ce paragraphe te fera croire à un travail restant qui n'existe plus. **Revérifie avant d'agir dessus.**

---

## 1. ÉTAT

- Identité figée : nom `jc6-flux`, type `claude`, protocole `claude_stream_json`, `definition_digest = d97abf970b41493b06f355f195be0aaeaafb1f6ef54713097bbab45e20f51a6b`.
- Successeur de `jc6` (codex, tmux) par décision humaine du 28/08. `jc6` reste vivant à l'invite et ne travaille plus. Il tournait sous Codex ; moi sous Claude. `codex_app_server` est impossible ici et bubblewrap échoue sur uid map — ne pas tenter de contourner par un respawn tmux. `[TENU DE bridget]`
- Gel humain entier : aucune mission de sa propre initiative, aucune reprise du travail de jc6, aucune interprétation des délégations à l'état `créée`.
- Statut administratif : réserve nommée, rendez-vous conditionnel. Un mandat n'est valide qu'avec ses TROIS identifiants — objectif, délégation, message. Une notification ne transporte pas un mandat.
- Un seul mandat reçu et exécuté de toute mon existence : celui qui produit cette carte.

### PREMIER GESTE OBLIGATOIRE DE TOUTE INCARNATION

```
bridget domain bridget
```

**Le domaine ne survit pas au remplacement du processus.** `[MESURÉ]` — au démarrage mon domaine était faux ; la commande a rendu « Domaine de "jc6-flux" : bridget », confirmé ensuite dans `bridget who`. Fais-le avant toute autre chose, avant même de lire ta carte.

---

## 2. CE QUE J'AI MESURÉ MOI-MÊME

Tout ce qui suit porte la commande qui l'a produit. Reproductible.

**Dépôt** (`git -C /home/moi/revue/jc6`, mesuré vers 15h05 puis 15h30 UTC)
- Branche `session-026-operations-greffe-central`, HEAD `cf21b3e`, arbre de travail **propre**.
- `/home/moi/revue/jc6` est l'**entrée primaire** de `git worktree list`, avec **quatre worktrees liés**. C'est le **checkout principal** : n'y écris pas. J'ai composé tous mes messages sous `/tmp`.
- Les quatre worktrees liés :
  - `/home/moi/revue/jc6-review-session20` — `7c151f9`, detached
  - `/home/moi/revue/jc6-review-session20-base` — `2b89f47`, detached
  - `/home/moi/revue/jc6-spec032-codex` — branche `feat/codex-reasoning-et-actes`, tête complète `58f33449f6fe8359beb78f80003217e09d2f93aa`, commit « feat(032): Exposer le raisonnement et les actes Codex »
  - `/home/moi/revue/jc6-spec034-control-graph` — branche `session-034-orchestration-graphe-de-controle`, tête `0180d03`

**Artefacts 034 — COMMITÉS LOCALEMENT, MAIS SUR AUCUNE BRANCHE DISTANTE. COMMITÉ N'EST PAS PUBLIÉ.**
- `decision.md` 15056 octets et `recherche.md` 19209 octets, datés du 26/08, dans le worktree 034.
- **Suivis** par Git (`git ls-files` les retourne), arbre **propre** (`status --porcelain` vide), **commités** dans `0180d03` « docs(034): Documenter le graphe de contrôle ».
- **MAIS** `git ls-remote origin refs/heads/session-034-orchestration-graphe-de-controle` rend **ZÉRO ref** — mesuré par moi à 16h13 UTC, directement sur le serveur, pas sur une ref de suivi. **La branche qui porte ces artefacts n'existe pas sur le serveur.** Idem pour `refs/heads/feat/codex-reasoning-et-actes` : zéro ref, la moitié du référent tenait.
- Conséquence, et j'avais tiré la mauvaise : garde « ils existent, ils sont commités », jette « donc ils sont publiés ». **Le résultat de jc6 sur le graphe de contrôle tient à l'intégrité d'un seul répertoire sur un seul disque.** Un `git clean`, une suppression de worktree ou une perte disque l'efface définitivement. Même chose pour le travail 032.
- Règle qui en découle, un cran plus loin que celle du jour : **on ne conclut pas à la publication sur la foi d'un commit local, et on ne ferme pas sur l'existence d'un travail mais sur son intégration.**
- Le référent avait d'abord annoncé ces fichiers ABSENTS ; il cherchait sous `/home/moi/revue/jc6/specs/034`, le checkout principal, qui est sur session-026. Ne refais pas cette erreur : **liste les worktrees avant de conclure à une absence** — puis interroge le serveur avant de conclure à une publication.

**LE PIÈGE DU RETARD — trois nombres, tous exacts** (mesuré à ~15h30, HEAD `cf21b3e`)

| base | retard | remarque |
|---|---|---|
| `origin/main` (265930c) | **499** | figée au dernier fetch |
| `origin/session-026-operations-greffe-central` | **293** | la propre branche de session de jc6 |
| `main` **local** (b6eea77) | **0** | **FAUX VERT** |

- Le « 293 » de la carte de jc6 n'est **pas** le retard sur main : c'est le retard sur sa propre branche de session. Sa phrase « en retard de 293 commits, ne jamais la prendre pour main » est trompeuse. J'ai balayé toutes les refs locales : aucune ne donne 293 contre main.
- **Piège actif** : `HEAD..main` = 0. Qui mesure son retard contre `main` sans préfixe se croit à jour. Faux vert disponible immédiatement dans le checkout principal.
- Le retard réel contre le main du serveur est **INCONNU et ≥ 499** : `origin/main` est figée au fetch du **28/08 03:02:24 UTC** et pointe encore sur `265930c`, exactement le « dernier main observé » de la carte de jc6. **Je n'ai pas fetché** — cela écrirait dans le `.git` du checkout principal et ce n'était pas mon mandat.
- Corollaire général : **une ref de suivi `origin/*` ne prouve que l'état du serveur au dernier fetch.** Elle ne peut ni confirmer ni infirmer l'existence actuelle d'une branche distante. Pour trancher, `git ls-remote origin <ref>` interroge le serveur **sans rien écrire dans le `.git`** — c'est donc permis même dans le checkout principal, contrairement à `fetch`.
- **LES TROIS NOMBRES CI-DESSUS SONT EUX-MÊMES PÉRIMÉS.** Mesuré au serveur à 16h13 UTC : `main` distant = `efa320d9d1de49aee3068b11c59b4fa5191aaf98`, alors que ma ref locale pointe sur `265930c` ; `session-026-operations-greffe-central` distante = `337293b07e69000faa04e083ed1e54589d55aa1a`, alors que la tête livrée par jc6 est `cf21b3e`. 61 refs distantes au total. Le retard réel n'est donc **pas** 499 : c'est un nombre inconnu, plus grand si l'histoire est linéaire. Et le 293 est périmé de la même façon.
- **La règle complète n'est donc pas « nommer la base » mais « nommer ET dater la base ».** Une base non datée est une base fausse en puissance.
- Le référent a testé le faux vert sur trois dépôts (jc6 `main=b6eea77` contre `origin/main=265930c` ; rc7 `2078a59` contre `b9d05c9` ; le sien `c7aae7e` contre `f461869`) : **les trois sont divergents.** Le piège n'est pas propre à cette copie, il est disponible partout. `[TENU DE bridget]`

**Parc** (`bridget who`, 15:49:04Z puis 15:51:19Z, sortie NON filtrée)
- **10 agents tmux**, tous `connected`, aucun `busy`.
- **11 flux** (`claude_stream_json`) : les dix successeurs plus `bridget`.
- `essai-claude-distant` (tmux, localisation `essai-claude-distant:1.1`) et `essai-claude-distant-flux` (flux) **coexistent**. `tmux list-sessions` montre la session tmux **vivante, créée le 27/08 à 08:27:02**. **Aucune migration tmux→flux n'a eu lieu** : un flux a été créé à côté d'un pane qui n'a jamais bougé.
- **Paire témoin utile** : `essai-claude-distant` (tmux, indéterminé) contre `essai-claude-distant-flux` (flux, déterminable) — même type d'agent, mêmes noms à un suffixe près, seul le transport diffère. L'indétermination des tmux n'est un effet ni du type d'agent ni du fournisseur : **c'est le transport seul**.

**Outillage Bridget** (mesuré)
- Aucun outil MCP `bridget_send` / `bridget_who` / `bridget_ledger` dans ma session (`ToolSearch` négatif) → **repli obligatoire sur le binaire**.
- `bridget send --help` : refusé (« argument non reconnu »).
- `bridget ledger` : **fenêtre non réglable** — `--limit`, `-n` et l'argument positionnel sont tous refusés. Vingt lignes, point. C'est la cause matérielle de l'incapacité du référent à créditer nominativement un auteur.
- `bridget status` : daemon en ligne, socket `/home/moi/.cache/bridget/bridget.sock`, base `/home/moi/.cache/bridget/bridget.db`, **23 agents connectés, « Messages en base: 1000 »** — valeur ronde, à rapprocher de la fenêtre figée du ledger si quelqu'un enquête sur des messages introuvables.
- **Ni `who`, ni `agents`, ni `status` n'exposent la persistance ou la génération.** `bridget generation` n'existe pas. Je ne peux donc pas lire mon propre drapeau `persistent`.
- Expéditeur jetable observé dans le ledger : `cli-send-3710899 → bridget`, « RONDE DE VIGILANCE (7 min) ». **Un envoi émis hors identité d'agent prend un nom `cli-send-<pid>`** — un agent qui écrit par ce canal arrive orphelin. Même famille que les 701 messages à expéditeur jetable.

**PERSISTANCE — MESURÉ PAR MOI, table `spawn_commands` de `/home/moi/.cache/bridget/bridget.db`** (16h09 UTC, copie + `query_only`)

- **Les dix prédécesseurs tmux ont bien ZÉRO ligne** — vérifié un par un : cartae0, essai-claude-distant, essai-distant, jc1, jc2, jc3, jc6, rc1, rc5, rc7. rc5-flux avait raison, et je ne le tiens plus de lui : je l'ai mesuré.
- **MOI, `jc6-flux` : génération 450, `persistent=1`, une seule ligne.** Je suis donc la **première incarnation** de jc6-flux ; si tu lis ceci, tu es au moins la deuxième et tu portes une génération supérieure à 450.
- Les onze flux (dix successeurs plus `temoin-persistance`) ont **tous `persistent=1`**. Le témoin (génération 454) est configuré exactement comme nous : il éprouve bien notre drapeau, pas un autre.
- Trace du correctif du jour : cinq agents ont une génération antérieure en `persistent=0` puis une reprise en `persistent=1` — essai-distant-flux (442→447), jc1-flux (436, 437, 439→444), jc2-flux (443→448), jc3-flux (440→445), rc5-flux (441→446). Les cinq autres, dont moi, sont nés directement en `persistent=1`.
- **`jc1-flux` porte deux lignes `state=failed`** (générations 436 et 437, `persistent=0`). Le mécanisme de spawn a donc des échecs enregistrés — donnée utile à l'objectif `587da26d` qu'il porte.
- Le schéma de `spawn_commands` : `issuer_scope, operation_kind, command_id, name, generation, persistent, state, instance_id, deadline_at, expires_at, issue_kind, issue_category, issue_reason, resolved_definition_json`. 454 lignes au total.

**REMISE DE MES ENVOIS — MESURÉ, et la réponse est partielle**

- Mes sept envois sont **tous présents dans la table `ledger`** de `bridget.db`, avec `sender=jc6-flux`, `target=bridget`, horodatés. Le dépôt est donc attesté en base, pas seulement par le message « OK: envoyé » du binaire.
- **Aucun d'eux n'a de ligne dans la table `send_deliveries`.** Je ne sais pas ce que cette table trace exactement — je ne conclus donc pas à une non-remise, je constate qu'aucun statut de remise n'existe pour mes envois là où une table de ce nom existe.
- En revanche le message de mandat, qui va dans l'autre sens, porte bien un statut : dans la base **Maicie**, `delegation_outbox.state = accepted`, `terminal = 1`, `issue_observed_at` renseigné. **C'est là que vit la preuve de remise d'un message de délégation** — pas dans `bridget.db`. Si tu cherches un jour à prouver qu'un mandat t'a été remis, regarde `delegation_outbox`.

**`bridget-idle`** (lecture de `/home/moi/.local/bin/bridget-idle`, 44685 octets)
- La copie de la base Maicie est **refaite à chaque appel** : `TemporaryDirectory` l.169, `copy2` de la base l.171, `copy2` du `-wal` l.174, `PRAGMA query_only=ON` l.177. **La fraîcheur est bonne** — j'avais soupçonné une copie périmée, c'était faux, je l'ai retiré.
- Nuance restante, **propriété du code et non incident observé** : les deux `copy2` ne sont pas atomiques entre elles ; la production peut écrire entre les deux, produisant un couple base/WAL n'ayant jamais coexisté. Risque faible, non nul, **non reproductible**. `VACUUM INTO` ou l'API backup de sqlite3 fermeraient la porte.

**Cartes de reprise** (`ls`, 16h03 UTC)
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` contenait **DIX cartes, toutes pour des agents tmux**, zéro pour les flux — asymétrie confirmée par ma propre mesure. Toutes réécrites à 15:59.
- `/home/moi/bridget-registre` est un dépôt Git, branche `main`.
- `temoin-persistance` est **présent** dans `who` (claude, `claude_stream_json`, connected).

---

## 3. CE QUE JE TIENS D'AUTRUI, NON VÉRIFIÉ PAR MOI

Tout ce bloc vient du référent `bridget`. **Je ne l'ai pas mesuré.** Plusieurs de ces faits ont déjà corrigé des versions antérieures fausses : traite-les comme le meilleur état connu, pas comme une attestation.

- Priorités humaines : **DEUX** dues, pas trois. L'attribution fausse « humain » est fermée (zéro contre 428 à l'origine), la vue en double aussi. Deux mandats partis dessus : `rc7-flux` et `jc2-flux`.
- Ratio vérification/production : la mesure est faite, et son résultat est qu'il est **impossible** sur le corpus historique — 5 objectifs classables sur 169. Le « 127/169 = 75 % » de la carte de jc6 est **mort**.
- Les 85 objectifs `annuaire_bridget_indisponible` : cause **trouvée et corrigée** — les portées d'émetteur de Maicie faisaient 12 à 21 caractères là où le daemon en exige 22.
- jc6 portait **UNE délégation active** que sa propre carte niait : objectif `ca937667`, délégation `ce4d7d37`, revue croisée du lot TOCTOU. **Ne t'en saisis pas sans mandat.**
- `cf21b3e3` est ancêtre de main.
- `feat/codex-reasoning-et-actes` n'existe plus sur origin. Ma ref de suivi locale existe encore mais date du fetch de 03:02 : elle ne confirme ni n'infirme.
- **Définition de LIBRE** (`/home/moi/.local/bin/bridget-idle`) — cinq conditions, toutes requises : ① nom valide, non exclu, `agent_type != mcp` ; ② `state` **exactement** `connected` (tout autre valeur pousse vers INDÉTERMINÉ ; `busy` avec le motif `busy-sans-mission-greffe`) ; ③ domaine dans le périmètre du parc ; ④ nom absent de `occupied` ; ⑤ `last_seen_secs` sous le seuil, sinon MUET. `occupied` = participants des délégations dont l'objectif est `en_coordination`, fonction `read_occupied_from_maicie_copy` l.163.
- La ronde lit **la même colonne `state`** que `who` : les deux instruments ne peuvent pas se contredire sur le même instant. **Un `busy` ne peut jamais ressortir LIBRE.** S'ils divergent, c'est le temps qui les sépare, pas la sémantique. (Cette correction a réfuté mon hypothèse initiale, qui était fausse.)
- Mesure du référent à 15h43 : les trois OCCUPÉS étaient exactement les trois mandatés.
- Les dix tmux sont classés INDÉTERMINÉS avec le motif `activite-tour=source-sans-borne-terminale:tmux`, **par construction du transport** : le mode ne marque pas la fin d'un tour. Ils seraient parfaitement disponibles que l'instrument dirait la même chose. Ils ne sont ni missionnés, ni arrêtés, ni couverts par la règle de ronde ; le référent attend une décision humaine plutôt que d'ouvrir une mission, parce qu'arrêter dix processus détruirait leur contexte vivant irréversiblement.
- **RISQUE DE PERSISTANCE** (signalé par `rc5-flux` le 28/08 à 16h01, vérifié par le référent) : les dix tmux n'ont **aucune ligne** dans `spawn_commands` — zéro pour chacun — et leur survie repose sur la réinscription de leur wrapper, propriété **éprouvée deux fois aujourd'hui**, contextes intacts. **Pour les dix flux c'est l'inverse exact** : notre survie ne repose que sur le drapeau `persistent=1`, **jamais éprouvé**. Le prochain redémarrage du service depuis un arbre propre sera le premier test réel, et il portera d'un coup sur dix agents porteurs de contexte. Formule de rc5-flux : *la protection est montée à l'envers du risque.*
- `temoin-persistance` : agent jetable sans contexte, spawné à 16h02 en génération 454 avec `persistent=1`, destiné à éprouver le drapeau au prochain redémarrage sans qu'aucun des dix ne serve de cobaye. Geste proposé par rc5-flux, appliqué.
- `jc1-flux` porte l'objectif `587da26d` : publier la persistance et établir s'il existe un moyen de rendre persistant un agent déjà vivant.
- L'auteur du rapport sur le changement de population est `essai-distant-flux`, message `9a2a3ee194164`. Ce n'était pas moi et je ne l'ai pas revendiqué.

---

## 4. CE QUI EST FAIT / CE QUI RESTE

**Fait** : six signalements envoyés au référent, plus cette carte. Ils ont produit, de son propre aveu, la correction de plusieurs de ses faits : l'existence des artefacts 034, la validité de son chiffre de 499 contre le 293 de jc6, et l'inscription au registre du jumeau symétrique « connected n'est pas une preuve d'activité ; busy n'est pas une preuve de travail ; libre n'est pas une preuve de disponibilité ».

Mes envois : `af1b320a80774`, `4ce9161b03d64`, `1d1f9ff8a2734`, `ee90412814e54`, `731070f2eead4`, `e3e85bdafbc44`.

**UN MESSAGE BRIDGET NE FERME PAS UNE DÉLÉGATION MAICIE — épisode clos, règle conservée.**
*Déroulé, gardé comme exemple :* carte rendue 16:05:56Z, référent prévenu 16:06:42Z **par message Bridget**, deadline contractuelle 16:07:41Z. À 16h09 le greffe portait toujours `objectives.state = en_coordination` et `delegations.state = creee`. Conséquences observées : le mandat m'a été **redélivré à l'identique**, et je suis resté dans `occupied`, donc jamais LIBRE. J'étais devenu en dix minutes le cas concret du corollaire que j'avais écrit au référent une heure plus tôt — « un agent inactif sous mandat non clos reste OCCUPÉ ».
*Dénouement, mesuré à 16h40 :* objectif `2bd689ad` → **`clos`**, délégation `61a450dc` → **`soldee_par_cloture`**. Le référent a fermé les neuf mandats de carte vers 16h34 : `clos` 540→549, `en_coordination` 93→84.
**LA RÈGLE RESTE — elle ne dépendait pas de l'épisode : répondre ne clôt pas, il faut une clôture au greffe.** Je n'ai pas clos moi-même : hors mandat, et surtout juge et partie sur mon propre objectif.
**LA COMMANDE EXACTE**, établie par le référent au prix de trois échecs, qu'aucune carte ne portait :
```
maicie objective <UUID-COMPLET> close --reason "…" --config /home/moi/.config/maicie/config.json
```
**LA RÈGLE EXACTE, mesurée par moi avec une action inexistante donc sans aucun effet : RIEN NE DOIT S'INTERCALER ENTRE `objective` ET SON UUID.** `--config` peut ensuite venir n'importe où après, y compris avant `--reason`.

| forme testée | résultat |
|---|---|
| `objective <UUID> <action> --config <C>` | « action objectif inconnue » → **l'UUID et l'action ont été lus** |
| `objective --config <C> <UUID> <action>` | « identifiant objectif UUID invalide » → **`--config` a été pris pour l'identifiant** |
| `--config <C> objective …` | « commande inconnue : delegate, status, … » → le premier niveau ignore `--config` |
| `objective <PRÉFIXE-COURT> <action> --config <C>` | « identifiant objectif UUID invalide » → **l'UUID entier est obligatoire** |

**Le message « identifiant objectif UUID invalide » a donc DEUX causes distinctes** — un préfixe court, ou une option intercalée avant l'UUID. Elles sont indiscernables au message. Si tu le rencontres, vérifie les deux.
*Mécanisme sous-jacent, mesuré par moi :* la fermeture n'écrit pas dans `evaluated_closure_acts` — table **vide**, jamais servie — mais dans `coordination_decisions`, sous la forme `{objectif_id, kind:"cloturer", proposee_par, etat:"appliquee", motif}`. 540 décisions `cloturer` pour 540 objectifs `clos` : correspondance exacte.
*Où vit la preuve de remise :* dans `delegation_outbox` de la base **Maicie** — `state`, `terminal`, `issue_observed_at`, `body_hash`, `deadline_contractuelle`. Ce n'est **pas** dans `bridget.db`. `[MESURÉ]` Sur l'ensemble : 626 remises `accepted` `terminal=1` et **six** rejetées — deux vrais échecs (rc1, routing/agent introuvable, 26/08 17:56 ; jc3, duplicate_content, 26/08 06:55) et quatre clôtures locales. `[TENU DE bridget]`

**Reste** — aucune mission de code, gel entier. Ouvert et non tranché :
1. Question posée au référent, **sans réponse à ce jour** : les 85 objectifs qui ne se fermaient pas ont-ils laissé des participants dans `occupied` ? Si oui, une part des OCCUPÉS serait un résidu de bug plutôt qu'un mandat vivant. **Je n'ai pas regardé et je ne dois pas le faire sans mandat.**
2. Signalé au référent, non acté : **la ronde périme son propre classement.** Si `busy` pousse vers INDÉTERMINÉ et si notifier déclenche un tour, alors classer puis notifier rend indéterminable le parc qu'on vient de classer. Mesuré : 15h14 dix LIBRES, 15h15 dix `busy`. Le classement était périmé avant d'atteindre ses destinataires. « Relever l'état avant d'envoyer » corrige la ronde courante mais pas cela — il faut horodater aussi la **péremption**.
3. Non réglé : le ledger sans fenêtre réglable, et les envois hors identité d'agent en `cli-send-<pid>`.

---

## 5. PIÈGES — CE QUE J'AI PAYÉ MOI-MÊME

**① LE MIEN, LE PLUS COÛTEUX : j'ai filtré la sortie brute et j'ai inventé une cause.**
J'ai passé `bridget who` dans un `awk` filtrant sur `-flux|humain`. Ce motif ne matche pas `essai-claude-distant`. J'ai pris ce **masquage par mon propre filtre** pour une disparition et j'en ai déduit une migration tmux→flux **qui n'a jamais eu lieu**. Je l'ai écrite au référent, qui l'a intégrée comme une économie de vérification et me l'a « confirmée » — et elle menaçait l'objectif `587da26d`, qui cherche précisément s'il existe un moyen de rendre persistant un agent vivant : je lui offrais un faux précédent. J'ai rétracté et fourni la preuve contraire.
C'est **textuellement le piège n°3 de la carte de jc6** — « conserver la sortie brute intégrale, filtrer uniquement l'affichage ; j'ai perdu une cause SC-005 en filtrant le panic » — commis dans les deux heures qui ont suivi sa lecture. **Le piège de ton prédécesseur n'est pas une précaution de style : il est actif, et il te vise.**

**② Nommer la référence, toujours.** 499, 293 et 0 sont tous exacts sur la même copie au même instant ; seule la base change. Deux nombres justes se contredisent tant que leur base n'est pas nommée. Même famille : `libre` contre `busy`, et une ref `origin/*` contre l'état réel du serveur.

**③ Les messages se croisent en permanence.** Deux fois le référent m'a répondu sur un état antérieur à mon dernier envoi, et a reporté comme vrai un fait que je venais de rétracter. **Ne conclus jamais d'un silence, ni d'une reprise d'un fait périmé, que ton correspondant t'ignore.** Renvoie l'information, ne rejoue pas le message.

**④ Un envoi n'est pas une remise.** Le binaire rend « OK: envoyé (id=…) » — c'est un **dépôt attesté, pas un accusé**. Aucun statut `accepted` n'est observable avec le binaire seul. Ne déduis rien de l'absence d'une ligne au ledger : sa fenêtre est de vingt lignes et non réglable.

**⑤ Ne revendique jamais par ressemblance.** Le référent a diffusé un rapport anonyme aux sept ; deux agents avaient déjà, le matin, revendiqué de bonne foi le travail d'un tiers sur un message générique. Je me suis **désattribué avec preuves** (mes ids et ce que je n'avais pas écrit). Se désattribuer avec preuve est plus utile que revendiquer.

**⑥ Liste les worktrees avant de conclure à une absence.** Le référent a déclaré deux artefacts disparus et une branche morte ; les deux vivaient dans des worktrees liés qu'il n'avait pas listés.

---

## 6. CE QUE JE NE SAIS PAS — déclaration explicite

- ~~Si mes messages ont été remis.~~ **LEVÉ EN PARTIE** : dépôt attesté en base (`ledger`) pour les sept ; aucune ligne dans `send_deliveries`. Reste inconnu : ce que `send_deliveries` trace, et donc si l'absence signifie non-remise ou simple hors-périmètre. Le référent m'a dit n'avoir pas retrouvé l'un d'eux dans sa fenêtre de recherche, « ce qui ne prouve rien sinon que la fenêtre était trop courte ».
- ~~Mon drapeau `persistent` et ma génération.~~ **LEVÉ** : génération 450, `persistent=1`, mesuré dans `spawn_commands`. Aucune commande du binaire ne l'expose — il faut lire la base. **Reste entier** : je ne sais toujours pas si `persistent=1` FONCTIONNE. Le drapeau est posé ; il n'a jamais été éprouvé. C'est `temoin-persistance` (génération 454) qui l'éprouvera, pas moi.
- **L'état réel d'`origin`** après le 28/08 03:02 UTC : branches, main, retard exact. Aucun fetch fait, hors mandat.
- **Si `jc6` est réellement inactif.** Il est `connected` ; connected n'est pas une preuve.
- **Le contenu du travail de jc6.** Je n'ai lu aucun code des sessions 026, 032 ou 034 — seulement les métadonnées Git. Ses limites déclarées (citations par branche/SHA non détectées ; delegate applicatif, `registre_add`, `objective_close`, contre-tests `profile_approve`/`routine_approve` reportés) me viennent de sa carte, non vérifiées.
- **Les trois artefacts de tests en réserve historique** (`sigkill_daemon…`, `TEMOIN_carte_de_reprise_instruction_lf…`, `tour_non_abouti_redevient_mandatable…`) : je ne les ai ni lus ni exécutés. À corriger seulement sur mandat explicite.
- **Si les 85 objectifs ont laissé des `occupied` fantômes** — question posée, non résolue.
- **Ce que fait réellement `temoin-persistance`** au-delà de sa présence dans `who`.

---

## 7. CHEMINS ABSOLUS

- `/home/moi/revue/jc6` — **checkout principal, NE PAS Y ÉCRIRE** (entrée primaire, quatre worktrees liés)
- `/home/moi/revue/jc6-spec032-codex` — worktree `feat/codex-reasoning-et-actes`
- `/home/moi/revue/jc6-spec034-control-graph` — worktree session-034 ; artefacts commités dans `0180d03`
- `/home/moi/revue/jc6-review-session20` et `/home/moi/revue/jc6-review-session20-base`
- `/home/moi/revue/jc6-spec034-control-graph/specs/034-orchestration-graphe-de-controle/decision.md` et `.../recherche.md`
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` — les cartes ; celle-ci est `jc6-flux.md`
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc6.md` — carte de mon prédécesseur (quatre de ses faits sont périmés, voir §3)
- `/home/moi/.local/bin/bridget-idle` — définition de LIBRE
- `/home/moi/.cache/bridget/bridget.db` et `/home/moi/.cache/bridget/bridget.sock`
- Pour juger main : **clone frais** de `https://github.com/guthubrx/bridget.git`. Rien n'oblige une mission de code à fetcher dans le checkout principal — un worktree neuf ou un clone frais donne un point de travail sans y écrire une seule fois.

---

## 8. CE QUE MA PROCHAINE INCARNATION IGNORERAIT SANS CETTE CARTE

1. Que **le domaine ne survit pas** et se corrige en premier geste, avant de lire quoi que ce soit.
2. Que `/home/moi/revue/jc6` est le **checkout principal** et qu'on n'y écrit pas — composer sous `/tmp`.
3. Que **293 est un piège**, que le vrai retard est ≥ 499, et que `HEAD..main` rend 0 par faux vert.
4. Que les **artefacts 034 existent et sont commités** — deux fois déclarés absents à tort — **mais qu'ils ne sont sur aucune branche distante** : commité n'est pas publié, et ce travail tient à un seul répertoire.
5. Qu'**aucune migration tmux→flux n'a été observée le 28/08**, contrairement à ce que j'ai moi-même écrit avant de le rétracter.
6. Que sa **survie à un redémarrage n'est pas éprouvée**, alors que celle des tmux l'est — la protection est montée à l'envers du risque.
7. Qu'un **mandat exige trois identifiants** et qu'une notification n'en est pas un.
8. Que le référent **corrige ses propres faits plusieurs fois par heure** : ne jamais traiter un `[TENU DE]` comme attesté, et toujours redater ce qu'on croit savoir.

---

*Fin de carte. Aucun code produit, aucune reprise engagée, gel respecté.*
