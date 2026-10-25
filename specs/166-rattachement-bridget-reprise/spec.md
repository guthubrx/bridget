# Feature Specification: Rattachement Bridget après reprise adaptateur de bureau

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-2).

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-166-rattachement-bridget-reprise`  
**Created**: 2026-10-15  
**Status**: Implemented
**Priority**: P1  
**Dependencies**: SPEC-149, SPEC-163, SPEC-165

## Problème

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-3).

La réinstallation du pont ou l'augmentation de limites système ne corrige pas cette incohérence. L'utilisateur doit pouvoir reprendre un fil ancien, une bifurcation ou une conversation principale sans réparer manuellement le connecteur.

## Histoires utilisateur

### US1 — Reprendre un fil sans réparer Bridget (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-4).

**Test indépendant** : un fil reçoit une identité, devient inactif, perd sa session fournisseur, puis reprend un tour. Son premier appel Bridget réussit avec l'identité du fil repris.

**Scénarios d'acceptation** :

1. Étant donné un fil inactif dont la session fournisseur a été recyclée, quand un nouveau tour démarre, alors les outils Bridget utilisent une preuve privée valide pour ce tour.
2. Étant donné un fil repris dans une session fournisseur partagée, quand une conversation voisine est déjà active, alors chaque fil conserve sa propre identité.
3. Étant donné une reprise réussie, quand plusieurs appels Bridget sont effectués pendant le même tour, alors ils restent autorisés sans rotation inutile.

### US2 — Préserver les refus de sécurité (P1)

Comme utilisateur, je veux que la réparation automatique ne permette jamais à un fil d'emprunter l'identité d'un autre fil.

**Test indépendant** : une preuve ancienne, révoquée ou appartenant à un autre fil reste refusée après la reprise.

**Scénarios d'acceptation** :

1. Étant donné une ancienne preuve privée, quand la reprise a chargé une preuve neuve, alors l'ancienne preuve reste refusée.
2. Étant donné deux fils attachés à la même session fournisseur, quand ils appellent Bridget, alors chaque attestation désigne son propre fil.
3. Étant donné une reprise sans fait de droits actif, quand une opération exige des droits hérités, alors elle reste refusée selon les règles existantes.

### US3 — Conserver la continuité des conversations (P1)

Comme utilisateur qui reprend un travail long, je veux conserver l'historique, le contexte et le processus courant pendant la réparation du rattachement.

**Test indépendant** : la reprise ne crée pas une nouvelle conversation et ne supprime aucun message, résultat ou lien Bridget existant.

**Scénarios d'acceptation** :

1. Étant donné un fil ancien avec un historique, quand son identité est renouvelée, alors son identifiant et son historique restent inchangés.
2. Étant donné un processus fournisseur partagé encore vivant, quand un seul fil reprend, alors les autres fils ne sont pas interrompus.
3. Étant donné un échec de renouvellement, quand le fil appelle Bridget, alors il reçoit un refus explicite et aucun repli vers une identité voisine.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-5).

## Cas limites

- Le fil reprend pendant qu'une autre conversation utilise déjà la session fournisseur partagée.
- La preuve enregistrée existe encore, mais le registre actif ne la reconnaît plus.
- Le processus fournisseur conserve une configuration ancienne après une rotation.
- Deux reprises du même fil arrivent presque en même temps.
- Le nouveau tour se termine pendant la vérification de la preuve.
- La reprise échoue avant le chargement complet des outils.
- Le fil est supprimé ou archivé pendant la reprise.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-6).

## Hypothèses

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-7).

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-8).

## Dépendances

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/spec.md#extrait-9).
