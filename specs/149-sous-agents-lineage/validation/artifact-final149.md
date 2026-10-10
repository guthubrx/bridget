# Contrôle du paquet final 149

## Périmètre

- Commit T3 : `f6f3d1998c686b20e316f304aff7f5df6a14592c`
- Version : `0.0.45-local.149`
- Dossier de paquet : `/Users/moi/.cache/t3-final149.K5dHqa`
- Application contrôlée (stage) : `/private/tmp/b149.l0KEOR/t3code-desktop-mac-stage-RSh9aq/app/dist/mac-arm64/T3 Code (Alpha).app`
- Zip contrôlé : `/Users/moi/.cache/t3-final149.K5dHqa/artifacts/T3-Code-0.0.45-local.149-arm64.zip`

Aucune installation, aucun redémarrage, aucun GUI, aucun serveur, aucun modèle et aucune base de production n'ont été utilisés.

## Résultats

| Contrôle | Résultat | Preuve |
|---|---|---|
| Vérificateur final (`verify-final.mjs`) | Réussi : 2576 contrôles OK, 0 échec | `/Users/moi/.cache/t3-final149.K5dHqa/final-artifact-check.json` |
| Installation et redémarrage | Non faits (`installed: false`, `restarted: false`) | même fichier |
| Smoke SQLite (Electron en mode Node) | Réussi : lecture durable OK, `quick_check` = ok | `/Users/moi/.cache/t3-final149.K5dHqa/sqlite-smoke-final.json` |
| Base SQLite du smoke | Nouvelle base privée dans packageRoot | `/Users/moi/.cache/t3-final149.K5dHqa/sqlite-smoke-final.db` |
| `codesign --verify --deep --strict` | Réussi : "valid on disk" et "satisfies its Designated Requirement" | sortie de commande, code retour 0 |

Détails du smoke SQLite : Electron 44.4.5, Node 24.21.0, modules 149, SQLite 3.53.4. `guiStarted`, `serverStarted` et `modelCalled` valent `false`.

## Incident de manipulation

Mon premier appel du smoke SQLite a utilisé `--help` comme chemin de base. Le fichier `./--help` a donc été créé dans `/Users/moi/Nextcloud/10.Scripts/64.bridget`. Ce fichier de 8192 octets venait de mon appel. Je l'ai supprimé. Le smoke final a ensuite été relancé avec la base privée décrite plus haut. Ce résultat n'a pas été utilisé comme preuve.

## Limites

- Le vérificateur indique lui-même que la signature doit être contrôlée par une commande séparée. Cette commande a été lancée et a réussi.
- Le smoke SQLite ne lance pas l'application complète.
- Les erreurs anciennes de lint et de typecheck (serveur 16, web 10) restent la référence de base. Aucun nouveau lint ni typecheck n'a été lancé.
- Les recettes Rust 1840 et les autres recettes déjà validées n'ont pas été rejouées.
- Aucun contrôle git, cargo ni build n'a été lancé.
