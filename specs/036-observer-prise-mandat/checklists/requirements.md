# Checklist qualité — Observer la prise d'un mandat

**But** : valider la complétude de la spécification avant implémentation.

**Feature** : `specs/036-observer-prise-mandat/spec.md`

## Qualité du contenu

- [x] Le problème et la valeur opérateur précèdent les détails techniques.
- [x] Les scénarios couvrent le cas bloqué, le cas sain et l'instrument muet.
- [x] Toutes les sections obligatoires sont renseignées.

## Complétude des exigences

- [x] Aucun marqueur de clarification ne subsiste.
- [x] Les exigences sont testables et non ambiguës.
- [x] Les critères de succès sont mesurables.
- [x] Les scénarios d'acceptation sont définis dans les deux sens.
- [x] Les cas source absente, vide, illisible, ambiguë et distante sont bornés.
- [x] La portée locale et le lot de fédération séparé sont explicites.
- [x] La dépendance SPEC-023 est déclarée dans l'en-tête.

## Préparation de la feature

- [x] Le seuil de 60 secondes est justifié par trois bornes mesurées.
- [x] Le spécimen à 720,857 secondes est documenté avec ses sources.
- [x] Les formes avec et sans `[Pasted Content]` sont couvertes sans lire le
  rendu de la console.
- [x] Le seuil idle et le steering sont séparés par deux spécimens réels
  opposés (`cartae0` / `rc7`).
- [x] Le cardinal obligatoire empêche un succès vide.
- [x] Aucun changement du transport de remise n'est autorisé.
