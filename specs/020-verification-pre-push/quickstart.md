# Quickstart — Spec 020

## Vérifier le lot

```bash
bash scripts/test-git-pre-push-authorship.sh
```

Le banc doit annoncer sept témoins passés, zéro rouge et zéro ignoré.

## Limite d'activation

Le lot ne doit ni créer ni remplacer `/home/moi/.git-hooks/pre-push`.
L'activation est une étape postérieure au jury et au merge, gouvernée par la
session 018.
