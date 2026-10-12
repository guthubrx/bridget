# Recherche 158B — base 158A livrée

Statut : candidat documentaire en cache. Autorisation ROOT de planification B. Base code :80b43407936110526b28f2c38ea72e1b34055154. Métadonnées de livraison A :9a94334a109b4d5f939a8ebea464530abc28f4d6. Aucun code ni test B exécuté. Les références ci-dessous visent le checkout principal actuel, pas les worktrees historiques supprimés.

## Décisions de réemploi

| Besoin | Preuve fraîche | Décision et limite |
|---|---|---|
| Tick et arrêt | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5167 | Greffer le réveil sur la boucle1s et la maintenance existantes. Aucun timer, service ou thread par programmation. |
| Pause | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/referent_control.rs:131 | Ajouter ScheduledWake à la garde pure existante ; recontrôler au départ. Halt A reste limité à Delegate/Continue. |
| Réservation099 | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:887 | Rejeu avant insertion disponible. Ce reserve n'inclut ni COUNT de quota ni issued_at dans son retour. Étendre le même magasin pour le CLI unitaire, sans deuxième registre. |
| Transaction de remise | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:89 | begin_send_delivery réunit phase dispatching, remise et ledger. Conserver OperationKind::Send. Aucun nouveau kind SQL. |
| Confirmation099 | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:768 | ACK vérifie delivery_id, instance et génération ; le cutoff de résumé avance sur cette preuve durable, pas sur socketwrite. |
| Auteur du fil | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1067 | lookup_operation avant effet, puis load_thread_for_member. Réemployer la transaction102, kindNone, vrai propriétaire membre. Aucun auteur bridget privilégié. |
| Identité CLI-fil | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:950 | Le chemin thread résout déjà une identité auxiliaire. CLI-fil exige ce contexte attesté, même headless ; sinon refus avant réservation. ROOT a retenu ce prérequis. |
| Notification de fil | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8550 | System et TriggerTurn sont créés par le daemon. L'intention102 peut fusionner plusieurs contributions ; ne pas lui appliquer une expiration B de10minutes. |
| Source du fait | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:423 | Le filtre et once existent. Notification ne garde actuellement que owner/body. Étendre la provenance typée avant retrait once ; ne jamais parser le texte modèle. |
| Échec tardif | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:1724 | Worker unique, file64, deadline et writer bornés. Ajouter retour typé vers le tick ; pas nouveau worker. |
| Snapshot | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:38 | Snapshot des abonnements plafonné262144octets. Il ne peut contenir8192 propriétaires, leurs dates et leurs cutoffs. Deux tables de domaine sont nécessaires : faits et métadonnées, sur le même Store. ROOT confirme ce choix. |
| Connexions SQLite | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:3425 | Store et IdempotencyStore ouvrent le même db_path, mais deux connexions. Pas transaction imbriquée cross-store. Chaque reprise suit une preuve gelée ; le commit102 et les réservations099 restent distincts. |
| Canon d'envoi | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication.rs:184 | canonical_send est binaire cadré, pas JSON récupérable. Pour CLI en attente, conserver un domaine fermé FrozenWake récupérable dans canonical_bytes du reçu099, scope interneCLI seulement. Aucun décodeur universel du canon historique. |

## SQL du quota CLI

Choix : une seule transaction Immediate de IdempotencyStore. Lire d'abord le reçu du scopeCLI interne stable. Un canon différent refuse ; un reçu existant rend son issued_at et son état. Pour un ID neuf seulement, COUNT des reçusCLI dont issued_at appartient à(now-60,now], puis INSERTprepared si count<60. issued_at est fixé par le daemon et jamais choisi par le client. Refus avant réservation ne compte pas. Réservation et COUNT ne sont pas deux appels non transactionnels.

Le rejeu reconstruit les métadonnées serveur depuis le reçu existant avant comparaison du nouveau target/label/acteur. Il ne remplace pas issued_at par now. Le scope réservé appartient au daemon et ne se fournit pas dans l'outil.

Pas index ou cache par défaut. Mesurer le vrai COUNT sur la rétention maximale avant choix ; l'index expires_at existant peut servir si le prédicat reste correct pour l'horizonCLI fixé. La mesure décide du besoin d'un index ciblé, sans lecture des corps ni extraction JSON de labels. Aucun résultat de coût B n'est revendiqué.

