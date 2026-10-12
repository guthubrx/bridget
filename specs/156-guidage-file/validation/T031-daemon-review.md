# T031 — lecture complète du diff daemon attribué

Lecture terminée le 2026-10-12T05:23:09.977698+00:00. Périmètre : les hunks des deux fichiers ci-dessous, tests compris. Aucun défaut nouveau156 confirmé dans ce périmètre. Ce constat ne certifie pas une revue complète du projet.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/crates/bridget-daemon/src/daemon.rs : patch lignes418–7963, tous les hunks lus. Le hunk802–1212 réutilise la lecture précédente du code8950–9375 ; ses410 lignes ajoutées/de contexte et zéro suppression correspondent exactement au segment actuel8951–9360. Les autres hunks ont été lus directement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/crates/bridget-daemon/src/daemon/native_child_requests155_tests.rs : patch lignes7964–8152, tous les hunks lus directement. Empreinte actuelle identique au fichier figé267f17533fbf0df0f99393f35bcdbae031347369b26f6564c91b94bc0441398f.

Diff source : /Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/diff-156-review.patch. SHA2561991ad415fa4573d6cfd6a3eea077528f322e6d95bb4d9d62e7695fd91dbfaa0. Le manifeste détaille chaque hunk, les plages réellement vues et la reprise des sorties tronquées.

## Constats de lecture

Les changements de forme couvrent plusieurs blocs historiques : attente, récupération, droits149/153 et fixtures de ronde153. La taille du diff seule ne constitue pas une erreur. Des anciens tests semblent supprimés au début d'un gros hunk mais réapparaissent dans les lignes7110–7453 du patch ; leurs assertions sont conservées.

Les changements sémantiques156 portent sur les gardes de guidage, l'origine attestée, les limites de rythme, la trace admitted liée au début de remise idempotente, les champs facultatifs de remise et la lecture de file. La lecture vérifie notamment les chemins Send/SendIdempotent/ControlExecution/Cancel, le refus steer avec réponse suivie avant réservation, la reprise par reçu existant, le contrôle avant émission physique et la capacité explicite QueueListV1. Elle constate le réemploi des stores, liens149 et fixtures existants. Aucun nouveau cadre générique n'est proposé.

Les tests ajoutés couvrent les droits, les parents réellement liés, les étiquettes d'origine hostiles, la pause, les bornes30s/600s, la persistance, l'ordre de file après réassignation, les limites de lecture et la négociation de capacité. Les quatre cas natifs156 réutilisent RequestFixture155. Les deux cas Cancel à socket synthétique prouvent l'annulation durable et la valeur du drapeau interrupt_active. Ils ne prouvent pas, seuls, un retrait physique dans le fournisseur ; cet effet relève de l'oracle ACP attribué ailleurs.

## Correctif des deux anciennes fixtures après FULL

Le delta /Users/moi/.cache/bridget-delivery156.iuy6stuo/daemon-guard-tests-prep/T030-two-old-fixtures-delta.patch a été relu intégralement. SHA256718af90339990a1037c4933916fb2d7226ca92f5a1636148016e4507c836ffad.

Le premier test inscrit un parent primaire puis un client auxiliaire avec son véritable credential et crée le lien parent/enfant durable. Le second remplace l'injection directe dans le routeur par une inscription attestée et le même lien. Les assertions historiques de remise, de refus fournisseur et de distinction des intentions restent présentes. Le helper déjà existant devient pub(super) uniquement dans le module de tests.

La reconstruction en mémoire depuis be7957ee et le patch figé retrouve exactement l'empreinte daemon initiale726d16d387cface2d882df55f563b40ec74ee14fb014036219a86ef154571268. Après application du delta, la comparaison avec le fichier actuel présente uniquement cinq hunks de mise en forme du helper et de ces deux fixtures. Tous ont été lus. Aucun changement de production supplémentaire n'apparaît dans cette comparaison.

Le reçu root /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-two-old-fixtures-green.json confirme RC0,2PASS/0FAIL en8,27s. Son journal /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-two-old-fixtures-green.log a l'empreinte3e848047dcac5c540d3d0a7d32767a6fc39ee912407abc7f4233d27a22b39aac, vérifiée à la lecture. Ce rejeu ciblé ne transforme pas le FULL antérieur en succès global.

## Limites

Les plages attestées sont celles du diff attribué, pas le corps entier historique des deux fichiers. ROOT couvre la production complémentaire et un autre owner couvre les autres fichiers Rust. Les éditions documentaires postérieures à la capture ne sont pas relues ici. Aucun Cargo, test, build, service ou écriture Git/source n'a été lancé par cet owner. Aucune affirmation de FULL global vert ni de Converge n'est faite.

Manifeste de lecture : /Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/T031-daemon-readmanifest.json.
