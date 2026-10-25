# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/validation/livraison162.md](../../../../../../../specs/162-execution-distante/validation/livraison162.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le worktree Bridget 162 et sa branche ont été retirés sans force. Le worktree et la branche T3 du lot avaient déjà été retirés. Les 45 fichiers ignorés inventoriés possèdent chacun une copie durable byte-identique. Preuve de retrait : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-worktree-result.json. Les chemins de worktrees dans la chronologie sont désormais historiques ; aucun worktree n’a été recréé.

<a id="extrait-2"></a>

## Extrait 2

Les intermédiaires du stage Mac ont été retirés par la commande existante build.py installed : 0,31 Gio logiques, binaire source conservé. Le retrait antérieur du stage T3 représente 840 737 986 octets logiques et son archive de 59 preuves reste vérifiée. Aucun gain physique de volume n’est déduit de ces tailles.

<a id="extrait-3"></a>

## Extrait 3

- Cache debug Mac de 44,96 Gio, utile pour 163, budget temporaire de rétention 46 Gio ; cache Cargo Linux utile également conservé.
- Stage Git distant d’environ 3,45 Go et préfixes de validation d’environ 276 Mo : garde d’usage lsof incomplète, y compris avec root, à cause du montage /run/user/1002/doc. Aucun retrait forcé ni masquage de cette limite.
- Git 2.56 installé, profil /etc/apparmor.d/bwrap-userns-restrict, sauvegardes de rollback, bases, preuves et cache source T3 contenant cache.db : nécessaires au fonctionnement, au retour arrière ou à 163.

<a id="extrait-4"></a>

## Extrait 4

L’inventaire et la qualification sont conservés dans /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-inventory.md et /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-priority-inventory.json. L’inventaire initial était en lecture seule ; les retraits réellement effectués sont distingués ici des conservations. Les 17 worktrees T3 étrangers restent hors périmètre.

<a id="extrait-5"></a>

## Extrait 5

L’activation est attestée par /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3/progress.json : phase activate réussie ; daemon Mac 71469 né à 17:53:53 CEST, pont 71567 né à 17:53:55 ; worker Linux 471321, naissance 142199350, exécutable du préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d. Les SHA des binaires actifs sont d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44 sur Mac et 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352 sur Linux. DB quick_check ok, zéro erreur de clé étrangère, schémas 7/12 et Fleet 5. T3 704/969 et le tunnel 1384 restent inchangés par rapport au préflight après le redémarrage externe.

<a id="extrait-6"></a>

## Extrait 6

| Surface | Dernière preuve transmise par ROOT |
|---|---|
| Build Mac | RC0, 169,694 s ; binaire signé /Users/moi/.cache/bridget-build-162-cancel.8rq76kny/release/bridget ; SHA-256 d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44. |
| Build Linux | RC0, 70,866 s ; préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d ; SHA-256 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352. |
| Source précédemment active | 611bfa7557b9. Deux préflights avaient expiré pendant un ancien sweep tenant le mutex et un appel pread, dans un contexte de swap. L’arrêt du pont a rétabli la CLI ; admissions fermées à 17:13, génération 5, raison livraison162-correctif-annulation. |
| Redémarrage externe du Mac | Observé vers 17:29–17:31, avec uptime de 18 minutes à 17:47. T3 PID 704/969, nés à 17:31:27/33 ; daemon 1362 et pont 1374 nés à 17:31:50 ; tunnel 1384. Ces identités remplacent les anciennes observations. La procédure du correctif ne relance pas T3 et ne modifie pas le tunnel. |
| Gel et sauvegarde R3 | Préflight réussi ; worker, daemon et pont réellement arrêtés. Sauvegarde Linux réalisée, SHA-256 4f206849c7af476ac7257319365992de5d3c3c7d26d6e72953e126744e8a9876. Dossier de reprise : /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3. La sauvegarde jointe Mac n’est pas déclarée achevée. |
| Garde SQLite | La garde Mac a arrêté la procédure : le lecteur SQLite du contrôle de quiescence conservait sa propre connexion ouverte. Diagnostic reproduit : lsof trouve le PID lecteur avant collecte mémoire, puis aucun après. Correction de fermeture explicite dans le script privé et reprise en préparation ; aucune nouvelle modification produit nécessaire. |

<a id="extrait-7"></a>

## Extrait 7

