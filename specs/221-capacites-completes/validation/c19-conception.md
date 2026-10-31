# C19 — proposition indépendante à éprouver
30 octobre 2026. ClaudeSonnetSYS Task70c5a3fd-2b81-403d-9481-ccf052d438e6, lecture seule ; résultat intégral lu/ACK et clôture demandée. Proposition brute privée : /Users/moi/.cache/bridget221/c19-design-proposal.md. Aucune implémentation ou qualification d'accès fournisseur déduite.

Constats retenus : Shared peut demander un rafraîchissement de HEAD et refuse un descendant actif ; créer un worktreeGit n'est pas le résultat demandé. Les parties de sélection du profil compact peuvent être réutilisées, pas son audit d'index comme attestation absolue d'un dépôt enfant modifiable.
Option proposée, non adoptée : dépôt indépendant avec objets partagés et HEAD détaché. Elle exige une ancre contre le GC et modifie la racine par cette ref ; ce n'est donc pas un simple export sans métadonnées. L'accès aux objets par SHA dépasse la seule arborescence A ; ses droits doivent être réellement testés, pas supposés. Un .git contrôlé par l'enfant ne doit jamais servir seul de preuve d'intégrité.
Avant contrat : prototype privé, contrôle des liens, bornes exactes, GC, accès Git nécessaires et droits réels. Nouvelle stratégie et capacité imposent de vérifier les reçus à deny_unknown_fields et le retour vers un ancien lecteur même pour des tâches terminées conservées ; absence de tâche active ne suffit pas.
T174 puis T030/T031/T032 restent ouvertes. Aucun fallbackWorktree ni service/proxy ajouté ; interfaces complètes et programme global toujours à réaliser.

## Prototype privé après la reprise du programme

Source de travail inchangée : 37a11ed39ca701af7a5c4afc8e4537458cafd0b4.
Script : /home/moi/.cache/bridget221/c19-prototype/prototype.py.
Résultat : /home/moi/.cache/bridget221/c19-prototype/prototype-result.json.
Le script et le résultat restent hors Git. Un programme de prototype a exécuté
29 assertions sur des dépôts temporaires possédés. Ce ne sont pas 29 tests
de production et ce n'est pas une qualification du fournisseur.

Résultat observé : export de 200895 octets, 200000 octets explicitement omis.
Deux exports de la même base ont le même manifeste et les mêmes octets.
La source reste sur B, avec une modification non commitée, un fichier non suivi
et un worktree Git descendant déjà inscrit. HEAD, index, références, types,
modes et octets de la source sont comparés avant/après. Aucun processus ou
garde de mission réel n'est simulé par cette seule inscription.

Les lectures utilisent ls-tree puis un seul cat-file --batch. Les entrées
binaires gardent leurs octets ; export-ignore et export-subst ne changent
aucun fichier. La sélection reproduit les règles de compact207 dans ce
prototype seulement ; elle ne réutilise pas encore son code Rust.
Les fichiers de code volumineux restent sélectionnés, le gros binaire est
omis et nommé. Les liens internes et internes pendants restent des liens.
Les liens sortants, chaînes sortantes, cycles, collisions de casse exactes,
chemins traversants, .git et noms non UTF-8 sont refusés.

Source absente, objet absent, objet non commit, accès réellement refusé
par les droits du répertoire d'objets, budgets d'entrées/octet et destination
existante sont éprouvés. Leur distinction diagnostique n'est pas encore
implémentée : le prototype regroupe certaines erreurs Git sous un refus
générique. Aucun fallback worktree ni publication de vue sur lien sortant.
Une modification de fichier est détectée par l'inventaire privé. Les fichiers
sélectionnés restent lisibles sans accès à la source. Le Git CLI ne fonctionne
pas depuis la vue : cette limite est testée et doit être traitée dans le
contrat de lecture, pas cachée par un dépôt Git fabriqué.

Limites bloquant la clôture C19 : politique fournisseur réellement readOnly,
corrélation et reprise durable après coupure, contrôle du lecteur minimal
avant retour ancien, lecture native des objets omis, budgets à
l'échelle du projet, portabilité macOS et concurrence hostile du système
de fichiers. chmod0444 ne prouve pas une interdiction d'écriture sous le même UID.
Le manifeste doit rester hors de la vue de l'enfant. Les 29 assertions ne
font cocher ni T174 ni T030/T031/T032. T177 reste également ouverte.

Contre-revue de contrat demandée par Bridget, clé
bridget221-c19-contract-review-20261030-01, tâche
1078ab03-86c7-4905-b51b-bcda44cb31e8, enfant
d2318a93-0ea9-409b-8d63-4287016a6a31, Claude Sonnet.
L'admission a rendu starting sans erreur ; le résultat a ensuite été lu comme indiqué ci-dessous.


## Traitement de la contre-revue de contrat

Résultat intégral de 5927 octets lu, ACK effectué et clôture native demandée.
Verdict APPROVE_WITH_CHANGES. Méthode limitée : spec et conception lues,
symboles localisés ; ni research D1 ni types protocole/record relus, aucun
test lancé. Ce verdict ne qualifie ni un diff d'implémentation ni une activation.
Le compte rendu intégral reste privé :
/home/moi/.cache/bridget221/c19-contract-review.md.

| Observation | Vérification et décision |
| --- | --- |
| Carte d'omissions à rendre visible au modèle | Retenue. Résumé borné dans le contexte et carte lisible ; copie publique distincte de l'ancre privée, collision de nom refusée. |
| Ancienne racine conservée comme autorité | Risque conditionnel retenu : remplacement explicite du mapping, pas ajout de racine. La preuve négative d'accès à tout autre chemin dépasse les politiques readOnly existantes ; aucune isolation générale de lecture n'est promise. Lecture native_permissions.rs : la projection traduit writableRoots et additional_directories, pas une liste globale des lectures système. |
| recordNone sans issue de reprise | Retenue : chemin déterministe, adoption seulement avec preuve privée complète et audit ; sinon conservation et refus guidé, jamais suppression/recréation. |
| Concurrence Shared à préciser | Retenue : pas de shared_refresh ni garde Shared pour la vue ; lecture par SHA, aucune garde étrangère retirée. Le test avec vraie commande active reste requis. |
| Rendre les omissions définitivement inaccessibles | Non retenue : cela laisserait un manque de lecture. Prévoir une lecture native du seul blob du commit, sans écrasement. L'absence future d'objet reste explicitement refusée ; aucune référence GC cachée. |
| Ancien lecteur et retour | Retenue comme gate avant activation : formats conservés, même terminaux, vérifiés avant Prepare. L'absence de mission active n'est pas une preuve de compatibilité. |

Le contrat correspondant est dans le fichier commun de cette spécification,
section C19. Les points retenus sont des exigences avant intégration, pas des
défauts de production déclarés corrigés. Les cinq assertions ajoutées après
le premier appel de revue (29 au lieu de 24) n'ont pas été relues par cet agent.


## Noyau Rust en cours — aucun nouveau parcours admis

Base de travail : f80fe1a748091785f1a3f453f39b8f75258e21ed.
Implémentation dans workspace_commit_view.rs, intégré comme module interne de
workspace_git, sans appelant de production et sans variante de Task nouvelle.
La sélection pure de compact207 est réutilisée. Construction de commande Git
factorisée : les appels historiques gardent leur environnement ; la lecture
de vue seule ajoute GIT_NO_REPLACE_OBJECTS=1. Aucun dépôt enfant ni alternates.

