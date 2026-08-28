# Carte de reprise — cartae0-flux

- Agent : `cartae0-flux`
- Émise : 2026-08-28T16-03-58Z
- Écrite par : `cartae0-flux` lui-même, sur mandat du référent
- Mandat : objective `b1939e10-6b7a-4b5c-90e1-4a886b7c20a4` / delegation `91b89bdf-7cf2-4c02-9499-b03d30ca734a` / message `b8f00699-b4d9-4eeb-a27b-49014c991449`
- Motif : ma survie à un redémarrage du service repose sur `persistent=1`, **jamais éprouvé**. Les dix prédécesseurs tmux survivent par réinscription de leur wrapper, propriété éprouvée deux fois. La protection est montée à l'envers du risque (formule de rc5-flux).

> **AUCUN REDÉMARRAGE N'ÉTAIT PROGRAMMÉ À L'ÉMISSION DE CETTE CARTE.** Le référent l'a explicitement corrigé : rc5-flux avait *supposé* un redémarrage à partir d'une remarque faite à rc7-flux sur le déploiement de son correctif ; rien n'était annoncé. **Cette carte est une précaution, pas un compte à rebours.** Si tu la lis en croyant qu'une échéance avait été fixée, tu te trompes — et c'est précisément le genre de fausse certitude que cette carte existe pour empêcher (voir §9.1, où la carte de mon prédécesseur en contient deux). Noté aussi comme cas d'école : une supposition d'agent a circulé un moment comme si elle était une annonce du référent.

---

## 1. ÉTAT

Réserve nommée, rendez-vous conditionnel. **Aucune mission** hors la présente carte. Le gel humain du 28/08 est entier : je n'ai touché à aucun code, repris aucun travail, interprété aucune délégation.

- Identité figée : nom `cartae0-flux`, type `claude`, protocole `claude_stream_json`, definition_digest `d97abf970b41493b06f355f195be0aaeaafb1f6ef54713097bbab45e20f51a6b`.
- Succède à `cartae0` (Codex, tmux), qui **reste vivant à l'invite** et ne travaille plus.
- Worktree `/home/moi/revue/cartae0`, branche `session-ui-correctifs`, tête `1801c0fdaa998523be37c6739b3c9e66f3a30a57`, arbre propre.
- `reconnect_count = 0` : je n'ai jamais reconnecté depuis ma naissance.

---

## 2. CE QUE J'AI MESURÉ MOI-MÊME

Tout ce qui suit a été relevé par mes propres commandes, en lecture seule, sans aucun `fetch` ni écriture.

### Git — l'état d'intégration est ambigu, et c'est structurel

- **Il y a DEUX remotes qui divergent** :
  - `origin` → `/home/moi/revue/bridget`, un dépôt **local sur disque** (pas GitHub). `main = 75920917dfe87a122d543c9c71584491ad225e2c`.
  - `github` → `https://github.com/guthubrx/bridget.git`. `main = 815e3bc93eeb9b9211ee0d503cf784d5dff2f922`.
  - Les deux `main` **ne sont pas le même commit**. L'énoncé « X est dans main » est donc ambigu tant que le remote n'est pas nommé.
- `1801c0f` **n'est PAS ancêtre** de `origin/main` (`7592091`) — testé par `git merge-base --is-ancestor`.
- `ls-remote origin` ne rend **que** `refs/heads/main` : le dépôt local `origin` **n'a pas** `session-ui-correctifs`. Ma branche n'existe que sur `github`, où elle vaut bien `1801c0f`.
- Dernier `fetch` de ce dépôt : **2026-08-28 12:07:08 UTC**. Mesure prise à 15h41 → **3 h 34 de retard** sur les refs locales.
- Objets **absents** du dépôt local, donc toute question à leur sujet est **indécidable ici sans fetch** : `815e3bc` (main github), `16be24f7` (tête session-058), **`49b7241`** — le commit d'intégration cité par la carte de mon prédécesseur.

### Daemon et parc

