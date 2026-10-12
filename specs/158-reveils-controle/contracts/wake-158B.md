# Contrats 158B — extension de la base158A livrée

## Surfaces

Un seul nouvel outil MCP : bridget_wake. Une famille CLI bridget wake. Une action missed ajoutée à bridget_events. Aucun contrôle humain MCP, HTTP, outilsecret, daemon ou service supplémentaire.

## MCP wake version1

Objet fermé. Actions fermées : schedule,list,cancel. Champs d'identité/owner/from/issued_at/scope refusés. Null n'est pas omission pour cible/cadence/label/IDs obligatoires. Les cibles sont fermées :

```json
{"kind":"self"}
{"kind":"agent","id":"UUID canonique"}
{"kind":"thread","id":"UUID canonique"}
```

Cadences :

```json
{"kind":"interval","seconds":60}
{"kind":"at","time":"09:00","weekdays":[1,2,3,4,5]}
```

Schedule prend target,cadence,label facultatif≤2048 octets. List ne lit que les propres schedules. Cancel prend schedule_id et vérifieowner avantmutation ; rejeu d'une annulation connue rend sa preuve. Pas command_id de création : chaque schedule admis reçoit un UUID neuf. Cancel est une projection idempotente du même schedule connu, sans registre supplémentaire.

Portée de base Sends ; readOnlyHintfalse. Schedule self et list/cancelpropres sont WritesOwnState. Schedule agent/thread reste Sends, même si le UUID fourni ressemble à l'appelant. Unknownaction/champs sont InvalidParams ; cible/propriété inconnue refuse sans effet. La tableexhaustive 158A inclut le nouveloutil. Le plafondDiscovery permet self et gestionpropre ; refuse les autres effets avantmatch. Les droits de projet et de membre sont revérifiés au départ.

Le transport utilise une familleWakeRequest/WakeResult fermée dans les enumsprotocoles existants. L'identité vient de registered_connection+instance/live_connection_identity. Le CLI d'administrationhumaine utilise la négociation et control_client_actor existants, pas l'autoritéMCP. Aucun indicateur booléen client ne remplace ces gardes.

## CLI

`bridget wake <agent|fil> [--label T] [--id K]` est un déclenchementunitaire, via socketUnix, sans autostart de modèle et sans terminalinteractif obligatoire. Cible agent/fil doit être connue. Résolution par le daemon vers sa clé canonique ; alias ambigu refuse. --id omis crée uneclé unique ; --id conservé au retry. Rejeu du mêmeID/canon conserve issued_at original, quotas et opération 102. Canon différent refuse.

CLI vers fil : contexte d'identitéauxiliaire existant requis. Le daemon l'atteste, vérifie projet et membre, puis attribue le post à cet acteur. Sans ce contexte, refusalidentity avant réservation. Aucun --from, auteurhumain/bridget ou créateurdufil usurpé. Scripts/launchd/ssh doivent recevoir le même contexte attesté que la CLIthread existante ; headless ne signifie pas anonyme. Agentdirectlocal garde la borneUIDADR011 et Systemcrééparle daemon.

Gestionhumaine prévue : bridget wake list --all et bridget wake cancel <schedule_id> surchemin de contrôlehumain existant, négociation et acteurvalidés. Un agent ne gère que ses programmations. Les flagsadministratifs ne sont jamais ajoutés à MCP. Ces formes doivent réemployer require_interactive_terminal/cmd_control pour mutationsadministratives, pas pour déclenchementCLI unitaire.

Limiteglobale :60 nouveauxIDsCLI/minuteglissante, scopeinterne stable réservédaemon,issued_atdaemon. Lookup/replay/mismatch avantCOUNT,COUNT+INSERT dans1TXImmediate 099. Occurrencesschedules et appelsMCP exclus. Refusavantreservation necomptepas. Le reçu durable survit au redémarrage pendant7jours à partir du issued_at initial. Un reçu expiré refuse le rejeu ; son horizon ne se renouvelle pas.

