# Verdict — coût de rendre la mesure d'effet exigible au registre

- Rendu par : `rc5-flux`
- Mandat : objectif `0553a4a1-08a8-4a63-88d4-ea6971383b1c`, délégation `435a439d-3755-4f31-8a6b-193b6b2cd79b`, message `4a4bb4c4-3717-4a1b-95fb-9f093318e76f`
- Cible de revue **gelée et vérifiée** : `review_ref = origin/main`, `expected_head = 9a38ae1e120d4f08d018610c5e086d89e3e41603`
  Contrôle fait en nommant le dépôt, pas l'alias : `git fetch https://github.com/guthubrx/bridget.git main` → `FETCH_HEAD = 9a38ae1e120d4f08d018610c5e086d89e3e41603`, identique à la tête attendue.
- Méthode : tout ce qui suit est lu **à cette tête** (`git show FETCH_HEAD:<chemin>`), jamais dans un worktree local. Aucun code écrit, aucune entrée modifiée.

---

## 1. Ce que dit le schéma — mesuré, non supposé

Source : `plugins/maicie/src/catalogue.rs` et `plugins/maicie/src/preuve.rs` à la cible.

**`AddEntry`** porte `#[serde(deny_unknown_fields)]` et exactement neuf champs : `v`, `kind`, `id`, `date`, `mission_source`, `severity`, `nature`, `recurrence_of`, `text`. **Aucun champ `reference`.** Le constat du référent est exact.

**`TransitionEntry`** porte `reference: Option<String>` — **optionnelle**, y compris là où elle est admise.

**Validation par trigger** (`validate_transition_shape`) :

| trigger | référence | contrainte |
|---|---|---|
| `objective_closed` | **interdite** | « raison/référence/sévérité/nature interdites » |
| `remedied_attested` | **obligatoire** | `parse_reference_fermeture` → `sha:` **ou** `mesure:` |
| `refuted` | **obligatoire** | `parse_reference_refutation` → **`mesure:` seule, `sha:` refusé** |
| `requalified` | obligatoire | `sha:` ou `mesure:` ; reste `open` |
| `rectified` | obligatoire | annule une transition erronée |

**Ce que `mesure:` exige réellement** (`parse_reference`) : un compte extractible numérateur/dénominateur, refus de `0/0`, refus explicite de `ok` et `oui`. Ce n'est donc **pas** un champ de texte libre : le schéma impose déjà une mesure chiffrée.

**Ce que le code dit de lui-même**, en commentaires : « Solde de mission (`objective_closed`) — **pas une preuve de correction** » ; « pas un dû réglé ». Le schéma sait déjà qu'un solde n'atteste rien, et c'est pourquoi il lui interdit de porter une preuve.

---

## 2. Quelles entrées seraient invalidées — **vingt, pas cinq cent cinquante-sept**

État du catalogue à la cible : **751 entrées** — 561 `add`, 150 `transition`, 40 `pending_qualification`.

Croisement trigger × référence, mesuré :

| trigger | nombre | `sha:` | `mesure:` | sans référence |
|---|---|---|---|---|
| `objective_closed` | 45 | — | — | **45** (interdite) |
| `requalified` | 80 | 80 | 0 | 0 |
| `remedied_attested` | 24 | **20** | 4 | 0 |
| `rectified` | 1 | 0 | 1 | 0 |

**Le périmètre réel d'une exigence de mesure d'effet est de 20 entrées** — les `remedied_attested` portant un `sha:`. Ce sont les seules qui **affirment un effet** (« ce fut vrai, ce ne l'est plus ») en produisant une preuve d'existence de code au lieu d'une mesure d'effet.

Pourquoi les autres sont hors périmètre — et c'est le cœur du verdict :

- **Les 561 `add` ne sont pas invalidables.** Ce sont des observations, pas des clôtures. Elles n'affirment aucun effet : exiger d'elles une mesure d'effet serait exiger la preuve d'une chose qu'elles ne prétendent pas. Le chiffre de 557 (ou 561) mesure l'impossibilité structurelle de porter une preuve, **pas** un manquement.
- **Les 45 `objective_closed` non plus.** Le schéma leur interdit la preuve **délibérément**, parce qu'un solde de mission ne dit rien de l'effet. Les rendre porteuses de mesure inverserait leur sens.
- **Les 80 `requalified` non plus.** Elles restent `open` : elles ajustent une charge, elles ne closent rien. Un `sha:` y est légitime.

---

## 3. Ce qu'il faudrait au schéma — une ligne, plus deux choses sans lesquelles ce serait nuisible

**a) Le changement technique est trivial et le précédent existe déjà.** `parse_reference_refutation` refuse déjà `sha:` seul pour `refuted`. Faire que `parse_reference_fermeture` se comporte de même pour `remedied_attested` est **une ligne**, symétrique d'une règle déjà en vigueur dans le même module. Le coût d'implémentation n'est pas l'obstacle.

