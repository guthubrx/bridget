# 260 - Versions de schéma des bases SQLite
Date : 10 novembre 2026. Statut : Proposé. Décision à prendre par François.
## Contexte
L'inventaire (`specs/260-assainissement-code/validation/lot12-inventaire-stockage.md`) relève quatre façons de connaître la version d'une base. Une table de migrations sert au magasin d'exécution. Une constante maximale (version 8) et un préflight en lecture seule servent au magasin d'idempotence. `PRAGMA user_version` sert au magasin mobile. La présence de colonnes (`PRAGMA table_info`) sert à plusieurs magasins. Deux petits magasins de remise utilisent un journal `DELETE` avec `synchronous=FULL`, la base du démon utilise `WAL`. 111 ouvertures de base sont réparties dans de nombreux fichiers.
## Options
- A : ne rien changer. Le fonctionnement actuel est conservé.
- B : viser un seul mécanisme à terme, avec une migration base par base. Chaque migration est un changement de format durable.
- C : documenter seulement, et ajouter un test qui échoue si un magasin n'a aucune version lisible.
## Conséquences de chaque option
- A : aucun risque nouveau. Le prochain mainteneur doit connaître les quatre mécanismes. Réversible sans effort.
- B : risque sur les données existantes de chaque machine. Effort élevé. Une migration mal faite est difficile à défaire. Demande un plan de retour par base.
- C : risque faible. Effort modéré. Rend l'écart visible sans modifier un format. Ne réduit pas le nombre de mécanismes.
## Recommandation
Choisir C comme première étape, puis décider de B avec un inventaire complet des magasins non lus.
## Ce qui n'est pas décidé
Le choix du mécanisme cible, l'ordre des migrations, le sort des deux magasins en journal `DELETE`, et le point d'entrée unique d'ouverture des bases. Aucun code ni schéma n'a été modifié par le lot 12.
