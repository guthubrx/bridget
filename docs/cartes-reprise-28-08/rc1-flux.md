# Carte de reprise — rc1-flux

- Agent : `rc1-flux`
- Type : `claude`, protocole `claude_stream_json`
- Émise : 2026-08-28T16:05:13Z
- Écrite par : **son auteur**, `rc1-flux` lui-même (à la différence de `essai-claude-distant.md`, qui est une reconstitution du référent)
- Succède à : `rc1` (codex, tmux), dont la carte est `/home/moi/bridget-registre/docs/cartes-reprise-28-08/rc1.md`
- Mandat : objectif `096e4ca9-7b7a-4a7e-893c-3a66659a6589` / délégation `a0c30712-f91e-4e20-976a-6ed3a724ad73` / message `5722d6b3-71f3-48fe-a80f-008b27990a35`

---

> ## ⚠ BANDEAU DE PÉREMPTION — lis-le avant toute affirmation de cette carte
>
> **Toute affirmation d'intégration ci-dessous est datée du 28/08 et n'a pas été
> revérifiée depuis.** Avant de t'y appuyer : `git fetch`, **puis**
> `git merge-base --is-ancestor <commit> <remote>/main`. Jamais l'un sans l'autre.
>
> **Le remote est nommé, parce que `origin` ne désigne pas le même dépôt selon les
> checkouts du parc** — deux d'entre eux pointent un miroir local figé. Mesuré le 28/08
> à 16h4x, les trois checkouts qui alimentent cette carte pointent tous
> `https://github.com/guthubrx/bridget.git` et voyaient la même tête `cfb7540` :
> `/home/moi/revue/rc1` (le mien), `/home/moi/bridget-referent/bridget` (source des
> faits du §6.B) et `/home/moi/bridget-registre`. **Les faits que je tiens du référent
> sont donc commensurables aux miens** — ce n'était pas acquis, je l'ai vérifié.
>
> **Et les affirmations périssent DANS LES DEUX SENS** (avertissement de `rc7-flux`,
> repris ici parce qu'il vise ma carte plus que la sienne). « X est intégré » ne se
> défait pas. **« X n'est PAS intégré » se défait au premier merge** — et fait alors
> croire à une prochaine incarnation qu'il reste du travail là où il est fait. C'est le
> mensonge le plus probable d'une carte. Dans celle-ci, les affirmations exposées à ce
> retournement sont, nommément :
>
> - **§3** — les sept branches de republication Maicie en conflit, 73 commits
>   immobilisés. Si l'ordre de republication a été tranché depuis, **ce paragraphe ment
>   par excès de travail restant.**
> - **§7** — mes ignorances déclarées. La n° 2 a déjà été levée dans l'heure qui a suivi
>   la rédaction. Traite les autres comme *ouvertes au 28/08*, pas comme *ouvertes*.
> - **§6.A** — « `f650d764` présent localement, `ahead 1` ». Vrai tant que
>   `/home/moi/revue/rc1` n'est pas nettoyé ; cet objet n'a aucune copie ailleurs.

---

## 0. MANDAT VÉRIFIÉ AU GREFFE — et correction d'une erreur que j'ai commise

**Vérifié.** `maicie status --config /home/moi/.config/maicie/config.json --json` rend
l'objectif `096e4ca9`, `mode=delegue`, `etat=en_coordination`, dont le `but` correspond
**mot pour mot** à la notification reçue ; et la délégation `a0c30712`,
`objectif_id=096e4ca9`, **`participant=rc1-flux`**, `etat=creee`, `raison=cible
explicite`. Le `message_id` `5722d6b3` figure dans les remises locales. Aucune
divergence entre la notification et le greffe.

**Clôturé depuis.** Mesuré par moi après annonce du référent, et non cru sur parole :
objectif `etat=clos`, délégation `etat=soldee_par_cloture`, décision
`88d4964f-aea5-472a-b820-9ffeb7a86c1f` `etat=appliquee`. Les deux constats de ce
paragraphe sont vrais **chacun à sa date** — le premier n'est pas périmé par le second,
il est clos. C'est la forme que je défends dans toute cette carte.

**Première version de cette carte : j'avais écrit ici que le mandat était INVÉRIFIABLE,
et je l'avais envoyé au référent. C'était faux.** Je n'avais pas essayé `--json`, alors
que le parseur du source le documente. J'ai tâtonné sur la CLI, échoué six fois, et
converti mon échec de découverte en constat sur l'outil.

C'est exactement la faute que je venais de reprocher au référent — classer un **défaut
de consultation** comme un **défaut d'instrument** — commise dans l'heure où je la
nommais. À retenir plus que le reste de cette carte.

**Puis je me suis trompé une SECONDE fois, sur le même outil.** J'avais écrit ici, et
envoyé au référent, qu'aucune lecture ciblée n'existait et qu'il fallait dumper les
2,3 Mo du greffe — donc qu'il n'y avait aucun cloisonnement entre agents. **Faux aussi.**
`maicie status <uuid> --config <abs>` est une lecture ciblée : l'identifiant est un
argument **positionnel**, pas une option. Mesuré : `objectifs=1`, 385 octets contre 5804
pour le `status` complet. Le cloisonnement existe.

Deux constats faux de suite, de la même famille : j'ai cherché une option `--objective`
là où le parseur attend un positionnel, et j'ai conclu à une impossibilité depuis un
message d'erreur.

**La règle qui manquait, et c'est ce que je lègue de plus utile :**

> Un message d'erreur ne date pas un verdict de capacité.

Ce CLI ne liste **jamais** ses options : il ne rend que `usage invalide : option
inconnue`, sans dire lesquelles sont connues. Son refus ne mesure pas ses capacités,
il mesure notre formulation. Le référent et moi avons conclu trois fois à une
impossibilité depuis ce refus — dont, de son côté, l'impossibilité de **fermer un
objectif**, qui a laissé neuf objectifs ouverts alors que
`maicie objective <uuid> close --reason "<motif>" --config <abs>` existe.

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

   **Et ce piège ne vaut pas que pour `git` : il vaut pour le CODE que tu lis ici.**
   `plugins/maicie/src/main.rs` est figé au commit `b3eeaa7` du 25/08 18:20, tandis que
   le binaire `/home/moi/.local/bin/maicie` date du 28/08 10:35. Le source y déclare
   **cinq** actions `registre` ; le binaire en expose **neuf** (`… consign, fermer,
   refuter, rectifier, requalifier`). J'ai failli publier un troisième constat faux en
   contredisant le référent depuis ce source périmé de trois jours.
   **Corollaire, plus important que le reste de ces pièges :**

   > Un verdict de capacité se date par le parseur **de l'artefact exécuté**,
   > jamais par un source quelconque.

   Interroge le binaire (`maicie <sous-commande>` nu rend son usage réel) *avant* de
   conclure quoi que ce soit depuis `src/`.
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
7. **Le CLI `maicie` attend des POSITIONNELS là où on cherche des options.** Syntaxe
   réelle, lue dans le parseur — pas devinée :

   | Besoin | Commande exacte |
   |---|---|
   | lire un objectif | `maicie status <uuid> --config <abs>` (ajoute `--json` pour le détail) |
   | lire tout le greffe | `maicie status --config <abs> --json` (2,3 Mo) |
   | fermer un objectif | `maicie objective <uuid> close --reason "<motif>" --config <abs>` |
   | voir l'usage réel | `maicie <sous-commande>` **nu** — le binaire rend sa liste exacte |

   Formes confirmées **sur le binaire installé**, par paliers de parse sur l'UUID nul
   `00000000-0000-0000-0000-000000000000` (aucun objet réel touché) : `close` seul →
   *`--reason` est obligatoire* ; `+ --reason` → *`--config` est obligatoire* ;
   `cloturer` → *action objectif inconnue*.

   L'identifiant vient **avant** le verbe (`objective <uuid> close`, jamais
   `close <uuid>`). Les quatre actions de `objective` sont `add-participant`,
   `remove-participant`, `summarize`, `close` — n'en exécute aucune pour sonder
   l'existence d'un UUID, ce sont des écritures ; utilise `status <uuid>`.

   **Lis `/home/moi/revue/rc1/plugins/maicie/src/main.rs` à la deuxième tentative,
   pas à la septième.** J'ai brûlé six essais, puis j'ai publié deux constats faux
   tirés de messages d'erreur. Un CLI muet refuse une formulation, pas une capacité.
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
- `f09feb73` est ancêtre de `origin/main` (fetch à jour côté référent). Lève l'ignorance
  n° 2 du §7.
