# Quickstart 162 — recettes après implémentation

Ce document décrit les validations à réaliser. Il ne prouve aucun lancement, test ou déploiement 162 déjà exécuté.

## Préflight

Relever versions, PID et naissances de Bridget et T3. Vérifier les sauvegardes privées et le registre. Avant activation Bridget, inventorier les missions vivantes et vérifier leur reprise sans perte ni duplication. Ne pas forcer un arrêt dans une situation inconnue. L’activation Bridget 162 est autorisée par le pipeline ; T3 reste sans relance.

Le choix utilisateur autorise Bridget, Roadie et Opus2D ; Cartae est exclu. Vérifier racines, identité Git, commit de base et modifications à préserver. Le dépôt Cartae distant observé comporte une modification ; ne pas la supprimer. Contrôler les chevauchements canoniques avec credential, répertoire privé et état worker à Prepare puis RELEASE. Tester les deux sens du chevauchement et une résolution impossible. Les modes privés ne constituent pas une isolation entre fournisseurs du même UID ; ne pas prétendre confiner full-access. Vérifier le fournisseur réel et son authentification, pas seulement la présence de son binaire. Provisionner les credentials privés sans les afficher. Routes désactivées par défaut ; paramètres manquants : tests isolés seulement.

## Validations prévues

1. Étendre registre, protocole et permissions : absence de machines, refus de route, identité invalide, double connexion, versions incompatibles, traversal, mauvais projet, parent déjà distant, règle opaque, restrictions Codex et sources Claude Linux. Tester une source apparue après préparation, un fragment changé, les sources de l’autre OS ignorées et une lecture interdite.
2. Exécuter les frontières du contrat worker : reçu avant bootstrap, marqueur avant ready, permis unique, stockage saturé ou corrompu, PID recyclé et nouvelle époque. Compter créations, RELEASE et complétions durables. Vérifier writer unique par fichier, blob immutable, budget sérialisé, temporaires comptés, publication pendant relance worker et collecte après fin du producteur.
3. Intégration native : placement local et distant explicites, isolation 154, journal, résultat, demandes 155, annulation et coupure. Vérifier la réémission après résultat durable mais avant publication, puis après persistance maître mais avant confirmation. Un parent absent retrouve son résultat. Couvrir accepted, discarded_terminal, rejected_conflict et deferred ; quota_hold est différé puis accepté ou définitivement classé après preuve. Vérifier le relais après mort wrapper sans fausse connexion enfant. Send et SendIdempotent d’une mission distante courante ou passée doivent produire un Nack sans mutation ni dispatch. Vérifier reserve_result_slot avant RELEASE, ronde et reprise quota ; saturation ou refus ne crée ni dispatch ni UUID.
4. Reprises 152/160 : nouvelle génération seulement après preuve terminale ; quota conserve instance, placement et échéance. Aucun changement d’hôte après coupure.
5. T3 : tests ciblés 149 des contrats, reader, orchestrateur et composants. Fixture v1 gelée, ancien décodeur strict, v2, repli autorisé, abandon de toutes les pages v2, contact worker distinct du maître. Fermeture et reprise sans lancement fournisseur. Aucun navigateur ou contrôle graphique sans accord exprès.
6. Recette réelle Linux sur chaque projet autorisé avec mission synthétique bornée. Relever machine, cwd, moteur, résultat, unicité du lancement et arrêt attesté. Distinguer résultat de mission et intégration Git. Le journal perdu avec le contact est signalé ; le résultat reçu reste au maître.

Cargo : jobs=2, nice=10, threads de test=2 et une compilation active. Au plus une suite Rust finale si requise ; ensuite seulement les replays affectés par une correction. T3 : suites ciblées uniquement, pas de suite globale. Conserver commandes, RC, durées, sources et limites. Aucun succès global déduit de tests partiels.

## Livraison

ROOT gère commit, intégration et push. Bridget vers github/main. T3 vers fork/local/main-20261009, depuis /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant ; aucune PR ni push upstream. Vérifier les références distantes, construire, signer et installer selon les procédures existantes.

Activer Bridget et le worker après sauvegarde, recettes et contrôle des missions actives. Installer l’unité utilisateur /home/moi/.config/systemd/user/bridget-worker-cartae.service, credential /home/moi/.config/bridget/worker-cartae.credential en 0600, données /home/moi/.local/share/bridget-worker/cartae en 0700. Le tunnel 095 reste sur le Mac. KillMode=process, SIGTERM et SendSIGKILL=no : la relance du worker ne tue pas arbitrairement ses wrappers. Reconnexion : même credential remplace la connexion fantôme, ancienne époque refusée ; aucun permis restauré. Arrêter le worker reste distinct d’annuler ses missions. T3 : installation sur disque seulement, ancien runtime conservé. Les sorties v1 doivent rester compatibles. Le reçu distingue preuves Git, fichiers, runtime Bridget et rendu T3 isolé.

