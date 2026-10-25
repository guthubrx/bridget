# Préparation de livraison - Session 149 - Tour haiku r1

Date : 2026-10-10
Périmètre : lecture seule. Ce fichier est le seul écrit.

**Non fait dans ce tour :** aucun build, test, lint, installation, signature,
redémarrage, écriture Git, recherche réseau, ni exécution de binaire.
Seules des lectures ont été faites (`git status`, `rg`, `ls`, `shasum`,
`defaults read`, `codesign -dv`, `df`, `du`).

**Important :** aucune livraison n'est prête. Rien ci-dessous ne dit qu'un build a réussi.

## Constats à lire d'abord

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-1).

---

## 1. Pipeline adaptateur de bureau réutilisable depuis 148

### Ce qui existe (preuves archivées, lecture seule)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-4).

### Commandes cibles pour 149 (non exécutées, à adapter)

Variables : `R` est le dossier privé. `SHA` est le commit fusionné propre
(donné par le principal).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-5).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-6).

Extraction et renommage du ZIP dans un dossier distinct du stage :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-7).

Signature : `sign-stage.py` avec la racine paramétrée sur le stage natif (`--keep-stage`), puis la copie.
Vérification :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-8).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-9).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-10).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-11).

Signature : ad hoc. Il n'existe pas d'identité développeur (`TeamIdentifier not set`).
La méthode 148 est :

- signature de 38 objets (29 Mach-O, 8 conteneurs, l'application) ;
- droits relus avec `codesign -d --entitlements`, puis comparés à un instantané
  (`signing-before.json`) ;
- `--preserve-metadata=entitlements,flags,runtime` seulement si l'objet était déjà signé ;
- contrôle `codesign --verify --strict --deep` (code retour 0).

Sauvegarde et remplacement (méthode à confirmer par le principal) :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-12).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-13).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-14).

Synchronisation minimale proposée :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-15).

Aucune édition faite dans ce tour. La spec 149 indique 0/45 tâches.

---

## 5. Artefacts non suivis et preuves

### Worktree T (`/Users/moi/11.Repositories/adaptateur de bureau-local/.worktrees/149-sous-agents-lineage`)

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-16).

---

## Connu contre restant

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-17).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/delivery-preparation149-haiku-r1.md#extrait-18).

Aucune promesse de build réussi. Aucune nouvelle demande d'approbation dans ce rapport.