Tests sur Cartae : 5 noms C19 PASS, puis suite workspace_git entière :
33 PASS, 2 ignorés. Ces nombres se recoupent ; ne pas les additionner.
Les deux premières compilations ont échoué sur des erreurs de types/emprunts,
corrigées avant les tests. Aucun RED comportemental n'est revendiqué.
Journaux privés : /home/moi/.cache/bridget221/c19-kernel-tests.log et
/home/moi/.cache/bridget221/c19-git-regressions.log.
Aucun Cargo direct ; le helper a contrôlé les caches avant/après.
La suite dure 1,20 s hors compilation. Ce n'est pas une mesure à l'échelle
d'Opus2D ni une qualification fournisseur.

Éprouvé : export exact A depuis un Shared B modifié et un linked checkout
indépendant ; conservation HEAD/index/références/statut/worktree list ; octets
binaires, sous-dossier source, absence d'effet export-ignore/export-subst,
remplacement d'objet ignoré ; audit sans source ; ajout/modification détectés ;
lien sortant/cycle/chemin interdit/base absente/objet non commit refusés ;
destination déjà présente jamais écrasée ; ancrage par descripteur sous
substitution d'un dossier par un lien, sans écrire dans sa cible.
Bornes actuelles à qualifier : 65 536 entrées, arbre 8 Mio, sélection 512 Mio,
liens 4096 octets, profondeur64, processus Git et audit60s.
L'audit borne aussi les octets des omissions devenues présentes.

Gate toujours ouvert : aucun reçu privé durable, aucune admission de vue,
aucune reprise de vue publiée, aucun nettoyage natif ni matérialisation liée
à Task/instance ; comportement fournisseur et macOS non qualifiés. Les liens
internes pendants restent conservés mais leur détail dans la carte est à finir.
Un Git dont l'arrêt n'est pas confirmé doit rester détenu dans la préparation
native ; le seul plan de données n'établit pas cette rétention durable.
Les tâches T174/T030/T031/T032 et le gate T033 ne sont pas cochés.
Aucun service, mission ni ancien reçu modifié. Aucune livraison C19 annoncée.

Contre-revue de ce seul noyau demandée : clé
bridget221-c19-kernel-review-20261030-01 ; patch privé de 47444 octets,
SHA25644c95f6d4352d02fbdc9832dff24a038a3eceeefec476c08d227b26091a5fcfe.
Le patch inclut tous les nouveaux fichiers ; le checkout du relecteur reste
sur la base. Aucun verdict ou activation n'est déduit de l'admission.

## Retour de version : revue distincte

Revue32b693b1-8ac3-4410-b649-de1478af1847, 8637 octets lus/ACK et clôture
native demandée ; APPROVE_WITH_CHANGES, sources lues sans test/build.
Rapport privé : /home/moi/.cache/bridget221/c19-min-reader-review.md.

Constats retenus : Report209 refuse les champs inconnus ; ne pas y ajouter
les formats. Le chemin avant Prepare d'update.py est réutilisable.
Le registre ne déclare pas les state_root custom des workers. Une absence
de racine dans ce registre ne prouve donc pas une absence de reçu.
Le pointeur CE08 reste qualifié pour T180, pas pour de futurs reçus C19.

La proposition d'inventaire explicite de toutes les racines est encore à
arbitrer avec une alternative plus conservatrice : le lecteur installé
pourrait imposer à tous ses successeurs la lecture de tous les formats qu'il
sait écrire, même sans scanner les données. Cela éviterait de traiter une
racine custom oubliée comme vide. Cette alternative n'est pas implémentée
et ne lève pas le gate de retour. La transition initiale et le retour
automatique pendant la maintenance restent à éprouver avant tout premier reçu.


### Traitement de la revue du noyau, ronde1

Tâche28951149-17f3-4046-9d60-2d16086ff3b7, 6273 octets lus/ACK,
clôture native demandée. APPROVE_WITH_CHANGES, patch complet lu ; aucun
test/build exécuté par le relecteur. Rapport privé :
/home/moi/.cache/bridget221/c19-kernel-review.md.

| Objection | Vérification et décision |
| --- | --- |
| Destination protégée seulement sous cwd, avec double canonicalize/unwrap | Confirmée et corrigée. Résolution unique de source ; toplevel, git-dir absolu et common-dir tous protégés. Test depuis sub et linked checkout, destinations dans racine ou metadata refusées avant écriture. |
| 60 s insuffisantes par principe | Non établie à cette échelle ; ne pas retirer les fsync de fichiers au profit d'un fsync de dossier, qui ne garantit pas leurs données. Mesure réelle de la nouvelle fixture10006entrées : export+audit3,577s sur Cartae. Mesure Opus2D et SYS/KS encore nécessaire avant activation, pas de délai historique modifié. |
| Incohérence65536entrées/8Mio arbre | Deux bornes indépendantes, pas promesse d'accepter tous les arbres de65536chemins de4096octets. ls-tree est bien en argument0, stdout8Mio constaté dans capture. Fixture10006entrées passe. Ne pas augmenter la sortie pour supprimer une borne de mémoire. Les deux refus restent distincts. |
| Omettre les liens sortants pour accepter le dépôt | Non retenu : le contrat approuvé impose leur refus. Ne pas changer silencieusement cette règle. L'action précise du refus reste à projeter dans le parcours natif. |
| Carte sans borne | Borne explicite8Mio ajoutée, contrôlée avant création. Carte entière conservée ou préparation refusée ; aucun chemin omis caché par troncature. |
| Modes exacts vs modeGit | L'audit vérifie le type Git et son bit exécutable. chmod ne constitue pas une autorité. La qualification des droits du fournisseur reste distincte. |
| CollisionsUnicode/casse à préclasser en minuscules | Pas de normalisation globale inventée. Le FS cible et les créations exclusives font autorité ; conservation/refus natif d'une vue partielle requis. macOS reste non qualifié. |

Après ces changements : 8 tests C19 PASS,0ignoré,4,79s au total hors
compilation ; le test d'arrêt emploie un vrai cat-file possédé et confirme
sa récolte après expiration de l'appel. La précédente suite33PASS incluait
5des8tests : aucune union33+8 revendiquée.
Journal : /home/moi/.cache/bridget221/c19-kernel-tests-r2.log.
T174/T030/T031/T032 toujours ouvertes.

## Noyau Linux et macOS, campagne suivante
Les types CommitView sont lisibles dans les structures fermées, mais la
création reste refusée avant tout reçu ou canon durable. Le rapport Git
historique refuse cette stratégie : il ne peut pas inventer un dépôt Git.
La réouverture SQLite est testée sur la valeur réellement persistée, après
normalisation de Task ; une première assertion de fixture a été corrigée.
Cette lecture de types seule ne qualifie pas un lecteur de retour C19.

Campagne finale actuelle du noyau : 17 tests daemon et 1 test transport passent
sur Cartae et Mac. Journaux privés c19-sync-tests.log sur Cartae et tests-r3.log
sous /Users/moi/.cache/bridget221-c19-mac sur Mac. Les campagnes se recoupent.
TMPDIR Mac est le chemin canonique court /private/tmp/cfabcc0 : l'alias système
et la longueur des sockets ont fait refuser une première campagne. Les gardes
de canonicalisation et de longueur restent strictes.

La carte décrit la sélection initiale, les omissions et les liens pendants ;
elle ne change pas après matérialisation. L'audit valide aussi les liens réels
sur le système cible : casse ou alias Unicode ne doivent pas permettre une
sortie ou un cycle. Le principal et tous les linked checkouts sont protégés
quand la source est elle-même un linked checkout.

