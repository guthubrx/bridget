# Checklist qualité de spécification: Profils, extensions et secrets bornés par projet

**But**: valider la complétude avant planification
**Créée**: 2026-08-29
**Feature**: [spec.md](../spec.md)

## Qualité du contenu

- [x] Valeur et risque principal explicités
- [x] Frontière de confiance projet compréhensible
- [x] Limite intra-projet annoncée sans promesse trompeuse
- [x] Sections obligatoires complètes

## Complétude des exigences

- [x] Aucun marqueur NEEDS CLARIFICATION
- [x] Exigences testables et bornées
- [x] Critères mesurables sans dépendre du code
- [x] Approbation, extension, secret, rotation et portabilité couverts
- [x] Cas limites permissions, symlinks, collision et fuite couverts
- [x] Hors périmètre broker, mémoire et conteneur par agent explicite
- [x] Dépendances 064-066 identifiées

## Préparation de la feature

- [x] Valeurs secrètes absentes des autorités durables
- [x] Approbation locale obligatoire
- [x] Recréation et rollback définis
- [x] Contrat générique Codex, Claude et Cursor
- [x] Rebind invalide génération, politique et approbation avant toute ressource
- [x] Redaction process-env causale avant tout sink durable
- [x] Redaction binaire conserve son état entre fragments et canaux
- [x] Rebind conserve les exécutions actives mais refuse toute admission
- [x] source_ref possède un catalogue hôte fermé et une liste de projets autorisés
- [x] SecretSourceStamp détecte une mutation hors rotation sans contenu secret
- [x] runtime_policy_version est épinglée sur toutes les décisions
- [x] Switch backend et changement de politique rendent le profil stale
- [x] L'UID/GID runtime est compatible avec les secrets privés `0600`
- [x] Les incidents SPEC-068 ne conservent que code fermé, référence pseudonymisée et ProjectReference
- [x] Les scanners couvrent trame, store, notification, rejeu et acquittement SPEC-068
- [x] L'absence de `.specify` est non bloquante et ne déclenche aucune mise à jour

## Notes

- Validation effectuée le 2026-08-29, itération 2 après clarification de
  l'accessibilité des secrets à tous les agents du projet.
- Le statut reste `Draft` jusqu'à approbation humaine.
- Complément RC8 validé le 2026-08-30 sans credential réel.
- Relecture indépendante intégrée contre `main` à `d589b24`; aucune valeur
  secrète ni tâche d'implémentation n'a été ajoutée.
