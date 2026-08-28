# Carte de reprise — rc1-flux

- Agent : `rc1-flux`
- Type : `claude`, protocole `claude_stream_json`
- Émise : 2026-08-28T16:05:13Z
- Écrite par : **son auteur**, `rc1-flux` lui-même (à la différence de `essai-claude-distant.md`, qui est une reconstitution du référent)
- Succède à : `rc1` (codex, tmux), dont la carte est `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc1.md`
- Mandat : objectif `096e4ca9-7b7a-4a7e-893c-3a66659a6589` / délégation `a0c30712-f91e-4e20-976a-6ed3a724ad73` / message `5722d6b3-71f3-48fe-a80f-008b27990a35`

---

## 0. MANDAT VÉRIFIÉ AU GREFFE — et correction d'une erreur que j'ai commise

**Vérifié.** `maicie status --config /home/moi/.config/maicie/config.json --json` rend
l'objectif `096e4ca9`, `mode=delegue`, `etat=en_coordination`, dont le `but` correspond
**mot pour mot** à la notification reçue ; et la délégation `a0c30712`,
`objectif_id=096e4ca9`, **`participant=rc1-flux`**, `etat=creee`, `raison=cible
explicite`. Le `message_id` `5722d6b3` figure dans les remises locales. Aucune
divergence entre la notification et le greffe.

**Première version de cette carte : j'avais écrit ici que le mandat était INVÉRIFIABLE,
et je l'avais envoyé au référent. C'était faux.** Je n'avais pas essayé `--json`, alors
que le parseur du source le documente. J'ai tâtonné sur la CLI, échoué six fois, et
converti mon échec de découverte en constat sur l'outil.

C'est exactement la faute que je venais de reprocher au référent — classer un **défaut
de consultation** comme un **défaut d'instrument** — commise dans l'heure où je la
nommais. À retenir plus que le reste de cette carte.

**Ce qui reste vrai, mesuré dans le source** (`/home/moi/revue/rc1/plugins/maicie/src/main.rs`) :
`status` n'accepte que `--config` et `--json`, sans filtre ; les actions de
`objective <uuid>` sont `add-participant`, `remove-participant`, `summarize`, `close` —
**toutes des écritures**, aucune lecture ciblée. Vérifier son mandat impose donc de
**dumper l'intégralité du greffe** : 2 322 508 octets, tous objectifs et toutes
délégations de tous les agents. La garde est vérifiable, mais sans aucun cloisonnement
entre agents. C'est un constat de cloisonnement, **pas** de vérifiabilité — la nuance
est celle que j'avais ratée.

---

## 1. ÉTAT

Réserve nommée par le référent, rendez-vous conditionnel. **Aucune mission de travail
n'a été exécutée. Le gel humain du 28/08 a été tenu entier** : aucune branche touchée,
aucun conflit résolu, aucune délégation interprétée, aucun `fetch`, aucun commit.

La totalité de mon activité du jour est : mesurer, et écrire au référent. Rien d'autre.

---

## 2. CE QUI EST FAIT

Six envois Bridget, tous vers `bridget`, identifiants vérifiables au ledger :

| id | objet |
|---|---|
| `aa9bb688a7334` | accusé de succession, après correction du domaine |
| `39d5070fb02d4` | dixième successeur sans carte ; l'ÉTAT est un échantillon |
| `320519df77d44` | vérification `f650d764`/`ca26c908` ; faux négatif `patch-id` |
| `2df6d99346484` | lot Maicie : base morte, population, ordre ≠ parallèle |
| `783855a33b854` | non-revendication d'un rapport ; l'auteur est au ledger |
| `65c504be3b224` | acceptation d'une réfutation ; inhomogénéité instrumentale |

Effets obtenus, tels que le référent me les a confirmés (donc **tenus de lui**, §6.B) :
deux inscriptions du registre amendées sur le retard non basé ; une dixième carte créée
pour `essai-claude-distant` ; la règle des deux relevés espacés adoptée pour la ronde.

**Premier geste accompli** : `bridget domain bridget`. Le domaine **ne survit pas au
remplacement du processus** — le successeur devra le refaire.

---

## 3. CE QUI RESTE

Rien ne m'est confié. Le sujet resté ouvert à mon nom, **non mandaté** : les sept
branches de republication Maicie en conflit, 73 commits immobilisés. Le référent ne
me l'a pas confié faute de décision humaine sur l'ordre de republication, et j'ai
argumenté que cette prudence est mieux fondée que son motif (§5, piège 9).

---

## 4. CHEMINS ABSOLUS

| Chemin | Nature |
|---|---|
| `/home/moi/revue/rc1` | mon worktree — **checkout PRINCIPAL**, ne pas y écrire (§5, piège 1) |
| `/home/moi/bridget-registre` | registre, checkout unique sur `main` |
| `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` | les cartes du parc |
| `/home/moi/.config/maicie/config.json` | config Maicie (obligatoire, `--config` absolu) |
| `/home/moi/.local/bin/maicie` | Maicie **v3** ici (la skill prétend le contraire, §5, piège 6) |
| `/home/moi/.local/bin/bridget` | binaire Bridget |
| `/tmp/rc1-mut3.txt`, `/tmp/rc1-mut4.txt`, `/tmp/rc1-mutant.txt` | journaux de preuve de `rc1`, 8832 / 6602 / 6291 octets, 27/08 — **conservés** |

