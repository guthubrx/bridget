# Lot 9 - Fonctions lisibles (T099-T113)

But : plus aucune fonction de plus de 1000 lignes ; les 9 fonctions de 15 paramètres ou plus reçoivent une structure d'arguments ; les autres fonctions longues sont listées avec leur sort. Mesures et liste complète des 43 fonctions : `releves.md` section 8 (`sed -n '99,108p'`).
Branche : `session-260-lot09-fonctions`. Worktree : `.worktrees/260-lot09-fonctions`. Prérequis : lots 4 et 5 intégrés (la fonction `handle_wrapper_message_with_peer` est déjà traitée au lot 5). Code exécutable modifié : « activation en attente » (R8).
Chercher par **nom de fonction** (`rg -n 'fn <nom>\(' crates --glob '*.rs'`), jamais par numéro de ligne.
Aucun comportement ne change. Aucune signature **publique du protocole** ne change (R6.1). Les verrous gardent le même ordre et la même portée (R6.4 : pas de changement de garde).

## Outil de mesure

`specs/260-assainissement-code/outils/longues_fonctions.py` (lecture seule, sans Cargo) :

```sh
L=specs/260-assainissement-code/outils/longues_fonctions.py
python3 -I $L crates                       # fonctions de 300 lignes ou plus : lignes, paramètres, lieu, nom
python3 -I $L crates --min-lignes 1000     # fonctions de 1000 lignes ou plus
python3 -I $L crates --min-params 15       # fonctions de 15 paramètres ou plus
```

L'outil est une mesure de contrôle : il peut s'écarter de 1 à 2 fonctions du relevé. Il peut rater une fonction de paramètres : pour les 9 fonctions du tableau T100 à T104, chercher par nom.

## Méthode A - structure d'arguments (fonction de 15 paramètres ou plus)

1. Compter les appels : `rg -c '\b<nom>\(' crates --glob '*.rs'`. Plus de 30 appels : arrêter la fonction et la noter.
2. Lire la signature. Si un paramètre est générique ou `impl Trait` : arrêter la fonction et la noter.
3. Juste au-dessus de la fonction, définir :
   ```rust
   pub(crate) struct <Nom>Args<'a> {
       pub champ_a: TypeA,
       pub champ_b: &'a TypeB,
       // un champ par paramètre, mêmes noms, mêmes types
   }
   ```
   `<'a>` seulement si un champ est une référence. Visibilité : la même que la fonction (`pub` si la fonction est `pub`).
4. Changer la signature : `fn <nom>(args: <Nom>Args<'_>) -> <Retour>`. Première ligne du corps : `let <Nom>Args { champ_a, champ_b, .. } = args;` (liste de **tous** les champs, sans `..`). Le reste du corps ne change pas.
5. Les appels : le compilateur les liste (`C-CHECK`). Remplacer `<nom>(a, b, c)` par `<nom>(<Nom>Args { champ_a: a, champ_b: b, .. })`. Si une variable porte déjà le nom du champ : forme courte `champ_a,`.
6. Vérifier : `C-CHECK` (avec les tests). Contrôle : `git diff -U0 | rg '^[-+]' | rg -c '\b<nom>\('` n'augmente pas le nombre d'appels.
Justification (une structure pour une fonction) à écrire dans le journal : le besoin est la lisibilité des appels à 15 paramètres et plus (FR18), pas une abstraction réutilisable.

## Méthode B - découpage d'une fonction longue

1. Lire la fonction d'un bout à l'autre. Repérer les **sections** : suites d'instructions séparées par un commentaire ou une ligne vide, avec un but (ex. « préparer », « valider », « émettre »).
2. Choisir une coupure là où peu de variables locales traversent (6 au plus). Pour chaque section candidate, lister : les variables qu'elle lit (venant d'avant), les variables qu'elle produit (utilisées après).
3. Extraire la section dans une fonction voisine `fn <nom>_<but>(<variables lues>) -> <variables produites>` (un seul résultat, ou un tuple, ou une petite structure). Le corps de la section est copié sans changement. Un `return` dans la section qui sort de la fonction d'origine : ne pas extraire cette section (noter).
4. Une variable `let mut` modifiée par la section et utilisée après : la passer en `&mut`. Un verrou (`MutexGuard`) tenu pendant la section : le passer en paramètre (`&mut` sur la garde déréférencée). **Ne jamais prendre ou relâcher un verrou ailleurs qu'avant.**
5. `C-CHECK-LEGER` après chaque extraction, un commit par section : `refactor(<crate>): Extraire la phase <but> de <nom>`.
6. Objectif par fonction : passer sous 1000 lignes (SC09), de préférence sous 600. Arrêter dès que l'objectif est atteint. Au plus 6 allers-retours de compilation par section, puis arrêt de la section (R4).

## T099 - Constat