## Remise et résultat

Agent : prepared/deferred avantdépart jusqu'àdue_at+600 s ; gardeScheduledWake fraîche et route/instance/capacité/DND. Aucun départ àexpiration. Après départ, utiliser 099 : confirmed ou outcome_unknown. Un receipt préparé n'est pas un succèsexécuté.

Précision de preuve après revue R2 : si la garde du callback refuse avant
begin_send_delivery, aucun départ n'a été enregistré et le retour peut rester
Deferred. La lecture durable lors d’un passage ultérieur qui relit cette occurrence avec now >= grace_until conduit
au parcours MissedBeforeDeparture ; le parcours CLI finalise alors Rejected,
sous réserve des lectures et écritures SQL réussies. Le contrat n'exige pas
un état terminal immédiat dans le callback. Le GREEN G5 prouve Deferred,
absence de remise099 et absence de trame. Il ne teste pas la finalisation au
passage ultérieur qui relit cette occurrence. Cette dernière reste une preuve par lecture, pas un PASS runtime.


Fil : membre/project puis entrée+intentions commit 102. Body préfixecourt+[label entier2048] ; kindNone,notifyall valide 102 ; auteurowner. L'UUIDoperation estpersistéavecoccurrence. Alertes 102 conservent leurs phases/générations/ACK et WakeRowpartagée. Ne pas lui donnerTTL10min B ni effacer sollicitationsordinaires.

Pause : aucune nouvelle occurrence programmée n’est gelée pendant la pause. À sa levée, un intervalle choisit sa dernière échéance due et une seule occurrence de rattrapage ; une heure fixe retardée de plus de600s est sautée. Une occurrence déjà gelée garde sa grâce initiale de600s, sans prolongation. Après son résultat terminal, next_at est avancé strictement au-delà de now sur son ancre pour éviter un second rattrapage immédiat. Le déclenchement CLI unitaire expiré ne crée aucun rattrapage. Halt 158A ne bloque pas les réveils par effetimplicite. Cancel empêche prochainesoccurrences etpendingnonparties ; aprèsremise, résultat conservé, pas fausseannulationd'effet.

## Missed et résumé

bridget_events action=missed est Reads. Retourne faits du propriétaireauthentifié, abonnementoncehistorique compris, lacunesowner et lacuneglobale agrégée. Pagination propre : 20 faits par défaut, 64 au plus, after_seq dans le domaine i64 ; réponse JSON complète ≤65536 octets et faits entiers ≤16384 octets, jamais tronqués. Les champs sont fermés, sans owner libre. Pas deleteàlecture. Aucun corps/UUID/crossowner dans lacuneglobale.

Les états distinguent notification jamaispartie prouvée, écritureincertaine et lacunedesource. Journal_only/throttlevolontaire ne deviennent pas missed. Tout échecde persistance refuse le succèsde sauvegarde ; annonce la lacuneconnue.

Retour Register confirmé ou DNDlevé :1générationparépisode, résumé des faits/gaps nouveaux depuiscutoffconfirmé. Pendingcanon/key/issued_at/cutoffimmuables. Seul ACK099 validédelivery/instance/generation avancecutoff. Unknown ne signifiepas appliquénireçu. Lecturemissed n'avancepas ce seuil.

## Calendrier et changements

Intervalle≥60 s :dernièredue≤now,unique rattrapage,next>now. Heurefixe :HH:MM,journonvide,calendrierdaemonannoncé. Trou :aucuneoccurrence. Heure double : recherche UTC bornée et retourcivil strict décrits ci-dessous ; minimumUTCvalid. previous_calendar_datepersistée évitedeuxpassagesd'unemêmedate. La résolution non-DST est fermée dans l'addendum ci-dessous, avec domaine de fuseaux explicitement borné.

Changement TZ OBSERVÉ au tick et avant gel via fingerprint libc : nouvelle zone pour échéances futures non gelées seulement. FrozenWake etdateprécédenteinchangés. Toute zone/civilnonrésolu rend erreurcalendar avantréservation, pas approximationminuteaffichée.

