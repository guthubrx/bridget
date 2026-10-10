# Préparation de livraison - Session 149 - Tour haiku r1

Date : 2026-10-10
Périmètre : lecture seule. Ce fichier est le seul écrit.

**Non fait dans ce tour :** aucun build, test, lint, installation, signature,
redémarrage, écriture Git, recherche réseau, ni exécution de binaire.
Seules des lectures ont été faites (`git status`, `rg`, `ls`, `shasum`,
`defaults read`, `codesign -dv`, `df`, `du`).

**Important :** aucune livraison n'est prête. Rien ci-dessous ne dit qu'un build a réussi.

## Constats à lire d'abord

1. Les deux worktrees 149 contiennent des modifications suivies non committées.
   Le build final doit partir d'un commit propre, fusionné par le principal.
   Un build depuis un worktree sale donne `-dirty` dans l'identifiant Bridget.
2. Les scripts 148 sont figés sur les chemins 148 (`t3-final148`, `bridget-final148`,
   `/tmp/b148.*`). Le cache de build 148 a été supprimé. Il faut une copie
   paramétrée des scripts. Ne pas réutiliser le dossier des preuves comme dossier de travail.
3. Les trois défauts de documentation relevés en 148 ne sont plus trouvés sur `main`
   (`Seize outils`, `claude_glm`, littéral `development_protocol_unavailable` en exemple).
   Je n'ai pas vérifié l'historique commit par commit.
4. `apps/server/.vitest/` est non suivi dans le worktree T (2 839 octets).
   Un `git add -A` l'ajouterait au commit.
5. Espace mesuré aujourd'hui : disque interne 52 Gio libres (95 % utilisé),
   volume 8TB2 161 Gio libres (96 % utilisé). Les valeurs 55 et 157 du mandat sont obsolètes.

---

## 1. Pipeline T3 réutilisable depuis 148

### Ce qui existe (preuves archivées, lecture seule)

Dossier : `/Users/moi/.cache/bridget-cleanup149-build148-proofs/t3-final148.XnkCUZ/`

| Fichier | Rôle | Adaptation nécessaire |
| --- | --- | --- |
| `build-final.sh` (37 lignes) | install, rebuild, version, build:desktop, package | Chemins 148 et `APP_VERSION=0.0.45-local.148` en dur |
| `sign-stage.py` (66 lignes) | signature ad hoc par objet, droits restaurés | Racine `t3-final148` en dur |
| `verify-final.mjs` (221 lignes) | vérification source, sous-modules, stage, copie, ZIP | Réutilisable avec d'autres arguments |
| `sqlite-smoke.cjs` | smoke SQLite sous Electron | Vérifier les chemins avant usage |
| `build-plan.txt` | ordre des étapes 148 | Référence seulement |

Toolchain conservée : `/Users/moi/.cache/t3-toolchains/148/` (240 Mo).
Présence confirmée : `node-v24.13.1-darwin-arm64/bin/node`, `SHASUMS256.txt`,
`verification.json`, et le shim `pnpm` de corepack. Version pnpm 11.10.0 non exécutée.
Le `packageManager` de `package.json` la déclare.

### Commandes cibles pour 149 (non exécutées, à adapter)

Variables : `R` est le dossier privé. `SHA` est le commit fusionné propre
(donné par le principal).

```bash
# Préconditions : main (B) et local/main-20261009 (T) propres, SHA fourni par le principal
export R=/Users/moi/.cache/t3-final149.<suffixe>   # suffixe aléatoire, dossier neuf
umask 077
mkdir -m 700 "$R"

# Copie privée au commit, pas de worktree de collaboration
git clone --no-local /Users/moi/11.Repositories/t3code-local "$R/source"
git -C "$R/source" checkout <SHA>
git -C "$R/source" submodule update --init  # si des sous-modules existent (2 pointeurs gitlink)
git clone --no-local /Users/moi/11.Repositories/t3code-local "$R/original"
git -C "$R/original" checkout <SHA>

# Version : seuls les 4 manifestes sont modifiés
export PATH="/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH"
node "$R/source/scripts/update-release-package-versions.ts" 0.0.45-local.149 --root "$R/source"

# Dépendances dans la copie seulement (jamais dans le node_modules partagé de 74 Go)
cd "$R/source" && pnpm install --frozen-lockfile > "$R/install-dependencies.log" 2>&1
pnpm rebuild --recursive > "$R/rebuild-dependencies.log" 2>&1
pnpm run build:desktop > "$R/build-desktop.log" 2>&1
node scripts/build-desktop-artifact.ts --platform mac --target zip --arch arm64 \
  --build-version 0.0.45-local.149 --output-dir "$R/artifacts" --skip-build --keep-stage --verbose \
  > "$R/packaging.log" 2>&1
```

