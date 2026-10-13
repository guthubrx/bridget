# Contrat worker 162 — remote_worker_v1, révision R2

## Transport, connexion et autorité

Réutiliser la socket Unix fédérée par SSH. Le handshake worker est distinct du guichet Service. Il lie machine, worker, révision et credential privé. Une seule connexion fait autorité. Même identité et credential autorisés : remplacer atomiquement l’ancienne connexion, la fermer, attribuer une nouvelle époque et rejeter ses trames tardives. Credential divergent : refus. Toute opération doit viser un placement durable de ce worker. Le verrou local du store maintient un seul worker écrivain.

La nouvelle époque ne réautorise jamais un permis précédent ni un lancement ambigu. Aucun fallback local. NativeDelegationRequest refuse déjà les champs inconnus ; l’ancien MCP refuse aussi les clés inconnues. Seul l’ancien registre ignore machines. Tester ces comportements historiques. Le maître valide capacité et placement lors de la demande courante, pas seulement lors d’un préflight séparé.

Le worker, les wrappers et les fournisseurs sous le même UID partagent un domaine de confiance. Les modes 0600/0700 ne les isolent pas entre eux. Un fournisseur full-access compromis peut lire le credential ou modifier l’état ; l’authentification prouve la possession du credential, pas l’intégrité du processus. À Prepare et avant RELEASE, refuser tout chevauchement canonique dans les deux sens entre cwd, mappings, writableRoots ou additional_directories et le credential, son répertoire privé ou l’état worker. Une résolution impossible entraîne un refus. Cette garde prévient l’exposition explicite ou accidentelle, pas un accès global dangerFullAccess/bypassPermissions. Le wrapper d’infrastructure peut publier uniquement dans son slot préalloué ; ce slot n’est jamais ajouté aux droits du fournisseur. Un UID séparé ou une sandbox OS restent hors périmètre. Journaliser les remplacements de connexion avec machine, worker et anciennes/nouvelles époques, jamais le secret. Le credential est chargé depuis le fichier privé dans la seule mémoire du worker. Il ne passe jamais par l’environnement du wrapper ou du fournisseur. build_environment/pass_env ne constitue pas un filtre générique de secrets ; ne pas créer une variable d’environnement contenant ce credential.

## Messages fermés

Les enveloppes WorkerRequest/WorkerResponse réemploient le framing et portent version, correlation_id et identité complète de commande. L’époque provient de la connexion authentifiée et est vérifiée pour chaque réponse ou observation.

| Opération | Sens et fonction |
|---|---|
| prepare | Maître vers destination ; observer configuration, cwd, fournisseur et sources de droits, sans processus. |
| parent_proof | Maître vers hôte d’origine attesté ; relire les sources du parent et retourner leurs révisions, sans secret ni mutation. |
| start | Maître vers destination ; reçu avant bootstrap, marqueur avant ready_to_release. |
| reserve_result_slot | Maître vers worker ; réservation idempotente du budget et slot avant tout dispatch initial, de ronde ou de reprise quota. |
| release | Maître vers destination ; permis frais après revalidation, une seule libération. |
| reconcile | Maître vers worker ; reçus et observations corrélés, aucune nouvelle exécution implicite. |
| cancel | Maître vers worker ; arrêt de la seule identité attestée. |
| journal_page | Maître vers worker ; page bornée du journal local, autorisée par le maître. |
| final_result | Worker vers maître ; enveloppe canonique durable d’un résultat, y compris après mort du wrapper. |
| result_disposition | Maître vers worker ; résultat de capture ou de classement, lié au canon exact. |

Réemployer Heartbeat unitaire. L’époque vient de la connexion, pas d’un nouveau champ Heartbeat. Toutes les E/S réseau se font hors mutex DaemonState : probe, E/S, puis revalidation avant mutation. Cela couvre les preuves parent, journaux, annulations, réconciliations et résultats.

## Préparation et lancement

prepare reçoit projet, racines approuvées, chemin relatif, moteur et contraintes parent sans secrets. Le worker renvoie empreintes et token. Le maître relit aussi parent_proof sur l’hôte réel du parent, même distant, avec profondeur actuelle maximale de 8. Aucune réduction aux parents locaux. Revalidation avant autorisation Release ; délai dépassé signifie refus sûr, pas promesse de latence interhôtes.

