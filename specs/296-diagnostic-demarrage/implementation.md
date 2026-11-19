# Diagnostic des démarrages silencieux

Incident documentaire du 16 novembre 2026, vérifié le 17 : quinze journaux du worker Mac contiennent uniquement le début du tour, l’écriture locale de consigne puis une échéance (onze cas) ou une interruption propriétaire (quatre cas). Aucun texte fournisseur ni acte de lecture n’est enregistré. Les stderr sont vides. La cause historique reste inconnue : le pipe écrit ne prouve ni lecture par le client ni requête acceptée par le fournisseur.

Le transport partagé journalise désormais la première trame JSON observée pendant le tour, sans recopier son contenu. Une trame n’est pas une preuve de travail métier. Après trente secondes sans trame, un diagnostic unique explique l’attente. Il ne change ni le délai de la mission, ni son état, ni les droits et ne relance rien. À l’échéance, le suffixe fermé `provider_output_unobserved` distingue cette absence de retour. Le statut conserve le code, la durée et le parcours de conservation, avec l’action de diagnostic utile. Coût constant par tour, deux écritures de diagnostic au plus ; pas de polling supplémentaire.

Deux reprises documentaires ont aussi un reçu worker `negotiation_failed`, sortie 1, tandis que le maître conserve `remote_start_failed`. Le stderr corrélé atteste `thread/start` expiré, pas sa cause fournisseur. La lecture de statut excluait les tâches failed de tout contrôle ExecutionStatus. Elle peut maintenant relire ce reçu pour le seul échec générique avant remise de mission, même après clôture logique. Propriétaire, instance, commande, ronde et époque worker restent vérifiés avant et après la lecture. Aucun Start, Continue, signal ou nouvelle admission n’est émis. Le diagnostic fermé du worker est projeté sous `startup_check`; un worker indisponible reste explicitement inconnu. Le mécanisme existant de réception d’observation peut conserver la cause fermée dans l’erreur durable. Aucun stderr arbitraire n’est publié et aucun format wire n’est ajouté. Le détail `thread/start` reste à lire dans le stderr corrélé, il n’est pas inféré depuis `negotiation_failed`.

Qualification en cours : transport silencieux réel simulé par un processus local, avertissement non terminal, annulation et canal fermé ; statut avec diagnostic, capacité absente et changement d’époque. Le premier test daemon a rencontré une socket temporaire macOS trop longue ; les tests daemon utilisent TMPDIR=/tmp. La fixture a été étendue aux tâches failed et son invariant de continuabilité adapté. Les deux tests transport 296 et le test statut étendu passent.

Revue indépendante demandée via le port natif. La première enveloppe a été refusée sans tâche car brief.patch n’appartient pas au contrat ; la seconde porte le diff dans la mission. Aucun document métier n’est relancé. T3 reste ouvert. Activation non attestée à ce stade ; le daemon observé est encore 288.

Revue native 8c5c1879-51e1-45d8-8b3d-305e09647d2c rendue, ACK et clôture propriétaire effectués. La remarque sur le refus d’identité a été corrigée : startup_check porte maintenant command_identity, et un changement pendant l’aller-retour rend observation_changed. Le verdict « forme nue impossible » est infirmé par les deux tâches réelles et par tick, qui choisit explicitement remote_start_failed lorsque Desired Stopped précède le diagnostic. Élargir à tous les préfixes remplacerait des causes déjà établies ; le filtre exact est conservé. Les tests n’étaient pas inclus dans le diff envoyé au relecteur : leur absence dans sa vue n’est pas une absence dans le lot. La première trame est seulement une observation pendant le tour, pas une corrélation métier. Les échéances de 30 secondes ou moins expirent sans avertissement préalable, avec leur diagnostic final.

Qualification finale : quatre tests daemon passent (lecture worker avec neuf situations dont garde terminale et identité absente, projection de durée, guidage fermé, diagnostic tardif) ; six tests du transport passent (silence, avertissement non terminal, fin de fond, interruption, délai explicite supérieur au défaut). Le parcours DeliveryRejected conserve le motif via bound_delivery_rejection dans wrapper ; le diagnostic de transport et sa projection native sont contrôlés séparément. Clippy strict bibliothèque/tests a passé avant la dernière correction de projection ; contrôle final et build de livraison en cours.