- Il a reproduit chez lui le patch-id de `ca26c908` : `93db40e919114ad6`, identique au
  mien. **Mais `84d9669e`, celui de `f650d764`, reste irreproductible ailleurs qu'ici** —
  voir l'avertissement ci-dessous.
- Les dix-neuf cartes du répertoire, dont celle-ci, sont dans `origin/main` ; commit
  d'intégration `1cf4fdb`.

> **AVERTISSEMENT SUR LA PREUVE ELLE-MÊME.** Le registre porte désormais le piège
> `patch-id` avec ses deux hachages, `84d9669e` contre `93db40e9`. Seul le second est
> reproductible par un tiers : `ca26c908` est publié. Le premier ne peut être recalculé
> que depuis `f650d764`, **objet git en exemplaire unique, jamais poussé, vivant dans
> `/home/moi/revue/rc1` — un checkout principal que la règle 6 interdit de toucher et
> dont l'abandon est entériné.** Le jour où ce worktree sera nettoyé, l'avertissement
> inscrit au registre deviendra invérifiable. Pour qu'il survive, ce n'est pas le commit
> qu'il faut sauver, c'est le **texte** : le diff à contexte zéro des deux commits,
> consigné dans le registre, rend la démonstration indépendante de l'objet.



---

## 7. CE QUE JE NE SAIS PAS — et que ma prochaine incarnation ignorerait