La mesure de 10006 fichiers sur Mac était 48,8929 s avec sync_all par fichier.
Après changement : 2,3847 s Mac et 3,4940 s Cartae. Ce sont des échantillons,
pas cinq répétitions ni une garantie de coupure électrique. Les fichiers et
leurs modes passent d'abord par fsync, puis leurs dossiers des descendants aux
parents, puis la racine et son parent. Une seule barrière forte sync_all finale
sur la notice du même volume remplace les milliers de F_FULLFSYNC Apple.
Matérialiser un fichier isolé conserve sa barrière forte propre.
L'objection de délai de la revue précédente est donc un risque mesuré sur Mac,
traité dans le noyau ; la recette native à l'échelle Opus2D reste à faire.
Références primaires :
- https://raw.githubusercontent.com/rust-lang/rust/1.92.0/library/std/src/sys/fs/unix.rs
- https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html
- https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html

À terminer : preuves privées et leur réservation dans le budget worker,
possession durable des processus d'export, rapport/reprise/cancel, lecteur
minimal et parcours de maintenance, capacité négociée, admission et lecture
native des omissions. Aucun appelant de l'export en production, aucune case
T174/T030/T031/T032/T033 cochée, aucune activation C19 annoncée.

## Preuves privées et source supervisée — 31 octobre 2026
Les structures fermées ViewBinding / ViewIntent / ViewPublished lient
worker, commande de préparation, digest du canon, source, destination et
manifeste. La publication porte aussi SHA256 des octets de l'intention,
périphérique et inode. Création exclusive, synchronisation et audit : aucun
remplacement d'une preuve tronquée ou d'une vue existante sans publication.
L'audit reste possible sans source ; une omission matérialisée est contrôlée
contre le même blob. La notice publique n'est jamais utilisée comme autorité.

Le helper workspace-view-export du même binaire exige son canal géré avant
lecture de chemins. ManagedLaunch, ManagedStatusReporter et marqueur existants
sont réutilisés. Ready est comparé au Child possédé, à sa naissance, à son
groupe et à l'identité attendue avant RELEASE. L'environnement garde seulement
PATH, HOME, TMPDIR et LC_ALL=C. Les refus du canal portent une enum fermée ;
aucun diagnostic Git libre n'y entre. La fin du leader ne vaut pas fin du
groupe. La fixture qui ignore TERM conserve son marqueur et son verrou, puis
coopère par son fichier de fin ; aucun SIGKILL. Une réussite de processus sans
publication ne devient pas une réussite d'export. Le test de contexte absent
refuse avant lecture des chemins de preuve.

La réservation est dans WorkspacePreparationReceipt.commit_view, option
omise pour les reçus historiques : clé, digest d'intention et octets réservés.
L'inventaire existant compte les fichiers, orphelins compris, et ajoute la
différence réservée non encore écrite. Le quota plein refuse avant reçu,
preuve ou vue. La réservation survit à la réouverture. Une preuve devenue
invalide refuse cette préparation sans mettre tout le store en quarantaine.
Primitive interne non appelée par une admission de production à ce stade.

Preuve Linux finale actuelle : 30 tests daemon + 1 transport PASS,
c19-store-tests-r5.log. Deux tests storage_compatibility PASS dans
c19-format-tests-r2.log. Nombres recoupés, aucune somme avec les campagnes
antérieures. Les nouvelles fixtures ont d'abord échoué : helper Worktree
exigeant .worktrees ignoré, forme Git hors liste blanche, puis données
placées sous state_root. Le journal c19-store-observation.log atteste
base/commande/digest/source/worker égaux et data_inside_state=true.
Seule la fixture a été réorganisée ; pas de retrait de garde. Une assertion
d'arrêt exigeait à tort exit0 ; elle vérifie maintenant ABANDONED_EXIT_CODE.
Ces erreurs de fixture ne sont pas des RED de défauts en production.
Dernière campagne Mac avant raccordement quota : 26 daemon + 1 transport
PASS (source-r4, tests-r5.log). La campagne du quota y reste à confirmer.

Contre-revue privée de synchronisation/preuves demandée sous
bridget221-c19-private-proofs-review-20261031-01, tâche
0d178c83-9845-4fd9-8c71-3a9eb3cb11e1, enfant
c9647ef6-9d50-49bb-94c5-9882e939f7ff. Le moteur automatique a choisi
claude-sysle2-haiku213. Retour remote_start_failed:remote_start:WorkerUnavailable,
puis statut failed, résultat absent. Cancel unique rend cancelling ; aucun
avis, ACK de résultat, remplacement ou rejeu. Worktree conservé sur SYS.
Patch fourni de 93976 octets, SHA256
a9c2304f2da83e2991161185c150fc1befa86f20a38ee42102e711ca698e10d3.
Ce patch précède le helper et la réservation ; aucune revue nouvelle ne
couvre leur code. Le refus de lancement n'établit pas sa cause amont.

T174/T030/T031/T032/T033 restent ouvertes. À raccorder : propriétaire du
processus dans WorkerRuntime et Cancel/reprise, métadonnées privées du
WorkspaceRecord et vrai rapport, lecture des omissions liée à l'instance,
capacité négociée, preuve de sous-dossier d'origine, plancher lecteur complet
avant Prepare et retour, puis admission/fournisseur et livraison.
Le parseur seul ne qualifie pas un lecteur de retour. Le schéma MCP n'annonce
pas commit_view et les créations demeurent refusées avant Task/canon.
Aucun binaire C19 ni service nouveau livré. Programme toujours In Progress.

## Rapport de la vue, checkpoint suivant
Le modèle connaît WorkspaceRecord.commit_view (omis dans les Worktree
historiques). ViewBinding est maintenant un type de données du modèle,
avec sa clé calculée dans le module de preuves. La référence conserve
répertoire privé, liaison et digest d'intention ; aucun HEAD ou branche
inventés. Le rapport vérifie cette référence et les fichiers, sans Git ni
source disponible. Une divergence refuse le rapport et conserve le dossier.
Changer la preuve sous le même chemin/base ne satisfait pas same_workspace_anchor.
La projection de rétention lit les faits de fichiers d'une vue, sans exiger
les champs Git absents ; des faits contradictoires restent report_unavailable.
La projection ne fait aucune E/S et ne modifie pas Task.

Campagne Linux actuelle : 33 tests daemon et 1 transport PASS
(c19-record-tests-r2.log). Suite workspace_git complète : 53 PASS, 3 ignorés
(c19-git-regressions-r2.log ; un helper enfant C19 ajouté parmi les ignorés,
appelé explicitement par ses tests superviseurs). Les recoupements sont
importants ; ces nombres ne s'additionnent pas. Les 64 tests remote_worker
passaient juste avant cet ajout d'ancre. Dernier Mac confirmé : 30 daemon et
1 transport PASS, source-r5/tests-r6.log ; source-r6/tests-r7.log est la
campagne de l'ancre et du rapport, à constater séparément.

Tous les types nouveaux sont encore en préparation. Le raccordement
WorkerRuntime/Cancel/reprise et le plancher lecteur doivent être éprouvés
avant les premières écritures d'admission. Pas d'activation C19 annoncée.


## Propriétaire du processus, reprise et relecture native — 31 octobre
Le helper d'export est conservé dans WorkerRuntime jusqu'à l'arrêt constaté
et durci dans le reçu. Le reçu conserve source_marker avant RELEASE et
source_stopped après disparition du groupe ; supprimer le sidecar ne suffit
pas à obtenir StartFenced. Le tick récolte, sans relancer d'export.
Après perte du worker, la clôture retrouve le marqueur du reçu. Elle compare
la naissance réelle avant TERM, sans SIGKILL ni signal à une identité ambiguë.
Le signal exige la même Task ET la même commande, pas la Task seule.
Un redémarrage ne permet jamais un deuxième export du même reçu.