Les sources sont intégrées et poussées. Bridget 162 est actif sur le Mac et le worker Cartae est démarré. T3 162 est installé sur disque ; ses processus conservent leur code chargé précédent. La recette distante Bridget, la survie du fournisseur à l’arrêt du worker et la récupération du résultat par le parent revenu sont attestées. Les routes Roadie et Opus2D ont également réussi. La livraison complète reste ouverte : l’annulation a laissé des descendants Linux vivants malgré cancelled/cleanup_done. Le correctif T045 est validé sur Linux isolé. Son installation et la recette réelle T046 restent nécessaires ; le nettoyage final reste ouvert.

<a id="extrait-8"></a>

## Extrait 8

| Surface | État attesté |
|---|---|
| Git Bridget | 611bfa7557b97587614687f6c6379d5961101fa6 intégré sur main et poussé vers github ; référence distante identique, 61 fichiers vérifiés. |
| Git T3 | 174a9d04554758aacffb4aade588e61c810faf5a intégré sur local/main-20261009 et poussé vers fork ; référence distante identique, 10 fichiers vérifiés. |
| Bridget Mac | Binaire installé et signé, Build ID 611bfa7557b9. Daemon 90803 et pont 90815 nés le 13 octobre à 15:11:21 CEST. SHA binaire installé : 1b2de5e0a2fe7aecc4637585928fbc611cd9fd957b7d93540713e00cf06de531. |
| Sauvegarde et données | Sauvegarde jointe vérifiée dans /Users/moi/.cache/bridget-install162-20261013-1515. Schémas DB idempotence 7 et exécutions 12 ; quick_check ok, zéro erreur de clé étrangère. Fleet sur disque reste en schéma 4, byte-identique à la sauvegarde ; la prochaine persistance écrit le schéma 5. |
| Worker Cartae | Unité bridget-worker-cartae.service active, PID actuel 340014 après arrêt/reprise de recette, release 162-611bfa7557b9. Lien client basculé vers 162. Configurations des trois projets Bridget, Roadie et Opus2D enabled=true ; aucun projet Cartae activé. |
| T3 sur disque | /Applications/T3 Code (Local).app installé en 0.0.45-local.162 par échange atomique ; signature stricte/deep RC0 et entitlements inchangés. Sauvegarde /Users/moi/.cache/bridget-install162-20261013-1515/T3-before.app. |
| T3 chargé | PID 60116/60228 et naissances du 11 octobre à 01:13:35/36 inchangés. Aucun redémarrage ni activation programmée. L’installation sur disque ne prouve pas l’affichage 162 dans le processus actuel. |
| Recette G8 | Bridget : mission 26465d28-fc14-40b5-8ae4-07ca375ab948 en result_available, résultat Linux/cartae/bon cwd/HEAD611bfa ; reprise worker et récupération après retour du parent attestées. Roadie et Opus2D réussies également. Annulation en échec réel : descendants Linux survivants malgré Terminal/cleanup_done ; T045 validée ; T046 ouverte. Les premiers refus Git et sandbox restent historiques. |
| Nettoyage G9 | Worktree et branche T3 162 retirés après vérification ; stage /private/tmp/b162.24BX8rMN retiré, 840 737 986 octets logiques. Archive de 59 preuves vérifiée. Cache source T3 avec DB et cache debug conservés pour163. Le gate global reste ouvert. |

<a id="extrait-9"></a>

## Extrait 9

- /Users/moi/.cache/bridget-delivery162-20261013/delivery162-git-sources.json
- /Users/moi/.cache/bridget-delivery162-20261013/delivery162-runtime-checkpoint.json
- /Users/moi/.cache/bridget-install162-20261013-1515/activation-progress.json
- /Users/moi/.cache/bridget-install162-20261013-1515/T3-install-receipt.json

<a id="extrait-10"></a>

## Extrait 10

Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant et la branche session-162-lineage-distant ont été retirés après contrôle de propreté, intégration/push et absence d’utilisation. Les worktrees étrangers restent hors périmètre. Preuve : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-worktree.json.

<a id="extrait-11"></a>

## Extrait 11

L’archive /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs.tgz contient **59 preuves vérifiées** ; SHA-256 **0e5e39b2614a0b286c979929ab5e4fdcbd2a69e49aaf1ab9dfeed87c7d2fc88c**. Le stage /private/tmp/b162.24BX8rMN a été retiré après contrôle de signature installée et de fichiers ouverts. Son inventaire compte **840 737 986 octets logiques**, sans revendication de gain physique APFS. Preuves : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs-verified.json ; /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs-manifest.json ; /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-stage-result.json.

<a id="extrait-12"></a>

## Extrait 12

