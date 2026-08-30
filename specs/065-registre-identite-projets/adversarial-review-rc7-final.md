# Contre-revue adverse RC7 - verdict final ciblé

**Périmètre relu** : fermeture de M1 et du finding LOW de l'itération 2 dans
les SPEC-065, SPEC-066, SPEC-067 et ADR-016.

## VERDICT: APPROVE

M1 et le finding LOW sur le rebind sont fermés. Aucun finding nouveau, aucune
incohérence introduite et aucun élément bloquant n'ont été trouvés dans le
périmètre ciblé.

Le passage à l'implémentation demeure conditionné aux gates SPEC-063 et
SPEC-064, aux gates propres aux specs et à une validation humaine explicite.

## État des findings

### M1 - redaction `process_env` par flux

**État : CLOSED.**

`OutputRedactionLease` définit désormais des motifs binaires, un état distinct
par canal, un tampon de suffixe candidat entre lectures et la règle de
fermeture du flux : aucun octet candidat ne sort avant décision, y compris à
une frontière de retour ligne. Seuls les octets filtrés atteignent
`JournalWriter`, les logs, métriques, événements et crash reports; le tampon et
la lease sont bornés puis détruits après fermeture complète.

Les tâches T023 et T029 imposent une sentinelle émise après le démarrage,
entière puis découpée au milieu et au retour ligne sur chaque canal, avec
inspection du `JournalWriter` réel. Le mutant qui réinitialise l'état ou
transmet avant comparaison est explicitement requis. Il tue donc le témoin à
l'assertion de fuite, et non au montage ni au spawn.

Références :

- `specs/067-profils-extensions-secrets-projet/data-model.md`,
  `OutputRedactionLease` et invariants;
- `specs/067-profils-extensions-secrets-projet/contracts/project-profile-v1.md`,
  `Secret process-env`;
- `specs/067-profils-extensions-secrets-projet/tasks.md`, T023 et T029;
- `specs/067-profils-extensions-secrets-projet/spec.md`, FR-038.

### LOW - rebind pendant une exécution active

**État : CLOSED.**

La politique est maintenant univoque et cohérente entre les trois specs : un
rebind rend le profil `stale` avant toute résolution de SecretRef ou montage,
classe l'environnement `recreate_required` et refuse toute nouvelle admission.
Les agents déjà actifs terminent toutefois sur leur ancienne génération; le
système ne relit aucune ressource pour eux et ne les arrête pas implicitement.

Cette politique est portée par les exigences, le modèle, le contrat et les
tâches de réconciliation et de preuve. Elle respecte simultanément
l'invalidation des nouvelles expositions et la non-destruction de SPEC-065.

Références :

- `specs/065-registre-identite-projets/spec.md`, FR-026 et T019/T021;
- `specs/066-environnement-partage-projet/spec.md`, FR-035 et T025;
- `specs/067-profils-extensions-secrets-projet/data-model.md`, invariants;
- `specs/067-profils-extensions-secrets-projet/contracts/project-profile-v1.md`,
  `Invalidation par rebind`;
- `specs/067-profils-extensions-secrets-projet/spec.md`, FR-037 et T030;
- `docs/decisions/016-environnements-partages-par-projet.md`, décision 11.

## Contrôle des incohérences introduites

- La redaction inter-fragments reste une frontière de fuite accidentelle, sans
  promesse d'isoler un agent non fiable qui possède légitimement le secret.
- La continuité des anciens agents n'autorise ni réadmission, ni réutilisation
  du conteneur divergent : `recreate_required` impose la recréation après leur
  terminalité.
- La règle de non-arrêt implicite n'entre pas en conflit avec rotation/révocation,
  qui restent des opérations explicites exigeant arrêt puis recréation.

Aucun finding additionnel n'est ouvert.