RED réel WIP : c19-source-fence-red.log montrait une clôture acceptée avec
export vivant. Corrigé, tests source-fence-green puis publication passés.
RED réel WIP suivant : c19-fence-attempt-red.log montre le signal à une
autre tentative de la même Task ; corrigé, c19-fence-attempt-green.log PASS.
Ce ne sont pas des défauts C19 activés en production : les admissions restent
fermées. c19-runtime-tests-r5.log était un échec de compilation de fixture
(Clone absent), pas un RED fonctionnel. r6 puis Mac tests-r9 montraient
StoreUnavailable au lieu du diagnostic source vivante ; r7 et Mac tests-r10
PASS après correction UnknownExecution/StopUnconfirmed.

La relecture de vue reprend les contrôles existants du worker : reçu de CE
store, même canon/base/commande, source arrêtée, discovery, preuve privée
dans ce store, aucun ancrage de réparation Worktree. Le mapping remplace sa
racine à partir des décalages Git privés ; il ne conserve pas l'ancienne
racine comme autorité nouvelle. La définition et les droits sont relus sur
le cwd de la vue. Le Shared peut avancer vers B et être sale : la vue reste
sur A ; aucun HEAD/index Shared n'est modifié. Preuve étrangère, posture
development, source non soldée et fichiers divergents restent refusés.
Le rapport seul reste possible sans source Git ; la relecture des droits
continue d'exiger un mapping courant valide.

Campagne Linux actuelle : 41 daemon C19 +1 transport PASS dans
/home/moi/.cache/bridget221/c19-reader-policy-tests-r4.log. Mac mêmes sources
source-r11 : 41 daemon +1 transport PASS dans
/Users/moi/.cache/bridget221-c19-mac/tests-r12.log. Suite remote_worker :
71 PASS dans /home/moi/.cache/bridget221/c19-worker-regressions-r5.log.
Ces suites se recoupent et ne s'additionnent pas. L'échec antérieur de
fixture (source sous le parent protégé de la clé) est observé dans r3,
source_inside_credential_parent=true ; seule la clé de fixture a été
isolée. Le refus CapabilityRequired d'inspect_candidate est conservé ;
le test construit son entrée conservée par les projecteurs purs existants.

Le compte rendu des vues distingue maintenant faits de fichiers et faits
Git : SHA complet, vue non Git, dossier conservé, contrôle conforme/
divergent/indisponible. Aucun nom de branche ou nombre de commits inventé.
c19-result-summary-tests.log : 41 daemon +1 transport PASS Linux, tests
intégrés au contrôle de rétention. Le dernier Mac ci-dessus précède ce seul
complément de texte ; il reste à confirmer sur le complément courant.
Les preuves d'interface et d'usage natif restent des gates avant livraison.

Contre-revue du noyau demandée sous
bridget221-c19-reader-lifecycle-review-20261031-01, tâche
2c1777b3-0195-4186-a149-898020277176, enfant
2c45d469-bac6-4348-a231-b4fe0f232e82. Patch de 274594 octets, SHA256
d01160187393d5e1a1b81bb95de0da0d125afcafad9f3ed1f5fd0e944b778cf3.
Retour WorkerUnavailable, puis failed sans résultat ; annulation unique
rend cancelling. Aucun avis ni nouvelle mission de remplacement.
Ce patch précède le complément de compte rendu. Worker SYS actif PID751532,
reçu avec bootstrap marqué puis terminal, sans RELEASE ni NativeReady.
Le journal de cet enfant rend AgentUnknown. Les journaux ciblés worker et
maître ne donnent pas la cause du refus ; cause précise inconnue.
Aucun service, modèle, compte ou droit changé pour contourner ce refus.
Les deux revues non démarrées 0d178c83 et 2c1777b3 et leurs dossiers restent
conservés ; aucun ACK de résultat absent, rejeu ou retrait forcé.

Restent : fichiers omis liés à l'instance/mission, retrait natif et quota,
origine/sous-dossier, négociation, lecteur de retour et contrôle avant
Prepare/retour, admission et recette fournisseur, livraison. Le plancher de
formats reste worktree.v1. 33/180 tâches closes, 147 ouvertes, In Progress.

## 31 octobre — matérialisation, durée de mission et verrou du contrôleur

Implémentation interne seulement : demande de blob liée à la commande complète,
mission, ronde et génération ; reçu de matérialisation avant RELEASE, marqueur
corrélé, aucun écrasement, résultat disponible seulement après arrêt confirmé
et audit du fichier. Le même créneau et le même propriétaire de processus sont
réutilisés. Annulation, Reconcile, fence et collecte ne libèrent pas une preuve
dont le helper est encore vivant. Après refus de Release, le passage suivant
range seulement le marqueur privé exact dont l'arrêt est durci.

Trois RED observés puis corrigés :
- c19-materialization-abandoned-marker-red.log : marqueur arrêté d'un bootstrap
  refusé bloquait une nouvelle exécution légitime ; green 49 daemon +1 transport.
- c19-file-launch-deadline-red.log : deadline de bootstrap traitée comme fin de
  mission active. Elle reste inchangée ; la preuve d'activité vient du slot
  corrélé non terminal et des gardes courantes.
- c19-file-precancel-red.log : annulation préalable modifiait le reçu ; mutex
  d'annulation empoisonné était présenté comme une demande d'annulation.
  Désormais refus avant réservation ; le cas illisible rend StoreUnavailable.

La campagne Linux c19-file-liveness-cancel-green.log a ensuite donné 51 PASS,
1 échec au lancement d'un helper (Unknown). Le cas isolé a passé dans
c19-managed-omission-observe.log ; la trace de la suite Mac r14 a identifié
export_busy. Aucun modèle, service ou droit modifié pour faire passer ce cas.

Le test déterministe c19-lease-inheritance-red.log reproduit flock conservé
par une description dupliquée après fermeture du descripteur du propriétaire.
ViewLease libère explicitement le verrou avec LOCK_UN à son Drop ; CLOEXEC
reste présent. Le marqueur privé reste une preuve indépendante et bloque
encore un nouveau helper même après libération du verrou local. Le contrat
ne déduit aucun arrêt de cette libération de verrou.

État courant vérifié :
- Linux : /home/moi/.cache/bridget221/c19-lease-marker-final.log :
  53 tests daemon C19 +1 test transport PASS.
- Mac : /Users/moi/.cache/bridget221-c19-mac/tests-r15.log :
  mêmes 53 +1 PASS, source immuable source-r15.
- Régressions worker : /home/moi/.cache/bridget221/c19-worker-regressions-r6.log :
  80 PASS ; elles précèdent la seule garde flock finale et se recoupent avec C19.
  Les chemins C19 modifiés de cette garde sont couverts par les suites finales.
- Archive source r15, 21 fichiers vérifiés sur les deux machines :
  SHA256 4d3f64da22e7cea534b714967d642b38d5d791c3e37da458fe11bd6088b7191e.
  Base f80fe1a748091785f1a3f453f39b8f75258e21ed + modifications non commitées.

La suite Mac r15 a aussi préservé les worktrees dont les fichiers ouverts sont
inconnus ; aucun retrait forcé. Entretien Cargo passé : Linux 3,34 Gio, Mac
5,36 Gio de caches logiques. Aucun installed : aucun nouveau binaire installé.

Ces tests emploient de vrais helpers, groupes et Git locaux avec données
synthétiques. La commande/mission Released de fixture est simulée ; ce n'est
pas une admission ni une qualification fournisseur. Les créations publiques
restent refusées. Aucun service, mission métier ou ancien canon modifié.