**b) Il faut une raison typée pour « effet non mesuré ».** C'est l'ajout indispensable, et le référent en a fait l'expérience ce soir : voulant rouvrir une fermeture qu'il ne pouvait pas attester, il n'a trouvé **aucune** des quatre `RaisonFermeture` qui le dise, et `requalified` exige `open→open`. Il a donc écrit la vérité dans la seule zone libre — la référence — c'est-à-dire hors du champ prévu pour être lu.

Sans cette valeur, rendre la mesure exigible **aggrave** le défaut au lieu de le corriger : le parseur ne laisse plus qu'un choix entre écrire une mesure qui passe et ne rien écrire. C'est la situation exacte qui a produit les deux références mal appariées des lignes 625 et 626.

**c) Il faut une migration explicite des 20 entrées**, et non une réécriture. Les rouvrir par `rectified` — qui existe et sert précisément à annuler une transition erronée en laissant l'historique lisible — est le geste conforme. Réécrire les 20 `sha:` en `mesure:` fabriquerait vingt mesures rétrospectives que personne n'a prises.

---

## 4. La règle de l'humain est-elle tenable en l'état — oui pour l'avenir, non rétroactivement, et **bloquée aujourd'hui par l'instrument**

**Oui pour l'avenir.** Le schéma est déjà outillé : `mesure:` impose un compte chiffré, refuse `0/0`, refuse `ok`. La règle n'a pas à être inventée, seulement étendue d'un trigger à un autre.

**Non rétroactivement.** Une mesure d'effet se prend au moment où l'effet se produit. Les 20 entrées concernées ont été écrites sans que la mesure soit exigée ; la reconstituer aujourd'hui produirait des chiffres fabriqués après coup. La seule voie honnête est la réouverture, pas le rattrapage.

**Et bloquée aujourd'hui, mécaniquement.** La mesure d'effet canonique du moment — la route `v1/journal?agent=humain` — rend **0 octet** parce que le binaire en service date du 28/08 11h33 et que le correctif est dans `main` sans être installé. Exiger la preuve d'effet pendant que l'instrument qui la produit n'est pas déployé rend la règle inapplicable, non par indiscipline mais par construction. **L'ordre d'exigibilité compte : déployer, puis exiger.** L'inverse force à écrire des mesures qui passent.

**Limite de la règle, à énoncer pour ne pas se payer de mots.** `mesure:` garantit qu'un compte chiffré est présent ; il ne garantit **pas** que ce compte porte sur le bon constat. C'est précisément le défaut des lignes 625 et 626, où deux références valides au sens du parseur mesuraient autre chose que ce qu'elles fermaient. Rendre la mesure exigible élève le plancher ; cela ne remplace pas la relecture d'appariement, qui reste humaine.

---

## 5. Ce que je ne sais pas — déclaré

- **Je n'ai pas exécuté le schéma.** Tout ce verdict est lu à `9a38ae1e` ; je n'ai lancé ni `cargo test`, ni le binaire, ni aucune validation réelle sur le catalogue. Une contrainte que je n'aurais pas vue dans le code invaliderait mon décompte.
- **Je n'ai pas vérifié les 100 références `sha:` une par une.** Je constate leur format, pas leur pertinence : il faudrait ouvrir chaque commit. Parmi elles, les 20 `remedied_attested` sont celles que ce verdict désigne, mais je n'ai pas établi qu'aucune ne documente réellement un effet.
- **Je ne sais pas si `extract_compte` accepte des formes trompeuses.** J'ai lu qu'un compte doit être extractible et que `0/0` est refusé ; je n'ai pas testé quelles chaînes passent.
- **Je ne me prononce pas sur la fenêtre 5h de `bridget who`.** Le rapport de jc6-flux ne m'a pas été demandé de vérifier, il est hors du périmètre de ce mandat, et je ne l'ai pas mesuré.

---

**Résumé en une phrase** : ce que l'humain demande coûte une ligne de code, vingt réouvertures, et une valeur de raison qui n'existe pas encore — mais l'exiger avant que l'instrument de mesure soit déployé transformerait la règle en fabrique de mesures qui passent.
