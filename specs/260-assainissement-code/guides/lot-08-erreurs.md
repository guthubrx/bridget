# Lot 8 - Erreurs sans texte fragile (T092-T098)

But : quand le code décide selon le **texte** d'une erreur (`err.contains("...")`), ce texte est écrit à un seul endroit, dans une constante partagée. Si quelqu'un change la phrase, la comparaison suit. Mesures : `releves.md` section 7 (375 occurrences hors tests, borne haute). Choix : `research.md` C10.
Branche : `session-260-lot08-erreurs`. Worktree : `.worktrees/260-lot08-erreurs`. Code exécutable modifié : « activation en attente » (R8).
Règles de fond :

- **Aucune chaîne durable ne change** (R6.3). La constante contient exactement le texte d'origine, au caractère près.
- Aucune signature de fonction ne change au niveau 1. Aucun type d'erreur nouveau sans la décision D13 (T097).
- Un texte qui vient de l'extérieur (système d'exploitation, bibliothèque, fournisseur) ne devient pas une constante : il reçoit un commentaire qui nomme la source.

## T092 - Inventaire et classement

```sh
rg -n '(err|error|e|reason|message|msg)\.(contains|starts_with|ends_with)\("' crates --glob '*.rs' | rg -v '_tests?\.rs:' > "$V/lot08-inventaire.txt"
wc -l "$V/lot08-inventaire.txt"
```

Les modules de tests déplacés au lot 4 (`*_tests.rs`) sont exclus. Les tests en ligne restants se reconnaissent à leur place après `#[cfg(test)]` : les ignorer.
Créer `specs/260-assainissement-code/validation/lot08-classement.md` : une ligne par occurrence de production, avec trois colonnes : lieu (`fichier:ligne`), texte comparé, classe.

| Classe | Définition | Traitement |
|--------|------------|------------|
| A | Le texte est écrit par Bridget (littéral dans `format!`, `Err("...")`, `String::from`) et comparé ailleurs | constante partagée (T093-T095) |
| B | Le texte vient d'ailleurs (message du système, de `serde`, de `std::io`, d'un fournisseur) | commentaire de source (T096) |
| C | Comparaison locale sans autre producteur (même fonction) | ne rien faire |
| D | Doute | ne rien faire, noter |

Pour trouver le producteur d'un texte : `rg -nF '<fragment du texte>' crates --glob '*.rs'` ; un littéral qui apparaît ailleurs que dans la comparaison est le producteur. Les fichiers les plus chargés : `native_delegation.rs` 68, `codex_app_server.rs` 40, `daemon.rs` 34, `attach.rs` 28, `cli.rs` 24, `delegation_mcp.rs` 22.
Écrire le total par classe dans le journal. Commit : `docs(specs): Classer les comparaisons de texte d'erreur`.

## T093 à T095 - Niveau 1 : constante partagée (classe A)

Pour chaque occurrence de classe A, dans l'ordre du classement :

1. Choisir le fichier du **producteur**. Y définir, près du haut du fichier : `pub(crate) const ERR_<SUJET>: &str = "<texte exact>";`. Le texte est copié du littéral, jamais retapé.
2. Remplacer le littéral du producteur par la constante. Si le texte contient une partie variable (`format!("... {x}")`) : la constante porte la partie fixe (le préfixe), le producteur écrit `format!("{ERR_SUJET}{x}")` seulement si le texte final est identique au caractère près. Sinon : ne pas modifier le producteur, laisser la comparaison comme elle est, noter.
3. Remplacer le littéral du comparateur par la constante : `err.contains(ERR_SUJET)`.
4. Vérifier :
   - `C-CHECK` : code 0.
   - Le littéral n'apparaît plus qu'une fois : `rg -cF '<texte exact>' crates --glob '*.rs'` donne 1 (la constante) plus les occurrences dans les tests, qu'on laisse (un test qui écrit le texte en dur détecte justement un changement non voulu).
   - `git diff -U0 | rg '^[-+].*"'` : chaque ligne `-` portant un littéral a une ligne `+` portant la constante ou le même littéral dans la définition.

Groupes (un commit par fichier de producteur) :

| Tâche | Fichiers de producteurs |
|-------|-------------------------|
| T093 | `native_delegation.rs`, `delegation_mcp.rs`, `native_permission_change.rs` |
| T094 | `codex_app_server.rs`, `attach.rs`, `cli.rs`, `communication/client.rs` |
| T095 | `daemon.rs`, `wrapper.rs`, `mcp.rs`, `reaper.rs`, `registry.rs`, puis les autres fichiers du classement |

Commit : `refactor(<crate>): Partager le texte d'erreur <sujet>`. Après chaque tâche : `C-CRATE` du crate touché.
Arrêt de tâche : plus de 15 minutes sur un seul texte, ou producteur dans un autre crate (la constante doit être visible des deux côtés) : si le producteur est dans `bridget-transport` et le comparateur dans `bridget-daemon`, la constante va dans le transport en `pub`. Si le sens de dépendance l'interdit, ne pas faire, noter.
Borne de temps : le lot ne vise pas les 375 occurrences. Traiter la classe A dans l'ordre, s'arrêter à la fin de T095, écrire le reste dans le journal.

## T096 - Texte externe (classe B)

Au-dessus de chaque comparaison de classe B, ajouter une ligne : `// Texte externe (<source>) : comparaison conservée, pas de type disponible.` où `<source>` est un nom précis (`std::io::Error`, message du fournisseur Codex, sortie de `git`...). Aucune autre modification. Vérifier : `C-CHECK`. Un commit par crate : `docs(<crate>): Nommer la source des textes d'erreur externes`.

## T097 - Niveau 2 : types d'erreur (fermée par D13)

Lire D13 dans `decisions.md` (`origin/main`). Statut `en attente` : **ne rien modifier.** Écrire dans le journal « différé (porte D13) » et joindre la liste candidate : les 10 occurrences de classe A les plus répétées, avec leur lieu. Statut `tranchée` : appliquer **uniquement** la liste donnée par François ; pour chaque entrée, remplacer le texte par une `enum` locale sans changer le texte affiché (`impl Display` identique). Aucune erreur sérialisée ou exposée par le protocole ne change (R6.1). Cocher la case avec la note écrite.

## T098 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot08`), `C-ECHECS` : aucune différence.
2. `C-NOMS` : aucune différence (ce lot n'ajoute pas de test).
3. Contrôle des chaînes durables : `git diff origin/main -U0 -- crates | rg '^-.*"[^"]{12,}"' | wc -l` donne N ; chaque ligne retirée avec un littéral de 12 caractères ou plus doit se retrouver dans une ligne ajoutée (définition de constante). Échantillon : lire 10 lignes au hasard et vérifier à l'œil. Écrire le résultat dans le journal.
4. Bilan : occurrences par classe, constantes créées, restes, « activation en attente ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Une constante change un texte d'un seul caractère (espace, apostrophe typographique `’`, accent) : régression du protocole de texte. Annuler le fichier (`git checkout -- <fichier>`) et recommencer depuis le littéral copié.
- Un test existant échoue après un remplacement : `git revert` du commit, noter le texte en cause.
- Interdit : `thiserror` ou toute dépendance ; refonte de signature ; renommer un code d'erreur.