La source de cache /Users/moi/.cache/t3-final162.2nosz6zd/source et sa DB /Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db sont conservées pour163. Le retrait du stage n’a modifié ni cette DB, ni le cache debug, ni l’ancien bundle, et n’a relancé aucun processus. Le nettoyage release Mac rapporte séparément 0,31 Gio logiques retirés et 44,96 Gio de caches restants protégés, sans suppression forcée : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-mac-stage.log. Les sauvegardes et les ressources encore nécessaires à la recette restent conservées.

<a id="extrait-13"></a>

## Extrait 13

- Bridget conserve un seul maître. Le worker Linux n’est pas une seconde base métier. Le pipeline autorise l’activation contrôlée de Bridget 162 après ses gates ; il n’autorise aucune relance T3.
- L’utilisateur a autorisé les projets Bridget, Roadie et Opus2D. Cartae est exclu de cette activation. Les trois clones Linux et leurs identités Git sont attestés dans provision-projects162.final.json ; les mappings privés sont préparés et désactivés. Codex login status retourne RC0, sans preuve de quota ni appel réel de modèle. Les routes restent désactivées jusqu’aux contrôles G8. Préserver /home/moi/20.cartae et sa modification existante.
- Le CLI worker T006 est écrit, compilé et exécuté dans les quatre E2E R3 réussis. ROOT confirme également les quatre E2E PASS dans la FULL unique ; cela reste distinct de son verdict global RC101 et d’une activation de production. **ExecStart doit viser le préfixe Linux162 réellement livré, puis être vérifié avant installation de l’unité.** Les identités candidates sont cartae/worker-cartae/révision 1 ; la socket et les configurations installées restent à réattester en G8.
- Les commandes de fermeture des admissions et de lecture des missions sont précisées en section2. Les chemins DB/Fleet/registre du maître sont observés ci-dessous et doivent être réattestés au préflight final. Les PID historiques ne sont jamais des cibles d’arrêt. L’absence de ces éléments bloque l’activation, pas le développement.
- Fleet écrit le schéma 5 ; les requêtes stockées sont strictes. Un ancien binaire ne doit pas lire l’état 162 inchangé. La compatibilité de lecture Lineage v1 ne signifie pas compatibilité des stores avec un ancien binaire.

<a id="extrait-14"></a>

## Extrait 14

1. Enregistrer les commits de sources Bridget et T3, les branches et le statut des deux worktrees. Ne pas mélanger SHA du binaire, SHA du paquet et commit documentaire final.
2. Intégrer Bridget vers github/main et T3 vers fork/local/main-20261009 avec les hooks normaux et l’identité utilisateur. Vérifier les références distantes. Aucun push upstream ni PR T3.
3. Construire depuis les sources intégrées propres. Les worktrees de développement ne deviennent pas une preuve de provenance du binaire installé sans correspondance exacte.
4. Relever les PID et naissances actuels du maître, du pont, du tunnel et de T3. Relever leurs binaires et configurations effectives sans secrets. Photographier l’état via des preuves textuelles privées ; aucune capture d’écran requise.
5. Enregistrer le compteur et les identités de missions actives, préparées, en quota, en attente humaine, avec résultat en attente, inconnues et à reprendre. Une commande status réussie ne suffit pas à prouver la quiescence métier.

<a id="extrait-15"></a>

## Extrait 15

| Élément | Chemin actif absolu vérifié | Copie privée absolue et SHA | Contrôle |
|---|---|---|---|
| Base(s) métier et sidecars applicables | /Users/moi/.cache/bridget-core/bridget.db observée ; annexes/sidecars à attester | Non créée | Non exécuté |
| Fleet et schéma | /Users/moi/.cache/bridget-core/bridget.fleet.json, 14 583 octets, mode 0600 observé | Non créée | Non exécuté |
| Registre maître privé | /Users/moi/.cache/bridget-core/agents.json, mode 0600 observé | Non créée | Non exécuté |
| État worker | /home/moi/.local/share/bridget-worker/cartae ou absence attestée | Non créée | Non exécuté |
| Credential worker, si déjà présent | /home/moi/.config/bridget/worker-cartae.credential | Copie privée seulement, aucune valeur exposée | Non exécuté |
| Binaire maître et configurations de services | À relever dans les processus réels | Non créée | Non exécuté |
| Application T3 actuelle | /Applications/T3 Code (Local).app | Non créée | Non exécuté |

<a id="extrait-16"></a>

## Extrait 16

