# Validation234 — 5 novembre 2026

Source exécutable aa3cbeb1bd4daf3e793bc8f6f8afaca5aa5c7882, version0.8.2,
build234-inventory-r3. Sept composants activés et empreintes chargées vérifiées.
Revue favorable acquittée, façades et lien natif revenus, release publique0.8.2
vérifiée. Sources et bilan intégrés àmain dans la transaction de livraison.
Le code exécutable reste strictement celui du commit aa3 ; les derniers
changements sont documentaires.

## SYS : faits et correction

Deux refus compact_plan/git_timeout, tâches396260c0 et a845e053 annulées,
cleanup_done=true. La commande historique exacte n’est pas conservée. Cette
étape lit deux réglages Git puis l’arbre ; elle ne compte pas les worktrees.
Le dépôt contient94worktrees, sous la borne128 ; liste6ms. L’inventaire actuel
de la base dcd7356f6ddd4d7a14e10165c4caea9f539e416a fait4739540octets :
Git worker2.56.0,160ms direct puis668ms avec l’ancienne cadence de lecture.
La latence initiale reste inconnue. Aucun effet fournisseur déduit.

Un inventaire valide11s échoue avec l’ancien délai10s puis passe avec le budget
explicite60s. Contrôles courts10s et autres délais explicites conservés. Les deux
flux restent bornés et équitables ; pause1ms seulement sans donnée. Diagnostic
fermé bridget_workspace_git_timeout : opération, commande normalisée, budget,
durée, volumes retenus et arrêt ; jamais les arguments, chemins ou stderr.
Contexte de tâche limité au thread de création, restauré par sa garde.

Plan natif réel du commitDCD :408ms,22519fichiers sélectionnés,612886253octets.
HEAD actuel a8c63ede131ffed75323f15c6e87de4f8bf9ecc7 et liste des94worktrees
strictement inchangés. Aucun worktree, branche ou mission créés pour ce contrôle.
La mission métier ad3baf84 surCartae avait déjà son résultat avant la bascule ;
son enveloppe reste identique et son travail suit son propre propriétaire.

## KS : défaut232 confirmé, récupération et disque

Le lot232 écrivait une réservation64Mio+21376octets mais la relisait encore
avec une borne8Mio+21376. La revue eb1b8fc3 n’a jamais démarré. Le worker est
entré en quarantaine. Défaut corrigé par une constante partagée.

Worker arrêté, sauvegarde complète conservée, puis récupération native froide
avec identité provisionnée, verrou exclusif existant, inventaire complet,
commandes terminal/refused, marqueurs arrêtés et témoin exact non adopté.
69commandes/68slots validés.465fichiers du magasin strictement inchangés.
Seule l’inscription de quarantaine est levée, avec sa sauvegarde ; processus0.
Le worker a ensuite repris. Aucun reçu ou marqueur effacé pour forcer le passage.

Le suivi SQL excluait cancelled+workspaceNULL malgré commit_view_prepare. Deux
cas réels. La correction laisse reprendre leur conservation native. eb1b8fc3
est désormais cancelled/cleanup_done=true, started_atnull, remote_preparedfalse.

La préparation suivante d4517e04 a rencontré un vrai manque d’espace :
137318400octets libres sur le volume racine deKS. Annulation native confirmée,
cleanup_done=true, aucun démarrage métier. Observation privilégiée en lecture
seule de tous les exécutables, descripteurs et mappings de processus ; contrôle
des références d’unités.28copies anciennes intégrées/inutilisées retirées,
1202173712octets logiques. Actif et retour conservés. Disponible après :
1339351040octets. Aucune donnée Opus2D, preuve, tâche ou vue supprimée.

Revue finale distincte0edbc32d-1dc1-4e3a-a626-9d0100c04c5d, même source,
lecture seule et axes libres, admise après correction physique et clôture des
précédentes. Sa copie privée est publiée, aucun ancien essai rejoué.

## Cartes, compatibilité et contrôles ciblés