## Nettoyage et rollback

Archiver les preuves avant retrait. Fleet passe au schéma 5 et les requêtes stockées sont strictes. Aucun downgrade de l’ancien binaire sur l’état 162 inchangé. Avant activation : sauvegarde cohérente DB/Fleet/registre/état worker après fermeture des admissions et quiescence attestée. Une restauration jointe n’est permise que sans perdre une activité postérieure ; sinon garder 162, désactiver la route et conserver l’état. Révoquer une route empêche les nouveaux lancements ; elle n’efface pas les reçus des exécutions actives ou ambiguës. Aucun nettoyage d’un worker perdu, d’un résultat non confirmé ou d’un commit non récupéré. Conserver les branches de tâche selon les instructions de leur mission jusqu’à intégration attestée.

Retirer seulement les deux worktrees 162 propres et intégrés, ainsi que leurs branches, sans force. Les 17 worktrees T3 préexistants restent hors périmètre. Nettoyer les caches avec les gardes du projet, conserver le rollback utile et documenter chaque conservation. Aucune base supprimée, aucun gain physique déduit d’une taille logique.


## Procédure opérationnelle préparée, NON EXÉCUTÉE

Le CLI worker est compilé et ses recettes isolées R3 passent. Son installation Linux et l’activation réelle restent à faire. La procédure canonique est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/livraison162.md. Elle produit ExecStart depuis le préfixe absolu /home/moi/.local/lib/bridget-communication/162-SHA12/bin/bridget, avec SHA complet et identités contrôlés ; aucune variable non développée dans l’unité systemd.

Avant toute bascule : lire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control status, fermer les admissions par control halt-delegations --reason livraison162 sans cancel-active, puis prouver la quiescence complète. La liste active limitée à 100 n’est pas une preuve exhaustive. Conserver l’état humain initial ; control resume-delegations ne doit pas rouvrir un arrêt humain préexistant ou postérieur.

Les stores maîtres observés sont /Users/moi/.cache/bridget-core/bridget.db, /Users/moi/.cache/bridget-core/bridget.fleet.json et /Users/moi/.cache/bridget-core/agents.json. Après arrêt attesté de leurs writers, sauvegarder SQLite et copier Fleet/registre dans une racine privée neuve, avec l’état worker cohérent ou sa preuve d’absence. Vérifier les copies et le manifeste joint avant remplacement. Les chemins et la quiescence doivent être réattestés au moment réel, pas déduits du préflight.

Les candidats privés restent désactivés dans /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate. Le registre worker prévu est /home/moi/.config/bridget/agents.json ; sa socket fédérée /home/moi/.cache/bridget-core/bridget.sock reste à réattester. Ne pas confondre ce registre Linux avec le registre maître sous bridget-core. Les secrets restent privés. L’authentification Codex observée ne vaut pas preuve de quota ni appel réel de modèle.

La rétention temporaire du cache utile 162/163 est fixée à 45 Gio, pas une cible de consommation. La FULL Rust unique est terminée avec RC101 ; seuls les replays affectés suivent les correctifs. Aucun deuxième passage global prévu. T3 reste sans relance.


Développement et préparation atteignent 42/42 après les replays ciblés et la lecture finale. Les 122 échecs de la FULL historique ont un rejeu PASS ; aucun GREEN global ou baseline ancienne n’est déduit. Le prochain passage est Converge, puis les autres gates et G8/G9. Les commandes de livraison ci-dessus restent NON EXÉCUTÉES ; T3 n’est pas relancé et les routes privées restent désactivées.

Checkpoint courant : Converge1 est clos non convergé sur une trace de remplacement worker manquante (T008). ROOT a ajouté T043, correctif et validation encore attendus. Le développement repasse In Progress, 42/43 ; le checkpoint 42/42 précédent est historique. Aucun changement aux gates G8/G9 ni activation. Attendre la preuve et le GO ROOT avant clôture 43/43.

État courant après GO ROOT T043 : trace worker corrigée et oracle ciblé réussi, développement/préparation Implemented, 43/43. Converge1 reste historiquement non convergé ; Converge2 ciblé doit encore statuer. Les gates G8/G9 restent NON EXÉCUTÉS ; aucune activation, installation ou livraison finale déduite de 43/43.

État courant : contre-revue code R1 BLOCKED, développement rouvert à 43/44 par T044 (réconciliation starting/refus durable et E2E sans état terminal injecté). Converge2 reste historique ; aucun passage 3 encore, audit pré-R1 suspendu. Attendre correction et validations avant reprise des gates. Aucun changement de l’autorisation des projets, aucune installation ni activation.
