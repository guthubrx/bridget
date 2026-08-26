# Plan 033 — Backlog des branches livrées non fusionnées

## Pourquoi ce dessin

Le besoin utile est une sonde Git en lecture seule, pas une nouvelle autorité
d'orchestration. `bridget-idle` connaît déjà la discipline d'indisponibilité et
la sortie texte/JSON ; l'extension reste dans ce script et son harnais.

Le verdict ne possède aujourd'hui aucune représentation structurée exploitable.
Le plan conserve ce manque comme un état explicite au lieu de parser le
catalogue ou les messages libres.

## Algorithme

1. Résoudre le dépôt source depuis le fichier réel du script, ou depuis
   `--git-repo` dans le harnais.
2. Fixer une échéance monotone globale de cinq secondes.
3. Lire `origin/main` et toutes les refs locales `origin/*` en une commande
   `for-each-ref`.
4. Pour chaque ref, exclure une tête ancêtre de main, calculer le `merge-base`
   et l'âge du commit de tête.
5. Exécuter `merge-tree --write-tree` avec `GIT_OBJECT_DIRECTORY` dans un
   répertoire temporaire du système et `GIT_ALTERNATE_OBJECT_DIRECTORIES`
   pointant vers les objets réels. Les éventuels nouveaux objets ne touchent
   jamais le dépôt observé et disparaissent avec le répertoire temporaire.
6. Trier les lignes par âge décroissant puis par référence.
7. Ajouter la vue au JSON v1 et au texte existants, avec les limites et
   l'indisponibilité de la greffe.

Tout retour inattendu, expiration du budget ou sortie Git invalide invalide la
vue complète. Une liste partielle serait plus dangereuse qu'une indisponibilité.

## Mapping du blocage

Ordre fermé :

1. aucun ancêtre commun → `base perimee, a rebaser` ;
2. conflit `merge-tree` → `en conflit` ;
3. sinon → `indetermine — greffe sans etat exploitable`.

Les états `attend un relecteur` et `attend le referent` ne sont pas émis tant
qu'un verdict structuré ne permet pas de les discriminer.

## Oracle

Le harnais construit un dépôt et un remote locaux avec :

- une branche fusionnée dont la ref distante subsiste ;
- une branche non fusionnée propre ;
- une branche en conflit ;
- une branche à histoire sans ancêtre commun ;
- une ref ajoutée au remote mais non récupérée localement.

Le contrôle positif vérifie d'abord le corps exact de la branche non fusionnée,
puis seulement l'absence de la branche fusionnée. Un faux binaire Git qui dort
valide la borne et le libellé d'indisponibilité. L'inventaire des fichiers
d'objets et des refs avant/après atteste la lecture seule.

## Complexité et frugalité

Pour `n` refs locales, le tri vaut O(n log n) et l'analyse lance O(n) commandes
Git. Le budget global borne le temps mural indépendamment de `n`. Une commande
par branche est retenue parce qu'elle conserve des codes de conflit simples et
auditables ; le protocole batch `merge-tree --stdin` ajouterait un parseur
complexe pour quelques dizaines de refs.

Aucune dépendance Python ou Git nouvelle, aucun cache durable et aucune
abstraction hors du script ne sont ajoutés.

## Validation

- `python3 -m py_compile scripts/bridget-idle.py` ;
- `bash -n scripts/test-bridget-idle.sh` ;
- harnais complet `scripts/test-bridget-idle.sh` ;
- mutant supprimant l'exclusion d'une branche fusionnée ;
- exécution sur le dépôt réel, avec temps mesuré et limites visibles ;
- `git diff --check` et self-review du diff complet.