## Refus fermés à intégrer au contrat des résultats

invalid_request,identity_unavailable,target_unknown,target_ambiguous,not_owner,thread_unavailable,project_scope_unavailable,capability_denied,schedule_limit,rate_limited,calendar_unavailable,storage_unavailable,envelope_mismatch. La liste métier peut réemployer le nom exact d'unrefus existant ; ne pas créercodealias inutile. Effet0avantadmissionrefusée, zérofaussepreuveaprès I/Oincertaine.

US8 non planifiée. HTTP public exclu. Les exemples sont uncontratcandidat, pas des outilsB déjàdisponibles.

## Résolution bornée des heures civiles, y compris sans DST

Les candidats mktime0/1 ne constituent pas un inverse universel de localtime. La règle finale choisit le minimum de tous les epochs correspondants dans une recherche UTC bornée. Support annoncé : fuseaux dont les offsets sont des minutes entières dans[-24 h,+24 h]. Chercher par pas60 s dans timegm(date,HH:MM:00)±24 h, vérifier le retourcivil exact localtime_r, puis retenir le pluspetitUTC. Aucunmatch : heure inexistante. Offsethorsborne ou àsecondes : calendar_unavailable, pas approximation. La recherche couvre aussi un fold dû à un changement d'offset sans changementtm_isdst.

Au plus2881 conversions par date et8dates civiles pour la prochaine journéehebdomadaire. Calcul horsverroudaemon lors création/avancement/changementTZ, jamais àchaquetick ; résultat lié au snapshotTZ et CASnext_at au commit. Mesurer le coût avant optimisation. Si le fuseau change pendant le calcul, résultat rejeté puis recalculé avantréservation. Les autreszones ne sont pas silencieusement prétendues supportées. Aucun crate neuf.

## Fermetures finales du contrat ROOT

**Fuseau observé.** Le daemon relève le fingerprint de la zone effectivement rendue par libc (nom de zone et offset) au tick et avant le gel. Un changement observé recalcule seulement les échéances futures non gelées. FrozenWake et previous_calendar_date restent inchangés. Le résultat d'un calcul hors verrou porte son fingerprint ; le CAS refuse un résultat devenu périmé. Aucune promesse de zone figée ou de changement uniquement au restart. Les changements DST du fingerprint n'autorisent pas une seconde occurrence de la même date.

**Retour du worker.** Le canal de résultat utilise try_send, borne64. Si le canal est plein, aucun résultat sans preuve n'est inventé : le pendingfact durable garde sa provenance et sa deadline initiale immuable. La deadline durable vaut observed_at+5s, fixée avant enqueue ; le budget I/O existant reste1s. Le tick solde un pending dépassant cette deadline en write_unknown, même si l'écriture a pu réussir ; il ne le qualifie jamais de non-écrit. Aucun once recréé. Si Store ne peut pas écrire ce résultat, garder la ligne pending et l'annonce agrégée de panne disponible, puis réessayer à la maintenance ; jamais annoncer une sauvegarde réussie. Le compteur global existant reste une preuve aggregate limitée, pas un reçu durable par fait.

**Cutoff.** Chaque fait porte un fact_seq SQLite AUTOINCREMENT durable et jamais réutilisé, en plus de son UUID de corrélation. Le cutoff du résumé capture ce fact_seq, plus owner_gap_version et global_gap_version séparés. Un fait arrivé après le gel au même observed_at reste au-delà du cutoff. ACK099 valide avance seulement les trois valeurs gelées ; aucune valeur plus récente n'est consommée. Pas triUUID ou horodatage en secondes seul pour ce seuil.