start exige la réservation durable du maître et l’égalité du canon. Persister reserved avant bootstrap. Réemployer managed_process ; persister le marqueur avant ready_to_release. Rejeu égal : restituer l’état sans recréer ; canon divergent : refus. Le quota Fleet est global toutes catégories, auquel s’ajoutent les limites fournisseur 157 par agent_type. Local et distant partagent ces plafonds ; aucun quota par machine. Les slots techniques sont une garde additionnelle de stockage.

release exige le permis corrélé au canon, à l’époque et aux révisions. Le worker relit ses sources, vérifie les délais, persiste release_authorized puis émet RELEASE une fois. La fenêtre de préparation est de 30 secondes au maximum, depuis avant Prepared jusqu’à ReadyToRelease inclus. Le permis expire au plus 5 secondes après cet événement local ReadyToRelease, jamais 5 secondes après une réception réseau tardive. Le maître contrôle aussi ses délais. Révocation avant permis : refus ; après permis : annulation corrélée. Crash ou reconnexion invalide les autorisations en mémoire. À expiration de Prepare ou Release, appeler abandon et fermer le canal RELEASE ; le bootstrap sort sur EOF. Constater sa fin avant de libérer la capacité.

## Store technique et résultat

WorkerCommandReceipt appartient à un store technique distinct de ReceiptStore Seen/Acked. Réemployer ses gardes, pas son format ni son handle partagé. Le worker écrit seul les métadonnées et confirmations. Le wrapper écrit seul un blob résultat immutable dans un slot privé préalloué pour mission, ronde et exécution. Aucun nouvel IPC ni usage du FD managed-status après démarrage.

Avant réseau, publier par écriture atomique et fsync l’enveloppe finale : message_id, tâche, mission, ronde, instance, génération, execution_id, génération d’exécution, corps et empreinte. Le journal asynchrone ne constitue pas cette preuve. Contrôler namespace privé, absence de symlink et producteur unique. Blob déjà présent : égal signifie rejeu en lecture ; divergent signifie quarantaine, jamais remplacement. Le worker ne modifie jamais ce blob.

Le wrapper distant publie seulement le blob et n’émet aucun résultat par Send ou SendIdempotent. Le worker est l’unique émetteur final_result, même si le wrapper est mort. capture_reply refuse toute mission distante courante ou passée par l’erreur remote_result_requires_worker avant ses branches de mutation ou terminales. Les appelants Send et SendIdempotent retournent Nack, jamais Ack ni Ok(false) qui continuerait vers prepare_dispatch. Ajouter l’entrée capture_remote_result : vérifier connexion et époque courantes, placement durable, toutes les identités et canon. Extraire la mutation partagée de capture_reply sans enlever ses gardes locales. La voie distante ne requiert pas conn_instances du wrapper mort. RetainedResultAuthority s’applique seulement après capture pour envoyer le résultat au parent.

La disposition et sa référence canonique sont persistées dans la même transaction que Task et sa projection, avant confirmation :

| Disposition | Effet |
|---|---|
| accepted | Résultat nouveau ou rejeu égal committé ; notification et wait existants. |
| discarded_terminal | cancelled, stale_round, obsolete_execution ou expired prouvés irréversibles ; pas un succès métier. |
| rejected_conflict | Canon divergent refusé, premier résultat préservé ; preuve de conflit conservée. |
| deferred | quota_hold ou execution_settlement_pending ; slot conservé, nouvel essai borné sur échéance ou fait nouveau. |

Une erreur de stockage ou une autorité non prouvée ne confirme rien de terminal. Un ACK legacy n’est pas une disposition. Un ACK perdu après commit restitue la même disposition persistée. Les issues finales arrêtent les essais ; deferred ne déclenche aucune boucle immédiate. Quota_hold ne doit jamais être classé comme discarded uniquement parce qu’il dure.

## Bornes et collecte

