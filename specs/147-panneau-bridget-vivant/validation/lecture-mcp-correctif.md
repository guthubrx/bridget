# Complément147 — accès MCP et pagination

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-1).

## Cause vérifiée

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-3).

La pagination est déjà disponible : lire toute la page, confirmer son reçu avec ack, puis refaire le même read sur le même fil. Le serveur reprend après la dernière page confirmée. history sert à relire des preuves exactes et ne déplace pas ce repère. Une page est limitée à200 entrées et60Kio ; atteindre la borne d'octets peut produire moins de200 entrées.

## Correction

- CLI : l'aide décrit read → ack → même read. Le refus de --from-seq/--to-seq dans read explique cette suite et réserve les bornes à history.
- MCP : la description de bridget_thread et celles des bornes donnent la même procédure.
- Alertes : le texte reçu indique la répétition du même read après ACK.
- Skill et référence : découvrir aussi les outils différés du harnais, distinguer l'annuaire Bridget des outils internes Claude, ne pas déduire le catalogue du modèle GLM. Ne jamais ACK une page dont les corps ont été filtrés ou tronqués.
- Aucun changement de contrat, de permissions, de notifications ciblées, de données, d'Agent Loop ou d'acquittement automatique.

## Preuves initiales

Le nouveau test CLI a échoué sur l'ancien message : il ne contenait pas la suite ack → même read. Après correction, ce test passe.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-5).

Les résultats Rust finaux et la publication des compétences sont consignés après exécution. Aucun succès de livraison active n'est déduit d'un paquet préparé.

## Suite globale : limite conservée

La première exécution Rust sans TMPDIR court a donné894 réussites,210 échecs et12 ignorés. De nombreux échecs signalent explicitement une socket Unix trop longue. Après correction du dossier temporaire du harnais, la commande `TMPDIR=/tmp CARGO_TARGET_DIR=/Volumes/SD1TO/bridget-build-147-rust cargo test -p bridget-daemon --lib -- --test-threads=4` donne1094 réussites,10 échecs et12 ignorés.

Les dix échecs concernent des tests de spawn, de supervision, de reprise et d'arrêt. Les erreurs comprennent auxiliary_credential_required, Operation not permitted sur des fichiers fleet privés et des délais de superviseur. Ils ne sont ni corrigés ni masqués par ce complément. La suite globale n'est pas déclarée verte. Les tests des zones modifiées, du montage MCP et de la pagination sont distingués de ce résultat.

## Vérification ciblée

- CLI de fils :11 tests passent.
- Moteur de fils :19 tests passent.
- MCP :54 tests passent,1 test explicite Agent Loop ignoré.
- Intégration réelle sur daemon privé :11 tests passent avec le filtre spec102_v1. Lecture incrémentale, page201, réponse perdue, lectures simultanées, matrice ACK, budget UTF-8 et history sans déplacement de repère sont couverts.
- Total Rust ciblé :95 réussites, aucun échec.
- Validation de la skill et format Rust : succès ; git diff --check sans erreur.

Les suites unitaires ciblées utilisent le binaire de test compilé par Cargo, avec TMPDIR=/tmp. L'intégration utilise cargo test -p bridget-daemon --test spec102_threads_test spec102_v1, avec le même CARGO_TARGET_DIR externe. Aucun daemon de production n'est lancé ou arrêté.

## Publication sans redémarrage

Le correctif de code ff168d1ca802df3d7c3e2bb347b621854642a6ee est fusionné sur main et poussé sur github/main.

La skill est publiée par le publisher existant, limité à SKILLS=bridget. Codex et le registre agents restent liés à la source canonique. Le guide Claude est identique octet par octet ; ses références étaient déjà liées à la source.

Un second fait est vérifié : le profil /Users/moi/.claude-glm ne possédait pas de dossier skills. Le même publisher, avec SOURCE_ROOT=/Users/moi/.claude-glm/skills, crée uniquement l'entrée bridget vers la source Codex canonique. Guide et référence sont identiques. Aucune configuration MCP, permission, clé ou session active n'est modifiée. Cette publication ne prouve pas le rechargement du catalogue d'un tour déjà ouvert.

Compilation release du commit ff168d1c : succès en47.92s. Le binaire signé est remplacé par renommage atomique, sans relance, à /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget. SHA256 b7b0bb0ccc36289a374f5d684c8ed83cc1131745c33293c4ff096d46c065e0c6. Le CLI canonique affiche la nouvelle procédure. L'ancien binaire est conservé dans /Users/moi/.cache/bridget-147-lecture-backup.LO6P9D/bridget-before-ff168d1c.

Contrôle après remplacement : daemon93353, pont93355 et serveur T350035 sont toujours les mêmes processus. Le daemon et les MCP déjà ouverts conservent leur code/catalogue chargé ; les prochains appels CLI utilisent le nouveau fichier. Aucun redémarrage, tour de modèle, notification ou mission envoyé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-6).

## Livraison finale sur disque, sans activation

Mandat suivant du2026-10-09 : livrer les builds pour la prochaine relance, puis nettoyer branches, worktrees et anciens builds. Aucun redémarrage demandé ou effectué.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-7).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-8).

Le binaire Bridget corrigé reste installé au chemin canonique. Une copie exacte est conservée dans /Users/moi/.cache/bridget-delivery147-final.X8rgoK/bridget-ff168d1c. Le paquet ZIP147 et les reçus de compilation sont conservés. Les versions installées sur disque ne prouvent pas un chargement par les processus actifs.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-9).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-10).

La branche distante Bridget session-146-bridget-panel-lisible est supprimée après vérification de son inclusion dans main. Le worktree du correctif147 avait déjà été retiré avec sa branche propre. Aucun autre worktree n'est supprimé : Bridget142/145/146 et T3145 sont fusionnés mais contiennent encore des modifications non committées ; T3139 n'est pas inclus par ascendance dans local/v0.0.45. Les autres projets et branches ne sont pas des cibles.

## Livraison reconstruite après intégration complète — 2026-10-09

Ce checkpoint remplace l'état de livraison précédent. Le nouveau mandat demande explicitement de committer les reliquats, intégrer les branches, reconstruire les deux programmes, pousser, installer et nettoyer, sans redémarrage.

Bridget : anciens travaux142/145/146 committés puis intégrés parfa07c3d4,f6a7e743 et3aa2a3c3. La source propre1163038fc25f864962e50a2893125c63ee27c3b7 est poussée sur github/main et reconstruite avec un nouveau cache Cargo. Le test145 ajouté deux fois par la fusion est conservé une seule fois ; le code Rust final est identique à celui déjà validé sur main.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-11).

Skills : origine142 intégrée dans dotfiles/main end0d8b04d. Checkout utilisateur capitalisé puis fusion de l'historique142 en6b157ef3, poussée sur origin/104-bridge-role-auth. Cette dernière fusion ne change aucun fichier ; les symlinks Claude existants sont conservés. Les travaux Vigi/Horizon et les index préexistants hors périmètre ne sont pas inclus dans les nouveaux commits de ce complément.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-12).

La sélection CLI plus large donne85PASS/2FAIL. Les deux échecs de spawn signalent auxiliary_credential_required, déjà rencontré dans la suite globale. Ils restent visibles dans les reçus ; aucune suite globale verte n'est revendiquée et aucun wrapper/agent n'est relancé pour les contourner.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-13).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-14).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-15).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md#extrait-16).
