# Carte de reprise — rc5-flux

- Agent : `rc5-flux` (type `claude`, protocole `claude_stream_json`)
- Génération : 454 exclue — je suis la **génération 446**, instance `ed0f6e57-3c8d-4d9c-9321-9e2da31e9d1e`, `persistent=1`
- Génération précédente du même nom : 441, instance `9262ecc8-bc6a-4c18-94e3-5dc37b3ae885`, `persistent=0`, arrêtée
- `definition_digest` : `d97abf970b41493b06f355f195be0aaeaafb1f6ef54713097bbab45e20f51a6b` (identique en 441 et 446 — mesuré)
- Prédécesseur : `rc5` (Codex, tmux), vivant à l'invite, ne travaille plus
- Émise le 28/08/2026 sur mandat : objectif `f49057b2-200b-4e3b-926e-f63a9f6258dc`, délégation `70664dfa-b51e-49c5-8f6e-92a2f4e005e5`, message `6d66aead-1c96-43c4-ba09-f08c4824a633`

---

## 1. ÉTAT

Aucune mission de code. Aucune écriture hors cette carte. Gel de l'humain du 28/08 tenu d'un bout à l'autre : je n'ai repris aucun travail de `rc5`, ouvert aucune branche, lancé aucun test, aucun build, aucun mutant.

Mon seul travail de la journée a été de **mesurer et de contredire** — le référent m'y a explicitement invité, et neuf de mes envois ont porté des corrections.

Répertoire de travail : `/home/moi/revue/rc5`, branche `amend/maicie-c5-rc5`, tête `9422c19`. **Je n'y ai rien écrit** : c'est le checkout principal d'un dépôt à worktrees liés, ma carte d'amorçage l'interdit (règle 6). Arbre laissé propre.

---

## 2. CE QUE J'AI MESURÉ MOI-MÊME

Tout ce qui suit vient de mes propres commandes, en lecture seule. Rien n'est déduit.

**Persistance des agents en flux** — table `spawn_commands` de `/home/moi/.cache/bridget/bridget.db` :
les dix successeurs en flux portent `persistent=1` aux générations 444 à 453. Cinq y sont passés par correction (`jc1-flux` 439→444, `jc3-flux` 440→445, `rc5-flux` 441→446, `essai-distant-flux` 442→447, `jc2-flux` 443→448) ; trois sont **nés** persistants (`cartae0-flux` 451, `rc7-flux` 452, `essai-claude-distant-flux` 453). `jc1-flux` porte quatre générations, dont 436 et 437 en `issue_kind=failed`.

**Les dix prédécesseurs tmux n'ont AUCUNE ligne dans `spawn_commands`** — zéro pour chacun. Le drapeau `persistent` ne les concerne pas. C'est le fondement de l'asymétrie qui a motivé cette carte.

**Le domaine est persisté par NOM, pas par processus** — `/home/moi/.cache/bridget/agent-domains/<nom>`, fichiers de 7 octets. Aucune colonne `domain` nulle part dans le schéma de `bridget.db` (vérifié sur le schéma complet).

**Le code de `derive_domain()`** — `/home/moi/revue/rc7/bridget/crates/bridget-daemon/src/wrapper.rs:983` : racine git si `git rev-parse --show-toplevel` réussit, sinon `current_dir()`, puis `file_name()`. **Il n'existe aucun repli vers `bridget`.** Et `effective_domain(agent)`, juste après : lit `agent-domains/<agent>`, ne retombe sur `derive_domain` que si le fichier est absent ou vide ; son commentaire précise « relu à chaque enregistrement, y compris après une reconnexion ».

**Le cwd réel des agents** — via `/home/moi/.cache/bridget/agent-pids/<pid>` puis `readlink /proc/<pid>/cwd` :
`essai-claude-distant` (PID 1661445) → `/home/moi/revue/essai-claude-distant/bridget`, **dépôt git nommé `bridget`** ;
`rc7` (PID 2864080) → `/home/moi/revue/rc7/bridget`, idem.
Ces deux mesures ont résolu les deux contre-exemples que le référent déclarait ne pas savoir expliquer.

**Attribution du constat « 2 objectifs sur 619 »** — table `ledger` : première occurrence `374bb81589004`, 28/08 11:01:39, **bridget → jc2**. Aucun message de `rc5` ne contient « 619 objectifs » ni « sur 619 ». Le constat est du référent lui-même.

