# Validation et diagnostic

Incident : requête opus2d-pietons-r56-capacite-index-revue-20261102-02, tâche
7024f34c-22df-4b6a-b16f-8b37907351b8, enfant
33b7e243-efa4-4734-ba74-35a5c1e79238, worker SYS.
Lecture du maître : starting, launch_issued_at1793642118, délai1793642418,
commande7024f34c, instance d9ce3451-e872-4a0a-9999-38baf0c4e19a, génération1890,
mission_issued_at null, résultat nul. Aucun champ de cette tâche réécrit.

Worker PID1528907 inchangé : dossier préparé au commit demandé, reçu de
préparation présent, prepare_digest2083ff2dd851f353efe470ae32399403abf554d38c13e44ce9788ce08f9c4e2d.
Aucun reçu de commande dans le magasin exact. Le code conserve ces reçus
terminaux au moins sept jours ; aucun fichier de journal ou processus corrélé
observé par les lectures ciblées. Aucun lancement métier attesté.
La cause précise de l’absence de confirmation reste inconnue. Les refus de
sondes contemporains ne sont pas attribués à ce démarrage sans preuve.
L’absence seule de reçu ne ferme pas une commande potentiellement en transit ;
aucun Fence, Cancel ou nouveau Start effectué pour qualifier cet incident.

Défaut confirmé : task_view n’expliquait pas les deux codes remote_start
WorkerUnavailable/StaleConnection. Le delta porte seulement la projection
d’erreur : portée, clé, machine connue, contrôles et faits non observés.
Le modèle de tâche, les délais, l’admission et les droits restent identiques.

Deux tests ciblés Mac passent en0,13s. Admission/relecture stable, reçu durable
inchangé, aucune remise ni nouvelle tâche ; erreur historique séparée après
remise ou résultat, famille ResourceLimit distincte, terminal sans faux arrêt.
Premier essai Mac : fixtures refusées pour socket trop longue sous le TMPDIR
système. Cet essai ne qualifie pas le correctif. Relance avec TMPDIR=/private/tmp.
Contrôle de non-régression de la confirmation Release : un test passe en0,07s.
Trois tests distincts acquis ; aucun autre test relancé. La version du paquet
est ensuite portée à0.7.6 sans autre changement du comportement testé.

Preuves privées : /Users/moi/.cache/bridget227-start-guidance.
Revue indépendante Bridget02fa9e11, commit8849081b : acceptable, aucun P1.
Deux P2 corrigés : clôture native encore requise après une erreur historique,
et mandat de lecture conditionnel. Deux remarques P3 traitées : présence du
résultat durable et placement éventuellement absent. Le petit doublon avec
Release est conservé : extraire un helper n'améliore pas cette correction
locale de projection. Le dernier delta a été relu par l'auteur.
La revue n'a ni exécuté les tests ni lu tout le dossier de spécification.
Résultat lu, ACK et clôture natifs effectués ; résultat conservé.
Les deux tests227 passent après ce delta en0,19s. Le contrôle Release précédent
reste applicable : sa branche est inchangée. Aucun second banc général.

Assemblages finaux Mac et Linux au commit723c7dbd, build227-start-guidance-r2.
Mac : d7ec031b2aea52019fb0e077b345d45a220a410cc6662ac64570908837e63de5.
Linux Debian12/glibc2.36 :
e57922159be48dd517b3bd440cbd19ca5be5c8faf0ece601642536178e629ba6.
Sept images chargées et vérifiées par les PID et leurs exécutables : worker et
pont Mac, maître/mobile/worker Cartae, workers SYS et KS. Coupure maître1,98s.
Trois superviseurs métier observés conservés, dont Cartae2500061 et deux SYS.
Registre worker Mac spécial conservé. Aucune nouvelle mission métier.
Compatibilité des formats dans les deux sens et des clients locaux vérifiée sur
les quatre hôtes. Retour226-r3 conservé, pointeurs rollback écrits avec le writer
atomique natif sous verrou après arrêt natif de l'updater. Updaters repris.
L'attestation du fil et une lecture native de tâche/annuaire fonctionnent après
chargement. Le guidage est rendu par le maître ; aucun nouveau schéma MCP.

7024 : payload et canonical strictement identiques au contrôle avant passage,
SHA payload24a44aa30255e0be97bd29ffea3c4ed2e77d6a0cfad8c71d4992a7adc6050887.
La commande reste starting/génération1890/même instance. Préparation toujours
présente, reçu Start absent. Aucun arrêt physique déclaré par cette seule absence.
Avis de reprise et diagnostic envoyés à piétons et au propriétaire du monde ;
remises initiales en vol, relecture unique du statut demandée à son propriétaire.

Android0.7.6/code7006 compilé sur Cartae, certificat de production et lignée
existants vérifiés. Empreinte APK :
62298909b9bae115779695bfc2adc618dc4d338b8841fc97fe20993a38bae649.
Aucune interface Android modifiée ; pas de nouvelle recette UI/FCM requise.
Correction technique de projection API : tests natifs habituels, pas de parcours
graphique modifié. Releasev0.7.6 publique, six actifs et empreintes relus.
Le candidat Mac R1 jamais activé a été retiré après contrôle des fichiers
ouverts :35971376octets. Les caches restent conservés lorsque l'observation
des processus est incomplète, notamment le refus installed sur SYS.

Passage client terminé : fermeture native de T3 confirmée puis réouverture.
Les huit anciennes façades MCP ont quitté. Les cinq façades actuellement
chargées utilisent227-r2, leurs images et empreintes sont vérifiées. Le worker
et le pont Mac restent à la même image. Attestation du même fil et annuaire
natifs revenus ; les sept services distants restent aux PID et SHA vérifiés.
L'autorisation permanente couvre cette courte interruption des sessions.
Aucun arrêt forcé ; travaux et reçus conservés avant passage. Le LaunchAgent
ponctuel a été retiré après sa sortie pour empêcher une répétition au login.
Avis de reprise envoyés aux six destinataires encore joignables. La septième
adresse, issue d'une revue T3 distincte, n'était plus reconnue à l'envoi ; aucune
nouvelle revue ni diffusion de remplacement.

Limite de remise : la demande suivie à piétons a expiré sans réponse. La
relecture strictement identique de son envoi rend outcome_unknown. Le diagnostic
est conservé dans le ledger, sans preuve de réception ou lecture par son
destinataire. La relecture native de7024 par son propriétaire n'est donc pas
déclarée acquise. Aucun nouvel envoi de cet incident, aucune mission rejouée.

Nettoyage de ce lot : cinq fichiers retirés,113283437octets logiques, dont le
candidat R1, les deux archives déjà publiées et vérifiées, la copie Linux de
transfert et la copie APK de publication. L'APK vérifié, les actifs installés,
le retour226-r3, les sources, tâches et preuves sont conservés. Les worktrees
de construction récents gardent la propriété de leurs caches ; les autres vues
et worktrees sous observation incomplète restent conservés. Branche227 intégrée
à main ; retrait local et distant vérifié à la clôture Git.
