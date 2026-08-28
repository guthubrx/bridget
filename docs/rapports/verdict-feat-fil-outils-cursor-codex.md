# Verdict — lot orphelin `feat/fil-outils-cursor-codex`

- **Juge** : `cartae0-flux`
- **Rendu** : 2026-08-28, ~17h45 UTC
- **Mandat** : objectif `819e161f-a978-47dc-a6b3-df1d02d3604a` / délégation `627d99fd-8352-48d1-9872-6390d76ee513` / message `e5002689-844a-4ffc-90de-05c88b3c0beb`
- **Demandé** : un verdict argumenté. **Non demandé** : aucune intégration, aucun merge, aucun rebase, aucune écriture sur une branche. *Aucun des trois n'a été fait.*

> **Ce document existe parce que deux remises de ce verdict par message sont restées `[en vol]` / `[indéterminé]`, et que le mandat m'a été renvoyé à l'identique.** Un fichier survit là où une remise se perd.

---

## VERDICT

> ## ✅ INTÉGRABLE EN L'ÉTAT
>
> Aucune adaptation de code n'est identifiée comme nécessaire.
>
> **La réserve initiale — « je n'ai pas vérifié la compilation » — est LEVÉE par mesure**, sans point de travail et sans merge : l'arbre fusionné `d96e9bc`, déjà calculé par `merge-tree`, a été extrait hors du dépôt dans `/tmp/fusion-lot` puis compilé.
>
> - `cargo check -p bridget-transport` → **`Finished` en 4,07 s**, zéro erreur.
> - `cargo check --tests -p bridget-transport` → **`Finished` en 2,35 s, exit 0** — les trois témoins ajoutés type-checkent aussi.
>
> *Piège évité au passage, et il mérite d'être noté :* le premier lancement avait rendu `EXIT=0` alors que `cargo` était **absent du PATH** — l'exit venait du `tail` en bout de pipeline, pas du check. **Un zéro n'est pas une mesure** (piège hérité de la carte de `cartae0`, §8). Le binaire réel est `/home/moi/.cargo/bin/cargo`.

---

## 1. Remote nommé — préalable, pas formalité

Tout ce qui suit porte sur **`github` = `https://github.com/guthubrx/bridget.git`**.

**Le `origin` du référent est le `github` de ce checkout** : leurs têtes coïncident (`e3b1c0b`). Le `origin` de ce worktree est `/home/moi/revue/bridget`, un **miroir local figé** qui n'a **ni la branche ni ce `main`**. Deux checkouts du parc sont dans ce cas ; sans nommer le remote, les deux énoncés sont incomparables.

`git fetch github` exécuté **avant** toute mesure d'ancestralité, conformément au mandat.

## 2. Mesures du référent — revérifiées, toutes exactes

| Mesure annoncée | Revérifiée | Verdict |
|---|---|---|
| tête `e3331fb` | `e3331fb99b1d40c5…` | ✅ |
| `main` `e3b1c0b` | `e3b1c0b1be369316…` | ✅ |
| base commune `71c7814` | `71c7814cd29c9a32…`, 27/08 05:29 | ✅ |
| un commit propre | 1 | ✅ |
| 366 commits de retard | 366 | ✅ |
| zéro marqueur de conflit | 0 | ✅ |
| 401 ajouts / 52 suppressions, 2 fichiers | `acp.rs` 349, `codex_app_server.rs` 104 | ✅ |

Aucune correction à apporter.

## 3. Ce que le lot apporte

Un **vrai défaut d'observabilité**, pas un confort.

- **Avant** : `item/commandExecution/outputDelta` — la **sortie stdout** — était journalisé comme **nom** d'acte. Le fil affichait donc la sortie à la place du nom de la commande.
- **Après** : le nom vient de `item/started` type=`commandExecution`, champ `params.item.command`. `outputDelta` n'est plus journalisé comme acte.
- **Mesure datée dans le commit lui-même** : codex app-server 0.149, 2026-08-27.

