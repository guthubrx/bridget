# Session149 — Sous-agents natifs Bridget visibles dans Lineage adaptateur de bureau

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-2).

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-149-sous-agents-lineage`
**Created**: 2026-10-10
**Status**: Livré sur disque, non activé (45/45 tâches validées ; aucun redémarrage)
**Priority**: P1
**Dependencies**: SPEC-148

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-3).

## Besoin et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-6).

L'héritage n'ajoute aucune politique restrictive nouvelle. Il n'introduit aucun
contournement global des permissions.

La livraison remplace des fichiers, comme en 147 et 148. Elle ne relance aucune
application ni aucun service. Les processus en cours continuent avec leur code
chargé. Le nouveau code s'active à la prochaine relance humaine. Aucune activation
différée n'est programmée.

## Scénarios utilisateur

### US1 — Enfant natif visible dans Lineage (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-7).

### US2 — Journal consultable sans effet de bord (P1)

L'utilisateur ouvre le journal de l'enfant depuis Lineage. Il lit les faits et
comptes rendus publiés par l'enfant. La lecture ne relance pas la mission, ne
fait pas avancer les phases et n'ajoute aucun message. La fin de tour avec travail
enfant actif ne clôt pas la mission, comme en 148.

### US3 — Héritage d'autorisations sans gain de droits (P1)

L'enfant GLM ou Codex lit et écrit dans les bornes effectives de son parent,
attestées au lancement. Sans posture explicite, l'enfant hérite de ces droits.
La posture découverte réduit à la lecture seule.

Aucun grant humain Bridget nouveau n'est demandé. Aucun dialogue de validation
de permission n'est introduit. Une action permise au parent réussit côté enfant.
Une action hors politique est refusée côté enfant, avec un refus nommé.

Les désactivations MCP et les configurations explicites de l'humain restent
respectées. Une politique dont la cible n'a pas d'équivalent sûr produit un
refus nommé. Aucune assertion de confinement que le fournisseur n'applique pas
n'est émise. La projection Lineage reste en lecture seule. La forme exacte des
contrats d'attestation et de lecture reste à verrouiller en planification.

### US4 — Mêmes capacités dans un terminal (P1)

Le même `bridget_delegate` depuis un agent par le parcours natif Bridget produit les mêmes capacités de
délégation et de lecture/écriture. dans un environnement natif isolé ne bloque ni la création, ni
l'exécution, ni le résultat, ni l'annulation. La visibilité Lineage est une
projection optionnelle, jamais une condition d'exécution.

### US5 — Opt-out, révocation et survie du moteur (P1)

Les opt-outs explicites existants de Bridget et de MCP restent en vigueur.
Aucun réglage nouveau de projection n'est inventé. Une preuve de session absente,
étrangère, expirée ou révoquée ne rend aucune projection et ferme l'accès aux
nouvelles opérations, comme en 148.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-8).

### US6 — Scénarios dégradés documentés (P2)

Six situations sont spécifiées et documentées :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-9).

### US7 — Livraison sans redémarrage (P2)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-10).

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-11).

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-12).

## Entités et hypothèses

Le store natif Bridget de la session148 reste la source de vérité : tâche liée à
son parent, enfant avec identifiant stable, reçu de mission, résultat final
corrélé, journal. La vue Lineage est une projection en lecture de ce store. Le
travail de lineage de 147 (PID, birth, résolution NativeTree) reste un socle
d'observation des processus ; il ne devient pas la source de vérité de la
projection. Le lien parent/enfant projeté dérive de l'enfant managed prouvé par
le store natif.

Les droits de l'enfant sont figés au lancement, depuis l'attestation effective
du parent. Une nouvelle délégation du même parent part des droits attestés au
moment de sa propre admission. La mission ne reçoit pas la conversation du
parent, comme en 148. Aucun nouveau panneau, aucun calendrier et aucun
recrutement interprojet automatique n'est ajouté. Le profil GLM et son
authentification existants ne sont ni recréés ni modifiés.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/spec.md#extrait-13).