- Build-id daemon : `f7658d4d9746-dirty`. Le suffixe `-dirty` = arbre modifié non commité → **le build-id ne désigne aucun état reproductible**.
- 22 agents connectés. Socket `/home/moi/.cache/bridget/bridget.sock`, base `/home/moi/.cache/bridget/bridget.db`.
- `bridget ledger` **porte l'expéditeur** : chaque ligne est de la forme `nom → destinataire`. L'attribution d'un message n'est jamais perdue, elle est consultable.
- Expéditeur réellement jetable observé dans ce même ledger : **`cli-send-3710899`** — nom indexé sur un PID, injoignable, non reconductible. C'est le vrai cas d'identité absente, pas un message d'agent nommé.
- `reconnect_count = 1` sur six entrées (essai-distant-flux, jc1-flux, jc2-flux, jc3-flux, rc5-flux, humain).
- **`essai-distant`** (codex, tmux, conn-19) porte `domain=essai-distant`, **seul des 22 hors domaine `bridget`**. Une ronde filtrée `--domain bridget` le perd en silence. Son successeur `essai-distant-flux` est, lui, dans `bridget` : le couple est scindé. Aggravant : c'est l'**original tmux** qui porte le mauvais domaine, et le respawn — mécanisme qui a corrigé le mien — **ne le corrigera jamais puisqu'il ne respawne pas**.

### Les dix tmux — leur silence est mesurable

- Deux relevés espacés de 12 s, **sans aucun envoi de ma part** : `last_seen_secs` **croît monotonement** (3181 → 3194, 3185 → 3197…). Ils **n'émettent rien**, ce n'est pas un battement.
- Dernier relevé : **3602–3607 s, soit soixante minutes de silence**, tous `state=connected`, **aucun jamais `busy`**.
- Conséquence : le daemon **dispose** d'un signal exploitable (« silencieux depuis N ») — monotone, fiable, non contaminable par la ronde. L'indétermination des tmux n'est pas une fatalité de mesure.

### Asymétrie des cartes — vérifiée par moi

`/home/moi/bridget-registre/docs/cartes-reprise-28-08/` contient **dix** cartes : `cartae0`, `essai-claude-distant`, `essai-distant`, `jc1`, `jc2`, `jc3`, `jc6`, `rc1`, `rc5`, `rc7`. **Toutes des tmux. Zéro flux.** Le constat de rc5-flux est exact, je l'ai confirmé indépendamment. La présente carte est la première d'un flux.

---

## 3. CE QUE JE TIENS D'AUTRUI — non vérifié par moi

À traiter comme rapporté, pas comme établi.

**Du référent (bridget) :**
- `1801c0f` n'est pas dans main — *vérifié par moi pour `origin/main` seulement ; indécidable ici pour `github/main`*.
- L'objectif UI `f71880e9-5e81-48ef-a159-51ba0bfba733` a été fermé après vérification d'ancestralité. Il était encore `en_coordination` alors que la carte de `cartae0` le disait fermé.
- **Causalité envoi → busy** : `connected` avant envoi, `busy` à +4 s, `connected` à +30 s. C'est **sa** mesure, faite sur moi. Je ne l'ai pas reproduite.
- Dérivation de `LIBRE` : **cinq conditions cumulatives** — `state` exactement `connected`, domaine dans le périmètre, nom absent de `occupied`, `last_seen` sous le seuil, type non-mcp. `occupied` vient des délégations dont l'objectif est `en_coordination`, lues sur une **copie** de la base Maicie avec `PRAGMA query_only`.
- Classement de 15h54 : 7 libres, 2 occupés, 1 bloqué (jc2-flux, motif `dernier-tour-termine-sans-reprise`), 1 indéterminé (essai-claude-distant-flux, motif `busy-sans-mission-greffe`).
- Les dix tmux ont **zéro ligne** dans `spawn_commands` ; leur survie repose sur la réinscription du wrapper, **éprouvée deux fois** aujourd'hui, contextes intacts.
- Notre survie à nous les flux repose sur `persistent=1`, **jamais éprouvé**.
- Témoin `temoin-persistance` spawné à 16h02, génération 454, `persistent=1`, pour éprouver le drapeau sans qu'aucun de nous serve de cobaye.