**Création et annulation.** schedule ne prend aucun command_id et n'est pas idempotent à la création. Chaque admission crée un schedule_id UUID neuf côté daemon. Un retry aveugle peut créer un second schedule ; après réponse perdue, relire list au lieu de promettre une création unique. FR103 protège les occurrences du schedule déjà créé, pas cette création. cancel prend seulement ce schedule_id canonique ; sur la même programmation connue déjà annulée, retourne already_cancelled sans mutation ni nouvelle émission. UUID inconnu refuse. Pas second registre de commandes et aucun command_id administratif gratuit. Les --id CLI unitaires conservent séparément la déduplication099 du contrat.

## Borne des programmations — décision ROOT

Limites physiques :128 lignes au total et16 par propriétaire, y compris les programmations annulées. Une admission peut retirer, dans sa transaction, les plus anciennes lignes annulées sans occurrence en attente ou en vol : d'abord celles du propriétaire si sa borne est atteinte, puis celles de la borne globale. Si aucune ligne n'est retirable, schedule_limit. Les preuves099/102 restent intactes. Cancel rend already_cancelled pour une ligne connue ; après retrait borné, son UUID est inconnu et refuse. Aucun historique perpétuel d'annulation.


## Transitions et reprise — précisions de la revue158B R1

Le résultat outcome_unknown est terminal pour une occurrence, sans preuve de réception. Il libère la génération suivante. Aucun renvoi de cette occurrence inconnue et aucune modification de son canon. Le magasin conserve uniquement le dernier résultat par programmation, avec les reçus099/102 soumis à leur rétention existante.

| Preuve durable retrouvée après crash | Résultat et action |
|---|---|
| FrozenWake préparé et lecture099 réussie prouvant absence de send_delivery | Différé si grâce active ; missed_before_departure à expiration. |
| send_delivery099 dispatching, sans ACK valide | outcome_unknown terminal ; ne pas renvoyer cette occurrence. |
| send_delivery099 acked valide | confirmed ; réappliquer seulement la clôture locale. |
| Erreur de lecture du Store/099 | storage_unavailable ; aucune absence ou réception déduite. |
| Reçu de post102 committé sous UUID gelé | committed_thread ; terminer la clôture099 CLI éventuelle sans republier. |
| Post102 non confirmé, lecture disponible | Rejouer le même UUID/canon102 selon son contrat ; aucune nouvelle opération. |

Pour un agent, la frontière de départ est le commit de begin_send_delivery099 avant toute écriture au wrapper. Pour un fil, elle est distincte : commit de la contribution et de ses intentions102. Le received/acked102 de ses alertes ne conditionne pas committed_thread.

CLI-fil : son UUID102 est créé avant reserve_cli_wake et inclus dans FrozenWake/canon099. Utiliser la transition099 prepared→dispatching avant post102, puis finalize existant après preuve102. Dispatching du reçu de commande ne signifie pas send_delivery agent. Après crash, retrouver102 sous son UUID et achever099 ; ne pas inventer de remise socket.

Chaque notification programmée vers agent conserve l'origineSystem du daemon exigée par US5. Son corps figé annonce « Réveil programmé par <identité propriétaire attestée> », puis le libellé entier. Le libellé reste une donnée d'agent ; il n'accorde aucun droit et ne modifie ni l'intention ni les gardes. Le CLI local est attribué au déclencheur local. Aucun auteur fourni par le client.

Résumé : prepared/deferred est limité à600s à partir de son gel. Dispatching sans ACK valide devient outcome_unknown terminal sans avancer le cutoff. Une nouvelle génération est possible seulement lors d'un nouveau véritable épisode de retour, avec annonce du résultat incertain précédent. Elle ne renvoie pas automatiquement l'ancien canon. Une même génération ne produit jamais un second départ. Un ACK tardif n'avance le cutoff que s'il correspond encore à la génération et au delivery_id conservés. Les preuves099 déjà acked sont recherchées avant clôture inconnue ou supersession.

L'entretien protège les owners avec résumé non terminal et des faits vivants. Il retire d'abord les owners sans faits et sans résumé non terminal. Si toutes les lignes sont protégées, la nouvelle attribution ne crée pas une8193e ligne : annoncer une lacune globale datée dans la même transaction. Un pending n'est pas protégé au-delà de sa deadline : sa clôture inconnue permet l'entretien normal. La borne physique8192 reste prioritaire ; un callback ou ACK visant un owner retiré ne recrée aucune ligne et n'avance aucun autre cutoff.

