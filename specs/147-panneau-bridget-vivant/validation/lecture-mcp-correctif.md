# Complément147 — accès MCP et pagination

Mandat du 2026-10-09 : corriger, committer, fusionner et pousser. Ne redémarrer ni T3 ni Bridget. Ce complément reste dans la session147.

## Cause vérifiée

L'agent Relance utilise Claude/GLM. L'application installée porte encore la version0.0.45-local.146. Le montage MCP Claude/GLM est déjà corrigé dans les sources147 de T3 ; aucun second montage ni changement global des configurations fournisseur n'est nécessaire.

L'activation147 précédente a échoué avant le remplacement de l'application, à la phase staging_path. Le reçu conserve aborted_before_replacement ; aucune base T3 n'a été restaurée ou copiée. Le job com.bridget.install147, arrêté en échec, a été retiré pour empêcher une activation ultérieure non demandée.

La pagination est déjà disponible : lire toute la page, confirmer son reçu avec ack, puis refaire le même read sur le même fil. Le serveur reprend après la dernière page confirmée. history sert à relire des preuves exactes et ne déplace pas ce repère. Une page est limitée à200 entrées et60Kio ; atteindre la borne d'octets peut produire moins de200 entrées.

## Correction

- CLI : l'aide décrit read → ack → même read. Le refus de --from-seq/--to-seq dans read explique cette suite et réserve les bornes à history.
- MCP : la description de bridget_thread et celles des bornes donnent la même procédure.
- Alertes : le texte reçu indique la répétition du même read après ACK.
- Skill et référence : découvrir aussi les outils différés du harnais, distinguer l'annuaire Bridget des outils internes Claude, ne pas déduire le catalogue du modèle GLM. Ne jamais ACK une page dont les corps ont été filtrés ou tronqués.
- Aucun changement de contrat, de permissions, de notifications ciblées, de données, d'Agent Loop ou d'acquittement automatique.

## Preuves initiales

Le nouveau test CLI a échoué sur l'ancien message : il ne contenait pas la suite ack → même read. Après correction, ce test passe.

Les tests T3 ClaudeMcp.test.ts et ClaudeAdapter.test.ts passent :194 réussites,1 ignoré. Ils vérifient le montage dans les sources147, pas la présence du MCP dans un agent actif sur146.

Le candidat T3 est reconstitué et sa signature est valide. Version0.0.45-local.147 ; SHA256 app.asar a1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2. Il reste en staging. L'application active146 n'a pas été remplacée ou relancée.

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

À ce premier checkpoint, T3 installé restait0.0.45-local.146 et le candidat147 était prêt, sans activation. La livraison suivante est consignée ci-dessous. Aucun restart différé n'est programmé.

## Livraison finale sur disque, sans activation

Mandat suivant du2026-10-09 : livrer les builds pour la prochaine relance, puis nettoyer branches, worktrees et anciens builds. Aucun redémarrage demandé ou effectué.

Le paquet T3 déjà construit correspond au HEAD source f4354fb0d5bee925338304cf3101d0853244f64b, déjà poussé sur fork/local/v0.0.45. Sa signature stricte, sa version et ses métadonnées sont vérifiées avant et après installation. Le serveur embarqué contient bridget_thread, setMcpServers et mcpServerStatus. Aucune compilation identique supplémentaire n'est nécessaire.

L'application /Applications/T3 Code (Local).app porte maintenant0.0.45-local.147. SHA256 app.asar inchangé : a1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2. L'ancienne146 est déplacée vers /Users/moi/.cache/bridget-delivery147-final.X8rgoK/T3-Code-before147.app.backup ; son suffixe évite de présenter une seconde application .app. LaunchServices enregistre le chemin canonique, sans ouvrir l'application.

Le binaire Bridget corrigé reste installé au chemin canonique. Une copie exacte est conservée dans /Users/moi/.cache/bridget-delivery147-final.X8rgoK/bridget-ff168d1c. Le paquet ZIP147 et les reçus de compilation sont conservés. Les versions installées sur disque ne prouvent pas un chargement par les processus actifs.

T3 conserve les PID49975/50035 ; daemon et pont conservent93353/93355. Leur prochaine relance chargera les fichiers installés. T3 et les deux services Bridget ont des cycles de vie distincts : redémarrer T3 seul ne redémarre pas ces services. Aucun tour, MCP de modèle actif, base, mission, Agent Loop ou configuration fournisseur modifié par la livraison.

Nettoyage : suppression du cache Cargo /Volumes/SD1TO/bridget-build-147-rust, du snapshot de build /Users/moi/.cache/t3-spec147-package.LIkn22/source et des anciens paquets141/143 sous leurs chemins exacts. Contrôles d'absence de fichiers ouverts réalisés avant suppression. Environ8,4Go de données reproductibles retirées, dont7,5Go sur le disque externe. Les binaires installés, ZIP147 et sauvegardes de retour arrière restent disponibles.