## Calendrier

libc est déjà une dépendance du daemon. localtime_r sert au rendu et timegm au parse UTC ; ce n'est pas un ordonnanceur actuel. Garder UTC pour échéances/occurrences et la date civile précédente pour unicité. Intervalle : calcul arithmétique depuis l'ancre, dernière échéance due, prochaine strictement future. Une occurrence en attente reste immuable ; ne pas accumuler les intervalles intermédiaires.

ROOT a vérifié les sources primaires suivantes. mktime interprète initialement tm_isdst=0 ou positif ; il normalise les champs. localtime_r permet de contrôler les champs civils depuis l'epoch. Le calcul B doit donc vérifier le retour civil exact. Le minimum des candidats DST valides fixe le premier instant UTC d'une heure double. Une heure normalisée vers une autre heure est rejetée comme trou. Cette règle B n'est pas une promesse native de POSIX. [POSIX mktime](https://pubs.opengroup.org/onlinepubs/009695399/functions/mktime.html), [POSIX2024 localtime_r](https://pubs.opengroup.org/onlinepubs/9799919799.2024 edition/functions/localtime.html), [projet C2024](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf).

L'addendum de résolution bornée ci-dessous ferme les transitions non-DST ; les seuls tm_isdst ne sont pas présentés comme universels. Aucun crate n'est ajouté. Changement de fuseau OBSERVÉ via fingerprint libc : seules échéances futures non gelées recalculées ; previous_calendar_date et occurrences gelées conservées.

## Faits, retour et durabilité

Avant de consommer once, enregistrer dans la même transaction Store le snapshot modifié et la provenance des notifications à partir. Le worker reçoit une référence typée. Son résultat revient dans un canal borné, drainé au tick. Succèswrite prouve seulement les octets écrits ; échec partiel/crash reste unknown. Un pending relu après redémarrage n'est jamais transformé en absence d'écriture prouvée.

observation_missed :8192 faits globaux,64 par subscription_id,7jours. observation_missed_owners :8192 owners,7jours après dernier fait/gap, plus une ligne globale bornée. Le propriétaire est protégé tant que ses faits ou un résumé non terminal restent. L'entretien retire seulement un ancien propriétaire sans faits et sans résumé non terminal. Si les 8192 propriétaires sont protégés, enregistrer une lacune globale datée dans la même transaction, sans supprimer leurs faits ni créer une 8193e ligne. La ligne globale ne contient ni UUID ni corps d'autres owners. Elle annonce compteur/période, sans reconstituer leur histoire.

Register confirmé et DNDlevé sont des signaux du même épisode de retour. La génération persistante déduplique l'épisode. Le résumé contient seulement les faits/gaps nouveaux depuis le cutoff confirmé. Génération,cutoff,key,issued_at,canon figés avant remise. ACK099 valide avance le cutoff ; lecture missed ne le fait pas. Unknown est terminal pour cette génération, sans avancer le cutoff ; nouvelle génération uniquement lors d’un nouveau retour réel, avec incertitude annoncée.

## Alternatives écartées et entretien

Écartés : compteurquotaRAM, nouveau scheduler/service, nouveau writer, faux membre système, expiration de WakeRow partagée, parse du corps modèle, métadonnées8192 dans snapshot262K, archive illimitée, nouvel outil missed, consommateur secret fictif, HTTP. Les nouvelles structures ont des responsabilités de domaine inspectables. Le mainteneur modifie cadence/SQL/façade séparément et peut supprimer B sans réécrire A/099/102.

La baseline de trente oracles est une extraction de FULL158A historique, pas une exécution B : /Users/moi/.cache/bridget-plan158B-tests-preparation-20261012/oracles-reuse-baseline158B.json. Aucun script de fixture avec SIGKILL/process-group n'est copié. US8 reste reportée faute de consommateur.

## Résolution bornée des heures civiles, y compris sans DST