reserve_result_slot porte tâche, mission, ronde, instance, génération, execution_id/génération et digest du canon d’admission, pas le digest du futur résultat. L’identité est figée avant l’échange. E/S hors verrou, puis revalidation sous verrou avant dispatch. Appeler avant RELEASE initial ; après begin_round mais avant send_as mission_pending ; après reconstruction quota et sauvegarde resume_execution_id mais avant deliver_reconstructed. Rejeu égal : même slot. Refus : identité conservée non dispatchée, aucun UUID neuf. Réservation caduque libérable seulement après preuve durable de non-dispatch ou annulation du slot, sans arrêt du wrapper réutilisé. Corps : 262 144 octets maximum. Enveloppe JSON sérialisée : 2 Mio maximum, métadonnées fermées et bornées. Le budget total de 16 Mio comprend réservations, blobs, métadonnées anciennes/nouvelles pendant écriture, fichiers de statut et temporaires orphelins. Plafond de nombre : 4 096 entrées, sans promesse de 4 096 résultats simultanés. Moins de huit slots maximaux tiennent dans 16 Mio. Refuser avant dispatch si la réservation manque.

La disposition finale solde la réserve de publication ; les octets conservés continuent à compter. Collecter le blob après disposition durable, preuve de fin du producteur et fermeture irrévocable de la commande ou du slot. Conserver 7 jours le reçu compact, l’empreinte et la disposition, dont les octets restent comptés. Un ACK seul ne permet pas de supprimer un blob pendant que son producteur peut encore publier. Résultats différés ou ambigus : jamais de purge automatique. Un slot encore vivant et non clôturé conserve son blob et peut saturer le budget. Après crash worker, reprendre son verrou et vérifier reçu/blob ; fichier temporaire ou absence ne prouve pas un résultat.

## Journal, annulation et inconnus

Attach lit aujourd’hui des fichiers locaux. Le worker sert journal_page avec ce lecteur ; la racine se dérive de l’instance et de la génération attestées, selon le patron ManagedStderrStore, sans champ de chemin fourni par le CLI ni racine supplémentaire stockée. Le maître autorise chaque demande. Le flux vivant réemploie LiveJournalFragment et AttachView déjà relayés au maître, avec leurs bornes et annulations d’abonnement. Aucune nouvelle opération follow/stop distante. Un gap impose une reprise par page froide autorisée ; si le worker est perdu, montrer le gap et l’indisponibilité. Aucun miroir permanent. Worker perdu : journal indisponible explicite ; dernier état et résultat déjà reçus conservés au maître.

cancel vérifie commande, machine, worker, instance, génération, PID et naissance via managed_process. Aucun kill par nom. Sans preuve de mort, l’annulation reste en attente. Unknown conserve la capacité jusqu’à observation terminale fiable ou intervention humaine hors périmètre ; cela réduit aussi la capacité locale sous le quota Fleet global ; aucun reset destructif de preuve ajouté.

Contact : Heartbeat toutes les 10 secondes, dégradation après 30 secondes selon horloge du maître. last_contact reste volatile ; notifier seulement les transitions de contact_state. Les transitions de contact_state incrémentent seq et invalident les curseurs. last_contact_at est observé lors de chaque page ; l’afficher comme tel, sans gel global ni âge vivant.

## Frontières à tester

Refus Send et SendIdempotent sur mission distante courante ou passée sans dispatch de remplacement ; reserve_result_slot initial/ronde/quota refusé avant dispatch, rejeu même identité et slot caduc prouvé ; avant prepare ; expiration avant ready ; retard réseau du permis ; réservation avant reçu ; reçu avant création ; création avant marqueur ; marqueur avant ready ; permis avant RELEASE ; RELEASE avant réponse ; Register avant fin ; résultat avant capture ; crash maître avant commit ; commit avant ACK ; ACK perdu ; résultat divergent ; ancienne ronde ou exécution ; quota_hold puis sortie vers acceptation ou disposition terminale ; wrapper mort avec relais après redémarrage worker ; publication pendant crash worker ; temporaires et saturation ; disposition finale avec producteur encore vivant ; collecte seulement après toutes ses gardes ; reconnexion avec ancienne connexion fantôme et trames tardives ; parent distant et journal froid distant et suivi vivant existant annulé.

Les oracles comptent bootstraps, RELEASE, commits et dispositions. Une nouvelle génération 152 exige une preuve terminale. Quota 160 ne devient pas un spawn. Aucun test futur n’est présenté comme exécuté.

## Précisions de raccordement — 13 octobre, implémentation en cours

Ces décisions précisent les frontières ci-dessus. Leur validation runtime reste à faire.

