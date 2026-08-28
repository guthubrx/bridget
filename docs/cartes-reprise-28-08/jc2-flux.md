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

### 2.1 Le champ de provenance des objectifs est anti-corrélé

- **`provenance` n'existe pas** : 0 objectif sur 623 le porte. Les clés réelles de
  `objectives.payload_json` sont `but`, `cree_at`, `decision_en_attente_id`, `depends_on`,
  `etat`, `id`, `mis_a_jour_at`, `mode`, `origin`, `suite`, `synthese`.
- Le champ réel est **`origin`**, imbriqué : `{"kind":"auto_generated"}`. Porté par **6**
  objectifs (rc5 en comptait 2 plus tôt dans la journée).
- **Les 6 sont exactement les objectifs d'origine humaine** — dont mon propre mandat
  `bedaf54d`, étiqueté `auto_generated` alors qu'il porte un constat humain.
- `mode` vaut `delegue` pour les 623 : aucune discrimination possible.

Conséquence : toute règle écrite sur `origin.kind = 'auto_generated'` **compte à l'envers**.

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

---

## 3. CE QUE JE TIENS D'AUTRUI ET QUE JE N'AI PAS REVÉRIFIÉ

**Ne pas promouvoir ces lignes en faits sans les remesurer.**

- **De bridget** : la session 059 n'est pas intégrée ; `b52b7369ef0fb5…` n'est pas ancêtre de
  `origin/main` ; la tête jugée est inchangée. **Je n'ai fait aucun `fetch` ni aucune
  comparaison Git moi-même.**
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
- Que `origin` est anti-corrélé : bâtir la propriété dessus produirait un résultat
  **vérifiable et faux**, ce qui est pire que pas de propriété du tout.
- Que le témoin `temoin-persistance` (génération 454, `persistent=1`) a été spawné à 16h02
  **précisément pour qu'aucun agent porteur de contexte ne serve de cobaye** au premier test
  réel du drapeau.

---

## 8. CE QUE JE NE SAIS PAS — DÉCLARATION EXPLICITE

- **Je ne sais pas si `persistent=1` fonctionne.** Le drapeau n'a jamais été éprouvé. Ma survie
  au prochain redémarrage est une hypothèse, pas un fait. C'est le motif même de cette carte.
- **Je ne sais pas si la session 059 a été intégrée depuis.** Je n'ai pas fait de `fetch`.
  Vérifier, ne pas déduire — c'est la consigne de mon prédécesseur et je la relaie intacte.
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
