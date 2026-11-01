# Validation224 — 1 novembre2026
## Cause et réalisation
SYS/KS : workers actifs mais aucun update-host.json ni watcher. Ancien updater
limité au rôle worker=bridget-worker-cartae.service. Statut commun limité aux
rapports déjà reçus, donc deux machines invisibles.

Correction : noms d'unités Bridget fermés, setup local sous verrou existant,
inventaire maître ajouté en projection CLI/mobile, rapports absents/périmés visibles,
dates manquantes jamais converties en1970. Aucun nouveau store/protocole/droit.
Service arrêté ou non confirmé : aucune activation ni remise en route automatique.
Les composants Windows restent clients ; aucun hôte natif Windows ajouté.

## Contrôles
26tests Python PASS, dont unités propres, noms hostiles refusés, vrai verrou local,
setup idempotent sans démarrer le worker et service arrêté conservé.
Commande directe et discovery exécutées : placement des tests corrigé.
Test Rust spec224_federation_never_hides_missing_or_stale_reports PASS.
Deux branches utilisent le même projecteur : administrationCLI et mobile authentifié.
Assemblages Linux/Mac et APK release PASS. Revue native5f200d81, Claude Haiku :
APPROVE_WITH_CHANGES, aucunP1. Date1970 et placement des tests corrigés ;
P3absence de date et verrou/ordre setup traités. Dernier delta relu directement.
Aucune campagne générale ni nouvelle mission de qualification métier.

## Activation réelle
Version0.7.3. Sept composants chargés, PID/empreintes vérifiés :
Cartae core1740329/mobile1740369/worker1744343 ; SYSworker1429029 ;
KSworker1734471 ; Macworker27189/pont27377.
Linux commun SHA256 :
49a5e066a8ca9f4e6b50dbdb0ac813be071cf0b6a393311209af9c1564a02ed2.
Mac SHA256 :
5bd1e2dff81e9021ee04873d5e0a0c1cc3e1c9eecfef417bd21ca8d7b61ff6a0.

Premier assemblage Cartae refusé sur SYS avant activation : GLIBC2.38/2.39 absentes.
Paquet commun reconstruit sur Debian12/glibc2.36 puis lancé et qualifié sur les
trois serveurs. Aucun système ni programme fournisseur mis à jour.
Passage Cartae initial lancé avant fin de copie du paquet portable ; image correcte
du code mais différente du paquet commun. Second passage contrôlé pour aligner
l'image chargée. Coupures2,66s puis2,68s. Erreur de séquencement corrigée.
Superviseur métierCartae2500061, même naissance270585460, conservé.
Pas de mission ni résultat rejoué, annulé ou remplacé par cette livraison.

Watchers SYS1429043, KS1734487 actifs via setup natif ; Cartae et Mac actifs.
Mac : Python Homebrew3.15 échouait à importer plistlib (libexpat incompatible).
Python système3.9.6 vérifié ; PATH changé seulement pour le LaunchAgent updater,
configuration sauvegardée. Aucun changement du PATH global ni installation tierce.
WatcherMac33363 actif, version courante confirmée par rapport.

Statut commun réel : cartae/ksstore/monordinateur/sysle2, tous0.7.3,
stage up_to_date, report_freshness fresh. Pas d'absence interprétée comme succès.
Politique automatique existante conservée.
Retours distincts reader-r1 (portable sous Linux), formats aller/retour contrôlés,
pointeurs rollback atomiques sous verrou sur les quatre hôtes ; activation absente.
Lien natif revenu, session réattestée et rôle mainteneur-bridget relu inchangé.

## Android et limites de preuve
APK7003 signé par le certificat existant, même lignée permission et configuration
Firebase publique ; garde223 PASS. APK final SHA256 :
792434dbc724d32b02435868d78d650c370f1a8ee7b8d7e86bc80463fd468296.
Installation release sur émulateur Android15 par ADB -r, sans désinstallation,
paramètres conservés, libellé Fédération Bridget visible.
L'autorisation serveur de cet émulateur était déjà révoquée avant le lot :
liste authentifiée Android non requalifiée ; projection commune prouvée viaCLI et
test Rust. Téléphone physique non connecté. Ne pas prétendre ce circuit vérifié.
Captures/manifestes/JSON originaux hors Git : /Users/moi/.cache/bridget224.
Diaporama autonome inclus avec la correction ; html_preview vérifié.

## Analyse / convergence / livraison
Analyse manuelle spec/plan/tasks/code : CONVERGED pour les corrections du lot.
Publication publique, push et entretien consignés ci-dessous.
Aucune prétention de terminer le programme221 complet.

## Reçu final
Source9653b21d intégrée/poussée sur main. Releasev0.7.3 publique, six actifs
dont taille/empreinte correspondent aux fichiers contrôlés. APK redéchargé depuis
GitHub : SHA792434dbc724d32b02435868d78d650c370f1a8ee7b8d7e86bc80463fd468296.
Le premier contrôle du brouillon par tag retournait404 après upload réussi.
Aucun nouvel upload : relecture des six actifs, puis garde étendu à la recherche
bornée du brouillon exact. Sept tests du garde PASS ; un brouillon vérifié ne
prétend plus être public. Contrôle après publication : assets_verified et
public_verified vrais. Ce delta porte sur l'outil de publication, sans nouveau
binaire ni redémarrage.
Une demande de vérification commune a été émise par updates check après
publication ; politique automatique conservée. Les quatre rapports sont récents.
Rôle mainteneur-bridget et périmètre bridget relus après retour du lien natif.

Entretien : treize archives de transfert retirées sur les quatre hôtes,
435184505 octets logiques. Actifs, retours compatibles, captures, manifestes et
sauvegardes restent conservés. installed Cartae terminé ; Mac et SYS refusent le
nettoyage sous observation des processus incomplète, sans forçage.
Premier appel installed SYS visait l'actif alors que le dernier assemblage était
le lecteur de retour : argument corrigé, empreinte égale puis garde d'observation
incomplète conservée. Worktree224/branche intégrés conservés pour ce même motif,
avec leurs fichiers ignorés de compilation. Aucun travail étranger supprimé.
Statut : Livré pour le lot224. Programme221 indépendant non déclaré terminé.
