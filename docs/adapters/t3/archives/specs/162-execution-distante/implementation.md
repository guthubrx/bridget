# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/implementation.md](../../../../../../specs/162-execution-distante/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Réemploi : requête versionnée, client négocié et méthode de validation du contrat existant. Aucun fichier produit, dépendance ou canal nouveau pour cette tâche. Le contrôle de version évite qu’un résultat ancien soit pris pour une projection enrichie. La réponse réelle du maître, le décodage T3 et les tests restent à vérifier. T033 reste ouverte.

<a id="extrait-2"></a>

## Extrait 2

ROOT a exécuté le groupe T3 Lineage :125 PASS sur3 fichiers, RC0,5,76s. Contrats et Reader de production relus conformes ; leur owner atteste les quatre fichiers inchangés depuis le run. Empreintes après exécution et commandes attestées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md. T035 et T036 cochées ; progression3/42 avec T001. Aucun runtimeT3 nouveau déduit.

<a id="extrait-3"></a>

## Extrait 3

Groupe T3 UI suivant :67 PASS sur3 fichiers, RC0,6,30s. Empreintes actuelles capturées ; relecture ROOT et raccordementRust T034 encore attendus. T037/T038/T039 restent ouvertes. Aucun GREEN global, aucun build/service/Git lancé par le rédacteur documentaire.

<a id="extrait-4"></a>

## Extrait 4

Les trois fichiers de production UI T3 et leurs trois tests ont été relus conformes par ROOT ; intégration 162 encore ouverte. Lint RC0 avec 8 warnings, dont 2 nouveaux corrigés ensuite ; types contrats RC0, serveur R1 RC1 sur nouvel oracle corrigé, web R1 RC1 hors diff en classification. Aucun test T3 final à jour n’est déduit après le correctif Reader. Preuves et limites : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md.

<a id="extrait-5"></a>

## Extrait 5

L’essai réel renamex_np RENAME_SWAP=2 a réussi sur deux dossiers synthétiques privés sous /Applications : permutation puis retour vérifiés sur le même volume, dossier de sonde supprimé, application intacte. La primitive peut servir à G8 après copie et signature ; aucune installation ni relance T3 n’est déduite.

<a id="extrait-6"></a>

## Extrait 6

La procédure /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/livraison162.md renseigne maintenant ExecStart avec préfixe162-SHA12 contrôlé, registre privé et socket à réattester. Elle conserve NON EXÉCUTÉ, projets en attente, routes désactivées et aucun redémarrage T3. Quatre points précis restent avant cochage T041 : commandes/chemins de quiescence et sauvegarde ; provisionnement et unité finalisés ; preuve de survie/reprise worker ; quickstart et relecture dépendante T032/T039. Aucun résultat de production inventé.

<a id="extrait-7"></a>

## Extrait 7

Les 122 échecs historiques de l’unique FULL ont un rejeu PASS : 112 dans R1 puis 10 dans R2, rapprochement vérifié par ROOT. R1 totalisait 173 PASS/10 FAIL ; le compteur 125 était son seul libdaemon. Aucun cumul de passages en tests uniques, aucune réécriture du RC101 de la FULL en GREEN global. Les limites web/T3 et les fixtures isolées restent explicites dans le registre de tests.

<a id="extrait-8"></a>

## Extrait 8

La lecture indépendante rapproche les 30 exigences FR et 12 critères SC. Mapping : FR001–002 admission/registre ; FR003/025 auth/époque/sonde ; FR004 autorité maître ; FR005/030 mapping/Git154 ; FR006–008 preuves/rechecks ; FR009–011/014 reçu/marqueur/RELEASE/Connected ; FR012–013 inconnus/bornes/collecte ; FR015 cancel ; FR016–017 reprise/slots/délais ; FR018–019/029 journal/blob/résultat/parent ; FR020–023 Lineage/contact ; FR024 configuration privée ; FR026–028 procédures G8/G9. SC001–002 admission/refus ; SC003 permissions ; SC004 frontières ; SC005 cancel ; SC006 reprise ; SC007 journal/résultat155 ; SC008–009 T3/wire ; SC010 bornes ; SC011–012 procédures de livraison/nettoyage. Les exigences opérationnelles restent attendues en G8/G9, pas réputées exécutées par 42/42.

