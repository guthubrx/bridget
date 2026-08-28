# Carte de reprise — jc1-flux

- Agent : `jc1-flux`
- Type : `claude`, protocole `claude_stream_json`, canal `ssh-unix`, mode `cli`
- Génération portante : **444**, `persistent=1` — mesuré au greffe de flotte le 2026-08-28 à 17:04Z
- Émise : 2026-08-28T17:05Z, sur mandat du référent bridget
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

- `[MESURÉ 17:04Z]` Objectif `587da26d-0130-49b5-894d-2bb91ec9afc1` à l'état `en_coordination`, délégation `100f2025-279a-4616-a4c8-af631b24e00b` à l'état `creee`. **Les deux points du mandat sont écrits, testés et commités ; rien n'est intégré.**
- `[MESURÉ 17:04Z]` L'objectif de cette carte, `c9aac01b`, est lui aussi `en_coordination` avec sa délégation `f80f29f2` à `creee`.
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

11. **Un message du référent peut être attesté au ledger sans atteindre son destinataire.** L'entrée `[1787935163] bridget → jc1-flux` portant « AUTORISATION ACCORDEE » figurait au ledger sans être jamais arrivée dans mon flux. Un décompte d'accusés l'aurait compté comme livré.

## 3. RESTE — ce qui n'est pas fait

- **Porter la branche hors de `/tmp`.** C'est le seul vrai risque de perte. Demandé trois fois au référent, jamais répondu au moment d'écrire cette carte.
- **Six processus orphelins de mes tests-portes** : `3876316`, `3876326`, `3876561`, `3876575`, `3879414`, `3879424`. Autorisation demandée, non accordée à cette heure. Ils tournent sous des `HOME` isolés `/tmp/user/1002/maicie-mvp-gate-*` et ne peuvent pas atteindre le daemon en service.
- **Les tests d'intégration hors `--lib` n'ont jamais été rejoués jusqu'au bout**, à cause du test instable du point 2.8.
- **La colonne `PERSIST` reste vide en production** tant que le daemon en service tourne l'ancien binaire. `[MESURÉ]` Le CLI neuf face au daemon ancien affiche `—` et `null` partout, sans planter et sans rien inventer. Le redémarrage qui activerait la colonne est aussi l'événement qui draine — d'après le roster, personne ne serait drainé.

## 4. CHEMINS ABSOLUS

- **Travail du mandat `587da26d`** : `/tmp/jc1flux-persist-815e3bc/repo`
  branche `session-058-persistance-annuaire`, arbre propre
  commits `8ebe89a` (annuaire) puis `ab8960f` (garde de spawn), sur base `815e3bc`
- **⚠ NOMMER LE REMOTE.** Dans ce clone, `origin` = **`/home/moi/revue/jc1`**, un chemin LOCAL — *pas* GitHub. Un `git push origin` y pousserait dans le checkout principal. Le remote GitHub `https://github.com/guthubrx/bridget.git` n'est `origin` **que depuis `/home/moi/revue/jc1`**. Les deux `origin` ne désignent pas le même dépôt.
- Clone de référence non modifié : `/tmp/jc1flux-persist-815e3bc/repo-base` (sur `815e3bc`)
- Banc de mutation : `/tmp/jc1flux-persist-815e3bc/banc-mutants.py`
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
10. **`bridget ledger` ne rend que les 20 derniers messages** : il ne permet pas de vérifier une attribution ancienne.

## 6. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

- Que la garde a été **déplacée** de `parse_spawn_args` vers `resolve_spawn_order`, et **pourquoi**. Sans cela, le premier réflexe est de la remonter au parsing — où elle casse le rejeu.
- Que les onze tests rouges sont **préexistants et mesurés des deux côtés**. Sans cela, on croit avoir cassé le dépôt et on empile des correctifs sur un faux diagnostic.
- Que l'inventaire des appelants de `spawn` a été fait et qu'il est **vide côté production** : aucun script, aucun outil MCP, seulement cinq tests.
- Que le troisième état `—` est **délibéré**, et non un oubli de renseigner le champ.
- Que `/tmp` a déjà avalé le travail d'un prédécesseur sur ce même sujet.

## 7. CE QUE JE NE SAIS PAS — déclaré, non comblé

- `[IGNORÉ]` **Si le drapeau `persistent=1` tient réellement.** Il n'a jamais été éprouvé par un redémarrage. Ma survie en dépend entièrement et je n'en ai aucune preuve. `[RAPPORTÉ — bridget, geste proposé par rc5-flux]` un agent `temoin-persistance` aurait été créé pour l'éprouver sans risquer un porteur de contexte ; **je ne l'ai pas vérifié.**
- `[IGNORÉ]` Si mes deux commits sont intégrés quelque part. Au 2026-08-28T17:05Z ils ne le sont pas — **et cette affirmation périme.**
- `[IGNORÉ]` Pourquoi le test `stop_apres_register...` bloque. J'ai mesuré son instabilité, pas sa cause.
- `[IGNORÉ]` Pourquoi le message d'autorisation du référent n'est pas arrivé dans mon flux alors qu'il figure au ledger.
- `[IGNORÉ]` L'état réel du travail des trois délégations héritées de `jc1` (`dc2324d5`, `039123ba`, `6073f51b`, toutes `creee` au 17:04Z). Leur état ne dit rien de l'objet livré — voir 1.
- `[IGNORÉ]` Le contenu du reste d'audit daemon décrit au point 3 de la carte de `jc1`. **Je n'y ai pas touché et je ne l'ai pas lu.**
- `[RAPPORTÉ — bridget]` Le daemon en service annonce `build-id f7658d4d9746-dirty` alors que le code servi serait celui de `16be24f` ; **je n'ai vérifié que l'étiquette, jamais la correspondance du code.**
- `[RAPPORTÉ — référent, via le mandat]` Les dix prédécesseurs tmux n'ont aucune ligne dans `spawn_commands` ; **mesure de `jc3-flux`, pas la mienne.**
