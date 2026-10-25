# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/livraison-release-0.2.1.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/livraison-release-0.2.1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le journal décrit le correctif à la source et la compatibilité T3 requise.
- Le guide d'installation déconseille la boucle de réinstallation et l'augmentation de la limite de fichiers.
- La référence de délégation décrit le remontage de la preuve privée avant `turn/start`.
- Le README reste inchangé. Sa règle de mise à jour conjointe et de redémarrage des processus est déjà exacte.

<a id="extrait-2"></a>

## Extrait 2

- Format ciblé des trois fichiers Rust affectés : conforme.
- Cycle de vie 165 : 13 PASS.
- Module T3 : 102 PASS.
- Contrats T3 : 22 PASS et 1 test ignoré.
- Build release Mac verrouillé et hors ligne : conforme.
- Build release Linux x86_64 verrouillé et hors ligne : conforme.
- Le format global reste rouge sur des fichiers historiques hors du correctif. Aucun fichier étranger n'a été reformaté.
- Aucun résultat GREEN global du workspace n'est revendiqué.

<a id="extrait-3"></a>

## Extrait 3

- Le pont T3 reste `running` avec 32 fils.
- Le pont conserve 171 fichiers ouverts lors du contrôle final. Le serveur T3 en conserve 59.
- La tâche d'activation `com.bridget.t3-activate166` est absente.
- Les dernières erreurs `Too many open files` du journal datent d'avant l'activation du correctif.
- Le daemon actif garde le BuildID `3b4f101e0732`. Il contient le correctif de cycle de vie.
- Le client sur disque est `0.2.1`. Le daemon n'a pas été relancé : 58 agents étaient connectés et la publication ne demandait pas cette interruption.
- T3 Code actif reste en version `0.0.45-local.166`, source `76780a20bb22b9390eab2389eaf0db87e1349e7e`.

<a id="extrait-4"></a>

## Extrait 4

- Les branches `session-165*` et `session-166*` sont absentes des dépôts Bridget et T3.
- Le dépôt Bridget ne garde que son worktree principal.
- Les worktrees T3 étrangers aux sessions 165 et 166 sont conservés.
- Les métadonnées de worktrees périmées ont été élaguées dans les deux dépôts.
- Le clone Linux de construction a été retiré après contrôle des actifs.
- Le script de construction temporaire et l'archive d'inspection de 0.2.0 ont été placés dans la corbeille.
- Les actifs 0.2.1 et le retour arrière T3 166 restent conservés.