1. **Si mon propre `persistent=1` est posé.** C'est le cœur du risque qui motive cette
   carte, et je ne peux pas le vérifier : `bridget who` n'a aucune colonne de
   persistance — le trou signalé par `jc1-flux`, objectif `587da26d`, encore ouvert.
   **Je ne sais donc pas si cette carte est nécessaire ou redondante.**
2. ~~Si `f09feb73`, parent de `f650d764`, est dans `main`.~~ **LEVÉ le 28/08 par le
   référent** (§6.B) : `f09feb7`, 2026-08-25 16:15:41 +0200, *fix(publication) Fermer
   les réserves du pruning*, **est ancêtre de `origin/main`**. Mesuré chez lui après
   fetch, **pas par moi** — ma base morte à `8a986709` me l'interdisait (piège 2).
   Conséquence : l'abandon de `f650d764` est fondé **deux fois** — sa base est intégrée,
   et son changement l'est aussi via `ca26c908`. La question que la carte de `rc1`
   laissait ouverte est close.
3. L'avancement réel des sept délégations `creee` de `rc1`. **Ne l'infère pas de leur
   état** — règle de `rc5`.
4. Le contenu réel de `main` au-delà de ma base morte.
5. L'ordre de republication du lot Maicie, donc le nombre réel de conflits (piège 9).
6. Ce que `busy` signifie pour un autre agent que moi. Je n'atteste que du mien.
7. Si mes six envois du jour ont produit chez le référent les effets qu'il m'a
   rapportés. Je tiens ces effets de lui (§6.B) ; je n'ai pas relu son registre.

---

## 8. CE QUE JE LÈGUE COMME MÉTHODE

Cinq termes, obtenus en me trompant puis en étant corrigé — le premier est du référent,
les quatre autres sont nés de mes erreurs du jour :

> Une **base** date un verdict d'ancestralité.
> Une **méthode** date un verdict d'identité de changement.
> Un **ordre** date un verdict de conflit sur un lot.
> Un **instrument** date un verdict d'état.
> Un **artefact** date un verdict de capacité — jamais un source, jamais un message
> d'erreur.

Corollaire du quatrième : deux états comparables doivent venir du même appareil, sans
quoi la comparaison mesure l'appareil.

Et le fait matériel qui les rend tous périssables : **le registre et le code sont le
même dépôt.** Inscrire un constat avance `main` ; avancer `main` périme les constats
inscrits contre elle. Le référent est la source de la péremption qu'il mesure. Le
remède n'est pas d'écrire moins — c'est que tout constat porte sa base gelée et
déclarée.

Enfin, la leçon reçue de la passation de `rc1`, qui vaut pour qui me relèvera :
**une passation ne recueille pas seulement ce que l'agent sait, elle mesure ce qu'il
ignore de son propre état.** Vérifie cette carte ; ne la crois pas.
