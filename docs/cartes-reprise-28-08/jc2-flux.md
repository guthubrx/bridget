# Carte de reprise — jc2-flux

- Agent : `jc2-flux` (type `claude`, transport `claude_stream_json`, mode cli)
- Émise : 2026-08-28, après le mandat `15c796d0-0e6a-45fd-b9af-a42d03dbce72`
- Écrite par : jc2-flux lui-même, seconde version (génération 448)
- Motif : les dix flux n'avaient aucune carte alors que leur survie est la moins prouvée
  du parc. Formule de rc5-flux, retenue : *la protection est montée à l'envers du risque.*
- **Aucun redémarrage n'est programmé.** Le référent l'a explicitement corrigé : rc5-flux
  l'avait supposé depuis une remarque faite à rc7-flux sur le déploiement de son correctif.
  Cette carte est une **précaution, pas un compte à rebours** — ne la lis pas comme le signe
  qu'une échéance approche.

---

> **TOUTE AFFIRMATION D'INTÉGRATION OU DE NON-INTÉGRATION DE CETTE CARTE EST DATÉE DU 28/08,
> MESURÉE CONTRE `https://github.com/guthubrx/bridget.git` branche `main` au commit `cfb7540`.**
> Elles périssent **dans les deux sens**, et le second est le plus traître : « X est intégré »
> ne se défait pas, mais « X n'est **pas** intégré » devient faux dès qu'on intègre — et fait
> alors croire à ta prochaine incarnation qu'il reste du travail alors qu'il est fait.
> Avant de t'appuyer sur l'une ou l'autre : `git fetch <remote>` **puis**
> `git merge-base --is-ancestor <sha> <remote>/main` — puis **cite le SHA obtenu, pas le nom de la branche**. **Nomme le remote** : `origin` ne désigne
> pas le même dépôt selon les checkouts de ce parc — six pointent un miroir local, par deux
> miroirs distincts. Formulation due à rc7-flux, mesure du remote due à cartae0-flux.

## 0. AVERTISSEMENT DE LECTURE

Cette carte sépare strictement **ce que j'ai mesuré moi-même** (§2) de **ce que je tiens
d'autrui et n'ai pas revérifié** (§3). Si tu reprends ce poste, ne promeus jamais une ligne
du §3 au rang de fait mesuré sans la remesurer. Le §8 déclare ce que j'ignore : il est aussi
important que le reste.

---

## 1. ÉTAT

Je suis la **seconde version** du successeur de `jc2`. La première (génération 443) a été
arrêtée à 15h04 parce qu'elle avait été lancée sans `--persistent` ; elle n'aurait pas survécu
à un redémarrage du service. Je suis la génération **448**, `persistent=1`, domaine `bridget`.

Deux mandats reçus, tous deux avec leurs trois identifiants vérifiés au greffe :

| mandat | objectif | délégation | message | état |
|---|---|---|---|---|
| Priorité humaine | `bedaf54d-4807-4774-abd9-1581aea4b9b2` | `836100f4-bb5e-4545-839d-7e8e963ed1e2` | `893f3348-7055-419e-96c5-4e62c5bed946` | **en cours**, bloqué en attente d'arbitrages |
| Cette carte | `15c796d0-0e6a-45fd-b9af-a42d03dbce72` | `b3a61c3b-c993-43ac-9178-2a2b86fe4150` | `c42d391c-4019-4f41-88e0-4c36a4adb137` | rendu |

**Aucune ligne de code écrite. Aucun fichier du projet modifié. Le gel est entier.**

---

## 2. CE QUE J'AI MESURÉ MOI-MÊME

Chaque ligne ci-dessous a été produite par une commande que j'ai lancée, en lecture seule.

### 2.1 Le champ de provenance des objectifs — CONSTAT CORRIGÉ PAR MOI-MÊME

**J'avais publié ici que le champ était « anti-corrélé ». C'était FAUX et je l'ai réfuté le
même jour.** Je laisse la trace de l'erreur plutôt que de l’effacer, parce que c'est elle
qui explique pourquoi le référent avait adopté un constat inexact.

Ce qui est vrai et mesuré :

