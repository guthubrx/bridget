# Quickstart de validation - SPEC-082

## Préconditions

1. Démarrer une instance Bridget locale avec base de test et répertoire canonique
   temporaire privés. La racine canonique attendue est dérivée de la base :
   `<base>.db` devient `<base>.artifacts`.
2. Ouvrir Bridget Desktop connecté à cette instance.
3. Utiliser un fournisseur test qui annonce l'outil de publication.
4. Préparer un jeu de données avec séries, unité, source, une valeur manquante
   et une transformation déclarée.

## Parcours P1 - Publication et lecture

1. Demander une visualisation de la série de test.
2. Vérifier un appel unique de publication et un reçu visible dans l'activité.
3. Vérifier le graphique inline, son résumé, les valeurs importantes et la table.
4. Ouvrir Données et source et vérifier provenance, date, unité, transformation
   et empreintes.
5. Exporter les données et comparer leur empreinte ou contenu à la version.

## Parcours P1 - Versions et cache

1. Actualiser une source identique et vérifier qu'aucune fausse version n'est
   créée.
2. Modifier la source, actualiser et vérifier une nouvelle version liée.
3. Épingler la version, déclencher le nettoyage cache et vérifier sa conservation.
4. Évincer une entrée non épinglée puis restaurer depuis le message d'origine.
5. Désactiver l'accès Bridget et vérifier la carte indisponible avec manifeste,
   cause explicite et action de récupération Bridget.

## Parcours sécurité et visibilité

1. Tenter une publication sans source, avec taille dépassée, puis avec données
   partielles déclarées.
2. Vérifier refus explicite des deux premiers et publication partielle du dernier.
3. Créer deux projets, vérifier l'onglet du projet courant, puis le filtre global
   opérateur.
4. Tenter une lecture depuis un agent non destinataire et vérifier son refus.
5. Supprimer une conversation contenant un artefact sans autre référence puis
   vérifier que ses blobs non référencés deviennent collectables.

## Preuves attendues

- tests Rust, Node et Desktop passants ;
- manifestes de fixture et exports correspondants ;
- capture de thèmes clair et sombre ;
- mesure de cache avant et après éviction ;
- aucun contenu, secret ou chemin canonique dans les diagnostics.

## Commandes de preuve

Depuis la racine du dépôt :

```text
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test artifact_lifecycle_test --test artifact_store_test --test artifact_publication_test
/Users/moi/.cargo/bin/cargo test -p bridget-daemon relais_artefact_pagine_exporte --lib
/Users/moi/.cargo/bin/cargo test -p bridget-daemon artifact_fetch --lib
cd /private/tmp/bridget-project-nav.JNWHqE/crates/bridget-daemon/assets/ui && npm test
```