```sh
python3 -I $L crates > "$V/lot09-depart-longues.txt"; python3 -I $L crates --min-params 12 > "$V/lot09-depart-params.txt"
wc -l "$V/lot09-depart-longues.txt"; python3 -I $L crates --min-lignes 1000
```

Après les lots 4 et 5, attendu : 3 fonctions de 1000 lignes ou plus au plus (`launch_session_with_status`, `spawn_reader` de `codex_app_server.rs`, et `handle_wrapper_message_with_peer` si le lot 5 est arrêté). Écrire les chiffres dans le journal. Commit : `docs(specs): Relever les fonctions longues avant le lot 9`.

## T100 à T104 - Structures d'arguments (méthode A)

| Tâche | Fonctions (fichier, nombre de paramètres) |
|-------|--------------------------------------------|
| T100 | `prompt_request` (`bridget-transport/src/acp.rs`, 21) |
| T101 | `handle_register_with_channel` (`bridget-daemon/src/daemon.rs`, 17) |
| T102 | `resolve_target` (`bridget-daemon/src/daemon/native_delegation.rs`, 17) |
| T103 | `reconnect_managed_session` (17), `connect_and_register_at` (16), `connect_and_register_with_domain_at` (15), tous dans `bridget-daemon/src/wrapper.rs` : un commit par fonction |
| T104 | `post` (`bridget-daemon/src/threads.rs`, 15), `apply` (`bridget-daemon/src/wrapper_workspace_repair.rs`, 15), `spawn_reader` (`bridget-transport/src/claude_stream_json.rs`, 15) : un commit par fonction |

Commit : `refactor(<crate>): Regrouper les arguments de <nom>`. `C-CRATE` du crate après T102 et T104.

## T105 à T111 - Fonctions longues (méthode B)

| Tâche | Fonction | Lignes | Remarque |
|-------|----------|--------|----------|
| T105 | `launch_session_with_status` (`wrapper.rs`) | 1371 | priorité : SC09 |
| T106 | `spawn_reader` (`bridget-transport/src/codex_app_server.rs`) | 1101 | priorité : SC09 |
| T107 | `tick` (`daemon/native_delegation.rs`) | 983 | **verrou global tenu** : ne pas déplacer une prise de verrou. Si la coupure exige de la déplacer : arrêter et noter |
| T108 | `launch` (`wrapper.rs`) | 796 | une section ou deux suffisent |
| T109 | `admit_delegate` (`native_delegation.rs`) | 777 | idem |
| T110 | `apply_specialist_inner` (`agent_profile/specialists.rs`) | 727 | idem |
| T111 | `handle_idempotent_send_with_result_authority` (`daemon.rs`) | 625 | idem |

Les autres fonctions de plus de 300 lignes ne sont pas traitées dans ce lot (voir T112). Après T106 : `C-CRATE` des deux crates. T105 et T106 sont les seules tâches obligatoires pour SC09. Si l'une est arrêtée, SC09 n'est pas atteint : l'écrire dans le bilan.

## T112 - Exemptions et restes

1. Exemptions de nature (FR19) : ajouter un commentaire d'une ligne au-dessus de la fonction : `// Exemption lot 9 : fonction de données (liste d'outils), sa longueur n'est pas de la logique.` pour `tools` de `mcp.rs` ; `// Exemption lot 9 : définition SQL du schéma, sa longueur n'est pas de la logique.` pour les deux `init_schema` (`idempotency.rs`, `store.rs`).
2. Fonctions de 12 à 14 paramètres : un commentaire d'une ligne au-dessus de chacune : `// <n> paramètres : regroupement différé (programme 260, lot 9).` Liste : `python3 -I $L crates --min-params 12` moins les 9 du tableau.
3. Restes : écrire `specs/260-assainissement-code/validation/lot09-restes.md` : un tableau `fonction / lignes / sort` pour chaque fonction de 300 lignes ou plus encore présente. Sort : « exemptée », « différée (hors périmètre du lot) » ou « arrêtée (cause) ».
Vérifier : `C-CHECK`. Commit : `docs(code): Documenter les exemptions et les restes du lot 9`.

## T113 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot09`), `C-ECHECS` : aucune différence.
2. `C-NOMS` : aucune différence.
3. `python3 -I $L crates --min-lignes 1000 | wc -l` : 0 (SC09). Sinon, écrire la fonction et la cause dans le bilan.
4. Bilan : fonctions traitées, restes, « activation en attente ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- La compilation échoue sur un emprunt (`cannot borrow ... as mutable`) après 2 essais de coupure : observer le code réel, choisir une autre coupure ou arrêter la section (R4).
- Un test change de résultat : `git revert` du dernier commit (R3).
- Interdit : changer l'ordre des appels, des verrous, des `save` ; fusionner ou supprimer une branche ; renommer une chaîne durable ; ajouter une dépendance.