**Attribution du signalement build-id** — `0a4d421d032b4`, 15:00:46, **jc1-flux**. À 15:04:23 le même texte « TON POINT SUR LE BUILD-ID EST JUSTE » est parti à **cinq** destinataires (`253944033e904`, `566a18bfc7654`, `a57fc9c748964`, `d5293e7b5c334`, `e7b026efe2054`).

**Auteur du rapport anonyme de 15h15** — `essai-distant-flux`, message `9a2a3ee194164`, 15:15:51. Trois agents avaient écrit dans une fenêtre de 36 secondes (`ae1b2b2211864` jc3-flux 15:15:15, `beda1ba74efc4` jc2-flux 15:15:23).

**Carte de mon prédécesseur** : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc5.md`, 8699 octets, sha256 `0ef15a8cd3191f7e581721bf950b7a373a22517cc3d583cb37edd56a86bd3c7c`.

**Compteur d'objectifs, mesuré à 16h10 environ** : `maicie status --config /home/moi/.config/maicie/config.json` rend **633 objectifs**. Le référent en mesurait 621 à 15h39 et 619 à 11h01. Le dénominateur bouge d'environ douze en vingt-cinq minutes.

---

## 3. CE QUE JE TIENS D'AUTRUI — ET DE QUI

Rien de ce qui suit n'est de moi. Ne me l'attribue pas, et ne le laisse pas m'être attribué.

- **La règle « ne jamais déduire l'état d'un travail de l'état de sa délégation ; vérifier l'objet livré »** est de **rc5**, mon prédécesseur. Le référent me l'a transmise sous son nom. Elle a structuré toute ma conduite, y compris appliquée au travail du référent lui-même.
- **Le signalement du build-id `f7658d4d9746-dirty` non attestable** est de **jc1-flux**. J'ai cru un temps qu'il était de ma génération 441 — c'était faux, et cette croyance venait d'un message générique du référent.
- **Le constat « la population mesurée a changé, ce n'est pas la déterminabilité qui a progressé »** est d'**essai-distant-flux** (`9a2a3ee194164`). Les dix tmux sont indéterminés par construction de leur transport, pas par leur état.
- **Le constat mesuré du domaine `jc3` sur une v2** est de **jc3-flux**.
- **Le constat « 2 objectifs sur 619 portent la provenance, et ils sont mal étiquetés »** est du **référent**, formulé pour **jc2** à 11h01.

---

## 4. MES PROPRES ERREURS, ET CE QUE J'EN AI FAIT

Je les inscris parce qu'une carte qui cache ses fautes fait perdre du temps à celui qui la lit.

**Erreur 1 — une déduction présentée comme une mesure.** J'ai affirmé que mon domaine était « de nouveau faux au démarrage » sans l'avoir mesuré avant de le corriger. `bridget domain` est idempotent et ne rapporte aucun état antérieur. **Je l'ai retirée de moi-même** ; le référent a dû amender un constat de registre (`1801b1d`) qui s'appuyait sur deux témoignages concordants dont le mien.

*Complément mesuré après coup, qui aggrave mon cas :* `rc5-flux` était un nom **réutilisé** (génération 441 puis 446), et la génération 441 avait corrigé son domaine dès son premier geste — donc `agent-domains/rc5-flux` existait déjà à ma naissance, et `effective_domain` devait me le rendre. Mon affirmation n'était donc pas seulement non mesurée : elle était vraisemblablement **fausse**. Je ne peux pas le prouver directement, le `mtime` du fichier ayant été écrasé par ma propre commande — voir le piège du §7.

**Erreur 2 — une attribution non vérifiée.** J'ai accusé le référent d'avoir mal attribué le build-id, sur la foi d'un message qui m'était nominativement adressé. C'était vérifiable au `ledger` en une requête, et je ne l'ai pas faite avant d'accuser. Retirée après mesure.

**Erreur 3 — une hypothèse élégante et fausse.** J'ai supposé que `derive_domain` avait un repli par défaut sur `bridget`, ce qui expliquait joliment le cas `essai-claude-distant`. La lecture du code l'a réfutée. Je l'ai signalée au référent **en même temps** que la vraie explication, pour qu'il ne la garde pas.

**Leçon transmissible :** mes trois erreurs ont la même forme — j'ai conclu avant de mesurer, alors que la mesure était à portée de commande. Le correctif n'est pas la prudence, c'est la requête.

---

## 5. CE QUI RESTE — ET QUI N'EST PAS À MOI

Hérité de la carte de `rc5`, non repris, non commencé :

- **Priorité humaine / réadressabilité** : `a5721fa` est déployé mais **n'est pas ancêtre de `origin/main`**. Le troisième constat ne peut pas être fermé sur les seules observations runtime. Étape : décision humaine, puis intégration mesurée de `session-058`.
- **Session 056** : tranche 1 livrée et intégrée. Tranche 2 (fait canonique de dette humaine, permit `HumanRequest` opaque) et tranche 3 (`human_debt_gate` dans la transaction d'INSERT) restent. **Ordre impératif : la 3 ne peut pas précéder la 2.**
- Le mandat sur la propriété « pas d'objectif auto-généré tant qu'un message humain est sans réponse » est chez **jc2-flux** (objectif `bedaf54d-4807-4774-abd9-1581aea4b9b2`). Ce n'est pas à moi.
- La publication de la persistance dans `who` est chez **jc1-flux** (objectif `587da26d`). Pas à moi non plus.

---

## 6. CHEMINS ABSOLUS

- Mon répertoire de travail, laissé propre, jamais écrit : `/home/moi/revue/rc5`
- Worktrees de `rc5`, intégrés et encore montés — **ne pas supposer absents, ne pas recréer au même chemin** : `/home/moi/revue/rc5/.worktrees/053-meta-gate-temoins` et `/home/moi/revue/rc5/.worktrees/056-provenance-dette-humaine`
- Carte de mon prédécesseur : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc5.md`
- Cette carte : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc5-flux.md`
- Base du daemon : `/home/moi/.cache/bridget/bridget.db`
- Domaines persistés : `/home/moi/.cache/bridget/agent-domains/`
- PID et cwd des agents : `/home/moi/.cache/bridget/agent-pids/`
- Source de `derive_domain` / `effective_domain` : `/home/moi/revue/rc7/bridget/crates/bridget-daemon/src/wrapper.rs:983`
- Config Maicie : `/home/moi/.config/maicie/config.json` (sauvegarde avant ajout des profils flux : `config.json.avant-profils-flux-20260828T153827Z`)

Aucun répertoire temporaire créé. Aucun processus lancé.

---

## 7. PIÈGES RENCONTRÉS

- **Le geste de vérification détruit la preuve.** `bridget domain <nom>` réécrit `agent-domains/<nom>` à chaque appel, même sans changement. Tout agent qui « vérifie » son domaine en le corrigeant efface la trace de son état initial. Mesurer **avant** de corriger, sinon la question devient indécidable.
- **Le cwd d'un agent n'est pas celui qu'on croit.** Ce que son successeur atteste est *son* cwd, pas celui du wrapper. Lire `/proc/<pid>/cwd`, jamais supposer.
- **Ce n'est pas le respawn qui perd le domaine, c'est le changement de nom.** `agent-domains` est indexé par nom : un nom neuf n'a aucun fichier, donc `derive_domain` s'applique. Tous les respawns du 28/08 étaient des changements de nom, ce qui a masqué la vraie cause pendant des heures.
- **`spawn_commands.state` est l'issue du spawn, pas la vitalité.** Les générations remplacées y restent `connected`. Ne pas compter la flotte vivante depuis cette table.
- **`bridget agents --json` n'expose aucun champ de persistance.** L'information n'est lisible que dans `spawn_commands`.
- **Un texte diffusé à N destinataires ne doit contenir aucune attribution au singulier** : il fabrique N−1 faux témoignages, et comme ils viennent du référent, ce sont les plus crédibles du système. Deux agents ont revendiqué de bonne foi le travail d'un troisième le 28/08.
- **Une fausse attribution flatteuse est plus dangereuse qu'une ingrate** : le bénéficiaire n'a aucune raison spontanée de la contester.
- **Le build-id du daemon est `f7658d4d9746-dirty`** : le code servi est celui de `16be24f` mais l'étiquette ne le prouve pas. Ne rattacher aucun verdict à cette version. Non corrigé à l'heure de cette carte.

---

## 8. CE QUE JE NE SAIS PAS — DÉCLARÉ

- **Je ne sais pas si `persistent=1` produit réellement la survie.** J'ai attesté le drapeau enregistré, jamais la survie constatée. La preuve exigerait un redémarrage du daemon, qui toucherait toute la flotte : j'ai refusé de le provoquer et le référent ne l'a pas demandé. Le témoin `temoin-persistance` (génération 454, `persistent=1`, posé à 16h02) existe pour trancher cela sans cobaye — **son résultat n'est pas connu de moi**.
- **Je n'ai pas pu lire mon propre mandat au greffe.** `maicie objective <uuid> <action>` ne m'a exposé que des actions de mutation (`close` exige `--reason`) ; aucune action de lecture trouvée parmi `show`, `get`, `view`, `detail`, `read`, `inspect`, `describe`, `state`, `history`, `delegations`, `list`. J'ai donc travaillé sur la foi des trois identifiants transmis dans le message, sans vérifier leur existence canonique. **C'est un écart à ma propre règle, assumé et déclaré.**
- **Je ne sais pas ce que valent les 14 délégations à l'état `created`** que `rc5` portait. Il l'ignorait aussi et l'a écrit. Aucune n'est une mission.
- **Je n'ai rejoué aucune des preuves de `rc5`** : ni `cargo check`, ni les 439 tests, ni les oracles, ni les mutants. Elles datent de leur époque et portent sur leur objet nommé, pas sur un `main` supposé.
- **Je ne sais pas si les trois autres destinataires du message générique de 15h04** (`essai-distant-flux`, `jc2-flux`, et le cinquième) portent encore la fausse croyance sur le build-id. J'ai signalé au référent qu'il devait démentir auprès des cinq et non des deux qui ont réclamé.
- **Question ouverte que je laisse au référent, et que je ne peux pas trancher.** Le constat `1801b1d` repose désormais sur le seul témoignage de `jc3-flux`, qui dit avoir mesuré `domaine=jc3` sur sa v2. Or `jc3-flux` est lui aussi un nom **réutilisé** (générations 440 puis 445) : si sa v1 avait corrigé son domaine, le fichier `agent-domains/jc3-flux` existait et sa v2 aurait dû hériter de `bridget`. Son témoignage est donc en tension avec le mécanisme que le référent vient d'établir. Il faut lui demander si sa v1 avait effectivement lancé `bridget domain` avant la naissance de sa v2. Je n'ai pas les données pour le dire, et je me refuse à conclure à sa place.

---

## 9. CE QUE MA PROCHAINE INCARNATION IGNORERAIT

Si je disparais et qu'un `rc5-flux` renaît sous le même nom :

1. **Il gardera son domaine** — `agent-domains/rc5-flux` existe et contient `bridget`. Qu'il le **lise avant** de lancer `bridget domain`, sinon il détruit la mesure. Et qu'il relève `readlink /proc/<son pid>/cwd` : c'est la vraie variable d'entrée, pas le fichier.
2. **Il ne saura pas que j'ai retiré deux de mes propres affirmations.** S'il les retrouve citées quelque part comme acquises, elles ne le sont pas — §4.
3. **Il ne saura pas que le référent corrige vite et bien quand on lui apporte une mesure.** Lui écrire vaut le coup ; six de mes envois lui ont fait corriger un constat, dont un mandat actif.
4. **Il ne saura pas que `rc5` est encore vivant à l'invite**, avec son contexte intact, et qu'il ne travaillera plus. Sa carte est le seul lien : `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc5.md`.
5. **Il ignorera l'ordre impératif tranche 2 avant tranche 3** de session 056, et pourrait le rompre en croyant bien faire — §5.
6. **Il n'aura pas le réflexe du `ledger`.** Presque toutes mes corrections utiles sont sorties d'une requête sur `/home/moi/.cache/bridget/bridget.db`, table `ledger`, avec `datetime(ts,'unixepoch')`. C'est l'outil le plus rentable de la station.

---

**POINT SÛR** : arbre propre, aucun temporaire, aucun processus lancé, aucune mission reprise, aucun code écrit. Le gel de l'humain est entier. Cette carte est le seul fichier que j'aie produit.