Carte publique8Mio→256Mio ; elle peut répéter les chemins d’un manifeste64Mio.
Référence close : lecture en flux du seul manifeste, autres listes ignorées.
Le contrôle d’espace compte aussi la carte. Nouveau refus mesuré NoticeBytes.
Formats requis worker.view.reservation.v2 et workspace.commit_view.notice.v2.

Retour234-view-capacity-reader-r0 : source289e84675c27cba3e7480e45b0cc7fb255da1737,
capacité/cartes corrigées, ancien budget Git et ancien prédicat pending conservés.
Sa compatibilité de lecture est vérifiée dans les deux sens. Le retour reste
conservateur pour les copies annulées non publiées ; il ne contient pas le
correctif de suivi aa3. Les anciens232 sont refusés comme lecteurs de retour.
Aucun retour réel ni réécriture de tâche pour tester.

Contrôles distincts utiles :8tests234,10régressions Git,3références closes,
1régression SQL pending et1lecture administrative de la base réelleDCD.
Les8tests234 et le pending passent aussi surLinux ; pas de double comptage.
Carte synthétique30000entrées/14791089octets relue, manifeste identique,
champ manifeste doublé refusé. Récupération : worker actif, témoin absent,
identité différente, bootstrap inconnu, marqueur vivant et reçu malformé refusés.

Premiers contrôlesLinux utilisaient le Git système trop ancien ; les4fixtures
concernées passent avec le Git2.56.0 du worker. Premiers contrôlesMac utilisaient
un TMPDIR trop long pour la socketUnix ; parcours acquis avec /private/tmp.
Ces erreurs de banc ne sont pas déclarées succès et n’ont affaibli aucune garde.

## Activation vérifiée

| Machine | Composant | PID |
|---|---|---|
| Cartae | maître |2167858|
| Cartae | mobile |2167919|
| Cartae | worker |2170434|
| SYS | worker |2069757|
| KS | worker |3077075|
| Mac | worker |99589|
| Mac | pont natif |99776|

SHA Linux :e9413ca540b3ee257feebe8d16941afb7e6ea76f812ff43981b6d517ff3a6909.
SHA Mac :1398d138271f68931113aeeb636459986f6c763832fbcec4b8dc6b7e37a1f2da.
Coupure maître2,19s. Superviseurs Cartae1978546/birth337233333 et
2500061/birth270585460 strictement conservés. Aucun superviseur actif SYS/KS/Mac
à cette bascule. Registre Mac dédié inchangé, SHA
bdee09c363292d1be730eecffad779a942cd75a0d45b69725472867ff3f3587f.
Quatre pointeurs rollback écrits par l’écrivain atomique natif sous verrou,
sauvegardes et redémarrage du seul watcher. Les quatre rapports0.8.2 sont frais.
Les métadonnées de compatibilité des clients peuvent rester en cache ; elles
ne servent pas à déduire l’image réellement chargée, contrôlée séparément.

Payloads intégraux7024 et77ba inchangés après activation. Canonique des six
tâches protégées inchangé ; la progression de leur propriétaire reste distincte.
18préavis envoyés, reçus in_flight conservés sans lecture inventée.

## Client, publication et avis

Fermeture/réouverture natives de la même application231, aucun remplacement.
Version0.1.7-local.20261103.attestation-r1, SHA app.asar
81a55cefac78c12208e99a53f6f5008bcc484b571bd4e45809a81a1c5eef6ff0.
PID application681→87901 ; backend même exécutable913 exclu de l’arrêt ciblé.
21anciennes façades MCP arrêtées ;4façades présentes après reprise vérifiées
234-r3/SHA1398. Preuve nativeV2/permissions provider_turn de ce même fil, puis
who same_project réussi, projetBridget nonnul/bridge1016. Déclencheur ponctuel
retiré après sortie0. Aucune réattestation ou relance en boucle.

Release publique https://github.com/guthubrx/bridget/releases/tag/v0.8.2,
id434847045, cibleaa3. Sixactifs, tailles et SHA API vérifiés. APK8002, SHA
 e8086ae5d663a8899b1eb2a8fe72a7173f7a9d950dc05c1b73b3e12b368e8346,