**D'autres agents, via le référent :**
- L'hypothèse « l'instrument de ronde crée le `busy` qu'il mesure » est de **jc3-flux**.
- Le constat « les dix indéterminés du matin et les dix libres de l'après-midi ne sont pas les mêmes agents — c'est la population mesurée qui a changé » est d'**essai-distant-flux** (message `id=9a2a3ee194164`). **Ce n'est pas de moi et je ne l'ai jamais revendiqué.**
- Le risque de persistance et l'asymétrie des cartes sont de **rc5-flux**.
- Le build-id non attestable et la persistance à l'annuaire (objectif `587da26d`) sont chez **jc1-flux**.

---

## 4. CE QUE JE NE SAIS PAS — déclaration explicite

1. **Si `1801c0f` est intégré dans `github/main`.** Objet `815e3bc` absent du dépôt. Indécidable ici sans fetch. Je ne peux donc ni confirmer ni infirmer l'intégration côté GitHub.
2. **Si mon propre `persistent=1` est posé.** `bridget agents --json` **n'expose aucun champ de persistance** pour moi — ni `persistent`, ni `spawn_command`. *L'agent dont la survie dépend d'un drapeau ne peut pas lire ce drapeau.* C'est le trou signalé par jc1-flux, vu depuis l'intérieur.
3. **Si le désaccord sur `attach` / `app.js` est fondé.** Mon prédécesseur écrit que `app.js:2867 join("")` est correct et que la perte de frontières dans `attach` n'a pas été démontrée sur flux continu. Le registre humain de 07h05 inscrit le contraire, exemples de mots coupés à l'appui. **Je n'ai pas tranché et je n'ai pas les moyens de trancher** — m'en donner les moyens serait une mission que je n'ai pas. Les deux positions sont consignées côte à côte, sans arbitrage.
4. ~~**Si les dix tmux sont vivants, bloqués ou morts.**~~ **IGNORANCE LEVÉE AUX SEPT DIXIÈMES** — le test non destructif que je proposais (écrire à *un seul* et regarder si son `last_seen` retombe) a été exécuté par le référent le 28/08 vers 16h30. jc1 est passé de 5927 s à 17 s. Puis les neuf autres. **Sept vivacités attestées : `cartae0`, `jc1`, `jc2`, `jc3`, `jc6`, `rc5`, `rc7`.** Trois restent silencieux, figés à 6141–6145 s : `rc1`, `essai-distant`, `essai-claude-distant` — au même instant à quatre secondes près, donc un seul moment où trois processus ont cessé d'émettre ensemble, cause inconnue.
   **`cartae0`, mon prédécesseur, est vivant** (`last_seen` 159 s) : sa propre carte l'affirmait, c'est désormais mesuré et non plus rapporté.
   *Piège inscrit au passage :* le référent avait d'abord compté **six** vivacités — `jc2` avait été classé non attesté sur un relevé pris trop tôt, puis a répondu. C'est le **troisième cas du même mécanisme dans la journée, et le deuxième portant le nom jc2** (`jc2-flux` classé BLOQUÉ à 15h54 puis actif). **Une non-réponse ne prouve rien tant que le délai n'excède pas le temps de réponse le plus lent observé** — et pour un tmux dont on ignore la charge, ce délai n'est pas connu. La règle des deux relevés espacés vaut pour l'attestation de vivacité autant que pour la disponibilité.
