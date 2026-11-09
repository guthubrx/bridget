# Validation 254 — activée et publiée

9 novembre 2026. Version finale 0.8.21 / 254-local-preparation-r3.
Source exécutable : 695921e76ed3a4611a2a4b4dfbf0411ae2f6b6ae.
La version intermédiaire 0.8.20/R2 reste publiée avec ses preuves propres.
T3 est resté ouvert. Aucune mission métier n’a été rejouée ou remplacée.

## Causes et corrections

Dossier documentaire local hors Git : le relais refusait origine=cible hors
commit_view. Il accepte désormais general, shared explicite ou implicite, sans
SHA. Cette forme utilise le helper local existant. Aucun tunnel vers soi-même,
aucune copie et aucun Git ne sont créés. Les autres formes locales non prises
en charge (Git, worktree, compilation) restent refusées.

Un second défaut a été constaté avant clôture : le profil générique Mac et le
registre du worker portent des identités différentes. R3 fournit au helper un
fait interne calculé par le worker. Le helper utilise alors son registre attesté
et vérifie activation, worker, révision et empreinte. Le profil générique reste
intact. Aucun paramètre public ne choisit ce fait. Aucun registre de production
n’a été modifié pour la correction.

Projet natif SYS : le nom système backup diffère du placement sysle2. Le projet
utilise maintenant le worker du placement et la commande native exacte. Même
si le nom déclaré égale celui du maître, une tâche native ne retombe pas sur une
lecture locale du maître. Commande, instance, cwd, révocations et état sont
vérifiés avant et après les E/S.

Combinaison commit_view/development : le refus local est correct. La condition
du schéma existait dans $defs, référencée par $ref. La description MCP précise
aussi discovery et expected_base_commit. Aucun validateur n’est assoupli. Un
vrai processus MCP neuf vérifie le texte et la condition référencée.

## Vérifications

- Neuf contrôles distincts R1/R2 passent sur Mac et Linux. Les ensembles
  successifs se recouvrent et ne sont pas additionnés.
- R3 renforce le test complet existant : handler worker → helper Python réel,
  HOME privé, deux registres concurrents, succès local et profil inchangé.
  Test acquis Mac/Linux, sans modèle, Git ni SSH.
- Preuve Python privée complémentaire : sans fait attesté, le profil concurrent
  est refusé project_preparation_machine_mismatch ; avec le fait attesté, le
  bon registre est choisi. Zéro écriture de registre en production.
- Builds Mac/Linux R3 réussis via scripts/build.py et caches partagés.
- APK8021 construit en46s, signé, certificat et lignée8020 vérifiés.
  Aucun changement d’interface Android, pas de nouveau diaporama nécessaire.
- Compatibilités forward/return/inverse qualifiées sur les binaires finaux ;
  ancien lecteur245 refusé. Versions des clients installés contrôlées sans
  authentification ni appel de modèle.

Un ancien test mobile négatif attendait un refus de chemin extérieur sous
dangerFullAccess. Il contredisait les sources déjà admises. Il utilise maintenant
un chemin relatif réellement interdit. Les permissions ne changent pas.

## Revues et dispositions

Trois revues indépendantes lues et acquittées. R1 conduit à fermer le repli
local en cas de collision de hostname. R2 ne confirme aucun P1/P2. R3 ne confirme
aucun P1 et propose deux remarques classées P2, plus un P3 :

- Le cas sans drapeau est reconnu inatteignable dans le flux worker corrigé.
  Le découvreur historique reste nécessaire aux autres parcours. Une exigence
  générale du drapeau casserait ces appels. La preuve Python vérifie aussi que
  le profil concurrent sans drapeau est refusé, sans faux succès.
- Le relecteur n’a pas relu le test annoncé. Le test complet Mac/Linux et la
  preuve Python complémentaire sont acquis. Aucun test indépendant n’est inventé.
- Deux appels du prédicat et booléen faux explicite : remarques de style acceptées.

