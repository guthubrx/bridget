# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/checklists/requirements.md](../../../../../../../specs/147-panneau-bridget-vivant/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] US147-07 P1 et exigences de catalogue/identité/opt-out/permissions approuvées.
- [x] Cible signalée identifiée : regional-wrkr-1 / claude_glm / home .claude-glm.
- [x] Configuration MCP effective et chemin du cas exact corroborés : regional-wrkr-1/claude_glm, home .claude-glm, mcp get bridget RC1 et injection T3 t3-code seulement.
- [x] Plan MCP concret et réutilisations validés par le principal avant tâches/code US7.
- [x] Tests catalogue/options SDK/ vraie façade MCP privée sans modèle définis sur les chemins effectifs, T035–T041.
- [x] Barrière avant sessions.set/ready/prompt ; timeout ferme candidate/consumer et interdit montage tardif.
- [x] Métadonnées MCP fail-closed : NotFound distinct d'I/O ; Skills conserve son comportement fail-open.

<a id="extrait-2"></a>

## Extrait 2

Cause locale précise vérifiée dans diagnostic-mcp.md. Reçus SDK294PASS/1SKIP finaux, façade OS3PASS et qualification refresh1PASS ; régressions identité22/14. Revues finales lifetime/flags APPROVE et GO principal autorisent T049/T050 cochées, preuves292/293 antérieures conservées. Les preuves restent distinctes, pas chaîne T3+SDK+modèle complète. Aucun modèle/restart/production. Clôture globale US8/US9 encore ouverte.

<a id="extrait-3"></a>

## Extrait 3

- [x] US147-09 P2 approuvée dans147, pas nouvelle session148.
- [x] Ligne choisie mise en valeur native T3, bloc inférieur répété retiré, détails accessibles.
- [x] Hors première page/recherche et retour A→B→A couverts par les critères, sans sélection automatique.
- [x] Pas nouveau contrat/API/store, rebrand ou modification des communications/permissions.
- [x] Plan Web court et gate5/5 de réutilisation approuvés avant tâches/code US9 : item27, wrappers/styles natifs relus.
- [x] Cinq groupes RED comportementaux et aperçu isolé définis sur les composants natifs existants.

<a id="extrait-4"></a>

## Extrait 4

- [x]31FR/20SC/9US mappées après deux passes code↔artefacts, aucun orphelin ni finding actif.
- [x] US8 RED/GREEN, interop CLI/MCP privée et non-régressions reçus ; limites de qualification conservées.
- [x] US9 correction FR31 RED/GREEN122PASS, build/statiqueRC0 et recette native finale validés ; titre/membres complets et sélection accessible.
- [x] Audits frais Rust/T3 validés RC0/0erreur/0warning, grades limités aux deltas et constats corrigés historiques conservés.
- [x] SHA tasks identique pendant Converge,54/54 cochées seulement sur GO final.
- [x] NON installé/NON activé ; preuves SDK, autorité, publication et DOM synthétique non assemblées en fausse preuve E2E.
