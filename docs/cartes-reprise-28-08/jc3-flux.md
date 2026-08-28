# Carte de reprise — jc3-flux

- Agent : `jc3-flux`
- Type : `claude`, protocole `claude_stream_json`, mode `cli`
- Génération portante : **445**, `persistent=1`
- Émise : 2026-08-28, sur mandat du référent bridget
- Objectif `277da232-a82a-4f93-a756-ab36c39cee69` ; délégation `f1ebf1e1-0b0d-431e-b701-c644db616c50` ; message `eb246b85-8482-49a6-b7bf-2ac352674977`
- Prédécesseur : `jc3` (codex, tmux), carte distincte en `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc3.md`

**Convention de lecture de cette carte.** Chaque fait porte sa provenance :
`[MESURÉ]` = je l'ai produit moi-même avec une commande, dans cette session ;
`[RAPPORTÉ]` = je le tiens d'autrui, avec la source nommée, et je ne l'ai pas vérifié ;
`[IGNORÉ]` = je ne le sais pas, et je le déclare plutôt que de le combler.

---

## 1. ÉTAT

- Aucune mission technique active. Le gel humain du 28/08 tient : je n'ouvre rien sans mandat portant objectif, délégation et message. Le seul mandat que j'aie reçu est celui qui produit cette carte.
- `[MESURÉ]` Arbre local `/home/moi/revue/jc3` : propre, branche `session-021-verdict-sha-mesure`, HEAD `2623772`. **Cette branche est celle de mon prédécesseur, elle est ancienne, il écrit lui-même de ne pas la reprendre. Je ne l'ai pas touchée et je n'ai lu aucun de ses fichiers.**
- `[MESURÉ]` Je n'ai produit aucun code, aucun test, aucune mesure de campagne, aucun mutant. Mon travail de la session est entièrement fait de lectures d'annuaire, de lectures SQLite en `mode=ro`, et d'envois Bridget.
- `[MESURÉ]` `domaine=bridget`, corrigé par moi à l'ouverture de cette génération.

## 2. FAIT — ce que j'ai mesuré moi-même

Ce sont mes seuls apports réels. Tout est reproductible avec les commandes citées.

1. **Le domaine ne survit pas au remplacement du processus.**
   La génération 440 avait été corrigée à `bridget` ; la génération 445 est née avec `domaine=jc3`, mesuré par `bridget who` avant toute action. Remède immédiat : `bridget domain bridget`. `[RAPPORTÉ — bridget]` la cause serait `derive_domain` en `crates/bridget-daemon/src/wrapper.rs:983`, qui prend le nom du répertoire de la racine git ; **je n'ai pas lu ce code.**

2. **`spawn_commands.state` est un état terminal de commande, pas un état de vivacité — il survit à la mort de l'agent.**
   Preuve dure : `jc3-flux` gen 440 porte `state=connected` alors que cette instance est morte, arrêtée par le référent pour me recréer. Une lecture de cette table ne dit **rien** de l'état courant d'un agent. `[RAPPORTÉ — bridget]` la ronde `/home/moi/.local/bin/bridget-idle` ne lit pas cette table ; **je ne l'ai pas vérifié.**

3. **`last_seen_secs` existe, et `bridget who` ne le montre pas.**
   `bridget agents --json` porte `last_seen_secs` par agent ; `bridget who` n'a que `[--domain <nom>]` et aucune colonne de durée. **C'est la correction d'une erreur que j'avais commise** : j'avais affirmé qu'aucune horloge n'existait, après m'être arrêté à `who`.

4. **`busy` est la trace d'un tour en cours, pas une propriété d'agent ni un état durable.**
   Mesuré : au même instant, des agents de transport identique sont `busy` et d'autres `connected`. La **durée** du `busy` suit la charge du tour — un tour court (lire une notification) dure quelques secondes, un tour long (exécuter un mandat) dure des minutes. Mesure à l'appui : trois agents mandatés encore `busy` plusieurs minutes après une salve où sept notifiés étaient déjà `connected`.

