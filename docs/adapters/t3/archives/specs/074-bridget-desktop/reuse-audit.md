# Archives techniques de l’adaptateur T3

Document d’origine : [specs/074-bridget-desktop/reuse-audit.md](../../../../../../specs/074-bridget-desktop/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Coque desktop et consultation multi-vues | Étude GUI T3 Code seulement documentaire | `docs/decisions/009-gui-plan-de-controle.md:20` | Créer le client minimal propre au dépôt: aucun composant source Bridget réutilisable n'existe. |
| Diagnostics sans secret | Projections redacted et tests de contenu secret | `crates/bridget-transport/src/codex_app_server.rs:2937`, `scripts/test-bridget-idle.sh:1584` | Réutiliser la règle de non-divulgation, créer le format desktop limité à son besoin. |