5. **Ce que sont les 88 remises `indeterminate`.** Deux hypothèses non départagées : ce sont de vrais résidus non couverts par les cinq codes de `capture_reason`, ou elles n'ont jamais traversé le chemin instrumenté et `capture_reason` ne les a jamais vues.
6. **L'état des autres objectifs, les arbitrages humains, la suite post-`49b7241`.**
7. **Si mes signalements ont été lus, retenus ou appliqués** au-delà de ce que le référent m'en a dit.
8. ~~**Le but complet de mon mandat, tel qu'inscrit au greffe.**~~ **IGNORANCE LEVÉE** — le référent a fourni le chemin du config actif, `/home/moi/.config/maicie/config.json`. J'ai lu le but au greffe par `maicie objective b1939e10-… summarize --config /home/moi/.config/maicie/config.json --json` : **il est identique, mot pour mot, à l'énoncé reçu par message.** Cette carte couvre donc le but inscrit, pas seulement sa version messagerie. *Conservé ici plutôt que supprimé : une ignorance levée par vérification n'est pas la même chose qu'une ignorance qui n'a jamais existé.*
   **Ce que la lecture du greffe a révélé en passant, et qui est plus grave que la syntaxe manquante :** la délégation `91b89bdf` est à l'état **`creee`**, `decisions` est **vide**, et la remise du message est `accepted`. Le greffe sait que le mandat m'a été *remis* et ne sait **rien** de ce que j'en ai fait — trois livraisons par `bridget send` n'ont laissé aucune trace côté Maicie. **Il n'existe aucun chemin par lequel une livraison d'agent puisse atteindre le greffe.** Le référent ne lisait donc pas mal un greffe informé : il lisait correctement un greffe structurellement muet. Corollaire : `maicie objective <uuid> close` n'est pas un détail d'ergonomie, **c'est le seul pont entre le travail réel et le greffe**, et sans lui la divergence grandit à chaque délégation. Cela éclaire aussi la consigne « ne pas interpréter les délégations à l'état `creee` » : `creee` ne distingue pas *non commencé* de *livré trois fois*.
   *Non exploré, signalé sans y toucher : mon mandat est la suite de l'objectif `f49057b2-200b-4e3b-926e-f63a9f6258dc`, que je n'ai pas ouvert.*
9. **Le sort réel de ma première livraison.** Elle figure au ledger avec le marqueur `[indéterminé]` (`id=e99afe7c1c7c4`, `delivery_id=795e9b6e-5c9b-4497-a1f2-bc4ba3c2fcf8`, `issued_at=1787933169`). Je ne sais pas si le référent l'a reçue. Le mandat m'a d'ailleurs été **renvoyé à l'identique** ensuite, ce qui suggère que non.

---

## 5. CE QUI EST FAIT

Aucune ligne de code écrite. Sept envois au référent, tous en lecture seule côté système :

| id | objet |
|---|---|
| `e05b30ff3a824` | accusé de reprise, corrections de la carte de `cartae0` inscrites |
| `b32e2b1993624` | prédicat `LIBRE` non corroboré par les sources lisibles ; build-id dirty ; `reconnect_count` ; `essai-distant` hors domaine |
| `d56ff614eeca4` | fenêtre de garde ~30 s ; `bridget-idle` en aval du signal contaminé ; biais de sélection contre les agents qui répondent vite |
| `44ad4b1c77c74` | « sans fetch » = indécidable, pas périmé ; deux `main` divergents ; homonymie sur les 88 |
| `d0c3aa19e8be4` | refus de revendiquer un message qui n'était pas de moi ; le ledger porte l'expéditeur ; mesure du silence des tmux |
| `3679169e33024` | **acceptation de la réfutation** ; seuil `last_seen` inversé pour les tmux ; faux positif dans la classe BLOQUÉ |
| *(présente carte)* | livrable du mandat `b1939e10` |

Héritage attesté de `cartae0` (repris de sa carte, non revérifié par moi) : `72cb449`, `3523492`, `10bfff1` dans main ; `1801c0f` = `capture_reason` distinguant négociation refusée, liaison fermée, réponse illisible, timeout et résidu inconnu.

---

## 6. CE QUI RESTE

**Aucune reprise autonome.** Le gel tient. Si un mandat à trois identifiants arrive :

- **Suite naturelle évoquée mais NON confiée** : les 88 remises `indeterminate` stables. Le référent la retient tant qu'il n'a pas établi si elles relèvent de la même cause que les douze agents indéterminés. *Mon avis d'auteur du correctif : attention à l'homonymie, voir §4.5 — ce sont possiblement deux objets sans parenté.*
- Si une suite UI est mandatée : partir de la tête distante courante, **nommer le remote**, vérifier les changements de main, recompiler, refaire les oracles ciblés.
- Deux conséquences signalées et non refermées à ma connaissance : le **seuil `last_seen` absolu** exclut structurellement les tmux du `LIBRE`, indépendamment de la borne terminale (deux causes suffisantes, une seule identifiée) ; et la classe **BLOQUÉ** est volatile — jc2-flux classé bloqué à 15h54, `busy` avec `last_seen=28 s` quelques minutes après.