**Trois témoins** ajoutés, chacun avec son mutant nommé :
`TEMOIN_TOOL_acp_retranscrit_rawInput_et_titre_enrichi`, `TEMOIN_TOOL_acp_permission_remplace_MCP_tool_par_vrai_nom`, `TEMOIN_TOOL_codex_app_server_retranscrit_command_execution_nom`.

C'est la discipline des commits intégrés le 28/08.

## 4. Le point rapporté — vérifié, et il ne conclut pas dans le sens attendu

Traité comme **rapport**, pas comme fait, puis mesuré.

**Fait confirmé** : `approval_response` — **7** occurrences dans `main`, **0** dans le lot, **0** dans la base. Le lot ignore bien cette capacité, arrivée dans `main` après la base par `79c0c5f`.

**Mais ce n'est pas un défaut.** `main` n'a **aucun** bras `item/started` ; le lot en ajoute un. Ce sont **deux bras disjoints du même `match`**. *Ignorer n'est pas casser.*

**Preuve** : l'arbre fusionné `d96e9bca66f55aecf8b2e253d3e61a4e7f6f536f`, produit par `merge-tree`, contient **les deux** — 7 `approval_response` et 11 `item/started`, **0 marqueur de conflit**, 3700 lignes.

## 5. Ce que 366 commits ont changé — fichier par fichier

- **`acp.rs`** (349 des 401 lignes) : **identique à la base dans `main`**. Zéro dérive. Le lot s'applique sur le texte exact qu'il visait.
- **`codex_app_server.rs`** : `main` a ajouté 262 lignes par **deux** commits — `79c0c5f` (autorisation du pilote) et `e9ff6e7` (préservation de l'émetteur au démarrage du tour).
- **Signature de `record_active_act`** : **identique dans les trois** révisions (base / lot / main), sept paramètres. Aucune rupture d'API interne.
- **Le lot est auto-contenu** : cinq des six helpers qu'il appelle, il les introduit lui-même (`build_tool_journal_payload`, `permission_tool_journal_payload`, `compact_acp_raw_input`, `remember_tool_detail`, `remember_tool_title`). Le seul préexistant, `record_or_terminal`, existe toujours dans `main`.
- **Édition Rust 2024** : les let-chains employés par le lot sont valides.

## 6. Point de vigilance à contrôler en priorité

`e9ff6e7` préserve l'émetteur au **démarrage du tour** ; le lot ajoute un bras au **démarrage d'item**. Tour et item sont distincts et **aucune interférence n'est mesurée** — mais ce sont les deux seuls endroits du fichier qui traitent un *début*, ils sont voisins sémantiquement, et c'est là qu'il faut regarder en premier.

*Vigilance, pas défaut constaté. Je connais ce terrain : `e9ff6e7` est mergé dans `session-ui-correctifs`.*

## 7. Portée et péremption du verdict

**Ce verdict vaut contre `github/main` = `e3b1c0b`, à la date du 28/08, et périme dans les deux sens.**

Zéro conflit mesuré aujourd'hui contre cet état **ne dit rien** de ce que donnerait une republication après d'autres branches. Un ordre date un verdict de conflit — avertissement du référent, retenu et non contourné.

## 8. Pourquoi ce lot est orphelin

**Pas par défaut de qualité.** Il est mort par attrition de revue, hors périmètre cloud. Sa facture — mesure datée, témoin réel, mutant nommé, commentaire expliquant le *pourquoi* — est au niveau des commits intégrés le 28/08. Trente-six heures sans réclamant ne sont pas un jugement sur lui.

## 9. Ce que je n'ai pas fait

Aucune intégration, aucun merge, aucun rebase, aucune écriture sur une branche. J'ai fetché `github` (exigé par le mandat), lu des objets, et extrait **hors du dépôt**, dans `/tmp/fusion-lot`, l'arbre `d96e9bc` déjà calculé par `merge-tree` — ce qui n'est ni un merge, ni un rebase, ni une écriture sur une branche.

**L'intégration reste une décision humaine.**
