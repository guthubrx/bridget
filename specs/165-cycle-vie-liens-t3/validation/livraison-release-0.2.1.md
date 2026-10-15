# Livraison de Bridget 0.2.1

Date : 2026-10-15.

## Publication

- Version : `0.2.1`.
- Commit de release : `fc227c4590b7cd3154e734140902c88b1cbdf268`.
- Tag annoté : `v0.2.1`, cible exacte du commit de release.
- Publication : https://github.com/guthubrx/bridget/releases/tag/v0.2.1
- État GitHub : publiée, non brouillon, non préversion et marquée `Latest`.

## Actifs

- Archive Linux x86_64 : `bridget-0.2.1-linux-x86_64.tar.gz`.
- SHA-256 de l'archive : `d5ed8f680dfe9f89718db5ed3a4f5271141381fbd8e3295488aae8dc54cd5668`.
- SHA-256 du binaire Linux : `f976b4f26cca3d5ecf9b64bfa38d315798dc43beae91d5e549b5d771f3dc48b8`.
- Le digest publié par GitHub correspond au fichier local vérifié.
- Le binaire Linux annonce `bridget 0.2.1` et embarque le commit de release.
- Binaire Mac construit : `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
- SHA-256 du binaire Mac : `e65736533bb81a30d256f9a77fb9b918e35deb68e7418c5bd33f4131d450d4ab`.
- La signature ad hoc du binaire Mac est valide. Le client annonce `bridget 0.2.1`.

## Documentation

- Le journal décrit le correctif à la source et la compatibilité T3 requise.
- Le guide d'installation déconseille la boucle de réinstallation et l'augmentation de la limite de fichiers.
- La référence de délégation décrit le remontage de la preuve privée avant `turn/start`.
- Le README reste inchangé. Sa règle de mise à jour conjointe et de redémarrage des processus est déjà exacte.

## Validation

- Format ciblé des trois fichiers Rust affectés : conforme.
- Cycle de vie 165 : 13 PASS.
- Module T3 : 102 PASS.
- Contrats T3 : 22 PASS et 1 test ignoré.
- Build release Mac verrouillé et hors ligne : conforme.
- Build release Linux x86_64 verrouillé et hors ligne : conforme.
- Le format global reste rouge sur des fichiers historiques hors du correctif. Aucun fichier étranger n'a été reformaté.
- Aucun résultat GREEN global du workspace n'est revendiqué.

## État actif

- Le pont T3 reste `running` avec 32 fils.
- Le pont conserve 171 fichiers ouverts lors du contrôle final. Le serveur T3 en conserve 59.
- La tâche d'activation `com.bridget.t3-activate166` est absente.
- Les dernières erreurs `Too many open files` du journal datent d'avant l'activation du correctif.
- Le daemon actif garde le BuildID `3b4f101e0732`. Il contient le correctif de cycle de vie.
- Le client sur disque est `0.2.1`. Le daemon n'a pas été relancé : 58 agents étaient connectés et la publication ne demandait pas cette interruption.
- T3 Code actif reste en version `0.0.45-local.166`, source `76780a20bb22b9390eab2389eaf0db87e1349e7e`.

## Nettoyage

- Les branches `session-165*` et `session-166*` sont absentes des dépôts Bridget et T3.
- Le dépôt Bridget ne garde que son worktree principal.
- Les worktrees T3 étrangers aux sessions 165 et 166 sont conservés.
- Les métadonnées de worktrees périmées ont été élaguées dans les deux dépôts.
- Le clone Linux de construction a été retiré après contrôle des actifs.
- Le script de construction temporaire et l'archive d'inspection de 0.2.0 ont été placés dans la corbeille.
- Les actifs 0.2.1 et le retour arrière T3 166 restent conservés.
