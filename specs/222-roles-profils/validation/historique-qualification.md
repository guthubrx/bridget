# Validation222 — In Progress
Dernière mise à jour : 2026-11-01. Aucun commit ni chargement222 annoncé.

## Contrôles acquis
- Analyse des artefacts : analyze.md ; pas de finding CRITICAL conservé.
- Revues de plan R1 puis R2 : objections retenues corrigées, limites conservées.
- cargo test -p bridget-daemon --lib spec222 via scripts/build.py : dernier groupe
  9 PASS, 0 échec (domain-test7.log). Profil/grant, versions, oubli, réserve prepared,
  garde RELEASE, réponse humaine corrélée, contact sans durabilité, périmètre,
  budget hérité et composition partielle sans doublon.
- Compilations intermédiaires library daemon PASS. Ne valent pas qualification
  de binaire installé, de client ou de services.
- Android : compilation Kotlin/debug passée jusqu'aux tests ; 8/9 premiers tests
  de modèle PASS. Le dernier test signalait la liste d'états finaux du client :
  correction ciblée conservée, nouveau résultat attendu. Aucun APK livré.

## Défauts réellement trouvés et corrigés pendant les raccords
- BEGIN IMMEDIATE imbriqué dans un savepoint : boucle d'identité extraite et
  réutilisée ; test d'activation passé.
- Remise humaine avec réponse : la garde native exigeait un agent expéditeur.
  Réemploi de tracked_requests/journal pour cette voie humaine attestée, sans
  faux agent. Le contrôle exact du message s'effectue avant le transport.
- L'observation réelle du test a montré hops réservé4 / transporté3 ; le rappel
  en mémoire ne refait plus le contrôle d'enveloppe après ce changement attendu.
- Le reçu déclaré soumis masquait un refus : retour maintenant fondé sur le
  résultat natif. Une réservation accepted n'est pas déclarée remise.

## Travail Android distinct
Tâche e9ce2aba-02a9-4c3b-b824-615b03220d73, Claude/Sonnet KS. Résultat lu,
ACK et clôture demandés une fois. Delta contre snapshot appliqué au worktree
principal222 ; corrections de contrat et vérification conservées par le parent.
L'auteur n'avait ni JDK ni SDK ; son avis ne prouvait donc aucune compilation.

## Encore non attesté
Transport fermé nouveau groupe, couverture HTTP/CLI réelle, capture paginée en
fonctionnement, contre-revue d'implémentation, Converge et audit final ; builds
Mac/Linux/Android finaux, Git, activation et nettoyage ; circuit UI et preuves.
Aucune mission métier ni incident historique221 rejoué.


## Raccords après revue R1
Groupe daemon222 : 12 tests distincts PASS, dont portée des déclarations limitée,
projection non propriétaire sans grant/owner et new_grant sans panique.
Premier lancement temporaire : un test ciblé PASS, prompt/canon figés,
premier emploi réservé sans processus et réemploi sous autre clé refusé.
Android : neuf tests de modèle PASS et assembleDebug PASS. Ajouts ultérieurs
de rôle officiel/historique/demandes natives : compilation Android PASS séparée.
Émulateur API35 ouvert. Les deux commandes agent-device ont quitté avec130 sans
message ; fallback adb. Installation du paquet ordinaire refusée pour certificat
différent, aucune désinstallation. Paquet de vérification updatefixture installé
en parallèle, application initiale et données conservées. Écran de connexion réel
observé ; aucun circuit authentifié du catalogue encore déclaré vérifié.


## Source après intégration amont
Main91083f5d intégré par autostash, modifications222 conservées. Trois conflits d'ajout documentaire, les deux côtés conservés ; aucun conflit fonctionnel. Premier check intégré a montré l'import Sha256 disparu dans lifecycle ; import local ajouté à la garde. Le test du plancher222 passe. Le test de portée humaine restreinte passe. Les trois échecs antérieurs restent dans les journaux, pas annoncés PASS. RevueR2 d51b1c2c réservée surSYS, starting ; complément non remis unknown_recipient, aucun rejeu.

## Contrôle final de source
Audit V15 calculé puis validé : pass, release_blocked=false sur FR001–030.13daemon222+1transport222+1autorité181 PASS. Les échecs du cadre V15 sont conservés : plafond de processus, graphe incrémental de256Mio tronqué, TMPDIR trop long ; dernier contrôle un job/sans incrémental/debug et TMPDIR privé court. Aucune assertion fonctionnelle retirée. Le N+1 de sélection a été remplacé par un JOIN et contrôlé.
Assemblage Android0.7.0 signé avec le certificat existant, SHA fc6de337a135eb707e26fa5879ea0493043eebe4ee13984ea7ef15deec62557d, installé par mise à jour sur l'émulateur sans suppression des données. Aucun circuit authentifié encore annoncé.
CLI réel sur namespace isolé : liste vide, proposition, approbation, versions, export et import ; deux définitions conservées, zéro spécialiste activé. Daemon de qualification arrêté proprement, PID1179681 identifié avant TERM et absence vérifiée après3s.
Assemblages Mac r2/reader-r2 acquis. Image Linux portable construite surSYS GLIBC2.36 ; l'image native Cartae exigeGLIBC2.39 et ne sera pas distribuée surSYS. Image active portable en cours d'assemblage, aucune activation222 encore.