Une seule compilation active, y compris le composant Rust du paquet T3. Cargo jobs 2, nice 10 et deux threads pour les validations. Le cache debug utile 162/163 est conservé sous budget temporaire de rétention de 45 Gio ; ce budget ne bloque pas mécaniquement la compilation. Ne pas purger tout le profil pour revenir arbitrairement à 20 Gio.

<a id="extrait-17"></a>

## Extrait 17

Ne pas exécuter avec une variable vide. Relever commit source, sortie Cargo, durée, RC, BuildID réellement embarqué et SHA. Réemployer le remplacement atomique gardé de 160 seulement après sauvegarde et contrôle d’identité du fichier installé. Relire le SHA et la signature de /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget après échange. L’activation maître/pont se fait ensuite, avec les identités de processus fraîches et les contrôles de reprise. Aucun arrêt T3 ni tunnel déduit de cette installation.

<a id="extrait-18"></a>

## Extrait 18

Sources intégrées sur fork/local/main-20261009, paquet depuis une copie privée du commit. Réemployer /Users/moi/11.Repositories/t3code-local/scripts/build-desktop-artifact.ts et /Users/moi/.cache/bridget-delivery149-tools/sign-stage.py. Le script 149 de build complet contient une version et des opérations anciennes : ne pas le relancer tel quel.

<a id="extrait-19"></a>

## Extrait 19

Sauvegarder /Applications/T3 Code (Local).app dans une destination privée neuve et contrôler la copie. Déposer le candidat sous /Applications/.T3 Code (Local).app.new-162 seulement si ce chemin n’existe pas déjà ; contrôler signature et contenu. Utiliser renamex_np avec RENAME_SWAP=2 après contrôle du candidat et du rollback. La sonde /Users/moi/.cache/bridget-delivery162-20261013/t3-atomic-swap-probe.json prouve la permutation puis le retour sur deux répertoires synthétiques du même volume sous /Applications ; aucun bundle réel n’a été touché. Ne pas prétendre que le script149 contenant cette primitive a été retrouvé. Comparer les identités des deux chemins juste avant l’échange et refuser un changement constaté. Ne pas inventer une commande mv non contrôlée. Relever SHA app.asar/Info.plist, version et signature après installation.

<a id="extrait-20"></a>

## Extrait 20

Comparer PID et naissances T3 avant/après : ils doivent rester inchangés. Aucun `open`, kill, restart, launchctl, activation différée ou écriture de base T3. Le nouveau rendu n’est attesté que par tests isolés tant que l’utilisateur n’a pas autorisé une relance. L’ancien reader doit continuer à recevoir v1.

<a id="extrait-21"></a>

## Extrait 21

| Ressource | Décision préparée | Condition avant action |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante | Retrait sans force après intégration | Propre, référence poussée, aucun processus utilisateur, preuves copiées dans le checkout durable. |
| /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant | Retrait sans force après intégration | Même contrôle, sans arrêter T3. |
| Branches session-162-execution-distante et session-162-lineage-distant | Retrait lorsqu’inutiles | Fusion et push attestés, aucun travail non récupéré. |
| Les 17 worktrees T3 préexistants listés dans le préflight | Conservation hors périmètre | Aucun mandat de nettoyage de ces travaux. |
| TMP et stages réellement créés pour 162 | Inventaire puis retrait ciblé ou conservation justifiée | Chemins exacts, propriétaire, absence d’usage, archives SHA vérifiées ; aucune suppression globale de TMP. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug | Conservation utile 162/163 | Budget temporaire45Gio ; nettoyage par build.py seulement après dernière réutilisation et gardes satisfaites. |
| /home/moi/.cargo/registry | Conservation utile | Cache partagé ; aucune purge générale. |
| Préfixe161, rollback096 et sauvegardes160/161 utiles | Conservation | Ne pas détruire un retour ou une preuve encore nécessaires. |
| État et blobs worker 162 | Collecte contractuelle seulement | Disposition finale durable, producteur terminal, commande/slot irrévocablement clos ; reçu compact/digest/disposition 7 jours conservés. |
| /Users/moi/.cache/bridget-delivery162-20261013 | Archivage durable des preuves utiles | Relire SHA avant retrait de copies temporaires ; pas de faux gain physique APFS. |

<a id="extrait-22"></a>

## Extrait 22

