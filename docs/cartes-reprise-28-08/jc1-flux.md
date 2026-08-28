# Carte de reprise — jc1-flux

- Agent : `jc1-flux`
- Type : `claude`, protocole `claude_stream_json`, canal `ssh-unix`, mode `cli`
- Génération portante : **444**, `persistent=1` — mesuré au greffe de flotte le 2026-08-28 à 17:04Z
- Émise : 2026-08-28T17:05Z, sur mandat du référent bridget
- Mise à jour : 2026-08-28T18:20Z, à la demande du référent — deux faits promus de `[RAPPORTÉ]` à `[MESURÉ]`, un troisième mandat livré, une limite d'observation déclarée
- Mise à jour : 2026-08-28T19:30Z — **le drapeau `persistent` a été éprouvé par un vrai redémarrage**, les trois correctifs sont intégrés et déployés, et trois erreurs à moi sont inscrites
- Mandat de cette carte : objectif `c9aac01b-cc8d-4520-a375-dcaec5c84844` ; délégation `f80f29f2-033b-4e6a-8dbb-75da7d4ddc18` ; message `f72325c2-aa49-4dd6-adb2-57878efe64a1`
- Prédécesseur : `jc1` (codex, tmux), carte distincte en `/home/moi/bridget-registre/docs/cartes-reprise-28-08/jc1.md`
- Seconde incarnation du successeur : la première avait été lancée sans `--persistent` et remplacée avant d'avoir travaillé

**Convention de lecture.** Chaque fait porte sa provenance :
`[MESURÉ]` = produit par moi, avec une commande, dans cette session ;
`[RAPPORTÉ — source]` = tenu d'autrui, source nommée, non vérifié par moi ;
`[IGNORÉ]` = je ne le sais pas, et je le déclare plutôt que de le combler.

**Toute affirmation de cette carte est datée du 2026-08-28 et périme dans les deux sens.**
« X est intégré » ne se défait pas ; « X n'est pas intégré » peut devenir faux à tout instant.
Vérifier avant d'agir sur l'une comme sur l'autre.

---

## 1. ÉTAT

- `[MESURÉ 17:04Z]` Objectif `587da26d-0130-49b5-894d-2bb91ec9afc1` à l'état `en_coordination`, délégation `100f2025-279a-4616-a4c8-af631b24e00b` à l'état `creee`. **Les deux points sont écrits, testés, commités — et `[MESURÉ 18:57Z]` INTÉGRÉS dans `main` du dépôt GitHub, puis déployés.**
- `[MESURÉ 17:04Z]` L'objectif de cette carte, `c9aac01b`, était lui aussi `en_coordination`. `[RAPPORTÉ — bridget, 18:04Z]` **il est depuis fermé**, la carte étant présente dans `main` du dépôt GitHub.
- `[MESURÉ 18:05Z]` Troisième mandat livré : objectif `522a044e-9a89-42d8-add9-f9ca6117737c`, délégation `04f80ced-7011-4733-bd34-f038551a6c87` — exposer la borne du ledger. Branche `session-059-borne-ledger`, commit `7fb0ecd`. `[MESURÉ 18:57Z]` **Intégré dans `main` GitHub et déployé** : `bridget ledger` rend 200 par défaut et annonce sa borne.
- `[MESURÉ 17:04Z]` **`creee` ne distingue pas une délégation dormante d'une délégation en cours de travail.** Mes deux délégations actives portent exactement l'état des trois délégations héritées de `jc1` que le référent m'a dit de ne pas reprendre. C'est la confirmation directe, par mesure, de la règle attribuée à `rc5` : ne jamais déduire l'état d'un travail de l'état de sa délégation — vérifier l'objet livré.
- `[MESURÉ]` `domaine=bridget`, corrigé par moi **deux fois** : à l'ouverture de la première incarnation, puis à nouveau après ma recréation.

## 2. FAIT — mesuré par moi

1. **L'annuaire ne publiait aucune persistance.** `bridget agents --json` rendait 16 clés, aucune ne portant la survie de l'agent ; la donnée existait pourtant dans `spawn_commands.persistent`. Collectée, non publiée. C'est ce signalement (message `0a4d421d032b4`, 15:00:46) qui a déclenché le mandat `587da26d`.

