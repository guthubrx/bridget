# Quickstart — Activation gouvernée

## Après jury et merge, sur la machine principale

```bash
cd /Users/moi/Nextcloud/10.Scripts/bridget
git fetch origin
git switch main
git status --short
/Users/moi/Nextcloud/10.Scripts/bridget/scripts/install-bridget-idle.sh --force
/Users/moi/Nextcloud/10.Scripts/bridget/scripts/install-bridget-ronde.sh \
  --config /Users/moi/.config/maicie/config.json \
  --report-dir /Users/moi/.cache/bridget/rondes \
  --force
```

Un `git status --short` non vide interdit l'activation.

## Lire l'origine active

```bash
readlink /Users/moi/.local/bin/bridget-idle
cat "$(readlink /Users/moi/.local/bin/bridget-idle).origin"
readlink /Users/moi/.local/bin/bridget-ronde
cat "$(readlink /Users/moi/.local/bin/bridget-ronde).origin"
```

Les deux cibles doivent vivre sous
`/Users/moi/.local/share/bridget/pilotage/releases/<SHA>/`, jamais sous
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/`.

## Validation Linux de la session

```bash
cd /home/moi/revue/ac3/.worktrees/session-18-activation-outils-pilotage
bash -n scripts/lib/pilotage-release.sh scripts/install-bridget-idle.sh \
  scripts/install-bridget-ronde.sh scripts/test-018-pilotage-install.sh
scripts/test-018-pilotage-install.sh
scripts/test-bridget-idle.sh
scripts/test-bridget-ronde.sh
cargo test --workspace --no-run
```