Les candidats mktime0/1 ne constituent pas un inverse universel de localtime. La règle finale choisit le minimum de tous les epochs correspondants dans une recherche UTC bornée. Support annoncé : fuseaux dont les offsets sont des minutes entières dans[-24 h,+24 h]. Chercher par pas60 s dans timegm(date,HH:MM:00)±24 h, vérifier le retourcivil exact localtime_r, puis retenir le pluspetitUTC. Aucunmatch : heure inexistante. Offsethorsborne ou àsecondes : calendar_unavailable, pas approximation. La recherche couvre aussi un fold dû à un changement d'offset sans changementtm_isdst.

Au plus2881 conversions par date et8dates civiles pour la prochaine journéehebdomadaire. Calcul horsverroudaemon lors création/avancement/changementTZ, jamais àchaquetick ; résultat lié au snapshotTZ et CASnext_at au commit. Mesurer le coût avant optimisation. Si le fuseau change pendant le calcul, résultat rejeté puis recalculé avantréservation. Les autreszones ne sont pas silencieusement prétendues supportées. Aucun crate neuf.

## Fermetures finales du contrat ROOT

**Fuseau observé.** Le daemon relève le fingerprint de la zone effectivement rendue par libc (nom de zone et offset) au tick et avant le gel. Un changement observé recalcule seulement les échéances futures non gelées. FrozenWake et previous_calendar_date restent inchangés. Le résultat d'un calcul hors verrou porte son fingerprint ; le CAS refuse un résultat devenu périmé. Aucune promesse de zone figée ou de changement uniquement au restart. Les changements DST du fingerprint n'autorisent pas une seconde occurrence de la même date.

**Retour du worker.** Le canal de résultat utilise try_send, borne64. Si le canal est plein, aucun résultat sans preuve n'est inventé : le pendingfact durable garde sa provenance et sa deadline initiale immuable. Le tick solde un pending dépassant cette deadline en write_unknown, même si l'écriture a pu réussir ; il ne le qualifie jamais de non-écrit. Aucun once recréé. Si Store ne peut pas écrire ce résultat, garder la ligne pending et l'annonce agrégée de panne disponible, puis réessayer à la maintenance ; jamais annoncer une sauvegarde réussie. Le compteur global existant reste une preuve aggregate limitée, pas un reçu durable par fait.

**Cutoff.** Chaque fait porte un fact_seq SQLite AUTOINCREMENT durable et jamais réutilisé, en plus de son UUID de corrélation. Le cutoff du résumé capture ce fact_seq, plus owner_gap_version et global_gap_version séparés. Un fait arrivé après le gel au même observed_at reste au-delà du cutoff. ACK099 valide avance seulement les trois valeurs gelées ; aucune valeur plus récente n'est consommée. Pas triUUID ou horodatage en secondes seul pour ce seuil.

**Création et annulation.** schedule ne prend aucun command_id et n'est pas idempotent à la création. Chaque admission crée un schedule_id UUID neuf côté daemon. Un retry aveugle peut créer un second schedule ; après réponse perdue, relire list au lieu de promettre une création unique. FR103 protège les occurrences du schedule déjà créé, pas cette création. cancel prend seulement ce schedule_id canonique ; sur la même programmation connue déjà annulée, retourne already_cancelled sans mutation ni nouvelle émission. UUID inconnu refuse. Pas second registre de commandes et aucun command_id administratif gratuit. Les --id CLI unitaires conservent séparément la déduplication099 du contrat.

## Borne des programmations — décision ROOT

Limites physiques :128 lignes au total et16 par propriétaire, y compris les programmations annulées. Une admission peut retirer, dans sa transaction, les plus anciennes lignes annulées sans occurrence en attente ou en vol : d'abord celles du propriétaire si sa borne est atteinte, puis celles de la borne globale. Si aucune ligne n'est retirable, schedule_limit. Les preuves099/102 restent intactes. Cancel rend already_cancelled pour une ligne connue ; après retrait borné, son UUID est inconnu et refuse. Aucun historique perpétuel d'annulation.


## Transitions finales158B R1

Les règles finales de pause, départ099 distinct du commit102, unknown terminal, deadlineworker5s/budgetI/O1s, horizonCLI7jours, attribution du label et entretien des résumés sont définies dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/contracts/wake-158B.md. Elles priment les descriptions préparatoires ci-dessus. Aucun nouveau service, worker, registre ou scénario de confort.