<a id="extrait-9"></a>

## Extrait 9

La correction T043 est relue sur daemon.rs SHA-256 71c5b7065eb8e1133edbbc6a56a14165a0768fd7bd89a1ae6317e3b9c389abe4 : trace après drop(st), machine/worker et deux époques, sans credential. L’oracle existant auth/epoch a 1 PASS, RC0 en 48,95 s. Le contenu du logger est prouvé par lecture, pas par une assertion de capture ajoutée. Le mapping 30 FR/12 SC et la lecture effective T3 du passage précédent sont conservés sur code inchangé. Aucun test, build, Git ou service exécuté pendant ce passage.

<a id="extrait-10"></a>

## Extrait 10

La lecture fraîche porte sur le delta T044 : une sonde starting par cadence hors mutex puis contrôle complet de l’identité au handler ; Terminal/Refused avant DesiredStopped ; conservation du Child sur les erreurs EOF/StartupFailed ou de marqueur ; clôture des slots après preuve d’arrêt ; inventaire groupé en maintenance. Le mapping 30 FR/12 SC et les lectures T3 des passages précédents sont réutilisés explicitement sur les parties inchangées. Le raccord T044 couvre FR-009/012/014/015, avec FR-013 pour les bornes, et SC-004/005/007.

<a id="extrait-11"></a>

## Extrait 11

La couverture de lecture est de **7 898 lignes uniques sur 186 764 lignes du périmètre, soit 4,228866 %**. Il s’agit d’un échantillon des sources Bridget et T3 : les lectures antérieures ne sont reprises que sur les lignes inchangées, avec lecture fraîche des deltas D1. Cette note ne certifie pas tout le produit, ne remplace pas les résultats de tests et ne constitue pas un GREEN global.

<a id="extrait-12"></a>

## Extrait 12

L’activation est attestée par /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3/progress.json : phase activate réussie ; daemon Mac 71469 né à 17:53:53 CEST, pont 71567 né à 17:53:55 ; worker Linux 471321, naissance 142199350, exécutable du préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d. Les SHA des binaires actifs sont d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44 sur Mac et 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352 sur Linux. DB quick_check ok, zéro erreur de clé étrangère, schémas 7/12 et Fleet 5. T3 704/969 et le tunnel 1384 restent inchangés par rapport au préflight après le redémarrage externe.

<a id="extrait-13"></a>

## Extrait 13

Le worktree Bridget 162 et sa branche ont été retirés sans force. Le worktree et la branche T3 du lot avaient déjà été retirés. Les 45 fichiers ignorés inventoriés possèdent chacun une copie durable byte-identique. Preuve de retrait : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-worktree-result.json. Les chemins de worktrees dans la chronologie sont désormais historiques ; aucun worktree n’a été recréé.

<a id="extrait-14"></a>

## Extrait 14

Les intermédiaires du stage Mac ont été retirés par la commande existante build.py installed : 0,31 Gio logiques, binaire source conservé. Le retrait antérieur du stage T3 représente 840 737 986 octets logiques et son archive de 59 preuves reste vérifiée. Aucun gain physique de volume n’est déduit de ces tailles.

<a id="extrait-15"></a>

## Extrait 15

- Cache debug Mac de 44,96 Gio, utile pour 163, budget temporaire de rétention 46 Gio ; cache Cargo Linux utile également conservé.
- Stage Git distant d’environ 3,45 Go et préfixes de validation d’environ 276 Mo : garde d’usage lsof incomplète, y compris avec root, à cause du montage /run/user/1002/doc. Aucun retrait forcé ni masquage de cette limite.
- Git 2.56 installé, profil /etc/apparmor.d/bwrap-userns-restrict, sauvegardes de rollback, bases, preuves et cache source T3 contenant cache.db : nécessaires au fonctionnement, au retour arrière ou à 163.

<a id="extrait-16"></a>

## Extrait 16

L’inventaire et la qualification sont conservés dans /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-inventory.md et /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-priority-inventory.json. L’inventaire initial était en lecture seule ; les retraits réellement effectués sont distingués ici des conservations. Les 17 worktrees T3 étrangers restent hors périmètre.