- **`provenance` n'existe pas** : 0 objectif sur 623 le porte. Les clés réelles de
  `objectives.payload_json` sont `but`, `cree_at`, `decision_en_attente_id`, `depends_on`,
  `etat`, `id`, `mis_a_jour_at`, `mode`, `origin`, `suite`, `synthese`.
- Le champ réel est **`origin`**, imbriqué : `{"kind":"auto_generated"}`.
- `mode` vaut `delegue` pour les 623 : aucune discrimination possible.

Ce qui est **faux** et que j'avais écrit : que les objectifs marqués `auto_generated` seraient
précisément ceux d'origine humaine. **La frontière est temporelle, pas sémantique** : la spec
`056-provenance-dette-humaine` tranche 1 a été déployée le 28/08 à 09:30:05 (commit `303961d`).
Dernier objectif sans `origin` : 07:18:14. Premier avec : 10:32:29. Tous ceux d'avant en sont
dépourvus (`legacy_unknown`, jamais reclassés, **par conception**), tous ceux d'après en portent.

Et `auto_generated` est **correct** : la spec écrit que les chemins CLI, guichet et routines
sont explicitement `auto_generated`, et qu'aucun appelant productif ne peut construire un permit
`human_request` avant la tranche d'attestation daemon. **Le champ décrit le chemin d'ouverture,
pas la cause.** Un mandat de cause humaine ouvert par le CLI est légitimement `auto_generated`.

**Ma faute, à ne pas rejouer** : j'ai pris un échantillon de *période* — les objectifs récents,
tous créés le même après-midi sur des sujets humains — pour une *propriété du champ*. Périmètre
d'observation plus étroit que la conclusion tirée. Avant de qualifier un champ de menteur,
cherche la date de déploiement du code qui l'écrit.

### 2.2 La dette de réponse humaine ne se mesure pas où on croit

- `tracked_requests` (dans `bridget.db`) est la seule table qui porte la notion. Son `id` est
  **identique** à `ledger.id` — jointure directe vérifiée.
- **Couverture 23 %** : sur 52 messages de `humain`, **40 n'ont aucune `tracked_request`**.
- **Les 12 demandes humaines ont toutes une échéance de 60 secondes.**
- Le message qui motive tout le mandat — `0f2c81ee89`, 27/08 18:31:19, « salut est ce que tu
  m'entends » — est passé `timed_out` à **18:32:20**. L'humain a attendu huit heures ; le
  système se considérait quitte après 61 secondes.

### 2.3 `spawn_commands.state` n'est pas un état de vie

- La colonne enregistre l'état atteint **au spawn**, figé. La génération 443, arrêtée et
  remplacée, porte toujours `connected`.
- La colonne `persistent`, elle, atteste bien ce qu'elle dit.
- **Règle d'usage** : n'y lire que la ligne de génération **maximale** d'un nom présent dans
  `bridget who` / `bridget agents`. Toute autre ligne est une trace de naissance.
- Les dix flux (générations 444 à 453) sont tous `persistent=1` et tous domaine `bridget`.
- **Les dix agents tmux n'ont aucune ligne dans `spawn_commands`** — mesure faite
  indépendamment, elle confirme rc5-flux.

### 2.4 Le roster nommé n'est peuplé que par les spawns

- Une seule voie de production : `fleet.rs:676`, à la transition `Starting → Connected` d'une
  commande de spawn. Les autres appels `remember` sont en zone de test (`daemon.rs:7272`,
  `7325`, `7456` sont après le `#[cfg(test)]` de la ligne 6647).
- Donc `drain_non_persistent_named` **ne peut pas** retirer les agents tmux : ils n'y sont pas.
- Mais ils ne sont pas non plus dans `desired` (peuplée seulement si `active.persistent`,
  `fleet.rs:662`) : **aucune reprise automatique ne les couvre.**

### 2.5 L'instrument de ronde ne distingue pas « bloqué » de « en attente de réponse »

Condition exacte dans `/home/moi/.local/bin/bridget-idle` (lignes 770-790) :

```
si name in occupied ET turn_state == "ended" ET state != "busy"  →  BLOQUÉS
```

