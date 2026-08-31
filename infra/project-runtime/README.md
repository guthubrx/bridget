# Image fixture du runtime projet

Cette image est uniquement une fixture de test de SPEC-066. Elle prépare les
répertoires ABI nécessaires à un conteneur à système de fichiers racine en
lecture seule. Aucun tag n'est une autorité d'identité : le test construit
l'image puis consomme exclusivement l'identifiant local écrit par Docker.

Depuis la racine du dépôt :

```sh
fixture_iid=/tmp/bridget-project-runtime.iid
docker build --iidfile "$fixture_iid" infra/project-runtime
cat "$fixture_iid"
```

Le contenu de `fixture_iid` est la référence `local_image_id` à placer dans le
document hôte `project-runtime-policy-config-v1`. Il est résolu et attesté
avant chaque création de conteneur.