Restent avant ouverture : retrait natif avec conservation sur divergence et
preuve des usages ; port non bloquant de lecture d'omission ; reprise fencée
et relation de commande actuelle à l'ancre source ; origine/sous-dossier et
stockage sur volume projet ; négociation ; lecteur et retour complets ;
contrôle avant Prepare/retour ; usages MCP réels puis livraison et preuves
d'interface. T174/T030/T031/T032/T033 restent ouverts. 33/180 clos, In Progress.

### Lecture en cours : sortie de la boucle bloquante

Le parcours privé est maintenant séparé en begin_view_file et poll_view_file.
Le jeton ne copie pas le manifeste. Seul l'adaptateur cfg(test) attend en
boucle. La collecte existante observe aussi la liveness du slot et le signal
d'annulation : un helper de mission terminée reste détenu jusqu'à son arrêt,
avec ProofUnavailable plutôt qu'une annulation utilisateur inventée.
Un autre Reconcile est traité pendant une lecture qui attend réellement.

c19-file-nonblocking-tests.log : 54 PASS,1 échec de fixture. Le test attendait
Reserved pour un autre bootstrap sans marqueur ; le vrai Reconcile rend
Unknown. La fixture a été corrigée pour exiger ce refus honnête ; aucune garde
de production n'a été assouplie. Le second test confirme qu'un poll ne réécrit
pas le reçu d'échec déjà publié par la collecte.

Preuves courantes, qui remplacent les comptes précédents sans les additionner :
- /home/moi/.cache/bridget221/c19-file-nonblocking-tests-r2.log :
  Linux55 daemon C19 +1 transport PASS.
- /home/moi/.cache/bridget221/c19-worker-regressions-r7.log :
  Linux82 remote_worker PASS.
- /Users/moi/.cache/bridget221-c19-mac/tests-r16.log :
  Mac55 daemon C19 +1 transport PASS.
- Source immuable /Users/moi/.cache/bridget221-c19-mac/source-r16 ;
  21 fichiers d'override vérifiés, archive SHA256
  073c0ecd804f60e93e8faebf14eba4f7b59a5bb48476ad99163b57f3049734c2.

Les audits de complétion restent synchrones et bornés. Leur coût doit être
mesuré au raccordement ; cette recette ne prouve pas une absence générale
d'attente du worker. Pas de nouveau thread, queue, store ou ordonnanceur.
Le port public, la négociation et la qualification fournisseur restent ouverts,
ainsi que le retrait et le retour de version. Aucune case de tâche cochée.

### Clôture native et trou avant READY

Le premier contrôle de retrait est en place, sans action destructive :
check_view_native_closure exige commandes et preuve d'arrêt correspondantes,
ainsi que dispositions finales des slots. Un Cancel sur un résultat non
acquitté ne suffit pas. Le store conserve désormais commandes et compacts de
slot des vues au-delà de sept jours, jusqu'au futur tombstone. La garde ne
prouve ni absence de lecteur externe ni permission de retrait.
Premier essai : 56 PASS,1 échec de fixture ; un slot dont la commande a été
retirée fait refuser inventory en Quarantined, pas Unknown. L'assertion de
fixture suit maintenant le vrai refus existant, sans assouplir ce refus.

c19-unready-bootstrap-red.log a montré Child retenu et encore vivant, sans
READY/marqueur, mais stop_confirmed=Ok(true) et source_stopped=true. Le reçu
durcit maintenant la tentative avant spawn. La source et la matérialisation
sans READY restent non confirmées ; une reprise sans propriétaire en RAM ne
relance pas ce helper. Le constructeur de production exige l'enregistrement ;
les démarrages non enregistrés ne subsistent que dans les fixtures cfg(test).
Le test conserve puis ferme réellement son Child de fixture ; pas de SIGKILL.

c19-settled-pid-reuse-red.log a ensuite reproduit une publication close bloquée
par un groupe étranger réutilisant le numéro historique. La reprise garde
l'arrêt durci et ne signale pas le groupe de fixture ; son stdin est fermé
par le test et cat termine normalement.

Dernière preuve Linux :
/home/moi/.cache/bridget221/c19-start-attempt-and-pid-final.log :
60 daemon C19 +1 transport PASS. Elle remplace les comptes précédents.
Le complément ci-dessous consigne ensuite Mac et les régressions worker.
Les états de commande/fournisseur des fixtures restent simulés ; seuls les
helpers, groupes, fichiers et lectures Git sont réels.

Inventaire de lecteurs, prototypes privés seulement :
- Mac lsof : sept cas de fixture passent après reconnaissance du séparateur
  f du format machine ; fd, cwd, répertoire, mmap et renommage détectés.
- Linux lsof : sortie partielle accompagnée de diagnostics, refus d'absence.
- Linux procfs utilisateur : lecteurs positifs détectés, mais 164 refus de
  métadonnées ; les PIDs sondés comprennent sd-pam et sshd du même UID.
  Ils ne sont pas ignorés d'après leur nom.
- sudo -n -l a confirmé un droit administratif déjà présent, sans mutation.
  Une recette de métadonnées procfs sous ce droit a supprimé ces erreurs.
  Un des deux cas sans lecteur a cependant vu changer l'ensemble des processus :
  refus conservé. L'autre cas était stable et vide. Aucun inventaire général
  toujours complet n'est attesté.
Aucun accès à un secret, aucun modèle ou service lancé, aucun droit modifié.
Les seuls dossiers retirés sont les fixtures privées créées par ces scripts,
après fermeture de leurs lecteurs. Aucun retrait de vue réelle.
Ces scripts ne sont pas du code produit ni une autorisation administrative
ajoutée au worker. Choix utilisateur confirmé : aucun nouveau privilège ; conserver les fichiers
si le contrôle des usages est incomplet. Aucun contrôle administratif produit.

Complément de preuve sur le même ensemble r20 : Mac tests-r20.log donne
60 daemon C19 +1 transport PASS ; Linux c19-worker-regressions-r8.log donne
87 régressions worker PASS. Les suites se recoupent, sans somme globale.
Choix utilisateur confirmé : conserver si le contrôle est incomplet, sans
ajout de privilège. Aucun contrôle administratif ne sera ajouté au worker.


### Conservation close et référence privée — preuves r22
Choix sans ajout de privilège appliqué au contrat et au code interne C19.
La clôture native est monotone et conserve les fichiers ; le lecteur reste
séparé du constructeur Active. Aucun Start/export/matérialisation sur une
référence close, ni réécriture de résultat tardif. Les ports C19 restent fermés.

Six nouveaux cas couvrent : reconstruction sans source ; carte corrompue ou
absente/liée ; inode remplacé ; petit temporaire partiel/lease occupé ;
coupure avant et après la réduction du reçu ; collecte après sept jours sans
perte du tombstone ; résultat tardif d'une matérialisation déjà close.
Les reçus de mission sont simulés ; helpers, fichiers et commandes Git réels.

Première compilation : deux erreurs de syntaxe/trait dans le nouveau code/test,
corrigées. Premier essai exécutable : 3PASS/2FAIL, le rapport utilisait encore
le constructeur Active ; séparation lecture/écriture corrigée. Dernier ensemble :
- /home/moi/.cache/bridget221/c19-closed-reference-final-r22.log :
  66 daemon C19 +1 protocole PASS.
- /Users/moi/.cache/bridget221-c19-mac/tests-r22.log :
  mêmes 66 daemon C19 +1 protocole PASS sur les21fichiers empreintés.
- /home/moi/.cache/bridget221/c19-worker-all-r22.log :
  89 tests du worker PASS, recoupant C19 ; pas de somme globale.
