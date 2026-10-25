# Archives techniques de l’adaptateur T3

Document d’origine : [specs/161-client-cartae/reuse-audit.md](../../../../../../specs/161-client-cartae/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/.speckit/constitution.md, I/III/X/XVI | Français, pipeline et worktree dédié | Artefacts161 en français ; pas d'édition du checkout main pour coder. |
| /Users/moi/.speckit/constitution.md, XVIII | Complexité explicite | Hash linéaire dans les octets ; liste réutilisée, pas scans inutiles répétés. |
| /Users/moi/.speckit/constitution.md, XIX/XX | Réemploi et charge future | Un déployeur et un harness ; pas de framework de packaging ou nettoyage. |
| /Users/moi/.speckit/constitution.md, XXI/XXII | Diagnostic réel et vérifications proportionnées | Après deux échecs de même hypothèse, observer ; pas de FULL Rust répété pour le script seul. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/AGENTS.md et consignes utilisateur de session | Préserver T3/maître/pont ; livraison et nettoyage requis | Aucun restart, aucune DB supprimée ; préserver travaux étrangers ; reçus distincts. CLAUDE.md absent, constaté par lecture. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/constitution.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/standards.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/user-layer.json | Constitution globale et couche utilisateur canonique | Pas de modification directe du toolkit. Références globales consultées selon ce lot script. |
| /Users/moi/.speckit/ref/code-quality-details.md ; /Users/moi/.speckit/ref/standards-tests.md | Contrats aux frontières, erreurs propagées, mocks techniques | Préflight, refus sans activation, pipefail ; réemploi du harnais technique existant, recette réelle séparée. |