Variables d'environnement : `T3CODE_DESKTOP_SIGNED=false`, `APP_VERSION=0.0.45-local.149`,
et les variables publiques de build du script 148 (relais, Clerk). Leurs valeurs ne sont pas
recopiées ici. Les lire dans `build-final.sh`.

Extraction et renommage du ZIP dans un dossier distinct du stage :

```bash
mkdir -m 700 "$R/staging"
ditto -x -k "$R/artifacts/T3-Code-0.0.45-local.149-arm64.zip" "$R/staging"
mv "$R/staging/T3 Code (Alpha).app" "$R/staging/T3 Code (Local).app"
```

Signature : `sign-stage.py` avec la racine paramétrée sur le stage natif (`--keep-stage`), puis la copie.
Vérification :

```bash
node "$R/verify-final.mjs" --commit <SHA> --version 0.0.45-local.149 \
  --stage "<chemin du stage natif lu dans packaging.log>" --original "$R/original" \
  --source "$R/source" --app "$R/staging/T3 Code (Local).app" \
  --zip "$R/artifacts/T3-Code-0.0.45-local.149-arm64.zip"
```

Smoke SQLite : Electron de l'application privée, base privée sous `$R`, sans GUI ni serveur.

### Caveats

- Espace : la copie avec dépendances occupe plusieurs Go (le cache 148 complet pesait 8,5 Gio).
  Le volume interne a 52 Gio libres. Le 8TB2 a 161 Gio libres, mais il est à 96 %.
- Node 24.13.1 est requis par `engines`. Le build 148 n'a pas affiché d'avertissement de version.
- Avertissements 148 toujours présents, non corrigés : chunks web au-dessus du seuil,
  `x11` externalisé, `import.meta` remplacé en sortie CommonJS.
- Pas de blockmap de mise à jour pour ce ZIP local.

### Connu et restant

- Connu : résultats 148 (commit `33f6d04e1164`, ZIP `1b3feb42…`, asar `60d30676…`).
  Ces valeurs ne valent pas pour 149.
- Restant : tout le pipeline 149. Le chiffre 2576 de 148 doit être recalculé.

---

## 2. Binaire Bridget

### Provenance de l'identifiant de build

Fichier : `crates/bridget-daemon/build.rs` et `src/build_identity.rs`.

- Si `BRIDGET_BUILD_ID` est absent, l'identifiant est `git rev-parse --short=12 HEAD`.
- Il reçoit le suffixe `-dirty` si des fichiers suivis sont modifiés. Les fichiers non suivis sont ignorés.
- La compilation se refait quand HEAD, l'index, les refs, `Cargo.toml`, `Cargo.lock`, `crates` ou `plugins` changent.

Conséquence : un build depuis un worktree sale donne `<sha12>-dirty`. Le build final doit
venir d'un commit propre fusionné dans `main`.

Preuve 148 (`bridget-build-receipt.json`) : `embedded_build_id: 347d788510b4`,
`sources_clean: true`, `build_info_source_changed: false`.

### Commandes cibles (non exécutées)

Le script 148 `run-build148.py` est lisible et paramétrable. Il fait :

1. Vérifie que HEAD est attendu, que le statut est propre et que le worktree est unique.
2. Copie `registry` et `git` de `~/.cargo` en clone APFS (`cp -c -R`) dans un `CARGO_HOME` privé.
3. Lance `python3 scripts/build.py cargo build --locked --release -p bridget-daemon --bin bridget`
   avec `CARGO_TARGET_DIR` privé, `TMPDIR` privé et `CARGO_NET_OFFLINE=true`.
4. Signe avec `codesign --force --sign -`, puis vérifie `--verify --strict`.
5. Vérifie que l'identifiant est embarqué et que `bridget --version` affiche `bridget <version>`.

```bash
export B=/Users/moi/.cache/bridget-final149.<suffixe>
mkdir -m 700 "$B"
git clone --no-local /Users/moi/Nextcloud/10.Scripts/64.bridget "$B/source"
git -C "$B/source" checkout <SHA>
# puis : même logique que run-build148.py, avec $B à la place de bridget-final148
```

`Cargo.toml` de 149 a été modifié et n'a pas été relu. Vérifier la version avant le build.
`scripts/build.py` nettoie des caches Cargo après succès. Le `CARGO_TARGET_DIR` privé
protège la cible racine.

### Installation (constat 148, non refait)