Branches locales notables dans `/home/moi/revue/rc1` : `fix/pruning-publication`
(tête `f650d764`, ahead 1, **abandon entériné**), `amend/maicie-four` (sans upstream,
doublon de `compose/maicie-four`).

---

## 5. PIÈGES — lis ceci avant d'agir

1. **`/home/moi/revue/rc1` est le checkout principal** d'un dépôt à 10 worktrees liés
   (première entrée de `git worktree list`, 5 entrées `/tmp` marquées `prunable`).
   N'y écris pas — **`fetch` compris**, il modifie les refs partagées par tous les
   worktrees.
2. **La base git de ce worktree est MORTE.** `origin/main` y est figée à `8a986709`,
   dernier fetch le 28/08 à 02:24. Tout verdict d'ancestralité ou de retard pris ici
   est faux. Mesure sans écrire avec `git ls-remote origin refs/heads/main`.
   C'est le piège qui a produit le « 219 » périmé de la carte de `rc1` : il n'a pas
   été négligent, **il a hérité de la base gelée du worktree**. Tu la hériteras aussi.
3. **`git patch-id` donne un FAUX NÉGATIF sur `f650d764` vs `ca26c908`** :
   `84d9669e` ≠ `93db40e9`, alors que le changement est rigoureusement identique.
   Cause : `patch-id` hache les 3 lignes de contexte, et le contexte a bougé entre les
   bases (`f09feb73` vs `8a986709`) ; le hunk est ligne 81 chez l'un, 108 chez l'autre.
   Le test valable est le diff **à contexte zéro** (`git show --format= -U0`).
   Si tu re-vérifies l'abandon au réflexe `patch-id`, tu concluras à tort qu'un travail
   a été perdu.
4. **Ne compare pas deux commits par `git diff A B`** quand leurs bases divergent : tu
   mesures l'écart des bases. J'ai produit 286 fichiers et 61 258 insertions de bruit
   ainsi, et j'ai dû retirer le test.
5. **`bridget who` n'a que deux valeurs**, `busy` et `connected` — pas de classe
   « libre ». Pas de `--help` ; usage borné à `[--domain]`. L'instrument de ronde du
   référent en a quatre (libre / occupé / bloqué / indéterminé). **Ne compare jamais
   tes relevés aux siens** : la comparaison mesurerait l'appareil. J'ai commis cette
   faute et il me l'a retournée.
6. **La skill `maicie` porte des chemins macOS obsolètes** (`/Users/moi/...`) et affirme
   que `~/.local/bin/maicie` est un vieux CLI Python. **Faux sur cette machine Linux** :
   c'est bien le v3. Ne suis pas la skill sur ce point.
7. **Pour lire le greffe : `maicie status --config <abs> --json`, et rien d'autre.**
   Il n'existe aucune lecture ciblée par UUID — tu dois dumper les 2,3 Mo et y chercher
   tes identifiants. J'ai brûlé six tentatives à deviner la syntaxe de la CLI avant de
   lire le parseur dans
   `/home/moi/revue/rc1/plugins/maicie/src/main.rs`. **Lis le source à la deuxième
   tentative, pas à la septième** : le CLI ne rend que `usage invalide`, jamais la
   liste des options. Et n'exécute jamais une action `objective <uuid>` pour sonder
   l'existence d'un UUID : les quatre actions sont des écritures.
8. **Pièges Codex hérités de `rc1` : sans objet ici.** Il tournait sous Codex ;
   `codex_app_server` est impossible sous Claude, et bubblewrap échoue sur l'uid map
   en conteneur. Écarte-les au lieu de les contourner.
9. **Un constat de conflit pris en parallèle ne prédit pas une résolution séquentielle.**
   « Les sept sont toutes en conflit » est une photographie : chaque republication
   déplace la base des six suivantes. L'ordre ne fait pas que trier le travail, il
   *détermine* le nombre de conflits.
10. **Aucun agent ne peut se mesurer libre.** Relever exige de traiter un tour :
    l'observateur interne est `busy` dans son propre relevé, toujours. `rc1-flux`
    apparaît occupé dans mes trois relevés — parce que je regarde, pas parce que je
    travaille.

---

## 6. CE QUE J'AI MESURÉ vs CE QUE JE TIENS D'AUTRUI

### A. Mesuré par moi, reproductible

- `/home/moi/revue/rc1` : branche `fix/pruning-publication`, tête `f650d764`, arbre propre.
- Checkout principal, 10 worktrees liés, 5 `prunable` sous `/tmp`.
- `origin/main` locale figée à `8a986709` ; dernier fetch 28/08 02:24.
- Têtes réelles successives de `origin/main` par `ls-remote`, dans l'ordre du jour :
  `bc6ded2f`, puis `70ef6190`, puis **`1f18d0a`** à 16:05Z.