Le contrôle natif du maître Cartae `updates prepare` a été exécuté le 17 novembre : refus update_missions_active_or_unconfirmed, sans bail ni arrêt de service. Le statut réel du daemon demeure 288-preparation-guidage-r1. Ce refus doit être levé par la clôture native des états concernés ; aucune réécriture du store ni interruption de tâche n’a été faite pour le contourner. La revue distante reste kept/review_required ; sa vue ne sera pas effacée sans contrôle natif d’usage.

Clippy strict final bibliothèque et tests : succès. Les dix tests ciblés du lot passent. Le cache automatique a retiré 10,37 Gio d’intermédiaires devenus inutilisés après les tests ; aucune copie de runtime active ni fichier métier n’a été retiré. La compilation release Mac est engagée sous l’identifiant 296-diagnostic-demarrage-r1. Les résultats d’installation et d’activation sont consignés séparément après leur exécution réelle.

Qualification de livraison achevée le 17 novembre : release Mac et Linux Debian 12 compilées. Les mêmes six tests transport et quatre tests daemon passent sur Linux. Le binaire Linux réellement construit dans le cache externe exige glibc 2.34, compatible avec le plafond 2.36. Le premier contrôle ABI portait sur un ancien target local ; il a été refait sur le binaire neuf avant copie. Sources exécutables : commit 8ae0ff4f728168cbff40dc5e63eb9cb95ce8b896, poussé sur main.

Candidats 296-diagnostic-demarrage-r1 installés et hachages vérifiés sur monordinateur, sysle2 et cartae, sans changement des liens actifs. Formats aller et retour compatibles sur les trois machines ; contrôle des versions clientes Mac compatible. Aucun accès fournisseur métier testé. Les reçus locaux sont dans le dossier privé .cache/bridget296-diagnostic de chaque machine.

Activation incomplète : le maître actif est toujours 288. Le contrôle durable à 1794892916 recensait quatorze tâches bloquantes (six cancelled, une failed, une mission_pending, une queued, trois starting et deux working) et trois admissions chosen. Un état terminal logique ne suffit pas : sept tâches terminales restent dans le prédicat natif pending. Ces états demandent une vérification et une clôture par leur propriétaire, pas une suppression en base. Le refus natif a été conservé ; aucune garde ni mission métier n’a été forcée. T3 n’a pas été arrêté.

Ménage : aucun worktree ni branche créé. Le passage installed Mac a réussi. Sur SYS, installed a conservé les intermédiaires parce que l’inventaire du processus 109486, descripteur 7, était incomplet (process_record_unverifiable). Le processus était Bridget ; aucun arrêt ni nettoyage manuel n’a été tenté. Le cache mesuré par le dernier build restait à 4,15 Gio, sous le budget. La vue de revue et les images actives/retour restent conservées.

## Complément du 19 novembre : échec terminal sans référence

Signalement Parix `mcp-69813-6afe5ae2-7d`, tâche
`28e95960-4e88-4edc-ad54-e8301307cac9` : échec générique et machine non
visible dans le statut rapporté. La cause fournisseur de cette tâche n'est pas
établie ; l'espace disque déclaré ne permet pas de la déduire.

Défaut reproduit : `current_task_view` ajoutait `detail.provider_failure` pour
plusieurs erreurs précises mais excluait `terminal Codex: failed`. Le nouveau
test via le statut propriétaire échoue avant correction parce que le contexte
est absent. Le cas générique utilise maintenant la même projection autorisée,
avec machine distante ou hôte local, références d'exécution, ronde et mission.
La cause reste `unobserved`, sans code HTTP inventé. Le guidage demande le
diagnostic corrélé, conserve les fichiers et interdit une reprise implicite.
Aucun changement de routage, de lancement, de droits ou d'état durable.

Test ciblé vert après correction : propriétaire, refus étranger, projection
abrégée sans détail, placement local et distant, conservation du store. La
première comparaison de non-mutation utilisait l'objet avant normalisation de
`save` ; elle compare désormais le store avant et après, comme le test voisin.
Le nettoyage automatique a retiré 10,39 Gio d'intermédiaires après ce test.

État observé à 03 h 13 Paris : maître Cartae toujours
`288-preparation-guidage-r1`. Le statut natif des mises à jour donne Cartae,
ksstore et sysle2 en `waiting_idle` pour 0.8.53. Ce complément n'est pas inclus
dans les paquets 0.8.53 déjà publiés et ne doit pas être présenté comme actif.