2. **Le piège de la ligne arbitraire, reproduit à l'identique.** Une requête `GROUP BY name` sans tri sur `spawn_commands` rend cinq agents `persistent=0` et `jc1-flux` en `state=failed` ; la même question triée par `MAX(generation)` rend les dix agents en flux à `persistent=1`, générations 444 à 453. **La table conserve toutes les générations d'un même nom : toute lecture non triée y est fausse par construction.**

3. **`named-roster.json` est la bonne source, et elle rend le tri inutile.** `/home/moi/.config/bridget/named-roster.json` ne garde qu'une entrée par nom, écrasée à chaque génération connectée par `remember` : la dernière valeur y est structurellement la courante. C'est aussi la source exacte que lit `drain_non_persistent`. `[MESURÉ 17:04Z]` 12 entrées, toutes `persistent=true`.

4. **Le `cwd` d'un équipier non persistant n'existe plus nulle part après sa connexion.** Quatre vérifications convergentes : `fleet.json` n'est pas écrit (l'`upsert` est conditionné à `persistent`) ; `spawn_commands` n'a pas de colonne `cwd` sur ses 14 colonnes ; `ResolvedAgentDefinition` n'a pas de champ `cwd` ; `complete_locked` retire l'`ActiveSpawn`, seul porteur du `cwd`, dès la complétion. **Conséquence : aucune promotion après coup n'est possible, une erreur de lancement est irrattrapable et non pas seulement non outillée.**

5. **Le rejeu d'un `--command-id` mémorisé est un appelant légitime que la garde aurait cassé.** Il ne redéclare aucune option : l'enveloppe est relue du disque. Une garde posée au parsing des arguments aurait refusé tout rappel d'un ordre mémorisé. Elle vit donc dans `resolve_spawn_order`, branche « ordre neuf », après le point de retour de l'ordre mémorisé.

6. **Le domaine ne survit pas au remplacement du processus.** Ma génération précédente avait été corrigée à `bridget` ; la génération 444 est née avec `domaine=jc1`. Mesuré aussi sur `jc3-flux`, `rc5-flux` et `essai-distant-flux` au même instant. Le domaine par défaut dérive du nom du répertoire de travail.

7. **Onze tests échouent sur la base 815e3bc, sans aucun rapport avec mon code.** Dix `daemon::presence_tests` et un témoin de carte de reprise. Comparaison à conditions égales : base **481 réussites / 0 échec**, mes modifications **486 / 0** — la différence est exactement mes cinq témoins.

8. **Le test `stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels` est instable, mesuré des deux côtés** : il échoue en 12 s sur certaines exécutions et se bloque indéfiniment sur d'autres, y compris sans mes modifications. **Je ne le requalifie ni en rouge ni en vert.**

9. **Deux tests que je croyais cassés par moi ne l'étaient pas** : le `HOME` de test était partagé entre exécutions. Avec un `HOME` neuf, la base passe 481/481. Mesuré, pas déduit.

10. **Mon banc de mutation a laissé un fichier muté.** Un mutant supprimait un bloc ; une suppression ne se restaure pas par motif, la restauration a échoué et le banc s'est arrêté **avant** de réparer. J'ai retrouvé l'arbre sans sa garde. Corrigé : motif vide interdit, restauration sous `finally`.

11. **Un message du référent peut être attesté au ledger sans atteindre son destinataire.** L'entrée `[1787935163] bridget → jc1-flux` portant « AUTORISATION ACCORDEE » figurait au ledger sans être jamais arrivée dans mon flux. Un décompte d'accusés l'aurait compté comme livré. **Deux cas en une heure** sur le seul canal référent → moi, le second étant `101def8e7b174`.

12. **Un message non parvenu n'est pas perdu : il est dans la table.** `SELECT ts, sender, target, body FROM ledger WHERE target='<nom>' ORDER BY ts DESC` sur `file:/home/moi/.cache/bridget/bridget.db?mode=ro` rend les messages que le canal n'a pas livrés. C'est ainsi que j'ai lu `101def8e7b174` et agi dessus **six minutes avant** que le canal ne me le remette. `mode=ro` est impératif : la base est ouverte par le daemon en service.