- /home/moi/.cache/bridget221/c19-worker-regressions-r22.log :
  sous-ensemble worker162,18PASS, inclus dans les89.

Fixture512fichiers : intention privée60685->605octets ; carte94496octets,
octets/inode/mtime inchangés et tous les fichiers conservés. Ce n'est pas un
benchmark de latence ni une mesure sur un projet métier.
Archive21fichiers r22 SHA256bf1d6338af06bddb91df1adabfe41ee61736f316bb0c2fc92eed7f840b659be9.
Entretien automatique : Linux3,34Gio / Mac5,36Gio logiques conservés.

Aucun nouveau privilège, dépendance, magasin ou processus administratif.
Aucune suppression de vue, aucun service/modèle/mission métier lancé ou relancé.
Pas de commit/activation C19 avant les gates restantes : retrait conservateur,
source asynchrone, reprise fencée, lecteur de retour, intégration et usage natif.
Les deux contre-revues WorkerUnavailable antérieures ne donnent aucun avis ;
aucun troisième remplacement demandé. Le clientMCP214 reste la dernière version
appelante constatée ; son rechargement complet dépend toujours de l'application.

### Préparation asynchrone et ancre de reprise — r24 à r26
Begin/poll source écrit, wrapper synchrone cfg(test) seulement. source_failure
fermé dans le reçu après arrêt ; résultat conservé même si le lecteur disparaît.
Les polls pending ne chargent pas le manifeste. Quatre cas r24 puis suite :
70daemon C19+1protocole PASS Linux.

Reprise privée : BindProof198/original_workspace_at, sans registre d'alias.
Une commandeB lit/matérialise dans la vueA seulement avec son slot actif,
son canon198 et son opération exacte. Une requête de mêmeTask sans preuve
ne passe pas le bootstrap ; les contrôles du helper relisent l'ancre.
CancelB cible son opérationB complète ; CancelA ne touche pasB.
Premier ensemble r25 :72PASS/1FAIL, entretien de stop_once resté sur l'ancienne
clé générique. Le RED a identifié le vrai retrait prématuré de cette clé.
L'entretien conserve maintenant l'identité d'opération exacte.

Dernière preuve Linux :
/home/moi/.cache/bridget221/c19-fenced-final-r26.log :
75daemon C19 +1protocole PASS. Tests ajoutés : canon/fence manquant/ancre close,
matérialisation B réelle dans A, CancelA/B et répétition, fence avant publication
puis redémarrage, résultat Stopped rejoué identique.
Mac reste à vérifier sur ce nouvel ensemble. Aucun Start fournisseur,
création publique, activation ou ancien travail métier relancé.

Contre-revue native64d7422f-19cc-4bd7-b892-62b22a83bc8d, ClaudeHaiku/SYS,
APPROVE_WITH_CHANGES sur quatre extraits r24 seulement. Résultat lu/ACK et
clôture demandée une fois ; dossier conservé, pas de retrait implicite.
Traitement :
- source close avec Child déjà arrêté : prise en compte sans réécrire le reçu ;
  le collecteur peut libérer son propriétaire ; FOP close vérifie opération/stop.
- résultat Stopped supposé perdu : test réel confirme sa répétition identique ;
  le résultat ne disparaît pas quand le durcissement doit encore attendre.
- contrôle pending léger : voulu et limité à une observation sans effet disque ;
  liaison complète/hash/racines relus à la publication.
- annulation mémoire : signal rapide seulement ; fence198durable est l'autorité,
  revérifiée sous transition avant publication. Nouveau test de redémarrage PASS.
- après spawn : chemins de refus relus, tous gardent Child jusqu'à arrêt confirmé ;
  le faux trou avant READY reste couvert par les tests réels.
Pas d'APPROVE global déduit de cette revue bornée ; intégration publique à faire.


## Suite C19 après livraison du catalogue —31octobre
Base intégrée f8691f98 ; sauvegarde des21fichiers précédents SHA256a5c0f45c9daad775e48a4c995ed28230872f86827845234d0ea566ffa6e805a1 et stash563647867f053185b5f304ff5931c7b00df8eeff conservés. Quatre conflits d'ajouts indépendants combinés (deux branches du refus, et trois documents),17autres fichiers vérifiés identiques. Première intégrationR27 :111daemon+3transportPASS,3privésignorés ; pas une qualification fournisseur.

Raccord de conservation : ConserveCommitView et CommitViewConserved, empreinte de canon seule, identité originale de préparation et capacité WorkspaceViewV1 négociées. Ancien pair : refus avant émission ; borne11de capacités, trames historiques sans champ ajouté. Le worker relit sa liaison et exige arrêt/disposition des résultats avant freeze. Aucun retrait de fichier, audit512Mio ou réduction de manifeste dans la boucle de contrôle. La réduction privée existante reste séparée sous son budget, réservation pleine conservée.

Le maître réserve le claim de rapport existant dans la clôture native ; aucune action depuis task_view ou ACKseul. cleanup_done restefalse tant que l'ACKexact n'est pas acquis ; CAS sur Task/mission/ronde/liaison et noncontinuabilité avant le bit final. Les erreurs métier sont conservées. Réponse tardive exacte absorbée sans effet ; étranger/époqueancienne refusés. Pas de champ de Task, table, file ou autorité supplémentaire. Une machine désactivée ne reçoit aucune admission : seulement une conservation de liaison connue si son lien est déjà négocié.

R36 :114testsdaemon+4transportPASS,3recettesprivéesignorées. Ce total recoupe C19,220etlecomplémentcatalogue ; ne pas additionner les anciens comptes. Test maître via vraie paireUnix : zérooctet pourcapacitéabsente, zéroeffet siTaskcontinuable, bitcleanup_doneencorefalseavantACK, vraientréehandle_worker_message, CASpuisréponses tardives etétrangères sansmutation. Le testworker emploie de vrais helpers d'export mais une mission simulée ; aucune qualification modèle revendiquée.
Échecs conservés :R28adaptateurd'erreurtypeincorrect,R30continuableprisàtortpourunchampau-lieudelaméthode,R31importReaddu testmanquant,R32fixturedu registre enmémoire sansfichier. Corrections ciblées après lecture des données/source ; assertions de sûreté conservées.

Statut natif des vues : résultat nonremis, conservation nonconfirmée et arrêt nonconfirmé distincts ; aucune preuve d'arrêt depuis un ACK absent. Le message nomme la vérification manquante et les autres travaux. Mac sur source figée en attente au moment de cette note. Lecteur de retour, formats, préparation publique et qualification Codex/Claude encore requis ; FORMATS reste workspace.worktree.v1 et créationsC19fermées.

Limite du canal du responsable : une réattestation réussie puis un capabilities refusé credential_mismatch après chargement221-catalogue. MCP21458296attesté par lsof ; aucun outil de reconnexion exposé, aucune boucle ni copie de secret. Les revues et qualifications natives nouvelles ne sont pas remplacées par un autre moteur/CLI. Rechargement complet de l'application T3 requis, puis reprise du même fil. Ce blocage ne constitue pas un verdict source favorable ou une réussite C19.

### Gate de lecteur avant bascule et retour — r37 à r39
Le contrôle de formats est raccordé avant Prepare dans l'updater, puis avant
retour automatique et récupération après coupure. L'image nouvelle impose le
plancher au retour, même quand le lien pointe encore vers l'ancienne.
Nonce neuf, SHA256 de l'image avant/après, rapport8Kio et validation par le
contrôleur exigés. Commande absente ou reçu mal formé : conservation, aucun
arrêt ni changement de lien ; message de portée et action du responsable.
Aucun inventaire manquant n'est pris pour un magasin vide.

