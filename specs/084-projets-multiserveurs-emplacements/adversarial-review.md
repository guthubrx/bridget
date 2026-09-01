# Revue adversariale interne - SPEC-084

## Verdict après corrections

PASS WITH GUARDS - Aucun blocker restant. Deux risques élevés ont été corrigés avant tasks.

## Findings

### HIGH-084-01 - Autorité inter-serveurs ambiguë

Le premier plan pouvait laisser croire que le panneau web d'un daemon pilote une autre source. Correction: la source est choisie dans la coque Bridget Desktop; chaque panneau reste strictement local à son daemon. FR-08425 et le plan ont été complétés.

### HIGH-084-02 - Migration v1 restrictive mais impact mal visible

Traiter v1 comme exact est sûr, mais peut empêcher de nouveaux imports sous des descendants historiquement admis. Correction: la prévisualisation doit inventorier les projets déjà liés sous chaque ancienne racine et présenter l'impact sans les modifier. FR-08424 a été ajoutée.

### MEDIUM-084-03 - Identités locales identiques

Risque de collision dans les clés de rendu Desktop. Couvert par `(source_id, project_id)` et des tests de libellés/identités identiques.

### MEDIUM-084-04 - Imbrication d'emplacements

Deux workspaces imbriqués peuvent produire une autorité incertaine. Couvert par refus des imbrications contradictoires et choix explicite par `location_id`.

## Points solides

- Aucun registre central.
- Aucun chemin par défaut codé en dur.
- Compatibilité v1 fail-closed.
- Retrait de catalogue sans destruction des projets existants.