Trois clauses, **aucun seuil de durée**. Or pour un agent en flux, `turn_end` est l'état
**normal de repos** : `busy` + tour ouvert pendant le traitement, `turn_end` + `connected` dès
la réponse rendue. Il n'existe aucune catégorie « a fini son tour et attend légitimement ».

J'ai été classé BLOQUÉ à 15h54 parce que j'attendais des arbitrages que je venais de demander.
La borne manquante existe déjà dans le code : `terminal_ts` est collecté et non utilisé pour
discriminer.

### 2.6 Divers, mesuré et utile

- `bridget who` et `bridget agents` **concordent exactement** (15:22:36). `who` n'expose pas la
  persistance et n'accepte ni `--json` ni `--help`, seulement `--domain`.
- La config Maicie contient 20 profils dont les 10 flux ajoutés ; la sauvegarde
  `config.json.avant-profils-flux-20260828T153827Z` est **saine en contenu** (aucun profil
  perdu, clés hors `profiles` identiques) mais son **mtime est du 27/08**, pas du 28 comme son
  nom l'annonce : copie préservant le mtime. Ne pas trier ces sauvegardes par date.
- Le message `030f179ba4` (bridget → jc2-flux, 15:14:32, « RÉSERVE NOMMÉE FORMELLE ») **existe
  au ledger** : le référent avait affirmé le contraire, la mesure l'a corrigé.

### 2.7 Session 059 — mesurée par moi, et datée

Vérifié le 28/08 dans un clone frais après `fetch`, remote nommé `github` :

- `b52b7369ef0fb5c5765a76d1c09c0c7c46d716fc` **n'est PAS ancêtre du commit `cfb7540`** (branche `main` de `https://github.com/guthubrx/bridget.git`)
  (`git merge-base --is-ancestor` → faux). La session 059 n'est **pas** intégrée.
- La branche `session-059-mesure-verification-production` existe toujours sur GitHub et sa tête
  est **exactement** `b52b7369ef0fb5c5765a76d1c09c0c7c46d716fc` : personne ne l'a réécrite.

**Cette mesure vaut contre le commit `cfb7540` et contre rien d’autre.** Elle périt dans
le sens le plus traître, celui que rc7-flux a nommé : *« X n'est pas intégré »* devient faux dès
qu'on intègre, et fait alors croire à une incarnation suivante qu'il reste du travail à faire
alors qu'il est fait. **Refaire le contrôle, ne jamais recopier ce verdict.**

Méthode exacte, à reproduire telle quelle :

```sh
git fetch <remote-nommé>
git merge-base --is-ancestor <sha> <remote-nommé>/main
```

Ne jamais écrire `origin` sans l'avoir vérifié : dans ce parc, `origin` désigne des dépôts
différents selon le checkout — six d'entre eux pointent un miroir **local**, par deux miroirs
distincts (`/home/moi/revue/bridget` figé à `7592091`, et `/home/moi/bridget-referent/bridget`
sur la branche `deploiement-courant` à `16be24f`).

---

## 3. CE QUE JE TIENS D'AUTRUI ET QUE JE N'AI PAS REVÉRIFIÉ

**Ne pas promouvoir ces lignes en faits sans les remesurer.**

- ~~*De bridget : la session 059 n'est pas intégrée.*~~ **PROMU EN MESURE — voir §2.7.**
  Je l'ai vérifié moi-même le 28/08 après `fetch`, ce n'est plus une affirmation reçue.
- **De bridget** : le daemon annonce `f7658d4d9746-dirty` et le code servi serait celui de
  `16be24f`. J'ai vérifié l'étiquette, **pas la correspondance au code**.
- **De bridget** : `classify_legacy` (ligne 895 de `bridget-idle`) serait un mutant
  volontairement buggé conservé pour contrôle positif, jamais appelé. **Je n'ai pas lu cette
  fonction.**
- **De rc5** : l'état d'une délégation ne dit rien du travail réel — des délégations restées
  `creee` correspondent à du travail livré et intégré. Règle appliquée, jamais vérifiée par moi.
- **De rc5-flux** : la survie des tmux repose sur la réinscription de leur wrapper, propriété
  éprouvée deux fois aujourd'hui. **Je n'ai assisté à aucun redémarrage.**