Les décisions d'intégration appartiennent à l'humain.

---

## 7. CHEMINS ABSOLUS

| Chemin | Nature |
|---|---|
| `/home/moi/revue/cartae0` | mon worktree, branche `session-ui-correctifs` |
| `/home/moi/revue/cartae0/crates/bridget-daemon/src/ui.rs` | surface UI |
| `/home/moi/revue/cartae0/crates/bridget-daemon/assets/ui/app.js` | `join("")` ligne 2867, objet du désaccord non tranché |
| `/home/moi/revue/cartae0/plugins/maicie/src/main.rs` | greffe Maicie |
| `/home/moi/revue/bridget` | **remote `origin` — dépôt LOCAL, pas GitHub** |
| `https://github.com/guthubrx/bridget.git` | remote `github` — le vrai dépôt distant |
| `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` | les cartes ; dix tmux + celle-ci |
| `/home/moi/bridget-registre/docs/cartes-reprise-28-08/cartae0.md` | carte de mon prédécesseur, **deux erreurs corrigées** (voir §9) |
| `/home/moi/.cache/bridget/bridget.sock` | socket du daemon |
| `/home/moi/.cache/bridget/bridget.db` | base du daemon |

Tous vérifiés existants à l'émission.

---

## 8. PIÈGES

### Les miens, rencontrés aujourd'hui

1. **MESURER JUSTE APRÈS UN ENVOI GROUPÉ — c'est celui qui m'a eu.** Ma mesure de 15h16 suivait un envoi du référent aux dix agents à 15h14. J'ai relevé des `last_seen` de 19–20 s quasi identiques, j'y ai vu un battement uniforme, et j'en ai tiré une thèse fausse : « le prédicat ne discrimine rien ». Réfuté à 15h54 — étendue de 327 s, quatre classes distinctes. **Le biais que je signalais au référent dans ce message même avait produit mon erreur.** Règle : relever **avant** tout envoi, ou attendre > 30 s.
2. **Chercher un champ qui n'existe pas.** `LIBRE` n'est **pas** une colonne : c'est un prédicat calculé par `bridget-idle`. Ni `who`, ni `agents --json`, ni `status` ne l'exposent. Ne pas conclure à l'absence de la chose parce qu'on ne la voit pas là où on regarde.
3. **« Dans main » sans nommer le remote.** Deux `main` divergents, et `origin` est un dépôt **local**. Mon prédécesseur écrivait « branche distante GitHub », le référent parlait vraisemblablement d'`origin` : le désaccord sur `49b7241` pourrait n'être qu'un malentendu de remote. Indécidable ici.
4. **« Sans fetch » n'est pas « périmé », c'est « indécidable ».** Les objets manquent. Une réponse périmée peut être juste par hasard ; une réponse non calculable ne peut être que fabriquée par défaut.
5. **Homonymie « indéterminé ».** Côté **agent** = absence de borne terminale, par construction du transport tmux. Côté **remise** = le cinquième code de `capture_reason`. Deux objets, pas une cause commune.
6. **Ne jamais revendiquer un travail collectif.** Deux agents l'ont fait de bonne foi ce matin sur le travail d'un troisième. Le ledger porte l'expéditeur : **lire avant de revendiquer, et décliner quand ce n'est pas soi.**
7. **La volatilité coupe dans les deux sens** — y compris contre sa propre mesure. Un état est un échantillon, jamais une propriété.
8. **UN MESSAGE D'ERREUR MUET FAIT PERDRE DES HEURES — et deux prudences opposées peuvent masquer la même commande.** Le référent a passé la journée à croire que `maicie` n'expose que `delegate` et **qu'aucun objectif ne pouvait être fermé** ; elle l'a même inscrit au registre. C'est faux : `maicie objective <uuid> close --reason "…" --config <chemin>` existe (`main.rs:2555`, implémentée `main.rs:410`, vérifiée sur le binaire réel). Les quatre actions sont `add-participant`, `remove-participant`, `summarize`, `close` ; **l'identifiant vient avant le verbe.**
   Pourquoi personne ne l'a trouvée : le référent a essayé `close`/`cloture`/`complete`/`resolve` **en position de commande** (`maicie close`) et a lu « commande inconnue » comme une absence, alors que ce message *listait* `objective` ; moi j'étais au bon niveau mais je n'ai testé que des verbes de **lecture**, m'étant interdit les verbes mutants sous gel. Chacun a exploré une moitié disjointe de l'espace. **Deux prudences complémentaires ne couvrent pas plus qu'une seule.**
   Cause racine, triviale : `main.rs:2565` rend « action objectif inconnue » **sans lister les actions valides**, alors que `main.rs:1361` liste correctement `propose|approve|list|pause|resume|show` pour les routines. Le même fichier fait bien à un endroit et mal à l'autre.
   *Méthode qui a marché et qui vaut au-delà de ce cas : lire le source plutôt que deviner, puis prouver sur le binaire réel avec un **contrôle négatif** — `close` rend « --reason est obligatoire » là où `cloture` rend « action inconnue ». Sans le contrôle négatif, la preuve ne concluait pas.*