| Preuve | État à cette rédaction |
|---|---|
| Source Bridget intégrée/poussée et référence distante | NON EXÉCUTÉ |
| Source T3 intégrée/poussée et référence distante | NON EXÉCUTÉ |
| Sauvegarde jointe cohérente et contrôles d’intégrité | NON EXÉCUTÉ |
| Stage Bridget signé, binaire installé et BuildID | NON EXÉCUTÉ |
| Build Linux, manifest et SHA candidat | NON EXÉCUTÉ |
| CLI worker exact et unité utilisateur validée | CLI compilé/exécuté en fixture ; unité de production NON EXÉCUTÉE |
| Activation Bridget/worker et réconciliation | NON EXÉCUTÉ |
| Projet autorisé et mission distante réelle | Bridget, Roadie et Opus2D autorisés ; mappings à attester, mission réelle NON EXÉCUTÉE ; Cartae exclu |
| Paquet T3 signé/installé, runtime T3 inchangé | NON EXÉCUTÉ |
| Nettoyage de chaque ressource ou conservation motivée | NON EXÉCUTÉ |
| Commit/push des métadonnées finales | NON EXÉCUTÉ |

<a id="extrait-23"></a>

## Extrait 23

L’inventaire TMP/worktrees/branches, le rollback joint sans perte, les conservations utiles et l’interdiction de relancer T3 restent décrits ci-dessus. Le gel de sources et le formatage ne prouvent pas la livraison.

<a id="extrait-24"></a>

## Extrait 24

1. Créer une racine privée neuve sous /Users/moi/.cache avec le préfixe bridget-install162-, enregistrer son chemin exact et son mode 0700. Pour l’état Linux, créer une racine de sauvegarde privée neuve sous /home/moi/.cache. Ce document ne fournit pas le nom d’une sauvegarde déjà créée.
2. Capturer l’état des admissions, les fermer sans annuler les missions, puis établir l’inventaire complet et la quiescence. Identifier les PID/naissances et tous les writers à partir des processus réels. Arrêter proprement les writers Bridget autorisés une fois la reprise sécurisée. Vérifier leur arrêt avant de copier ; ne pas toucher à T3 ni au tunnel. Si un writer ou producteur de blob reste incertain, différer la sauvegarde cohérente.
3. Après fermeture attestée des writers, utiliser la sauvegarde SQLite en lecture de /Users/moi/.cache/bridget-core/bridget.db vers une destination neuve de la racine privée. Cette méthode intègre les données visibles dans le WAL ; ne pas copier seulement bridget.db pendant qu’un WAL actif contient encore des transactions. Conserver dans le reçu les sidecars observés et la méthode choisie. Copier Fleet et registre durant la même période sans écriture. Les éventuelles bases annexes effectivement ouvertes doivent être inventoriées et traitées de même, sans en inventer une liste.
4. Sauvegarder ensemble l’état worker /home/moi/.local/share/bridget-worker/cartae, le registre worker et le credential s’ils existent, après fin attestée de tous leurs écrivains. Un service worker arrêté ne prouve pas la fin des wrappers. Si l’état worker n’existe pas, conserver la preuve d’absence ; ne pas créer un store vide pour le sauvegarder. Les secrets restent uniquement dans la sauvegarde privée.
5. Vérifier sur les copies SQLite quick_check=ok et foreign_key_check sans ligne, puis vérifier les SHA, modes et propriétaires des copies Fleet/registre/worker et des binaires de retour. Conserver un manifeste joint horodaté et la preuve de la période sans writer. Réattester les sources avant remplacement. Un écart arrête la bascule ; aucune sauvegarde fragmentaire n’est présentée comme cohérente.
6. Le retour à une version antérieure restaure cet ensemble uniquement si aucun état post-162 n’est perdu. Sinon conserver 162 et désactiver les nouvelles routes, en préservant toutes les preuves et mutations postérieures. La réouverture des admissions reste conditionnée à l’état initial et à l’absence d’arrêt humain intervenu depuis.

<a id="extrait-25"></a>

## Extrait 25

ROOT a rapproché la lecture finale et les replays : 83 PASS dans le dernier groupe, quatre défauts corrigés, 30 FR/12 SC mappés. T017/T019/T020/T040/T042 sont validées. Les 122 échecs de la FULL historique ont un rejeu PASS ; la FULL garde son RC101, sans succès global inventé. Les limites T3 (tests isolés, types web étrangers, aucun redémarrage) restent celles du registre de tests.

<a id="extrait-26"></a>

## Extrait 26

La couverture de lecture est de **7 898 lignes uniques sur 186 764 lignes du périmètre, soit 4,228866 %**. Il s’agit d’un échantillon des sources Bridget et T3 : les lectures antérieures ne sont reprises que sur les lignes inchangées, avec lecture fraîche des deltas D1. Cette note ne certifie pas tout le produit, ne remplace pas les résultats de tests et ne constitue pas un GREEN global.