13. **`spawn_commands.state` survit à la mort de l'agent.** Mesuré sur un cas neuf : `essai-garde-verif-referent`, génération 455, `persistent=0`, `state=connected` — alors que le référent l'avait arrêté quelques minutes plus tôt. Une lecture de cette table ne dit rien de la vivacité. `[MESURÉ]` — le même fait avait été établi par `jc3-flux` sur sa propre génération 440 ; je le confirme sur un agent créé et tué dans la même minute.

14. **Le témoin `temoin-persistance` existe** : génération 454, `persistent=1`, vu par moi dans `spawn_commands`. `[MESURÉ]` pour son existence ; `[RAPPORTÉ — bridget, geste proposé par rc5-flux]` pour son rôle attendu. **Ce rôle — éprouver le drapeau sans qu'un porteur de contexte serve de cobaye — s'est révélé non tenu : voir 18.**

15. **Le motif « donnée collectée, non exposée » se compte à quatre occurrences le même jour**, et deux d'entre elles ont un paramètre qui existe déjà : la persistance (absente des 16 clés de l'annuaire) ; la borne du ledger (`limit: 20` figé alors que le protocole porte `limit` et que le même fichier passe 200 ailleurs) ; le `busy` du greffe (connu pour refuser une délégation, introuvable pour observer un travail) ; la phase de remise (`send_deliveries.phase` — `acked` 5285, `dispatching` 33, `indeterminate` 94 — qu'aucune commande CLI n'expose). **Le coût n'a jamais été la donnée manquante, toujours le détour par un instrument improvisé.**

16. **LE DRAPEAU `persistent=1` A TENU À TRAVERS UN VRAI REDÉMARRAGE.** C'était l'ignorance centrale de cette carte. Le service a redémarré le 28/08 à 18:47:49 — `[RAPPORTÉ — bridget]` ancien PID 3588739 disparu, binaire remplacé à 18:46:50, trois vagues de réinscription par commandes `recovery-<uuid>` du superviseur. `[MESURÉ à 18:57Z]` **douze agents présents après redémarrage, tous `persistent=true`, aucun drainage** ; `essai-garde-verif-referent`, arrêté avant, est correctement absent ; roster à 12 entrées, aucune non persistante. **Mon contexte a survécu, et la preuve est un effet mesuré, pas une intégration.**

17. **Les trois correctifs sont intégrés ET déployés.** `[MESURÉ 18:57Z]` `8ebe89a`, `ab8960f`, `7fb0ecd` sont ancêtres de `main` du dépôt **GitHub** (`08d766e`), vérifié par `merge-base --is-ancestor` contre l'URL et non contre l'alias. En production : `bridget who` rend `PERSIST=oui` pour les agents gérés et `—` pour les tmux hors flotte ; `bridget ledger` rend 200 par défaut et se clôt par `maximum lisible atteint (200)`. **Intégré ne suffit pas — c'est le redéploiement du binaire et le redémarrage du daemon qui rendent un correctif vivant.**

18. **Le témoin de persistance prouve sans protéger, et son rang ne porte aucune intention.** `[MESURÉ]` `temoin-persistance` porte les générations 454, 469, 481, 493 ; la 454 est isolée (échéance 16:01, pose initiale). Les trois vagues de réinscription sont 458‑469, 470‑481, 482‑493, **douze entrées chacune**, et le témoin y est **douzième sur douze**. Les onze porteurs de contexte passent avant lui — il atteste la survie sans épargner personne : une attestation simultanée n'est pas une alerte précoce.
    **Mais il n'est pas dernier par conception : il est dernier parce que son nom commence par `t`.** `[MESURÉ]` L'ordre des douze est exactement l'ordre alphabétique, et la cause est dans le code : `fleet.rs:406`, `recovery_candidates()` trie par `lease.name`. Vérifié par les deux voies, source et données. **Aucun mécanisme ne distingue le témoin d'un agent quelconque** — un témoin qui doit son rang à son initiale est une treizième instance, pas un témoin.
    `[RAPPORTÉ — jc6-flux, message 340a6ea824ec4]` pour la chronologie des vagues ; `[RAPPORTÉ — bridget]` pour le groupement par `deadline_at` et le constat alphabétique ; formulation « prouve sans protéger » due à `bridget`.
    `[IGNORÉ]` Le rang est pilotable par le nom — un témoin nommé `aaa-…` passerait premier — mais **je ne sais pas si passer premier protégerait** : la vague de reprise semble atomique, et un rang précoce n'aide que si quelque chose peut s'interrompre entre deux réinscriptions. Non vérifié.