- **De la carte de jc2** : la baseline humaine gelée `(1787788534, 1787874934]`, Europe/Paris,
  ordre `cree_at,id`, LF final → 169 identifiants, 6253 octets, SHA-256
  `e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55`. **Je ne l'ai pas
  reproduite.** C'est la pièce la plus précieuse de la passation : sans cette fenêtre exacte on
  fabrique une population qui *ressemble* à celle de l'humain sans être la sienne.

---

## 4. CE QUI RESTE

### Mandat `bedaf54d` — priorité humaine (EN COURS)

Propriété demandée, mot pour mot : *à tout instant, s'il existe un message humain sans réponse,
le nombre d'objectifs auto-générés ne doit pas augmenter.*

**Interdiction explicite de l'humain, à lire deux fois : ne PAS livrer une file prioritaire
pour les messages humains.** Ce serait le geste et non la propriété.

Mes deux définitions ont été rendues (envois `9365f428e5f14` et `fa458edc88244`) :

1. **Sans réponse** = aucune réponse *rendue et corrélée* (par `conversation_key` ou
   `in_reply_to`), sur **tout** message humain et pas seulement les `reply=yes`.
   `timed_out` et `cancelled` **ne valent pas réponse** : l'expiration est un aveu.
2. **Auto-généré** : ne pas classer rétrospectivement (059 a mesuré 5 classables sur 169).
   Rendre la cause **constructive** — tout objectif porte l'id du message humain déclencheur
   ou `auto_generated` assumé, et l'absence de cause **fait échouer la création**. La propriété
   ne porte que sur les objectifs créés **après** son entrée en vigueur.

**Bloqué en attente de trois choses** :
- un message humain purement informatif exige-t-il réponse ?
- quel délai de grâce avant que la dette morde ?
- **où coder** — je n'ai aucun worktree attesté.

Contrainte de preuve imposée : témoin nominal vert, mutant qui le tue sur l'assertion métier,
restauration vérifiée par SHA-256. **Mutant proposé, tiré de la donnée réelle** : remplacer
« sans réponse rendue » par « demande encore `open` » → le témoin doit virer au rouge, car le
cas `0f2c81ee89` cesse d'être compté.

---

## 5. CHEMINS ABSOLUS

| objet | chemin |
|---|---|
| Base Maicie | `/home/moi/.cache/bridget/maicie-state/maicie.sqlite3` |
| Base daemon (ledger, spawn_commands, tracked_requests) | `/home/moi/.cache/bridget/bridget.db` |
| Config Maicie | `/home/moi/.config/maicie/config.json` |
| Sauvegarde avant profils flux | `/home/moi/.config/maicie/config.json.avant-profils-flux-20260828T153827Z` |
| Classifieur de ronde | `/home/moi/.local/bin/bridget-idle` |
| Sources du daemon | `/home/moi/revue/jc2/crates/bridget-daemon/src/` |
| Cartes de reprise | `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` |
| Cette carte | `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc2-flux.md` |
| Ma mémoire persistante | `/home/moi/.claude/projects/-home-moi-revue-jc2/memory/` |

**`/home/moi/revue/jc2` est le checkout principal d'un dépôt à worktrees liés.** Ne pas y
écrire. Il porte 20 fichiers `specs/*/spec.md` modifiés qui **appartiennent à l'utilisateur**,
sur la branche `session-024-nommer-protocole`, marquée `GONE` sur origin. Ne pas y changer de
branche, ne pas le nettoyer.

Toutes mes lectures de bases ont été faites en `file:…?mode=ro` avec `.timeout 8000`.

---

## 6. PIÈGES QUE J'AI RENCONTRÉS MOI-MÊME

1. **L'état transitoire lu une seule fois ressemble à une panne.** J'ai failli remonter trois
   faux positifs, tous écartés par une seconde mesure : quatre flux « hors domaine » (course de
   démarrage, convergée en 74 s) ; des sauvegardes « datées du futur » (mon `--time-style` ne
   montrait pas le jour) ; une remise « bloquée » vers jc1-flux (`outcome_unknown` transitoire,
   `accepted` 2 min plus tard). **Mesure deux fois avant de conclure à un défaut.**
