# Validation 223 — 1 novembre 2026
## Cause et correction active
Release GitHub v0.7.1 : ancien APK 0.7.0/7000, SHA256
fc6de337a135eb707e26fa5879ea0493043eebe4ee13984ea7ef15deec62557d.
Le refus d'intégrité est reproduit par le circuit réel sur Android 15.

APK reconstruit depuis bcc40a1fd04131eead16d09feaa3f0fd16b722bf, version Cargo
0.7.1, code7001. Build release Gradle Cartae PASS (57s, incrémental). Configuration
Firebase publique existante réutilisée ; les quatre options sont présentes dans le
DEX. L'ancien APK0.7.0 ne les contenait pas : ne pas présenter leur présence comme
un test de réception FCM. Notifications désactivées pendant la recette.

Signature V3, rotation dès API28 et même lignée historique avec capacité permission.
Certificat SHA256 :
0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588.
Aucune clé générée ni contrôle d'installation affaibli.
APK public corrigé : 3951164 octets, SHA256
4e0e6b47b9da8f72e4dbe0877150cab36918a708945e3c99cc60e6ad6237e538.
Seuls APK/checksum de v0.7.1 remplacés. Quatre actifs serveurs strictement identiques.
API GitHub, checksum et téléchargement public concordants.
La commande --verify-published relit les deux digests et tailles : PASS.

## Parcours réel
Émulateur Bridget_API35, Android15, application officielle release0.7.0.
Réglages → Vérifier les mises à jour → Télécharger → permission Android
d'installer depuis Bridget → refus initial attendu. Après correction publique :
Vérifier les mises à jour → Télécharger → confirmation Android Update.
Premier essai interrompu par réouverture trop tôt pendant Play Protect :
INSTALL_FAILED_VERIFICATION_FAILURE, pas une erreur de signature ; nouvelle
confirmation système du même APK réussie sans désactiver Play Protect.
Réouverture : Bridget0.7.1 et « Cette version est à jour ».
Dumpsys confirme code7001, version0.7.1 ; firstInstallTime inchangé
2026-10-25 13:21:44, lastUpdateTime2026-11-01 13:25:33.
Adresse serveur, notifications désactivées et mises à jour automatiques conservées.
Aucune désinstallation, aucun effacement. Le téléphone physique de François
n'est pas connecté. L'autorisation serveur de cet émulateur était déjà révoquée :
la recette ne prouve pas une nouvelle association ni un appel serveur authentifié.

## Contrôles du garde
Six tests unittest PASS : versions, mauvais package/version/code/certificat,
référence avec certificat non autorisé, apksigner en échec ou signataire absent,
publication partielle refusée. Ancien APK réel refusé pour v0.7.1 ; nouveau APK
réel validé. Contrôle public final PASS. Pas de campagne générale ni build Rust.

## Contre-revue
Mission0458b4aa-5b4b-4c06-bd54-49f3e27208b8, enfant2dfd87c5, Claude Haiku,
clébridget223-publication-android-review-20261101-01 : APPROVE_WITH_CHANGES,
aucunP1. Résultat lu/ACK/clôture une fois.
P2upload : retenu, budget600s et relecture obligatoire des deux actifs ; toute
issue d'envoi non confirmée l'annonce explicitement. API brute utilisée car l'ancien
gh release view de Cartae omet digest. Aucune seconde publication pour ce contrôle.
P2référence : risque de mauvaise référence retenu, certificat production ancré
dans le garde. La proposition de refuser un APK identique est rejetée : elle ne
prouve pas l'origine de la clé et interdit une republication idempotente valide.
Un certificat local différent reste refusé même si candidat et référence coïncident.
P3fences corrigées, test signataire absent ajouté. Delta relu et testé directement.

## Analyse et convergence
Analyse manuelle spec/plan/tasks/code/tests : aucune contradiction bloquante.
CONVERGED : version, signature, publication réelle, installation et conservation
ont chacune une preuve ci-dessus. Pas de changement de MobileUpdates.verifyApk.
Preuves HTML autonomes générées et contrôlées dans html_preview ; gestes natifs ADB,
pas de coordonnées DOM fabriquées. Captures/manifestes bruts hors Git :
/Users/moi/.cache/bridget-android071.

## Livraison Git et entretien
Correction68f47111 intégrée en avance rapide dans main, poussée sur origin GitHub ;
main Mac et Cartae synchronisés et propres. Aucun binaire Rust ni service remplacé :
la correction active est l'APK0.7.1 public, réellement installé sur l'émulateur.
Les pointeurs rollback des serveurs ne sont pas concernés et restent intacts.
APK0.7.0 officiel conservé hors Git comme référence de certificat et preuve.

Entretien examiné : worktree223 et branche intégrés mais conservés. lsof signale
une observation incomplète de montages sur Cartae ; aucun usage inconnu n'est supprimé.
Dossiers ignorés limités à Android .gradle/build et __pycache__. Pas de targetCargo
créé. Captures, manifestes et APK sont conservés dans le cache223 ; les autres
worktrees et binaires des sessions221/222 restent intacts.