## 3. RESTE — ce qui n'est pas fait

- ~~**Porter la branche hors de `/tmp`.**~~ **RÉSOLU à 17:29Z** : les branches sont poussées sur `https://github.com/guthubrx/bridget.git` (voir section 4). La cause n'était pas `/tmp` — le clone est un vrai dépôt git — mais un `origin` pointant un miroir local.
- ~~**Six processus orphelins de mes tests-portes.**~~ **RÉSOLU à 17:28Z**, sur autorisation du référent : trois signaux ont suffi pour six PID, les trois `managed-wrapper` étant déjà morts avec le daemon de leur groupe. `[MESURÉ]` **Tuer le daemon d'un groupe emporte son wrapper.**
- ~~**L'intégration des trois correctifs.**~~ **RÉSOLU vers 18:46Z** : les neuf branches du parc ont été intégrées une par une, les miennes comprises, puis le binaire a été remplacé et le daemon redémarré. `[MESURÉ 18:57Z]` **Il ne reste rien de ce mandat.**
- **Les tests d'intégration hors `--lib` n'ont jamais été rejoués jusqu'au bout**, à cause du test instable du point 2.8.
- **La colonne `PERSIST` reste vide en production** tant que le daemon en service tourne l'ancien binaire. `[MESURÉ]` Le CLI neuf face au daemon ancien affiche `—` et `null` partout, sans planter et sans rien inventer. Le redémarrage qui activerait la colonne est aussi l'événement qui draine — d'après le roster, personne ne serait drainé.

## 4. CHEMINS ABSOLUS

- **Le travail est publié — écrire l'URL, jamais l'alias :**
  `https://github.com/guthubrx/bridget.git`
  - mandat `587da26d` → branche `session-058-persistance-annuaire`, commits `8ebe89a` (annuaire) puis `ab8960f` (garde de spawn), base `815e3bc`
  - mandat `522a044e` → branche `session-059-borne-ledger`, commit `7fb0ecd`, base `1a7c381`
  - **aucun des trois n'est dans `main`** au 2026-08-28T18:20Z ; `main` y valait `07fbe92` à 18:13Z et bougeait plusieurs fois par heure
