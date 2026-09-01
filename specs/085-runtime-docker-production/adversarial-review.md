# Revue adversariale interne - SPEC-085

## Verdict après corrections

PASS WITH GUARDS - Aucun blocker restant. L'implémentation devra prouver la saga et la frontière Docker par tests réels.

## Findings

### HIGH-085-01 - Projet Host impossible à promouvoir avec le contrat actuel

Le moteur sait enregistrer Docker initialement, mais refuse `prepare` pour une liaison Host. Correction: `activate_docker` devient une opération v2 atomique avec publication tardive et compensation.

### HIGH-085-02 - Privilège Docker hôte

Donner accès au daemon Docker est une capacité sensible. Correction: seul Bridget l'obtient; aucun socket n'est monté; l'installation vérifie l'accès sans modifier automatiquement groupes ou permissions. FR-08531 a été ajoutée.

### HIGH-085-03 - Image trop liée aux fournisseurs

Embarquer tous les clients dans la base couple les mises à jour et augmente le risque de credentials résiduels. Correction: base neutre, clients versionnés approuvés par couche ou ressource de politique. FR-08532 a été ajoutée.

### MEDIUM-085-04 - Seconde source de vérité dans des volumes Docker

Correction: états persistants sous une racine hôte administrée par bind mounts, sans volume nommé opaque. FR-08530 a été ajoutée.

### MEDIUM-085-05 - Portabilité non définie

Correction: cette version supporte explicitement Linux amd64 et refuse les autres architectures sans émulation. FR-08529 a été ajoutée.

### MEDIUM-085-06 - Topologie worktree changeante

Couvert par digest de topologie et recreate obligatoire, avec chemins absolus identiques.

## Points solides

- Réutilisation directe de SPEC-066/067.
- Aucun fallback Host silencieux.
- Aucun argument Docker libre.
- Dépôt et worktrees restent sur l'hôte et réversibles.

