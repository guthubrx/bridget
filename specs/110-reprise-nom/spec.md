# Spécification 110 — Reprendre un nom d'affichage laissé par une identité éteinte

## Fiche synthèse

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/spec.md#extrait-1).

## Problème observé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/spec.md#extrait-2).

Cause : le nom d'affichage est unique et durable, et rien ne le libère quand son détenteur
disparaît. Chaque réimport de conversation crée une identité neuve qui ne peut pas reprendre son
ancien nom. Mesure du jour : 268 profils enregistrés pour 16 agents connectés, dont 253 dormants
depuis plus de deux jours.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/spec.md#extrait-3).

## Scénarios utilisateur

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/spec.md#extrait-4).

## Exigences fonctionnelles

- **FR-001** : une demande de nom déjà détenu est acceptée si et seulement si le détenteur est
  absent de l'annuaire vivant au moment de la demande.
- **FR-002** : si le détenteur est vivant, le refus reste `NameConflict`, inchangé.
- **FR-003** : lors d'un transfert, l'ancien détenteur reçoit un nom dérivé disponible, construit
  à partir de son nom courant ; il conserve son identifiant, son avatar et ses instructions.
- **FR-004** : le transfert est atomique : soit les deux profils changent, soit aucun.
- **FR-005** : la révision de chaque profil touché est incrémentée, pour que les lecteurs voient
  le changement.
- **FR-006** : la résolution d'un nom vers un identifiant continue de rendre le détenteur courant.
- **FR-007** : aucune identité n'est créée, supprimée ni fusionnée ; seul le libellé bouge.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/spec.md#extrait-5).

## Hors périmètre

Purge des profils dormants, fusion d'identités, libération automatique par ancienneté sans
consultation de l'annuaire, changement du format de sérialisation.