List annonce le fuseau, next_at, et la règle des heures inexistantes sautées. Aucun nouveau réveil ni alerte supplémentaire pour ce cas. Halt-delegations --cancel-active ne vise que les délégations ; il n'annule pas les programmations. Après purge d'un schedule annulé, target_unknown ne prouve pas que l'annulation précédente a échoué ; relire list. Pas de reçu administratif infini.

Les bornes64 sont distinctes :64faits par abonnement,64jobs dans la file de notification existante,64résultats maximum dans le canal de retour. Tous les pendingfacts restent inclus dans la borne globale8192.


Chaque fait structuré sérialisé est limité à16384octets, avant insertion. Un dépassement enregistre seulement une lacune propriétaire datée et son compteur, sans copie du corps ni troncature silencieuse ; la consommationonce et la lacune sont committées ensemble. La borne mémoire ne repose pas sur HOSTNAME, qui peut être arbitrairement long. Les8192faits incluent les pendingfacts.

Les occurrences réutilisent la garde du circuit-breaker existant lors du départ agent, avec le propriétaire attesté comme expéditeur logique : valeur par défaut8échanges/180s par couple, sans quotaB supplémentaire. Un refus reste différé dans sa grâce initiale, puis manqué seulement avec absence099prouvée. L’admission des128programmations ne garantit pas que128remises simultanées seront acceptées.


Le résumé est un agrégat borné≤16384octets : compteurs par type, période et lacunes, jamais concaténation des8192faits. Les détails restent paginés par missed. Au retour, enregistrer l’épisode, puis attendre que les pending_write du propriétaire soient soldés (deadline initiale5s, aucun renouvellement) avant de figer cutoff/génération/résumé. Un ACK ne peut donc pas consommer le seq d’un pending qui échouerait plus tard. Un fait entré après le gel garde un seq supérieur et attend le prochain retour.


FR104 conserve self/WritesOwnState sousDiscovery. À l’admission, une origine native connue est gelée avec sa Task et son instance ; aucun repli vers propriétaire ordinaire si cette preuve disparaît. Au gel et au vrai départ, DelegationStore::for_child et les preuves existantes revérifient la tâche et les ancêtres actifs, cleanup_done=false, instance/mission/exécution compatibles. Fin de vie connue : annuler les futures occurrences et solder les préparées non parties comme owner_unavailable. ErreurSQL ou preuve manquante ne devient pas une autorisation ; fermer le départ. Aucune programmation ne lance un processus ni une nouvelle ronde native. Le booléenCallerScope=false après cleanup n’est pas une preuve de droit au réveil.

La reprise d’un résumé relit send_delivery_phase099 par sa clé figée : send_delivery seul masque les phasesacked. Le callback de cutoff intervient aprèsOk d’acknowledge_send_delivery, même quand son returned answered_request vautNone. CAS owner/génération/delivery_id/cutoffs et UUID de résumé gelé empêchent un ACK ancien de toucher un owner recréé.

## Snapshots serveur et parcours CLI — raccord API final

Schedule et FrozenWake portent owner_project: Option<CommunicationProject>,
snapshot serveur d’admission revérifié au départ, jamais une autorité fournie
par le client. Schedule et CalendarSnapshot portent next_calendar_date:
Option<String>. Au changement de fuseau, conserver la date civile du next_at
déjà dû ; seules les échéances futures non gelées utilisent la nouvelle zone.
Le fingerprint/CAS et la date civile précédente gardent leurs règles existantes.

Le scan des pending CLI est paginé par 128 lignes, curseur `(issued_at,id)` en
RAM seulement. Il avance aussi lorsqu’une ligne reste Deferred et est remis
au début à la fin du parcours. Aucune nouvelle table, quota ou durée de grâce.