22testsPython updater PASS, dont5 nouveaux : liaison nonce/image/validateur,
rapport absent/tropgrand/falsifié, refus avant Prepare, refus de retour avec
intention conservée et récupération avant Prepare. Les fixtures ne qualifient
pas un lecteur de tous les états conservés.

Recette du vrai binaire privé r39, sans service ni fournisseur :
/home/moi/.cache/bridget221/c19-formats-real-r39.json.
SHA256097963a5724f3b9430990fb76bfd8a111a186201e304c0507dd03a05cb963a2d ;
formats policy ne déclare toujours que workspace.worktree.v1 ; contrôle avec
le même lecteur réussi, nonce falsifié refusé, ancien installé sans port
refusé update_storage_reader_unverified, lien actif221-catalogue inchangé.
L'installateur, le bootstrap du lecteurA et l'ensemble des formats C19 restent
à qualifier avant activation ou première création.

Deux limites de méthode conservées : première commande de build lancée par le
wrapper principal sans manifest explicite, donc mauvais checkout compilé ;
commande rectifiée. Puis dépendance transport ancienne réutilisée dans le
cache commun après changement de source : compilation refusée, aucune
activation. La construction réelle utilise désormais le cache enregistré
bridget-build-221-c19, partagé entre tous les essais C19 et distinct du lot
de livraison dont le protocole diffère. Pas de copie de target ni de cache par
test. Les sources du candidat sont confirmées dans le journal r39.

Le correctif T182 d'écritureMac, développé séparément, est repris dans la source
privée pour débloquer sa qualification. Les admissions C19 restent fermées ;
aucun ancien calcul, service métier, secret ou droit n'est modifié.


## Préparation possédée et audit différé — r55, 31 octobre
Source : /home/moi/.cache/bridget221/c19-overrides-r55.tar.
SHA256 e72dd65186bbf3ca5714c7812df40842a7472097b547a108168b1951e8838217.
24 fichiers, base 696d13af ; snapshot Mac exact vérifié avant tests.
Linux : /home/moi/.cache/bridget221/c19-plan-r55-all.log,
126 daemon + 4 transport PASS, 3 ignorés.
Mac : /Users/moi/.cache/bridget221-c19-mac/tests-r55.log,
122 daemon + 4 transport PASS, 2 ignorés. Aucun recoupement ajouté au total.
Les erreurs antérieures restent dans les logs r48/r50/r52/r53/r54. Dernier
échec : la fixture ouvrait le store encore verrouillé. Le runtime précédent
est désormais fermé avant réouverture ; le verrou produit reste inchangé.

Revue R51 : tâche 3e0ed74e-74a8-421b-8929-d8f3781cf4cb, résultat complet lu,
ACK confirmé après une réponse JSONL perdue, Cancel demandé une fois.
État result_available/review_required ; worktree conservé.
P2-A : cause PreparationExpired distincte, persistée après arrêt et testée.
P2-B : réservation avant graine conservée ; seule la même graine non tentée,
avec sa même empreinte et son même délai, peut être complétée.
P2-C : conservation native avant publication, si jamais tenté ou arrêt durci,
sans processus, commande ou remise active. Fichiers et tombstone conservés.
Le budget futur est libéré ; les octets physiques restent comptés.
ZERO utilise TERM seul, sans SIGKILL.
Audit différé : un thread de lecture par vue, joint seulement après sa fin ;
Cancel interdit publication, sans prétendre arrêter une lecture système.
Les contrôles de contexte précèdent l'attente et sont relus après audit.
Revue R56 : clé bridget221-c19-planification-revue-r56-20261031-01,
tâche ee6ef1f8-1cd7-49b9-b93a-399969c80b0e, complément r51→r56.
Aucun avis global ni d'activation déduit.
Binaire privé compilé : /home/moi/.cache/bridget-build-221-c19/debug/bridget.
Recette native hors fournisseur en cours. Aucun lancement public ni activation.


Recette native Linux r57 PASS : même binaire privé r55, SHA256
9a13cf341a61b9a304a63c0d33742a6d75299a9e479060b906b6cde85f91f1cc.
Le bootstrap de production et le dispatch CLI réel ont planifié, exporté et
audité la fixture. Source inchangée, pas de .git dans la vue, fichier omis
resté absent, groupe arrêté confirmé. Aucun fournisseur ni Task lancé.
Log : /home/moi/.cache/bridget221/c19-native-r57.log.
Preuve : /home/moi/.cache/bridget221/c19-native-r57.json.
La seule différence r57/r55 est l'opt-in de la recette sous cfg(test), sans
effet exécutable en production. Cette recette ne qualifie ni modèle ni mission.


## Raccord lecteur C19 — R103 et complément R107, 31 octobre

R103 : Linux153 tests daemon +6 transport PASS,4 ignorés ; Mac149 daemon +6
transport PASS,3 ignorés. Les trois exécutions de fixture enfant rapportées
séparément dans le journal Linux ne sont pas additionnées au total153.
Logs : /home/moi/.cache/bridget221/c19-combined-r103.log et
/Users/moi/.cache/bridget221-c19-mac/tests-r103.log.
Archive exacte32 fichiers SHA256
c4c1371b7af5a038ed1f71d727bc2abaa45d32af0b3e7a9f97a6d8e573c153ec.

La lecture de fichier natif vérifie parent propriétaire ou enfant courant,
Task active, pause/révocation, ronde, slot unique, exécution et ancre ; relecture
après réponse. L'écriture socket est bornée à1s, la réponse à125s et le worker
à120s. Un ancien pair ne reçoit aucune nouvelle trame. Le chemin rendu appartient
à la machine du worker, jamais implicitement à celle du parent.

Le contrôle d'intégrité du fichier passe par un thread de lecture sans store.
Le runtime durcit l'arrêt puis reste seul écrivain du CAS. Un arrêt inconnu ne
libère pas la possession. La reconnexion conserve les annulations liées à la
commande d'exécution, même si la vue vient d'une préparation précédente.
L'absence du Child en mémoire après échéance n'est pas une preuve d'arrêt.

Revue adverse R103 : Claude/Sonnet via claude-ksstore, tâche
2a671b18-f24b-4046-84ac-754ee80a4f76. Patch2279lignes lu en entier, sourcesLinux
ciblées, aucun test exécuté. APPROVE_WITH_CHANGES, pas de verdict d'activation.
Résultat lu entièrement, ACK confirmé, Cancel demandé une seule fois ; worktree
conservé sous contrôle d'usage non complet.

P1 de gate retenu : annoncer commit_view_files_v1 après le vrai raccord. Le
chemin MCP positif est ajouté et passe sous R105, deux formes parent/enfant.
Le maître annonce désormais cette feature de lecture sans ouvrir la création.
P2-a retenu : Locked devient OperationCapacity ; test à deux chemins pendant
le contrôle différé PASS sous R104. Le même fichier rejoint l'opération, aucun
second helper ; un autre fichier ne remplace pas le reçu encore contrôlé.
P2-b retenu : refus worker fermé et guidage par code, avec tests ciblés ; une
réponse indisponible ne devient pas une preuve de panne réseau ou d'arrêt.
P3 chemin : machine_id rendu et consigne de lecture sur place. P3 garde inconnue :
recours ciblé au responsable et à l'utilisateur, aucune attente infinie conseillée.
Instrumentation FENCED temporaire retirée. Aucun type commun pending nouveau :
les trois contextes ont des autorités et des durées distinctes. La borne des
lecteurs reste128 par famille ; aucune promesse de débit simultané. Un plafond
plus bas demanderait une mesure, ce n'est pas une correction démontrée par la
revue. Les lecteurs ne possèdent ni fournisseur, ni store ni droit d'écriture.