- Avant réservation Fleet, Prepare/Prepared ne prétend pas connaître l’instance ni la génération future. Son enveloppe porte command_id/task_id et la sentinelle instance_id vide/generation zéro. Seules ces deux opérations acceptent cette sentinelle. Start et les autres opérations exigent l’identité complète réservée.
- Start transmet aussi child_id, mission_id et forward_child_requests, nécessaires au bootstrap existant. Le credential worker reste hors environnement. Le slot initial est réservé après le reçu et avant RELEASE ; il n’a pas à précéder la création du bootstrap encore bloqué.
- Pour l’isolation Git, réemployer la séquence 154 : première préparation de la racine, réservation durable de Task, seconde préparation du worktree et reprojection des permissions, puis Fleet/Start. Task reste publiquement queued ; remote_prepared empêche son lancement prématuré. Le reçu technique Git est écrit avant création ; une création ambiguë n’est jamais répétée. WorkspaceRecord réel revient dans Prepared.workspace, valeur bornée désérialisée strictement au maître. Aucune observation Git n’est reconstruite depuis la demande.
- expected_git_identity est la valeur exacte de remote.origin.url lue localement par la commande Git fermée, sans includes et sans accès au réseau. Une URL absente ou divergente est refusée. La reprojection vers un worktree utilise seulement son chemin attesté, sans élargir les racines d’écriture.
- Le wrapper publie un marqueur immutable de conflit dans son slot si un second canon diverge. Le worker reste le seul écrivain de la quarantaine des métadonnées. Les octets et temporaires de ce marqueur et du reçu Git comptent dans les bornes communes.
- Le journal froid du worker réemploie le lecteur Attach. Il renvoie des fragments complets et la séquence atteinte ; une lacune produit un gap explicite au maître. La racine privée instance/génération/enfant est dérivée sur le worker, avec contrôle de composants. Le maître ne lit jamais sa propre session locale pour suppléer une tâche distante.

## Complément de raccordement vérifié par lecture — 13 octobre 2026

Ce complément fixe les obligations du raccordement. Le code correspondant est présent ; cette lecture ne vaut pas validation d’exécution.

1. **Négociation avant secret.** Sur chaque nouvelle connexion, émettre le message historique DaemonIdentityRequest. Exiger une réponse DaemonIdentityReport dont features contient remote_worker_v1. Réutiliser exactement cette connexion pour WorkerHello. Une réponse absente, invalide ou sans capacité interdit WorkerHello et tout envoi du credential ; aucun repli de lancement n’est permis.
2. **Preuve native distincte de RELEASE.** Observation porte native_ready avec défaut false. Le worker ne l’atteste qu’après fermeture du canal startup sans StartupFailed. Le maître combine cette observation corrélée avec le Register exact du wrapper associé au lease. Released seul, un Register seul ou une observation sans identité reconnue ne produit pas Connected.
3. **Git dans le store I40.** WorkspacePreparationReceipt réserve le budget commun et persiste le canon Prepare avant workspace_git::create. Le record absent après interruption signifie unknown. Le rejeu ne crée jamais un second worktree ; un record présent doit encore être vérifiable. Lire l’identité du dépôt par la liste d’arguments fermée git config --local --no-includes --get remote.origin.url, via run_git existant et son délai. Aucun accès réseau ni chemin de commande arbitraire.
4. **Conflit immutable.** Si un blob existant diffère, conserver ce blob et écrire une seule preuve payload/conflict sous le verrou de publication. Le wrapper ne modifie aucune métadonnée du worker. Conflit ou temporaire ambigu interdit une nouvelle publication ; tous les fichiers comptent dans les bornes communes.
5. **Préparation et preuves de résultat dans Task.** remote_prepared vaut false jusqu’à la préparation attestée. Persister slots et reçus résultat avec Task/projection. Les collections sont bornées à 64 par tâche ; le parcours normal maximal actuel emploie 26 identités. Aucune suppression de preuve active pour faire de la place. Ce plafond ne remplace pas la réservation des octets worker avant dispatch. Un reçu Deferred empêche le règlement et la purge de l’arbre maître, ainsi que la collecte terminale du résultat worker.

Les extractions load_from_path et prepare_spawn_parts réemploient les gardes et traitements existants. Elles ne constituent pas de nouvelles API publiques. Côté T3, le helper BridgetTaskPlacement est colocalisé dans le composant d’infobulle et partagé par trois vues ; il ne change pas le contrat de transport ni la dépendance de Bridget.
