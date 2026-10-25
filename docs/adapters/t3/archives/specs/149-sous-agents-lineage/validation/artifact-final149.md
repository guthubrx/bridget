# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/artifact-final149.md](../../../../../../../specs/149-sous-agents-lineage/validation/artifact-final149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Commit T3 : `f6f3d1998c686b20e316f304aff7f5df6a14592c`
- Version : `0.0.45-local.149`
- Dossier de paquet : `/Users/moi/.cache/t3-final149.K5dHqa`
- Application contrôlée (stage) : `/private/tmp/b149.l0KEOR/t3code-desktop-mac-stage-RSh9aq/app/dist/mac-arm64/T3 Code (Alpha).app`
- Zip contrôlé : `/Users/moi/.cache/t3-final149.K5dHqa/artifacts/T3-Code-0.0.45-local.149-arm64.zip`

<a id="extrait-2"></a>

## Extrait 2

| Contrôle | Résultat | Preuve |
|---|---|---|
| Vérificateur final (`verify-final.mjs`) | Réussi : 2576 contrôles OK, 0 échec | `/Users/moi/.cache/t3-final149.K5dHqa/final-artifact-check.json` |
| Installation et redémarrage | Non faits (`installed: false`, `restarted: false`) | même fichier |
| Smoke SQLite (Electron en mode Node) | Réussi : lecture durable OK, `quick_check` = ok | `/Users/moi/.cache/t3-final149.K5dHqa/sqlite-smoke-final.json` |
| Base SQLite du smoke | Nouvelle base privée dans packageRoot | `/Users/moi/.cache/t3-final149.K5dHqa/sqlite-smoke-final.db` |
| `codesign --verify --deep --strict` | Réussi : "valid on disk" et "satisfies its Designated Requirement" | sortie de commande, code retour 0 |
