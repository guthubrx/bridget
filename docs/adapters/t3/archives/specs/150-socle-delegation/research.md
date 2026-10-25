# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/research.md](../../../../../../specs/150-socle-delegation/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Décision** : garder au plus 4096 tâches au total. Quand le total est
  atteint, purger d'abord les arbres purgeables les plus anciens, même avant
  7 jours. Si rien n'est purgeable, refuser `retention_saturated`.
- **Raison** : 4096 est déjà publié aux lecteurs T3
  (`specs/149-sous-agents-lineage/contracts/lineage.md`, contrat 148). Les
  parcours bornés à 4096 (`delegation_lineage.rs:145`,
  `native_delegation.rs:1314`, `:1408`) restent inatteignables.
- **Remarque** : le plafond de 4096 reçus d'annulation
  (`delegation_lineage.rs:234`) reste indépendant ; la purge retire les reçus
  des arbres purgés.

<a id="extrait-2"></a>

## Extrait 2

- **Décision** : une purge qui supprime au moins un arbre change la génération
  de projection (nouvel UUID, seq 1). Après le commit, chaque propriétaire
  racine touché est signalé par le mécanisme existant `committed()`
  (`delegation_lineage.rs:107`).
- **Raison** : `publish` transforme un changement de génération en `Resync`
  (`native_lineage.rs:144-162`) et le contrat T3 exige un snapshot complet
  (`lineage.md:113-118, 217`). La contre-revue confirme la compatibilité
  (objection 10).
- **Limite** : la génération est globale. Les lecteurs non touchés verront un
  curseur périmé à leur prochaine lecture et feront un snapshot. Coût : au plus
  un snapshot par passage d'entretien.