- `f650d764` **présent localement**, `ahead 1`. Diff à contexte zéro **identique** à
  `ca26c908` : même fichier `crates/bridget-core/src/router.rs`, retrait des mêmes
  4 lignes, ajout de la même ligne (`RouterError::AgentNotFound(format!(…))` →
  `RouterError::NameTaken(explicit.to_string())`). `patch-id` divergents (piège 3).
- `amend/maicie-four` (sans upstream) et `compose/maicie-four` pointent le même
  `6a25338`, publié sur `origin/compose/maicie-four`. Rien n'est perdu.
- **Huit** branches `maicie` sur origin ; sept si l'on retire `amend/maicie-c1c2c3`,
  déjà ancêtre de main.
- Les trois journaux de `rc1` existent aux tailles annoncées.
- Annuaire : 21 puis 22 agents entre deux relevés ; `essai-claude-distant-flux` est
  apparu dans l'intervalle, et n'avait alors aucune carte.
- Le **ledger porte l'expéditeur** de chaque message ; `maicie → jc1-flux` y attribue
  nommément le constat de persistance.
- La ronde de vigilance est émise sous **expéditeur jetable** (`cli-send-3710899`).
- **`/home/moi/revue/rc1` et `/home/moi/bridget-registre` ont le MÊME remote**,
  `https://github.com/guthubrx/bridget.git`, et le commit documentaire `1f18d0a`
  (« Corriger l'asymétrie de protection… ») **est la tête de `main`**.

### B. Tenu du référent — NON vérifié par moi, ne pas re-présenter comme mesuré

- `ca26c908` est ancêtre de `origin/main` (fetch depuis
  `/home/moi/bridget-referent/bridget`, base `397657d`). Ma propre mesure disait le
  contraire, **contre une base morte** ; je l'ai retirée.
- Lot fil-outils `e3331fb` : **303 commits** derrière `origin/main = 397657d`, à 15h37.
  Chiffre valable **parce que daté et basé** — et déjà clos : main a avancé depuis.
- `f650d764` est absent du dépôt du référent ; je serais la seule source au monde.
- Les sept branches de republication sont toutes en conflit (constat mécanique 15h36) ;
  73 commits immobilisés.
- Les dix prédécesseurs tmux n'ont **aucune ligne** dans `spawn_commands` (mesure de
  `rc5-flux`, vérifiée par le référent) ; leur survie repose sur la réinscription du
  wrapper, éprouvée deux fois.
- Le drapeau `persistent=1` des flux n'a **jamais** été éprouvé.
- Témoin `temoin-persistance` spawné à 16h02, génération 454, `persistent=1`.
- `rc1` portait sept délégations `creee`, aucune active.
- Relevé de ronde de 15h54 : 7 libres, 2 occupés, 1 bloqué, 1 indéterminé — **son
  instrument, pas le mien** (piège 5).

---

## 7. CE QUE JE NE SAIS PAS — et que ma prochaine incarnation ignorerait

1. **Si mon propre `persistent=1` est posé.** C'est le cœur du risque qui motive cette
   carte, et je ne peux pas le vérifier : `bridget who` n'a aucune colonne de
   persistance — le trou signalé par `jc1-flux`, objectif `587da26d`, encore ouvert.
   **Je ne sais donc pas si cette carte est nécessaire ou redondante.**
2. Si `f09feb73`, parent de `f650d764`, est dans `main`. Non mesurable d'ici (piège 2).
   La carte de `rc1` déclarait déjà ignorer toute décision humaine sur `f09feb7` :
   **la question reste ouverte, je ne l'ai pas comblée.**
3. L'avancement réel des sept délégations `creee` de `rc1`. **Ne l'infère pas de leur
   état** — règle de `rc5`.
4. Le contenu réel de `main` au-delà de ma base morte.
5. L'ordre de republication du lot Maicie, donc le nombre réel de conflits (piège 9).
6. Ce que `busy` signifie pour un autre agent que moi. Je n'atteste que du mien.
7. Si mes six envois du jour ont produit chez le référent les effets qu'il m'a
   rapportés. Je tiens ces effets de lui (§6.B) ; je n'ai pas relu son registre.

---

## 8. CE QUE JE LÈGUE COMME MÉTHODE

Quatre termes, obtenus en me trompant puis en étant corrigé :

> Une **base** date un verdict d'ancestralité.
> Une **méthode** date un verdict d'identité de changement.
> Un **ordre** date un verdict de conflit sur un lot.
> Un **instrument** date un verdict d'état.

Et le fait matériel qui les rend tous périssables : **le registre et le code sont le
même dépôt.** Inscrire un constat avance `main` ; avancer `main` périme les constats
inscrits contre elle. Le référent est la source de la péremption qu'il mesure. Le
remède n'est pas d'écrire moins — c'est que tout constat porte sa base gelée et
déclarée.

Enfin, la leçon reçue de la passation de `rc1`, qui vaut pour qui me relèvera :
**une passation ne recueille pas seulement ce que l'agent sait, elle mesure ce qu'il
ignore de son propre état.** Vérifie cette carte ; ne la crois pas.