La branche distante Bridget session-146-bridget-panel-lisible est supprimée après vérification de son inclusion dans main. Le worktree du correctif147 avait déjà été retiré avec sa branche propre. Aucun autre worktree n'est supprimé : Bridget142/145/146 et T3145 sont fusionnés mais contiennent encore des modifications non committées ; T3139 n'est pas inclus par ascendance dans local/v0.0.45. Les autres projets et branches ne sont pas des cibles.

## Livraison reconstruite après intégration complète — 2026-10-09

Ce checkpoint remplace l'état de livraison précédent. Le nouveau mandat demande explicitement de committer les reliquats, intégrer les branches, reconstruire les deux programmes, pousser, installer et nettoyer, sans redémarrage.

Bridget : anciens travaux142/145/146 committés puis intégrés parfa07c3d4,f6a7e743 et3aa2a3c3. La source propre1163038fc25f864962e50a2893125c63ee27c3b7 est poussée sur github/main et reconstruite avec un nouveau cache Cargo. Le test145 ajouté deux fois par la fusion est conservé une seule fois ; le code Rust final est identique à celui déjà validé sur main.

T3 : les travaux145 et les en-têtes139 sont intégrés par0e9b54245a eteace00356d. Le HEAD eace00356d91b38ff950c7a100251d677773b0ea est poussé sur fork/local/v0.0.45. Les conflits145 étaient des copies exactes du commit d'importd381302ec1, hors modes de fichiers. Les deux fusions T3 ne changent aucun contenu de code. Une nouvelle compilation web/serveur/desktop et un nouveau paquet natif sont néanmoins réalisés depuis ce HEAD, comme demandé. Les dépendances sont installées avec le fichier de verrouillage, dans un clone privé.

Skills : origine142 intégrée dans dotfiles/main end0d8b04d. Checkout utilisateur capitalisé puis fusion de l'historique142 en6b157ef3, poussée sur origin/104-bridge-role-auth. Cette dernière fusion ne change aucun fichier ; les symlinks Claude existants sont conservés. Les travaux Vigi/Horizon et les index préexistants hors périmètre ne sont pas inclus dans les nouveaux commits de ce complément.

Vérifications finales réellement exécutées :16 tests de publication/catalogue dans chaque checkout skills ; T3 MCP/Claude194PASS/1SKIP et panneau50PASS ; paquet natif2413 contrôles réussis, zéro échec. Rust release : fils19PASS, CLI de fils11PASS, MCP54PASS/1SKIP et intégration daemon privé11PASS. Total ciblé Rust95PASS, zéro échec. Format Rust et git diff --check passent.

La sélection CLI plus large donne85PASS/2FAIL. Les deux échecs de spawn signalent auxiliary_credential_required, déjà rencontré dans la suite globale. Ils restent visibles dans les reçus ; aucune suite globale verte n'est revendiquée et aucun wrapper/agent n'est relancé pour les contourner.

Installation vérifiée : /Applications/T3 Code (Local).app, version0.0.45-local.147, commit embarqué eace00356d91, SHA256 app.asar01b46973b376b936ea2320fb565ac43acfe171724bdcffb4c5de9445273ec9c5. Signature ad hoc stricte valide. Binaire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget, SHA2569f70172027c06ca231f64496541a5ba14d48031985edb1d1a43a8d5094d21c11, signature valide. Le CLI /Users/moi/.local/bin/bridget et les deux services LaunchAgent pointent sur ce chemin.

Les PID T349975/50035 et Bridget93353/93355 gardent leurs dates de démarrage. Aucun redémarrage, job d'activation différé ou migration de base. La prochaine relance de T3 charge l'application installée ; la prochaine relance des services Bridget charge leur binaire installé. Ces cycles restent distincts.

Nettoyage final : worktrees Bridget142/145/146, T3139/145 et dotfiles142/deux intégrations147 propres retirés avec leurs branches, après vérification des fusions et des fichiers ouverts. Les branches et worktrees d'autres travaux restent intacts. Snapshots de compilation, cinq stages natifs145/146/147, ancien paquet145, paquet147 remplacé et copie intermédiaire147 retirés. Les gros caches Cargo temporaires sont retirés après conservation des logs. Aucun fichier de conversation ou base T3 n'est supprimé.

Paquet final et reçus : /Users/moi/.cache/t3-final147.zrm6Zc. ZIP SHA2566ea9b12373283a6b7a5cfcf7226e1cb3621804f491e31300c932b2e992fe5035. Binaire final et logs Rust conservés dans ce dossier. Sauvegarde146 de retour arrière : /Users/moi/.cache/bridget-delivery147-final.X8rgoK/T3-Code-before147.app.backup. Ancien binaire corrigé : /Users/moi/.cache/bridget-147-lecture-backup.LO6P9D/bridget-before-ff168d1c. La taille supprimée est une taille de données reproductibles, pas une promesse de gain physique identique sur APFS.