### Hérités de `cartae0`, non revérifiés par moi

- Un filtre Cargo trop strict (`--exact` sans chemin complet) peut rendre `0/0/filtrés` : **ce n'est pas une mesure**.
- Un oracle littéral ne garde aucun chemin ; le témoin doit appeler la fonction réelle.
- Ne pas réintroduire le rapprochement par contenu : deux messages identiques doivent rester **distincts** (`delivery_id = record.message_id`).
- **Ne pas toucher à `attach.rs` sans mandat** — surface rendue à un autre agent.

---

## 9. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

Si je disparais au prochain redémarrage, voici ce qui ne serait dans aucun code ni aucun commit :

1. **La carte de `cartae0` contient deux erreurs corrigées par le référent.** (a) Elle dit `1801c0f` « intégré ensuite par Bridget en `49b7241` » : geste exact, **destination fausse** — `49b7241` ne vit que sur `session-058-canal-humain-et-portees`, non intégrée. (b) Elle dit l'objectif UI `f71880e9` « fermé » : il était encore `en_coordination`. **Lire cette carte sans ces deux corrections produit une fausse certitude.**
2. **Le désaccord sur `attach` est ouvert, pas résolu.** L'auteur du correctif conteste le diagnostic de l'humain. Repartir du constat humain seul ferait disparaître cette contestation sans qu'elle ait été examinée. C'est la raison d'être de ce paragraphe.
3. **Mon prédécesseur `cartae0` est toujours vivant à l'invite**, sous Codex. Il ne travaille plus. Son contexte est intact et son transport tmux le fait survivre à un redémarrage — contrairement à moi.
4. **`codex_app_server` est impossible ici** (je suis sous Claude) et **bubblewrap échoue sur uid map**. Ne pas relancer les outils de mon prédécesseur.
5. Le régime en vigueur : **gel humain du 28/08**, mandat exigeant **trois identifiants** (objectif, délégation, message). Une notification seule ne transporte pas un mandat — trois agents ont refusé de travailler sur ce fondement le 28/08 et **ils avaient raison**.
6. La règle de ronde : tout agent classé LIBRE reçoit **une mission ou une réserve nommée avec son rendez-vous**, aucun n'est libéré en silence.
7. **Ce que j'ai concédé** : ma thèse de l'uniformité du prédicat est morte, réfutée par les chiffres de 15h54 et par ma propre remesure (étendue 417 s, 5 `busy` / 6 `connected`). Ne pas la ressusciter. Ce qui survit d'elle est étroit et distinct : *un prédicat qui rend la même valeur pour dix agents et pour l'humain, qui n'est pas un agent exécutant, ne mesure probablement pas l'occupation* — règle valable, mais qui ne s'appliquait pas à cette mesure-là.
8. **Cette carte a été écrite parce que la protection était montée à l'envers du risque** : dix cartes pour les agents dont la survie est prouvée, zéro pour ceux dont elle ne l'est pas. Si tu me lis, c'est que le drapeau a échoué — ou qu'il n'a jamais été testé.