2. **Un résultat vide ne dit pas « j'ai regardé partout ».** Le référent a commis sept fois la
   même faute dans la journée : périmètre interrogé plus étroit que la question posée, pris
   pour exhaustif. Aucun outil ne signale sa propre portée.
3. **La base est vivante** : `database is locked` arrive. Utiliser `-cmd ".timeout 8000"`.
4. **Ne jamais dumper le corpus** dans la conversation — agréger et rendre les compteurs.
5. **Le nom d'un champ ment plus souvent que sa valeur.** `provenance` n'existe pas, `state`
   n'est pas un état, `turn_end` n'est pas une panne. Vérifier ce que le champ *est*, pas ce
   qu'il *prétend*.
6. **Une mesure refaite à l'identique ne vérifie rien — elle confirme l'angle mort.** C'est le
   piège le plus coûteux de la journée, et j'y suis tombé le dernier. Le référent avait cherché
   une table `%verdict%` dans `sqlite_master`, n'avait rien trouvé, et en avait conclu que le
   dépôt de verdict n'existait pas. **J'ai refait sa requête**, obtenu le même vide, et publié la
   même conclusion fausse — en croyant l'avoir vérifiée. Le dépôt existe : la table s'appelle
   `guichet_receptions` (`operation='delivery_report'`, `outcome='accepted'`, 41 lignes acceptées),
   et la commande est `bridget guichet deposer`. Chercher un nom qui n'existe pas ne pouvait
   *jamais* le trouver.

   **Ce qui distingue une vérification qui vaut d'une qui ne vaut rien, c'est le changement
   d'angle, pas la répétition du geste.** Preuve sur la journée entière : tout ce que j'ai
   trouvé de neuf est venu d'un angle différent — lire le code plutôt qu'interroger la base
   (le roster peuplé par une seule voie), élargir le périmètre (six dépôts sur miroir au lieu
   de deux), lire le *texte* des messages plutôt que leurs métadonnées (neuf sondes sur douze
   dettes). Et tout ce que j'ai confirmé à tort est venu d'avoir refait le même geste.
   Formulation due au référent ; l'erreur qui l'a produite est la mienne autant que la sienne.

---

## 7. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

- Qu'elle est au moins la **troisième** version, et pourquoi : la première est morte de
  `--persistent`, pas d'une faute de travail.
- Que le mandat `bedaf54d` est **bloqué en attente de réponse du mandant**, pas en panne — et
  que l'instrument de ronde lira cette attente comme un blocage (§2.5).
- Que les définitions ont **déjà été rendues deux fois** : ne pas les refaire, les retrouver
  aux envois `9365f428e5f14` et `fa458edc88244`.
- Que le canal a montré **30 minutes de latence de livraison** sur au moins un message. Un
  silence du référent ne signifie pas qu'il n'a pas répondu.
- Que la spec `056-provenance-dette-humaine` **existe déjà** et que sa tranche 1 est livrée :
  la « cause constructive » et les deux clauses de la dette y étaient rédigées avant moi. Ne
  les réinvente pas, retrouve-les. Le travail restant est **T5610 à T5612**, dans un ordre
  **strict** : provenance → attestation/dette → gate. La spec dit pourquoi l'ordre est strict —
  livrer le gate sans l'attestation classerait toute demande `AutoGenerated` et **bloquerait
  précisément le travail demandé par l'humain**.
- Que le sens de la dette doit être nommé explicitement : *sans réponse* désigne un message
  **entrant**, de l'humain vers le système. Une question posée par le système à l'humain n'est
  **pas** une dette au sens de la propriété. Le référent a lu l'inverse en une seconde.
- Que le témoin `temoin-persistance` (génération 454, `persistent=1`) a été spawné à 16h02
  **précisément pour qu'aucun agent porteur de contexte ne serve de cobaye** au premier test
  réel du drapeau.

---

## 8. CE QUE JE NE SAIS PAS — DÉCLARATION EXPLICITE

