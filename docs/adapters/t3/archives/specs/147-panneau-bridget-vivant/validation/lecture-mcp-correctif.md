# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md](../../../../../../../specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Mandat du 2026-10-09 : corriger, committer, fusionner et pousser. Ne redémarrer ni T3 ni Bridget. Ce complément reste dans la session147.

<a id="extrait-2"></a>

## Extrait 2

L'agent Relance utilise Claude/GLM. L'application installée porte encore la version0.0.45-local.146. Le montage MCP Claude/GLM est déjà corrigé dans les sources147 de T3 ; aucun second montage ni changement global des configurations fournisseur n'est nécessaire.

<a id="extrait-3"></a>

## Extrait 3

L'activation147 précédente a échoué avant le remplacement de l'application, à la phase staging_path. Le reçu conserve aborted_before_replacement ; aucune base T3 n'a été restaurée ou copiée. Le job com.bridget.install147, arrêté en échec, a été retiré pour empêcher une activation ultérieure non demandée.

<a id="extrait-4"></a>

## Extrait 4

Les tests T3 ClaudeMcp.test.ts et ClaudeAdapter.test.ts passent :194 réussites,1 ignoré. Ils vérifient le montage dans les sources147, pas la présence du MCP dans un agent actif sur146.

<a id="extrait-5"></a>

## Extrait 5

Le candidat T3 est reconstitué et sa signature est valide. Version0.0.45-local.147 ; SHA256 app.asar a1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2. Il reste en staging. L'application active146 n'a pas été remplacée ou relancée.

<a id="extrait-6"></a>

## Extrait 6

À ce premier checkpoint, T3 installé restait0.0.45-local.146 et le candidat147 était prêt, sans activation. La livraison suivante est consignée ci-dessous. Aucun restart différé n'est programmé.

<a id="extrait-7"></a>

## Extrait 7

Le paquet T3 déjà construit correspond au HEAD source f4354fb0d5bee925338304cf3101d0853244f64b, déjà poussé sur fork/local/v0.0.45. Sa signature stricte, sa version et ses métadonnées sont vérifiées avant et après installation. Le serveur embarqué contient bridget_thread, setMcpServers et mcpServerStatus. Aucune compilation identique supplémentaire n'est nécessaire.

<a id="extrait-8"></a>

## Extrait 8

L'application /Applications/T3 Code (Local).app porte maintenant0.0.45-local.147. SHA256 app.asar inchangé : a1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2. L'ancienne146 est déplacée vers /Users/moi/.cache/bridget-delivery147-final.X8rgoK/T3-Code-before147.app.backup ; son suffixe évite de présenter une seconde application .app. LaunchServices enregistre le chemin canonique, sans ouvrir l'application.

<a id="extrait-9"></a>

## Extrait 9

T3 conserve les PID49975/50035 ; daemon et pont conservent93353/93355. Leur prochaine relance chargera les fichiers installés. T3 et les deux services Bridget ont des cycles de vie distincts : redémarrer T3 seul ne redémarre pas ces services. Aucun tour, MCP de modèle actif, base, mission, Agent Loop ou configuration fournisseur modifié par la livraison.

<a id="extrait-10"></a>

## Extrait 10

Nettoyage : suppression du cache Cargo /Volumes/SD1TO/bridget-build-147-rust, du snapshot de build /Users/moi/.cache/t3-spec147-package.LIkn22/source et des anciens paquets141/143 sous leurs chemins exacts. Contrôles d'absence de fichiers ouverts réalisés avant suppression. Environ8,4Go de données reproductibles retirées, dont7,5Go sur le disque externe. Les binaires installés, ZIP147 et sauvegardes de retour arrière restent disponibles.

<a id="extrait-11"></a>

## Extrait 11

T3 : les travaux145 et les en-têtes139 sont intégrés par0e9b54245a eteace00356d. Le HEAD eace00356d91b38ff950c7a100251d677773b0ea est poussé sur fork/local/v0.0.45. Les conflits145 étaient des copies exactes du commit d'importd381302ec1, hors modes de fichiers. Les deux fusions T3 ne changent aucun contenu de code. Une nouvelle compilation web/serveur/desktop et un nouveau paquet natif sont néanmoins réalisés depuis ce HEAD, comme demandé. Les dépendances sont installées avec le fichier de verrouillage, dans un clone privé.

<a id="extrait-12"></a>

## Extrait 12

Vérifications finales réellement exécutées :16 tests de publication/catalogue dans chaque checkout skills ; T3 MCP/Claude194PASS/1SKIP et panneau50PASS ; paquet natif2413 contrôles réussis, zéro échec. Rust release : fils19PASS, CLI de fils11PASS, MCP54PASS/1SKIP et intégration daemon privé11PASS. Total ciblé Rust95PASS, zéro échec. Format Rust et git diff --check passent.

<a id="extrait-13"></a>

## Extrait 13

Installation vérifiée : /Applications/T3 Code (Local).app, version0.0.45-local.147, commit embarqué eace00356d91, SHA256 app.asar01b46973b376b936ea2320fb565ac43acfe171724bdcffb4c5de9445273ec9c5. Signature ad hoc stricte valide. Binaire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget, SHA2569f70172027c06ca231f64496541a5ba14d48031985edb1d1a43a8d5094d21c11, signature valide. Le CLI /Users/moi/.local/bin/bridget et les deux services LaunchAgent pointent sur ce chemin.

<a id="extrait-14"></a>

## Extrait 14

Les PID T349975/50035 et Bridget93353/93355 gardent leurs dates de démarrage. Aucun redémarrage, job d'activation différé ou migration de base. La prochaine relance de T3 charge l'application installée ; la prochaine relance des services Bridget charge leur binaire installé. Ces cycles restent distincts.

<a id="extrait-15"></a>

## Extrait 15

Nettoyage final : worktrees Bridget142/145/146, T3139/145 et dotfiles142/deux intégrations147 propres retirés avec leurs branches, après vérification des fusions et des fichiers ouverts. Les branches et worktrees d'autres travaux restent intacts. Snapshots de compilation, cinq stages natifs145/146/147, ancien paquet145, paquet147 remplacé et copie intermédiaire147 retirés. Les gros caches Cargo temporaires sont retirés après conservation des logs. Aucun fichier de conversation ou base T3 n'est supprimé.

<a id="extrait-16"></a>

## Extrait 16

Paquet final et reçus : /Users/moi/.cache/t3-final147.zrm6Zc. ZIP SHA2566ea9b12373283a6b7a5cfcf7226e1cb3621804f491e31300c932b2e992fe5035. Binaire final et logs Rust conservés dans ce dossier. Sauvegarde146 de retour arrière : /Users/moi/.cache/bridget-delivery147-final.X8rgoK/T3-Code-before147.app.backup. Ancien binaire corrigé : /Users/moi/.cache/bridget-147-lecture-backup.LO6P9D/bridget-before-ff168d1c. La taille supprimée est une taille de données reproductibles, pas une promesse de gain physique identique sur APFS.
