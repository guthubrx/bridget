# Correction203 — Placement natif SYS sur le grand volume

## Besoin
Les futures préparations/worktrees Opus2D SYS doivent utiliser le dépôt déjà provisionné sur /mnt/storage. Le mapping actuel pointe vers / et provoque DiskFull : seulement7,2Gio libres pour9,24Gio de fichiers suivis auSHA49d905b81. Préserver tout travail, anciennes tâches et clés, Cartae et les services métier. Aucune mission de remplacement ni copie de clés.

## Périmètre
Correction de configuration existante, pas de nouvelle API ni de nouveau binaire. Garder project_id opus2d et source_root Mac inchangés. Modifier seulement destination et identitéGit SYS, incrémenter mapping_revision. Publication seulement après preuve qu'aucune tâche SYS ne possède encore une garde active et qu'aucune commande worker n'est non terminale. Une tâche historique fermée ne devient pas continuable par ce changement.

## Acceptation
Les autorités maître/worker/origine concordent ; les autres champs sont identiques. La préparation Git native importe/réutilise le SHA dans le dépôt du grand volume sans changer son HEAD. Les dépendances existantes restent présentes. Aucun worktree ancien ni service n'est déplacé/retiré/redémarré. Les propriétaires reçoivent le parcours des futures admissions et conservent les clés annulées.