- `/Users/moi/.local/bin/bridget` est un lien vers
  `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
- Selon le reçu 148, les LaunchAgents pointent vers ce même binaire. Je ne les ai pas relus.
- Empreinte actuelle, lue aujourd'hui : SHA256 de `target/release/bridget` =
  `8104a63b3e16784280a2da1aeef0a0632158f31b0bd08312db3f17ec8bd9f56e`.
  C'est le `signed_sha256` du reçu 148. Le binaire installé correspond au build 148.

Méthode 148 : copie voisine, signature, contrôles, sauvegarde comparée, puis `os.replace`.
Sauvegarde 148 : `/Users/moi/.cache/bridget-install148.KlIN2m/bridget.rollback`
(SHA `0db040b8…`).

Méthode proposée pour 149 (à confirmer par le principal) :

```bash
B_BIN=/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
cp -p "$B_BIN" /Users/moi/.cache/bridget-install149.<suffixe>/bridget.rollback
shasum -a 256 "$B_BIN" /Users/moi/.cache/bridget-install149.<suffixe>/bridget.rollback
cp "$B/source/target/release/bridget" "$B_BIN.new-149"   # copie voisine, même volume
codesign --force --sign - "$B_BIN.new-149"
codesign --verify --strict "$B_BIN.new-149"
"$B_BIN.new-149" --version
python3 -c "import os; os.replace('$B_BIN.new-149', '$B_BIN')"   # remplacement atomique
shasum -a 256 "$B_BIN"                                           # doit égaler le signed_sha256 149
```

Aucun `kill` ni relance. Les processus actifs gardent leur code chargé.
Les PID 148 sont 58468, 57109, 58394 et 58396. Les relever avant et après, sans les arrêter.

---

## 3. Permissions et signature de l'application

Lu aujourd'hui sur `/Applications/T3 Code (Local).app` (lecture seule) :

- `CFBundleIdentifier` : `com.t3tools.t3code`
- `CFBundleName` : `T3 Code (Alpha)`
- `CFBundleShortVersionString` : `0.0.45-local.148`
- Exécutable : `Contents/MacOS/T3 Code (Alpha)`
- `codesign -dv` : `Identifier=com.t3tools.t3code`, `Signature=adhoc`, 34 entrées Info.plist.
- SHA256 de `app.asar` : `60d30676…`, identique au reçu 148.

À conserver : identifiant `com.t3tools.t3code`, nom interne `T3 Code (Alpha)`,
exécutable identique. Seul le nom du dossier reste `T3 Code (Local).app`.

Signature : ad hoc. Il n'existe pas d'identité développeur (`TeamIdentifier not set`).
La méthode 148 est :

- signature de 38 objets (29 Mach-O, 8 conteneurs, l'application) ;
- droits relus avec `codesign -d --entitlements`, puis comparés à un instantané
  (`signing-before.json`) ;
- `--preserve-metadata=entitlements,flags,runtime` seulement si l'objet était déjà signé ;
- contrôle `codesign --verify --strict --deep` (code retour 0).

Sauvegarde et remplacement (méthode à confirmer par le principal) :

```bash
APP="/Applications/T3 Code (Local).app"
du -sk "$APP"    # taille seulement, avant copie
ditto "$APP" "/Users/moi/.cache/bridget-install149.<suffixe>/T3 Code (Local).app.rollback"
diff -qr "$APP" "/Users/moi/.cache/bridget-install149.<suffixe>/T3 Code (Local).app.rollback"
ditto "$R/staging/T3 Code (Local).app" "/Applications/.T3 Code (Local).app.new-149"
# contrôle de signature et diff -qr avec la copie, puis échange des deux noms
```

Ne pas copier ni modifier la base T3 (13 Gio). Ne pas toucher aux réglages utilisateur.

---

## 4. Guides et skills à synchroniser

Fichiers trouvés (liens symboliques vers le dépôt, une édition suffit) :

| Fichier | Lien depuis | Ce qu'il dit sur la délégation |
| --- | --- | --- |
| `docs/delegation-native.md` (84 lignes) | dépôt | Parcours un appel (l. 10), droits parent/enfant (l. 45), grant humain (l. 66-70), refus de posture développement (l. 62-64) |
| `skills/bridget/SKILL.md` | `~/.claude/skills/bridget`, `~/.agents/skills/bridget` | Mode d'emploi Bridget |
| `skills/bridget/references/commandes.md` | `~/.claude/skills/bridget/references` | Liste des outils |

Référence 148 : `specs/148-identite-delegation/validation/glm-relecture-guides-r1.md`
(verdict APPROVE avec trois corrections, non vérifiées dans leur historique).

Posture humaine, constat 148 : le grant se donne seulement dans un terminal interactif.
Le MCP ne peut pas créer de grant. Un agent est refusé, et ce refus est prouvé.
Ce point est correct dans la doc. Ne pas le changer.

Ce que 149 change pour l'utilisateur : l'enfant apparaît dans Lineage T3, sans appel
supplémentaire, sans gain de droits et sans nouveau grant. Le parcours reste
`bridget_delegate`. Ce n'est pas un spawn ordinaire.

Synchronisation minimale proposée :

1. `docs/delegation-native.md` : une section « Visibilité dans Lineage » (projection fidèle,
   T3 n'exécute rien et ne duplique rien, pas de conversation de premier niveau en doublon).
2. `skills/bridget/SKILL.md` : où voir l'enfant, et rappel qu'aucun grant nouveau n'est créé.
3. `skills/bridget/references/commandes.md` : seulement si 149 ajoute ou modifie un outil MCP.

Aucune édition faite dans ce tour. La spec 149 indique 0/45 tâches.

---

## 5. Artefacts non suivis et preuves

### Worktree T (`/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`)

- `apps/server/.vitest/json/output.json` : 2 839 octets, JSON de sortie de test.
  Type lu, contenu non lu. Transitoire. Non ignoré par Git : à exclure de tout commit.
  À supprimer seulement après archivage du résultat de test et accord du principal.
- Autres fichiers non suivis (`BridgetLineage.ts`, tests, `BridgetTaskJournal.tsx`, etc.) :
  ce sont des sources de 149 à committer. Ce ne sont pas des artefacts.

### Dossier de validation B (`specs/149-sous-agents-lineage/validation`)

- 151 entrées, 9 sous-dossiers, 13,6 Mo.
- Types : 59 `.md`, 52 `.log`, 12 `.json`, 9 `.txt`, 6 `.py`, 4 `.sh`, et des dossiers
  `ui149-screenshots*`, `ui-recipes`, `recipes`.
- Noms sensibles : aucun fichier nommé `token`, `secret`, `cookie`, `.key`, `.pem`,
  `.db`, `.sqlite`, `.wal` ou `.shm`.
- Deux fichiers ont `sqlite` dans leur nom : `native-fixture-sqlite-r7.diff` et
  `native-fixture-sqlite-sonnet-r7.md`. Contenu non lu.
- Captures `ui149-screenshots*` : elles peuvent montrer de vraies conversations.
  Une relecture humaine est requise avant tout partage.
- Gros logs (plus de 160 Ko) : `native-r7-daemon.log`, `native-r8-final-daemon.log`,
  `native-r6-final-daemon.log`. Ils peuvent contenir chemins et identifiants. Non lus.

Recommandation :

- Garder les preuves durables : `.md`, `.json`, reçus finaux et logs de la ronde finale.
- Supprimer les logs de build intermédiaires (`*build*.log`, `hash-latency*`) seulement
  après remplacement vérifié et accord du principal.

### Caches (volume 8TB2)

- `validation-cache/bridget149-target` : 5,04 Go. Cache de build réutilisable. À garder.
- `validation-cache/bridget149-debug` et `bridget149-release` : propres à 149. À supprimer
  seulement après remplacement vérifié et accord du principal.

### À préserver

- Caches de dépendances : `node_modules` partagé de 74 Go, registre Cargo.
- États de retour arrière : `bridget-install148`, sauvegardes d'application.
- Preuves de nettoyage 148 : `/Users/moi/.cache/bridget-cleanup149-build148-proofs`
  (3,2 Mo, 42 fichiers, `SHA256SUMS` vérifié à 42/42).
- Toolchain 148 : `/Users/moi/.cache/t3-toolchains/148` (240 Mo).

---

## Connu contre restant

**Connu (lu aujourd'hui) :**
- Binaire Bridget installé = SHA `8104a63b…` = build 148.
- Application T3 installée = asar `60d30676…` = build 148, identifiant `com.t3tools.t3code`.
- Procédure 148 complète, dossiers de preuves archivés, toolchain 148 présente.
- Contenu des fichiers d'artefacts non suivis : types et tailles seulement.

**Restant (non fait, à faire par le principal ou par un agent autorisé) :**
- Commit fusionné propre dans T et dans B (`main`).
- Build T3 149, build Bridget 149, signature, installation.
- Relecture de `Cargo.toml` (version) et de la configuration des LaunchAgents.
- Recalcul des valeurs de vérification 149. Les valeurs 148 ne servent pas de preuve.
- Mise à jour des docs, après relecture du diff 149.
- Décision sur les artefacts et les captures avant partage.

Aucune promesse de build réussi. Aucune nouvelle demande d'approbation dans ce rapport.
