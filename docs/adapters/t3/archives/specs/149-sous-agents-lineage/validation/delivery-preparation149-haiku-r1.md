# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

Dossier : `/Users/moi/.cache/bridget-cleanup149-build148-proofs/t3-final148.XnkCUZ/`

<a id="extrait-3"></a>

## Extrait 3

| Fichier | Rôle | Adaptation nécessaire |
| --- | --- | --- |
| `build-final.sh` (37 lignes) | install, rebuild, version, build:desktop, package | Chemins 148 et `APP_VERSION=0.0.45-local.148` en dur |
| `sign-stage.py` (66 lignes) | signature ad hoc par objet, droits restaurés | Racine `t3-final148` en dur |
| `verify-final.mjs` (221 lignes) | vérification source, sous-modules, stage, copie, ZIP | Réutilisable avec d'autres arguments |
| `sqlite-smoke.cjs` | smoke SQLite sous Electron | Vérifier les chemins avant usage |
| `build-plan.txt` | ordre des étapes 148 | Référence seulement |

<a id="extrait-4"></a>

## Extrait 4

Toolchain conservée : `/Users/moi/.cache/t3-toolchains/148/` (240 Mo).
Présence confirmée : `node-v24.13.1-darwin-arm64/bin/node`, `SHASUMS256.txt`,
`verification.json`, et le shim `pnpm` de corepack. Version pnpm 11.10.0 non exécutée.
Le `packageManager` de `package.json` la déclare.

<a id="extrait-5"></a>

## Extrait 5

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

<a id="extrait-6"></a>

## Extrait 6

Variables d'environnement : `T3CODE_DESKTOP_SIGNED=false`, `APP_VERSION=0.0.45-local.149`,
et les variables publiques de build du script 148 (relais, Clerk). Leurs valeurs ne sont pas
recopiées ici. Les lire dans `build-final.sh`.

<a id="extrait-7"></a>

## Extrait 7

```bash
mkdir -m 700 "$R/staging"
ditto -x -k "$R/artifacts/T3-Code-0.0.45-local.149-arm64.zip" "$R/staging"
mv "$R/staging/T3 Code (Alpha).app" "$R/staging/T3 Code (Local).app"
```

<a id="extrait-8"></a>

## Extrait 8

```bash
node "$R/verify-final.mjs" --commit <SHA> --version 0.0.45-local.149 \
  --stage "<chemin du stage natif lu dans packaging.log>" --original "$R/original" \
  --source "$R/source" --app "$R/staging/T3 Code (Local).app" \
  --zip "$R/artifacts/T3-Code-0.0.45-local.149-arm64.zip"
```

<a id="extrait-9"></a>

## Extrait 9

Lu aujourd'hui sur `/Applications/T3 Code (Local).app` (lecture seule) :

<a id="extrait-10"></a>

## Extrait 10

- `CFBundleIdentifier` : `com.t3tools.t3code`
- `CFBundleName` : `T3 Code (Alpha)`
- `CFBundleShortVersionString` : `0.0.45-local.148`
- Exécutable : `Contents/MacOS/T3 Code (Alpha)`
- `codesign -dv` : `Identifier=com.t3tools.t3code`, `Signature=adhoc`, 34 entrées Info.plist.
- SHA256 de `app.asar` : `60d30676…`, identique au reçu 148.

<a id="extrait-11"></a>

## Extrait 11

À conserver : identifiant `com.t3tools.t3code`, nom interne `T3 Code (Alpha)`,
exécutable identique. Seul le nom du dossier reste `T3 Code (Local).app`.

<a id="extrait-12"></a>

## Extrait 12

```bash
APP="/Applications/T3 Code (Local).app"
du -sk "$APP"    # taille seulement, avant copie
ditto "$APP" "/Users/moi/.cache/bridget-install149.<suffixe>/T3 Code (Local).app.rollback"
diff -qr "$APP" "/Users/moi/.cache/bridget-install149.<suffixe>/T3 Code (Local).app.rollback"
ditto "$R/staging/T3 Code (Local).app" "/Applications/.T3 Code (Local).app.new-149"
# contrôle de signature et diff -qr avec la copie, puis échange des deux noms
```

<a id="extrait-13"></a>

## Extrait 13

Ne pas copier ni modifier la base T3 (13 Gio). Ne pas toucher aux réglages utilisateur.

<a id="extrait-14"></a>

## Extrait 14

Ce que 149 change pour l'utilisateur : l'enfant apparaît dans Lineage T3, sans appel
supplémentaire, sans gain de droits et sans nouveau grant. Le parcours reste
`bridget_delegate`. Ce n'est pas un spawn ordinaire.

<a id="extrait-15"></a>

## Extrait 15

1. `docs/delegation-native.md` : une section « Visibilité dans Lineage » (projection fidèle,
   T3 n'exécute rien et ne duplique rien, pas de conversation de premier niveau en doublon).
2. `skills/bridget/SKILL.md` : où voir l'enfant, et rappel qu'aucun grant nouveau n'est créé.
3. `skills/bridget/references/commandes.md` : seulement si 149 ajoute ou modifie un outil MCP.

<a id="extrait-16"></a>

## Extrait 16

- Caches de dépendances : `node_modules` partagé de 74 Go, registre Cargo.
- États de retour arrière : `bridget-install148`, sauvegardes d'application.
- Preuves de nettoyage 148 : `/Users/moi/.cache/bridget-cleanup149-build148-proofs`
  (3,2 Mo, 42 fichiers, `SHA256SUMS` vérifié à 42/42).
- Toolchain 148 : `/Users/moi/.cache/t3-toolchains/148` (240 Mo).

<a id="extrait-17"></a>

## Extrait 17

**Connu (lu aujourd'hui) :**
- Binaire Bridget installé = SHA `8104a63b…` = build 148.
- Application T3 installée = asar `60d30676…` = build 148, identifiant `com.t3tools.t3code`.
- Procédure 148 complète, dossiers de preuves archivés, toolchain 148 présente.
- Contenu des fichiers d'artefacts non suivis : types et tailles seulement.

<a id="extrait-18"></a>

## Extrait 18

**Restant (non fait, à faire par le principal ou par un agent autorisé) :**
- Commit fusionné propre dans T et dans B (`main`).
- Build T3 149, build Bridget 149, signature, installation.
- Relecture de `Cargo.toml` (version) et de la configuration des LaunchAgents.
- Recalcul des valeurs de vérification 149. Les valeurs 148 ne servent pas de preuve.
- Mise à jour des docs, après relecture du diff 149.
- Décision sur les artefacts et les captures avant partage.