Qualification du complément : test ciblé, Clippy strict bibliothèque/tests et
compilation release Mac réussis. Image `296-diagnostic-generique-r2` installée
hors staging sans changer le lien actif, SHA-256
`d4a677cc75e2d42cfd1137ec4aa0994f6d9de41aa5022970ea545b2d7b69eac5`.
La revue indépendante native `01e3c488-83e8-469d-9764-fecb76adec29`
(clé `bridget-296-generic-codex-review-20261119-r1`) n'a pas démarré :
`remote_start_failed:remote_workspace_prepare:WorkerUnavailable`, SYS.
Annulation propriétaire demandée ; reçu `cancelling`, conservation non confirmée.
Aucune revue réussie ni fin physique n'est prétendue, aucun remplacement lancé.
La livraison serveur reste incomplète : garde de maintenance, pas de bascule
forcée, aucun arrêt de T3. L'image Mac seule ne qualifie pas le binaire Linux.

Relecture finale : revue `cancelled`, conservation native `confirmed` ; aucun
rapport n'autorise la suppression de sa préparation. Après cette clôture, une
tentative unique `updates prepare` sur Cartae refuse encore
`update_missions_active_or_unconfirmed`, sans bail obtenu ni service arrêté.
Le correctif source est poussé dans `89453b87`. Aucun nouveau worktree ou
branche ; le passage `installed` garde 1,37 Gio de caches, sous le budget.
Les images actives, de retour et les preuves sont conservées.

## Complément du 19 novembre : catalogues ajoutés et gardes existantes

Le diagnostic natif SYS du 19 novembre vers 15 h 31 Paris confirme une
régression de configuration : la tâche 45e24100-69c9-467d-a821-f50b221aa703
(génération 3751) refuse `DefinitionChanged` à `definition_identity`, tandis
que 3e31758d-1116-4405-afc7-24c9361c3e11 (génération 3778) passe le contrôle.
Le profil commun a reçu deux modèles après la première admission. Restaurer
ce profil global casserait la seconde garde ; aucun reçu n'a été réécrit.

Le recontrôle worker conserve désormais le catalogue figé de chaque admission
si le registre courant l'inclut exactement. Les ajouts seuls sont acceptés ;
retraits, efforts, niveaux de service, programme, compte, arguments et droits
restent soumis à l'égalité complète de la définition et de son digest. Le
registre réduit est éphémère, propre au contrôle ; le registre global et les
reçus persistés restent intacts. Le coût comprend O(n log m) pour l'inclusion,
O(n) pour la copie et O(B) pour la sérialisation canonique de la définition.

Deux tests ciblés passent : catalogues ancien/nouveau et huit mutations
refusées ; parcours worker de préparation et de continuation. Sur Mac, le
second exige TMPDIR=/tmp car le chemin de socket de la fixture dépasse la
limite Unix sous le répertoire temporaire par défaut. Clippy strict
bibliothèque/tests et compilation release Mac passent. Le wrapper build.py
ne prend pas en charge fmt ; rustfmt a été utilisé sans compilation directe.

Revue indépendante native d6aeb0fe-1189-4354-a831-322fdaa7df6f : aucun défaut
prouvé sur la continuation distante visée. Résultat lu, acquitté et garde
clôturée ; vue conservée review_required. La revue relève séparément une
attribution de compte local sensible à l'égalité du catalogue (daemon.rs) et
une question sur les clés de quota par digest. Ces chemins ne sont pas la
cause des deux gardes SYS et restent à traiter séparément, sans prétendre les
avoir corrigés ici.

Activation non effectuée : `updates prepare` sur le maître refuse encore
`update_missions_active_or_unconfirmed`. Aucun bail, aucun arrêt de service,
aucune nouvelle ronde métier. Le profil SYS courant est conservé pour la
tâche 6.2. La correction compilée ne prouve pas le chargement par le worker.

Qualification Linux : release compilée sur Debian 12, mêmes quatre sources
que le Mac (hachages comparés). Le diagnostic en lecture seule sur les deux
reçus réels franchit désormais definition_identity ; il refuse ensuite
launch_preflight pour les deux. La preuve existante du programme porte le
contrat a5c30e57..., alors que le candidat exige 4585e506... : cette preuve
devra être renouvelée par le collecteur natif du nouveau worker lors d'une
activation autorisée. La preuve de l'ancien worker n'a pas été remplacée.
Le binaire actif conserve son refus pour l'ancienne garde et son succès pour
la nouvelle. Aucun succès complet de continuation avec le candidat n'est
affirmé. Preuves privées :
/Users/moi/.cache/bridget-models-20261119/guards-before.json,
/Users/moi/.cache/bridget-models-20261119/guards-candidate.json et
/Users/moi/.cache/bridget-models-20261119/program-contract-sys.json.
