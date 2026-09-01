# Runtime Docker de production

Cette image est le socle neutre partagé par les agents d'un même projet. Elle
ne contient ni fournisseur, ni jeton, ni clé SSH, ni socket Docker. Les
clients Codex, Claude, Cursor/ACP ou Gemini ne peuvent être ajoutés que par un
catalogue de ressources approuvé et versionné côté hôte.

Construire et vérifier localement sur un hôte Linux amd64 :

```sh
docker build --platform linux/amd64 --iidfile /tmp/bridget-runtime.iid \
  -f infra/project-runtime/production/Dockerfile \
  -t bridget-project-runtime:local .
docker run --rm --read-only --tmpfs /tmp:rw,nosuid,nodev,noexec,size=64m \
  --user 1002:1002 --entrypoint /bin/bash bridget-project-runtime:local \
  /usr/local/bin/bridget-runtime-smoke
```

Le digest écrit dans `/tmp/bridget-runtime.iid` doit être recopié dans la
politique runtime hôte. Aucun tag libre transmis par un agent ou l'interface
ne constitue une autorité.

Le contexte de build est le sommet du dépôt Bridget : l'image compile alors
le binaire `/usr/local/bin/bridget` déclaré par la politique. Le fichier
`.dockerignore` exclut les secrets usuels, les caches et l'historique Git.