- Copie de travail : `/tmp/jc1flux-persist-815e3bc/repo`, arbre propre
- **⚠ NOMMER LE REMOTE.** Dans ce clone, `origin` = **`/home/moi/revue/jc1`**, un chemin LOCAL — *pas* GitHub ; le remote GitHub y est nommé `github`. Un `git push origin` pousserait dans le checkout principal, que personne ne fetch. Le remote GitHub n'est `origin` **que depuis `/home/moi/revue/jc1`**. Les deux `origin` ne désignent pas le même dépôt : c'est ce piège qui a rendu mes deux premiers commits invisibles au référent pendant une heure.
- Clone de référence non modifié : `/tmp/jc1flux-persist-815e3bc/repo-base` (sur `815e3bc`)
- Bancs de mutation : `/tmp/jc1flux-persist-815e3bc/banc-mutants.py` (mandat `587da26d`) et `banc-mutants-ledger.py` (mandat `522a044e`)
- Répertoires de compilation : `target-verif`, `target-base`, `target-base2` sous `/tmp/jc1flux-persist-815e3bc/`
- `HOME` de test isolés : `/tmp/jc1flux-persist-815e3bc/home`, `home-neuf`, `home-neuf2`
- **Tout ce qui précède est sous `/tmp` et ne survivra pas.** Le clone de mon prédécesseur `jc1`, `/tmp/audit-guard-review.ymR2/repo`, avait déjà disparu quand j'ai pris son relais.
- Checkout principal : `/home/moi/revue/jc1` — **interdit en écriture** (règle 6 : entrée primaire d'un dépôt à worktrees liés)
- Sources touchées : `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/{cli,daemon,fleet,recovery_trace,attach,reprise}.rs`, `plugins/maicie/tests/{mvp_gate.rs,integration/guichet_gate.rs}`

## 5. PIÈGES

1. **Ne jamais interroger `spawn_commands` sans trier par génération.** Voir 2.2. Préférer `named-roster.json`, indexé par nom.
2. **`—` n'est pas `non`.** Une persistance non attestée signifie qu'aucune entrée de flotte ne couvre l'agent, donc que rien ne le drainera. Les confondre pousse à relancer un agent sain, c'est-à-dire à détruire son contexte.
3. **Ne pas remonter la garde de spawn au parsing des arguments** : elle casserait le rejeu d'un `--command-id`. Un mutant du banc tue quiconque essaie.
4. **Isoler `HOME` avant toute suite de tests.** `DaemonConfig::default()` pointe sur `$HOME/.cache/bridget` — la vraie socket et la vraie base. Et ne pas réutiliser le même `HOME` entre deux exécutions : voir 2.9.
5. **Ne jamais partager un `CARGO_TARGET_DIR` entre deux arbres sources différents.** Cargo y réutilise des artefacts et fait échouer la compilation avec des erreurs qui semblent venir du code lu.
6. **Un banc de mutation doit restaurer sous `finally`** et refuser les motifs vides. Voir 2.10.
7. **Les tests-portes `maicie` laissent des daemons vivants derrière eux.** Contrôler les processus résiduels *après* chaque exécution.
8. **Ne tuer aucun processus sans autorisation explicite**, même le sien : `kill` sans `-9`, un à la fois, 3 s d'attente, vérification entre chaque, et arrêt si le premier résiste.
9. **`cargo fmt --check` n'est pas propre sur `main`** : 16 diffs préexistants au 2026-08-28. Comparer à la base plutôt que viser zéro.
10. **`bridget ledger` EN SERVICE ne rend que les 20 derniers messages** — treize minutes à 94 messages/heure. Il ne permet pas de vérifier une attribution ancienne. Recours immédiat : lire la table (voir 2.12). **PÉRIMÉ depuis 18:46Z** : le correctif est intégré et déployé, `bridget ledger` accepte `--limit` et rend 200 par défaut. Ligne conservée pour mémoire de la méthode : une affirmation datée doit être relue, pas recopiée.
11. **Écrire l'URL, pas l'alias.** `origin` ne désigne pas le même dépôt selon les checkouts. Une attestation formulée avec `origin` ne vaut pas pour son lecteur : un agent qui cherche sa carte via *son* `origin` conclura à une perte alors qu'il regarde le mauvais dépôt. Formulation due à `cartae0-flux`.
12. **Ne pas rebaser une branche dont les SHA ont déjà été vérifiés par autrui** sans le lui dire : le rebase invalide sa vérification et lui fait refaire son travail. Sans conflit, l'ancienneté de la base ne coûte rien.
13. **La fenêtre prise pour le tout — et je l'ai subie le jour où je la corrigeais dans l'outil.** J'ai mesuré la position du témoin avec `generation >= 465` : ce filtre ne montrait que les cinq dernières lignes d'une vague de douze, et j'en ai conclu « quatrième, au milieu du parc » au lieu de **douzième sur douze**. Le bon groupement n'est pas une fenêtre de lecture mais `deadline_at`, qui délimite les vraies vagues. **C'est exactement le défaut que je venais de corriger dans `bridget ledger`** — un plafond qui se lit comme « il n'y a rien d'autre » — et la même famille que le `tail -25` relevé par `cartae0-flux`. Corriger un piège dans un outil ne protège pas de le commettre dans sa propre lecture. Constat dû à `bridget`.
14. **Ne pas affirmer ce que sa propre carte marque comme non vérifié.** J'ai écrit que le témoin avait « rempli son office » alors que ma section 2.14 portait `[RAPPORTÉ]` et « je ne l'ai pas vérifié » sur ce rôle précis. J'avais l'avertissement sous les yeux, de ma main, et je l'ai enjambé parce que le résultat m'arrangeait. **Un marqueur de provenance ne sert à rien s'il n'est pas relu au moment de conclure.**

## 6. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

- Que la garde a été **déplacée** de `parse_spawn_args` vers `resolve_spawn_order`, et **pourquoi**. Sans cela, le premier réflexe est de la remonter au parsing — où elle casse le rejeu.
- Que les onze tests rouges sont **préexistants et mesurés des deux côtés**. Sans cela, on croit avoir cassé le dépôt et on empile des correctifs sur un faux diagnostic.
- Que l'inventaire des appelants de `spawn` a été fait et qu'il est **vide côté production** : aucun script, aucun outil MCP, seulement cinq tests.
- Que le troisième état `—` est **délibéré**, et non un oubli de renseigner le champ.
- Que `/tmp` a déjà avalé le travail d'un prédécesseur sur ce même sujet.

## 7. CE QUE JE NE SAIS PAS — déclaré, non comblé

- ~~`[IGNORÉ]` **Si le drapeau `persistent=1` tient réellement.**~~ **LEVÉ à 18:57Z, et c'était l'ignorance centrale de cette carte.** Le service a redémarré à 18:47:49 : douze agents réinscrits, tous `persistent=true`, aucun drainage — voir 2.16. **La preuve est un effet mesuré, pas un raisonnement.** Ce que la levée n'établit pas : que le témoin ait protégé quiconque (voir 2.18), ni que le drapeau tienne à un redémarrage d'un autre type — celui-ci s'est accompagné d'un remplacement de binaire.
- `[IGNORÉ]` **Je n'ai jamais vu ma colonne `PERSIST` afficher `non` sur un agent éphémère réellement VIVANT.** Le seul éphémère du jour, `essai-garde-verif-referent`, a vécu moins d'une minute, et le daemon en service tourne l'ancien binaire, sans la colonne. J'ai failli conclure que la colonne afficherait `—` au lieu de `non`, le roster ne portant aucune entrée non persistante ; j'ai lu le code plutôt que de conclure — dans `fleet.rs`, `connect()`, le `remember` est **hors** du `if active.persistent`, donc un éphémère vivant y est bien inscrit avec `false`, et le roster est vide de `false` simplement parce qu'aucun éphémère n'est vivant, l'arrêt retirant l'entrée via `roster.forget`. **C'est du code lu, pas du comportement observé**, et l'observation manquante ne sera possible qu'après déploiement.
- ~~`[IGNORÉ]` Si mes trois commits sont intégrés.~~ **LEVÉ à 18:57Z** : les trois sont ancêtres de `main` GitHub `08d766e`. L'affirmation contraire, écrite à 18:20Z, a péri en vingt-six minutes — c'est la démonstration la plus courte que « X n'est pas intégré » périme dans l'autre sens.
- `[IGNORÉ]` Si 200 messages suffisent au débit du référent. Deux heures de visibilité à 94 messages/heure est un calcul, pas une mesure de sa dette réelle.
- `[IGNORÉ]` Pourquoi le test `stop_apres_register...` bloque. J'ai mesuré son instabilité, pas sa cause.
- `[IGNORÉ]` Pourquoi le message d'autorisation du référent n'est pas arrivé dans mon flux alors qu'il figure au ledger.
- `[IGNORÉ]` L'état réel du travail des trois délégations héritées de `jc1` (`dc2324d5`, `039123ba`, `6073f51b`, toutes `creee` au 17:04Z). Leur état ne dit rien de l'objet livré — voir 1.
- `[IGNORÉ]` Le contenu du reste d'audit daemon décrit au point 3 de la carte de `jc1`. **Je n'y ai pas touché et je ne l'ai pas lu.**
- `[RAPPORTÉ — bridget]` Le daemon en service annonce `build-id f7658d4d9746-dirty` alors que le code servi serait celui de `16be24f` ; **je n'ai vérifié que l'étiquette, jamais la correspondance du code.**
- `[RAPPORTÉ — référent, via le mandat]` Les dix prédécesseurs tmux n'ont aucune ligne dans `spawn_commands` ; **mesure de `jc3-flux`, pas la mienne.**