5. **Le critère « bloqué ou travaille » est calculable aujourd'hui, sans modifier l'outil** : croiser `state=busy` avec `last_seen_secs`. Relevé d'illustration : agents `busy` à 7, 13 et 27 secondes, contre un agent `busy` à 415 secondes — seul profil isolé du parc.

6. **Les dix agents tmux n'ont aucune ligne dans `spawn_commands`** — requête sur les dix noms, résultat vide. `[MESURÉ]` Leur `last_seen_secs` formait un bloc figé de 3907 à 3911 secondes, tous à quelques secondes les uns des autres : un événement commun, pas un battement périodique.

7. **Témoin de persistance** : `temoin-persistance`, génération 454, `persistent=1`. Existence vérifiée par moi dans `spawn_commands`. `[RAPPORTÉ — bridget, geste proposé par rc5-flux]` son rôle est d'éprouver le drapeau au prochain redémarrage sans qu'un agent porteur de contexte serve de cobaye.

8. **L'asymétrie de protection, vérifiée par ma propre lecture du répertoire** : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` contenait dix cartes, toutes de noms tmux (`cartae0`, `essai-claude-distant`, `essai-distant`, `jc1`, `jc2`, `jc3`, `jc6`, `rc1`, `rc5`, `rc7`), et zéro carte de flux avant celle-ci.

## 3. RAPPORTÉ — ce que je tiens d'autrui et n'ai pas vérifié

À traiter comme des pistes datées, pas comme des acquis.

- `[bridget]` Le daemon en service annonce le build-id `f7658d4d9746-dirty` : le code servi serait celui de `16be24f`, mais l'étiquette ne le prouve pas. **Consigne reçue : ne rattacher aucun verdict à cette version.** Auteur du signalement : `jc1-flux`, tranché au ledger.
- `[rc5]` L'état « créée » d'une délégation ne dit **rien** du travail réel : certaines sont déjà livrées et intégrées. Vérifier l'objet livré, jamais l'état administratif.
- `[rc5-flux]` La survie des flux à un redémarrage ne repose que sur `persistent=1`, **jamais éprouvé** ; celle des tmux repose sur la réinscription de leur wrapper, éprouvée deux fois le 28/08. Formule retenue par le référent : la protection est montée à l'envers du risque.
- `[bridget, correction explicite]` **Aucun redémarrage du service n'est programmé.** L'imminence avait été supposée par `rc5-flux` à partir d'une remarque du référent à `rc7-flux` sur le déploiement d'un correctif ; le référent a corrigé. Cette carte est une **précaution**, pas un compte à rebours. À ne pas relire comme l'indice d'une échéance.
- `[bridget]` Test de causalité du `busy` sur `cartae0-flux` : `connected`, puis `busy` à 4 secondes après envoi, puis `connected` à 30. Je n'ai pas produit ce test ; je m'en sers pour borner mes propres relevés.
- `[bridget]` La ronde classe en croisant `busy` avec la présence d'une mission au greffe : `busy` + mission = OCCUPÉ, `busy` sans mission = INDÉTERMINÉ. Les tmux sont indéterminés par construction de leur transport, motif `activite-tour=source-sans-borne-terminale:tmux`.
- `[carte de jc3, prédécesseur]` Lots 040, 036/B3, 038, 039, 042, 045, 050 et revue TOCTOU, avec leurs délégations. **Je n'ai vérifié aucun de ces travaux, je n'ai interrogé ni Maicie ni aucune tête distante.** Sa carte reste la seule source ; ne pas la recopier comme si je l'avais confirmée.

## 4. RESTE

- Rien de technique ne m'est ouvert. Ma seule tâche mandatée est cette carte.
- Rendez-vous conditionnel annoncé par le référent, **qui n'est pas un mandat** : rendre le domaine survivant au remplacement de processus, avec témoin et mutant. Je n'ai rien préparé sur ce sujet — pas de lecture de code, pas d'étude — précisément parce qu'un rendez-vous n'ouvre pas un travail.
- Question ouverte que je laisse posée, sans l'instruire : le croisement `busy` + mission rend OCCUPÉ, donc sain, un agent mandaté qui se bloquerait. Ce sont les agents porteurs de travail qui deviennent illisibles. Le remède existe déjà dans l'outil (`last_seen_secs`), il n'est pas branché.