La conservation de la revue R3, d’abord inconnue, revient kept/review_required
après activation, dernier contrôle sans divergence. Vue conservée, aucun retrait.

## Activation et préservation

Sept composants R3 chargés sur quatre hôtes : core/mobile/worker Cartae,
workers SYS/KS, worker/connecteur Mac. Empreintes des exécutables chargés et
pointeurs rollback vérifiés. Outils de mise à jour relancés nativement après
écriture atomique sous verrou, sans activation concurrente.

Mac SHA : 8d738d8bde991603033c23b9ddead6c1e56254fab49273d1ad44678f6c949ec8.
Linux SHA : 19bfa475d21070cb60f9cf4959848e2c787f1c917d99e8172f54b850d158a09b.
Retour final254R2, source271d536bc4db8c691dd0be4a2df54054f17e8db9, compatible.
Coupure maître1,54s enR2 puis1,58s enR3. Aux passagesR3, deux wrappersMac,
deuxCartae et unSYS ont leurs PID et identités de démarrage conservés.
Onze processusT3 et leurs dates de démarrage inchangés, septpayloads historiques
protégés et registreMac inchangés. SHA registre :
60acca93483698c25d5b1aca4b52af20cd3ce24889b6ecf21ade87e2367daebe.

Preuve sur mission existante : agent3846 busy, même tâchef550356e-c3e7-4c55-88ef-b2b7d477c598,
même instancefe4a0fa4-c7e6-4746-a427-f4d266e0031c, wrapperSYSPID3016909 conservé.
Hostnamebackup conservé ; projet attesté parsysle2 :
/home/moi/.local/share/bridget-projects/auto-54c0347616de987c5493e28c7690a30a/repository/.git.
ParentParix busy/projetattesté. Aucun paquet métier déplacé.
Le wrapperSYS utilise encore l’image252, conservée. Son projet est corrigé côté
serveur sans arrêter le modèle. Les anciennes façadesMCP restent ouvertes :
le nouveau catalogue est vérifié sur MCP neuf, pas annoncé partout chargé.

## Publication, diagnostics et nettoyage

Release publique v0.8.21, six actifs téléchargés anonymement et SHA conformes.
APK8021 : 3967548octets, SHA308f8c599abe53c03ee3cad48a20d13dcdbd38691e7304b1c4ca0049daea4761.
Source intégrée et poussée surmain ; clôture documentaire alignée sur quatrehôtes.

DiagnosticsR2 envoyés àCDS, parentParix et3846. CDS/3846 acquittés par le transport,
sans lecture métier affirmée. ComplémentR3 utile envoyé àCDS :
mcp-56788-6af1aef7-c, delivery705df6b1-c9f3-46d5-9b05-b2b7e5fe66da, initialementin_flight.
Aucun broadcast ni nouvelle mission de qualification.

Deux propres worktreesMac254 et branches locales/distantes retirés après
intégration, contrôle des ignorés et absence d’usage. ImageR1 jamais activée
retirée après contrôle. Copies de transfert et téléchargements de vérification
R2/R3 retirés ; reçus, journaux, actifs publiés et APK signés conservés.

WorktreesbuildSYS/AndroidCartae propres et intégrés mais conservés :
cinq processusSYS et quaranteCartae illisibles au dernier contrôleR3.
Aucun processus étranger arrêté pour le ménage. ActifR3, retourR2 et anciennes
images utilisées ou sous observation incomplète conservés.
Derniers caches : Mac0,53Gio, SYS3,83Gio, Cartae4,46Gio, entretien commun appliqué.

Limite : aucune admission documentaire métier relancée pour créer une preuve.
Le flux complet est vérifié avec une fixture privée ; la prochaine demande
réelle conserve sa propre décision et ses propres reçus.
PreuvesR2 : /Users/moi/.cache/bridget254-preparation-locale/.
PreuvesR3 : /Users/moi/.cache/bridget254-registre-local/.