R107 Linux/Mac et contre-revue du complément restent à terminer. FORMATS ne
revendique pas encore C19 ; la création reste fermée. Aucun commit, activation,
qualification fournisseur ou clôture de T174/T030–033 depuis ce point privé.
Programme complet maintenu :35/182 vérifiées, In Progress.

R107 : 156 tests daemon + 6 transport PASS sur Linux, 4 ignorés ; 152 tests
daemon + 6 transport PASS sur macOS, 3 ignorés. La demande de contre-revue
conserve la clé bridget221-revue-c19-r107-20261031-01 après un refus
binding_transport_unavailable sans Task. Une lecture du ledger est revenue
(page partielle, aucune absence inférée), puis UN rejeu strict a repris cette
même enveloppe : tâche f70ba5d8-8d4d-4efb-883e-edb5f2cab3bd admise.
Aucun remplaçant. Le résultat reste à lire.

R110 : plancher de lecture C19 raccordé dans le candidat ; deux contre-épreuves
de formats PASS, 43 tests installateur PASS, 22 updater PASS, guides32/522/13PASS.
Les nouveaux cas couvrent le refus avant fournisseurs et services, le rapport
absent ou trop grand, et le paquet ancien refusé au-dessus du contrôleur
courant. L'exception de première installation v0.6.3 reste explicite et non
qualifiée pour les données préexistantes.

Le premier run R108 avait 20 échecs causés par un guillemet mal échappé dans
l'exécutable fixture. R109 n'avait plus qu'un échec : la fixture historique
simulait le code1 au lieu du vrai code2 pour une commande inconnue.
Les corrections ont rectifié ces fixtures, sans affaiblir la garde produit.
Aucune activation depuis ces contrôles.


## Fermeture des objections de publication — R114
Revue du complément R107 : tâche f70ba5d8-8d4d-4efb-883e-edb5f2cab3bd,
APPROVE_WITH_CHANGES, patch418lignes lu et sources Linux ciblées, sans exécution.
Résultat lu entièrement, ACK confirmé et Cancel demandé une fois. La tâche est
non continuable ; worktree conservé faute d'inventaire complet de ses usages.

P2-1 confirmé et corrigé : OperationCapacity commun conseille la relecture de
l'état de tâche ; seul le port fichier propose de demander ce fichier après
fin attestée. Assertions distinctes de préparation et de lecture ajoutées.
Le champ cause_kind distingue les codes worker des causes de vue, sans retirer
le code technique. Les guides disent que la copie crée aussi des marqueurs.

P2-2 : le collecteur conserve désormais un constat Ready jusqu'au CAS, sans
recommencer le helper ou l'audit des données. Réessai d'écriture au plus une
fois par seconde ; une clôture native exacte abandonne ce constat devenu
inutile sans inventer sa disponibilité. Le store et son propriétaire restent
uniques. Le thread de lecture ne reçoit toujours aucun droit d'écriture.
La contre-épreuve réelle rend le reçu illisible (PermissionDenied constaté),
laisse finir l'audit, vérifie Ready, puis rétablit cet accès de fixture et
termine le CAS au tick sans nouveau poll du demandeur. Le premier essai avait
retiré l'écriture du mauvais dossier parent : aucune publication n'était alors
refusée. Le chemin réel workspaces/<clé>.json a été lu avant correction.
Une vraie panne d'écriture de WorkerStore peut mettre ce store en quarantaine :
ce garde n'est jamais effacé par la reprise ; le constat reste conservé pour
maintenance autorisée et le refus StoreUnavailable conserve son recours ciblé.

R114 Linux :158tests daemon +6transportPASS,4ignorés. Log :
/home/moi/.cache/bridget221/c19-combined-r114.log. Un test ciblé du CAS est inclus
à ce total et n'est pas additionné une seconde fois.
Recette privée du vrai binaire : planification, export et fichier omis passent
par le bootstrap de production, sans fournisseur ni Task, sans .git et sans
modification du dépôt source ; groupe arrêté confirmé. Image testéeSHA256
5a0e349a9e2d21e63f380fabb58e52512a8ca401ed12566164fcca857f081872.
Log : /home/moi/.cache/bridget221/c19-native-r114.log.
Le premier essai R111 avait gardé l'objet d'export et donc son verrou : conflit
export_busy avant fork. La fixture le vérifie explicitement, le code conserve
maintenant la cause OperationCapacity, puis le détenteur arrêté est retiré
comme dans le runtime. La recette réussie ne qualifie aucun modèle.

Le vrai contrôle de formats de l'installateur passe aussi avec le candidat,
et refuse le contrôleur actif221-write sans port formats. Reçu :
/home/moi/.cache/bridget221/c19-installer-real-r110/receipt.json.
Cette preuve est bornée à l'image et à la fonction indiquées ; ce n'est ni une
installation réelle ni un verdict sur une image de retour encore à construire.
L'image doit être figée puis qualifiée avant la bascule ReaderA/WriterB.
La carte publiquev1 est canonique et auditée octet pour octet : son texte fait
partie du format initial. Après première admission, son évolution exigera un
lecteur de version compatible avant tout nouveau producteur.

Mac R114, contre-revue des corrections et du contrôle installateur, audit,
Git, images de retour et activation restent à terminer. Le créateur reste
false ; aucune mission métier ou ancienne demande n'a été relancée.
Statut du programme complet inchangé :35/182 vérifiées, In Progress.


R116 : trois contre-épreuves d'audit PASS sur Linux, dont la réouverture du
worker après arrêt durci mais avant publication. Ce démarrage reprend seulement
la lecture de données des opérations stopped && available=None ; aucun helper,
fournisseur, mission ou requête client n'est créé. Les opérations dont l'arrêt
reste inconnu sont exclues. Le constat est ensuite durci par le runtime.
La correction de compilation de R115 a repris la construction explicite de
WorkerRuntimeConfig déjà utilisée par les fixtures de réouverture ; aucun
nouveau trait Clone ni refactor de configuration.
Mac R114 :154tests daemon +6transportPASS,3ignorés. R116 complet et la revue des
corrections restent à vérifier. Aucune nouvelle activation.

R116 complet Linux :159tests daemon +6transportPASS,4ignorés. L'installateur passe aussi43tests sur macOS, dont les bornes de rapport avec son shell natif. Les refus de formats nomment désormais leur portée : cette installation candidate seulement ; les autres travaux suivent leur propre état. Cette précision ne change aucun format ou seuil. La source reste privée, le créateurfalse, le lot global In Progress.

## R118 — fermeture de la contre-revue R117
Le contrôle des formats précède désormais prepare_service, y compris enable-linger Linux. Le refus fermé storage_reader_incompatible est distinct du contrôle indisponible ; aucun diagnostic libre du validateur ne sort. Test réel de fixture linger ajouté avant la mutation. cause_kind est absent sans cause et documenté FR/EN. Le message fichier ne produit plus de double espace. Le lecteur d’audit réutilise la classification fermée du helper : une E/S ne devient plus une intégrité rompue par défaut.
Limites conservées : une reprise dont l’inventaire privé est invalide conserve la garde et demande au propriétaire une lecture ciblée après réparation ; elle ne saute pas un reçu corrompu. Après redémarrage, un helper arrêté dont la cause n’a pas été durcie peut donner fichier absent avec cause inconnue. Aucun nouvel export ni diagnostic de cause inventé. Créateur public toujours fermé, aucun déploiement revendiqué.