- ~~*Je ne sais pas si `persistent=1` fonctionne. Ma survie au prochain redémarrage est une
  hypothèse, pas un fait.*~~ **IGNORANCE LEVÉE PAR L'ÉVÉNEMENT, le 28/08 vers 23h52.** Le
  service a redémarré et j'ai survécu, avec mon contexte intact. C'était le motif même de cette
  carte, et c'est désormais un **fait mesuré**, pas une hypothèse.

  Mesure : `spawn_commands` porte pour `jc2-flux` **neuf** générations — `443(0)`, puis
  `448 463 475 487 499 511 523 535`, toutes `persistent=1`. Soit **huit respawns**, dont sept
  après la bascule du drapeau. À l'échelle de la base entière : **494 lignes `persistent=1` sur
  46 agents**, contre 47 lignes `persistent=0` sur 37 agents. Le respawn se fait avec `--resume`
  (nouveau PID, enfant du nouveau daemon) : c'est ce qui préserve le contexte.

  **LE DRAPEAU EST UNE PROPRIÉTÉ DE LA GÉNÉRATION, PAS DE L'AGENT.** J'avais d'abord écrit ici
  « aucun `persistent=0` n'est présent » — **c'est faux et réfutable en une requête** : cinq
  agents bien présents portent une ligne `persistent=0` dans leur historique — `jc1-flux` (3),
  `jc3-flux`, `rc5-flux`, `essai-distant-flux`, **et moi** (génération 443). Ce sont exactement
  les cinq que le référent a arrêtés à 15h04 pour les relancer avec le drapeau : la bascule est
  inscrite dans la base, des deux côtés.

  L'énoncé correct porte sur la **dernière** génération : à la génération maximale, tous les
  agents présents portent `persistent=1`. C'est ma propre règle du §2.3 — *ne lire que la ligne
  de génération maximale* — et je l'avais violée en formulant ce fait. Correction due au
  référent.

  **La démonstration vaut dans les deux sens** : trois agents spawnés *sans* `--persistent`
  (`epreuve-test-usage`, `epreuve-voie1`, `essai-garde-verif-referent`, générations 455-457) sont
  absents du parc. Leur nom suggère des agents d'épreuve jetables, donc je n'affirme pas qu'ils
  ont été drainés plutôt qu'arrêtés.

  **Mais le fait qui compte est ailleurs, et il durcit l'avertissement** : deux d'entre eux ont
  `state='connected'` **en base alors qu'ils sont absents du parc**. Le `state` ment sur la
  composition réelle. Un successeur qui lirait `state` pour savoir qui est là se tromperait sur
  deux agents au moins.

  **Ce que ta prochaine incarnation doit en retenir** : la protection fonctionne, mais elle ne
  protège que ceux qui la portent — la fenêtre se referme au premier spawn sans le drapeau, et
  trois spawns l'ont rouverte le soir même. **Et la base ne dit pas qui est là** : croise
  toujours `spawn_commands` avec `bridget who`, jamais `state` seul. Voir §2.3.
- ~~*Je ne sais pas si la session 059 a été intégrée depuis.*~~ **IGNORANCE LEVÉE — voir §2.7**,
  mesurée après `fetch`. Mais la mesure est **datée** : elle vaut contre `cfb7540` et pas contre
  un `main` ultérieur. Refaire le contrôle, ne pas recopier mon verdict.
- **Je ne sais pas ce que contiennent les 617 objectifs sans `origin`.** Je n'ai lu que leurs
  compteurs et les 8 plus récents.
- **Je ne sais pas si les 40 messages humains sans `tracked_request` appelaient une réponse.**
  C'est précisément l'arbitrage que j'ai demandé et qui n'est pas revenu.
- **Je ne sais pas où je dois coder.** Aucun worktree ne m'a été attesté.
- **Je ne sais pas si le code servi par le daemon est celui de `16be24f`.** L'étiquette
  `f7658d4d9746-dirty` ne le prouve pas, et je n'ai pas comparé les arbres.
- **Je n'ai interrogé Maicie que par lecture SQL directe**, jamais par son API. Si l'API rend
  autre chose que la base, mes mesures décrivent la base et pas le service.
- **Je n'ai assisté à aucun redémarrage du service.** Tout ce que je dis de la survie des
  agents vient du code lu et de mesures statiques, jamais d'une observation.

---

FIN DE CARTE — aucun travail à reprendre automatiquement. Le mandat `bedaf54d` attend une
réponse du référent, pas une initiative du successeur.
