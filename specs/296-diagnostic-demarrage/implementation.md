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