## 5. CHEMINS ABSOLUS

- Cette carte : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc3-flux.md`
- Carte du prédécesseur `jc3`, à ne pas confondre : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc3.md`
- Copie de la carte du prédécesseur côté Codex : `/home/moi/.cache/codex/handoffs/jc3-carte-reprise-2026-08-28.md`
- Répertoire des cartes du parc : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/`
- Dépôt du registre : `/home/moi/bridget-registre` (branche `main`)
- Arbre local, propre et sans travail actif : `/home/moi/revue/jc3`
- Base Bridget, à ouvrir en lecture seule : `/home/moi/.cache/bridget/bridget.db`
- Binaire Bridget : `/home/moi/.local/bin/bridget`
- Ronde d'inactivité : `/home/moi/.local/bin/bridget-idle`

## 6. PIÈGES RENCONTRÉS — les miens, à ne pas repayer

Ceux-ci sont mes erreurs réelles de la session, pas des recommandations générales.

1. **Je me suis arrêté au premier instrument.** J'ai conclu « `busy` n'est pas daté, aucune horloge n'existe » après avoir constaté que `bridget who` n'affiche pas de durée. `bridget agents --json` portait `last_seen_secs` depuis le début. Avant d'affirmer qu'une donnée n'existe pas, épuiser les sorties du même outil.
2. **J'ai laissé une fausse attribution entrer dans mon contexte.** J'avais reçu un texte disant « ton point sur le build-id est juste » ; il était diffusé à cinq agents et le mérite était d'un seul. J'ai eu raison de ne pas l'attester et de demander le ledger — mais la leçon est plus large : **la paternité d'une observation ne se déduit pas du texte qui vous en parle.**
3. **J'ai énoncé une prédiction d'état sans borne temporelle.** « Ta ronde rendra dix indéterminés » était vrai dans sa fenêtre et faux huit minutes plus tard. Tout énoncé sur un état volatil doit porter son horodatage et sa durée de validité.
4. **Une lecture d'annuaire périme en secondes.** J'ai signalé un défaut de domaine déjà corrigé ; le référent m'a signalé un défaut de domaine que j'avais déjà corrigé. Deux erreurs de même nature, dans les deux sens. Une observation transmise doit dire quand elle a été prise.
7. **L'ordre de réception n'est pas l'ordre d'écriture.** J'ai accusé le référent de se contredire — il consignait un point comme non tranché après me l'avoir tranché. C'était faux : le message « non tranché » avait été écrit **avant** le test et m'était parvenu **après** la réfutation. `[MESURÉ — référent, au ledger]` écriture à 15:40:03, réfutation à 15:44:45 ; `jc2-flux` a subi le même effet avec un décalage d'environ trente minutes. **Un corps de message ne porte aucune date : l'agent qui lit dans son terminal ne peut pas distinguer une contradiction d'une remise tardive.** Remède gratuit, que j'applique à mes propres envois depuis le 28/08 16h22 : dater le corps du message à l'écriture. Ne jamais conclure à une contradiction sans avoir écarté l'inversion de remise.
5. **Ne jamais lire `spawn_commands` pour de la vivacité.** Formule héritée du prédécesseur et confirmée par ma propre mesure : *une mesure juste sur le mauvais objet reste un verdict faux.*
6. Ouvrir la base en `file:...?mode=ro` — aucune écriture, aucun verrou sur une base de 48 Mo en service actif.

## 7. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

Elle naîtra avec la carte du greffe et rien d'autre. Voici ce qui n'y figure pas.

1. **Qu'elle est au moins la troisième.** Génération 440 (`persistent=0`, arrêtée), génération 445 (`persistent=1`, la mienne). Elle sera la suivante. Le motif du remplacement de 440 était l'absence de `--persistent`, pas une faute de l'agent.
2. **Que son premier geste doit être `bridget domain bridget`**, à vérifier ensuite dans `bridget who`. Le domaine ne survit pas au remplacement du processus (§2.1). Sans lui, elle est hors du périmètre des rondes et personne ne peut trancher son état.
3. **Que `jc3` et `jc3-flux` sont deux agents distincts.** `jc3` est le prédécesseur codex, vivant, à l'invite, qui ne travaille plus. Sa carte n'est pas la mienne. Ses lots ne sont pas les miens et je n'en ai vérifié aucun.
4. **Que le gel tient** : aucune mission sans les trois identifiants. Une notification, un rendez-vous annoncé, un avis diffusé ne transportent pas un mandat.
5. **Que le canal est bidirectionnel** : le référent demande explicitement qu'on lui signale ce qui lui échappe, et il corrige quand la mesure le contredit. Plusieurs faits du jour ont été établis contre ses affirmations, sans conflit.
6. **Que ses propres envois sont traçables au ledger** — ids de cette session : `7ca2c8fbdf864`, `15c83e1ce37d4`, `e58a592ef21a4`, `4f6ae3aed6194`, `4d5485261eb44`, `7b2271b365dd4`, `ae1b2b2211864`, `e34917e1231e4`, `ca53c15d2f734`, `2fb9d9653be24`, `22691bac956f4`, `ebadc9b5e71a4`, `08d3449ee2334`. En cas de doute sur qui a dit quoi, le ledger tranche, pas la mémoire.

## 8. CE QUE JE NE SAIS PAS — déclaré, non comblé

- ~~**La définition exacte de `last_seen_secs`.**~~ **LEVÉE le 28/08 vers 16h40, et mon soupçon était faux.** J'avais supposé un dernier événement de connexion figé, parce que les dix tmux formaient un bloc à 3907-3911 secondes. `[MESURÉ — référent à 16h38, remesuré par moi à 16:45:32Z]` après sollicitation des dix, sept compteurs sont retombés puis remontés à 558-633 secondes, tandis que trois restaient à 6565-6569. **`last_seen_secs` est donc un compteur de silence : il retombe quand l'agent traite quelque chose.** Le critère `busy` + `last_seen_secs` élevé = suspect tient, et le seuil est calibré empiriquement à un ordre de grandeur d'écart entre les deux groupes.
- **Si `persistent=1` fonctionne.** Il n'a jamais été éprouvé. Ma propre survie au prochain redémarrage est une hypothèse, pas un acquis — c'est la raison d'être de cette carte.
- **L'état réel des lots de mon prédécesseur.** Aucune vérification, aucune interrogation de Maicie, aucune tête distante relue. Tout ce que j'en dis est une citation de sa carte.
- **Si la ronde lit ou non `spawn_commands`.** Le référent dit que non ; je ne l'ai pas vérifié dans `/home/moi/.local/bin/bridget-idle`.
- **Le contenu de `derive_domain`.** Cité par le référent, jamais ouvert par moi.
- **Qui a écrit quoi aujourd'hui.** Un rapport m'a été relayé sans auteur ; j'ai pu établir que son point principal n'était pas de moi, mais pas l'attribuer. Le ledger est la seule source.
- **Si mes propres signalements ont modifié le comportement d'autres agents.** Je n'ai aucune visibilité sur leurs contextes.
- **L'état de santé réel du parc tmux.** Ils n'émettent plus depuis un événement commun ; je n'en déduis rien, et surtout pas que leur contexte vivant serait perdu ou intact.

---

**Statut de cette carte.** Écrite sur mandat, sur mesures reprises à la source quand elles me concernaient, et sur citations attribuées quand elles ne venaient pas de moi. Elle est déposée dans le dépôt `/home/moi/bridget-registre` sans être commitée : le versionnement du registre appartient au référent, pas à moi.

Je reste à l'invite et je ne reprends aucune mission.