certificat production0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588.
Lignée de signature et Firebase conservés. Aucun effet graphique dans ce lot,
aucune recette Android ni diaporama revendiqués ; réserve230 non effacée.

Diagnostic propriétairecb896 envoyé avec avis de reprise :
mcp-92739-6aec2a48-1, delivery1cc755b4-fd16-4168-aa99-3cfb2fc686cd,
in_flight au reçu, sans lecture inventée.17autres avis de reprise envoyés aux
agents prévenus et toujours présents. Aucun rejeu de la clé orpheline antérieure.

## Git, nettoyage et conservations

Les sources exécutables aa3 et ce bilan sont intégrés àmain et poussés.
L’alignement des quatre hôtes conserve les racines historiques détachées deSYS/KS.
Branche de développement234 retirée après intégration. Branche du lecteur de
retour234R0 conservée, source publiée et encore utile ; ne pas fusionner ses
retours au comportement Git/pending antérieur dans l’actifR3.

28copies obsolètes surKS,15surMac et2transfertsCartae retirés, après contrôle
complet des processus et usages.930fichiers de source/build Android temporaires
retirés :62sources identiques à leur archive et868produits générés, aucun usage
ou processus inconnu. Total1989224619octets logiques retirés.
Worktree du lecteur Mac propre, sans ignoré, conservé : sa lecture native des
processus reste incomplète. Aucun usage actif n’est déduit de ce seul manque
de preuve. Sa branche et son binaire de retour restent aussi conservés. Sauvegardes, journaux, reçus et paquets
publiés restent hors de ce nettoyage.

SYS conserve ses deux copies de transfert(88380168octets), les deux worktrees
propres de compilation234 et le cache partagé5,3Gio : le lecteur privilégié
natif sudo-n/lsof exige un mot de passe, donc l’observation est incomplète.
Aucune élévation de droits ni suppression forcée. Entretien installed appelé
surMac(succès, cache7,04Gio sous budget10Gio) etSYS(refus conservateur). Les
intermédiaires du checkout sale227 antérieur, vues de revue et anciennes images
hors namespace qualifié restent conservés. Leur nettoyage suit une observation
complète ultérieure, sans terminer le travail d’un autre responsable.

Preuves privées : /Users/moi/.cache/bridget234-inventory-timeout/.

## Revue indépendante et traitement des quatre P3

Revue0edbc32d : APPROVE avec réserves, aucunP1/P2, quatreP3. Résultat lu et
ACK natif conservé. Lecture statique bornée : aucun test indépendant, runtime,
protocole complet ou helper job revérifié ; limites intégrales dans revue.md.

P3-1 : mesure du vrai index complet SYS avec le Git du worker :27063entrées,
2104527octets,8ms, HEAD inchangé. Il couvre plus d’entrées que les22519fichiers
sélectionnés au plan DCD ; le worktree incident n’existe pas. Le budget d’audit
10s est conservé pour ce lot, marge mesurée1250fois sur cet index. Cette mesure
ne garantit pas toute charge future ; le journal nommera index_inventory si
cet audit dépasse sa borne. Aucun problème d’audit constaté sur ce parcours.

P3-2 : l’inscription ne conserve pas la cause historique de quarantaine. Le
retour cause=legacy_view_reservation_limit nomme le témoin de capacité contrôlé,
pas une preuve historique exclusive. La récupération exige aussi que toutes
les autres gardes actuelles passent ; aucun défaut restant connu n’est ignoré.
Dans l’incidentKS, les journaux et le reçu illisible par232 prouvent le défaut.

P3-3 : sauvegarde native .before-view-capacity-UUID.json conservée comme preuve,
avec l’image complète antérieure et les reçus de récupération. Aucun nettoyage
de preuve lors de cette livraison.

P3-4 : le chemin incident entre dans GitTask234 à l’intérieur de create, avant
compact::plan ; capture/run_git et les stages sont synchrones sur ce même thread.
Le thread de préparation est créé avant l’entrée dans create. Aucun transfert
asynchrone du contexte après la garde sur ce parcours. Les appels hors create
restent explicitement task_idnull ; aucune attribution inventée.
