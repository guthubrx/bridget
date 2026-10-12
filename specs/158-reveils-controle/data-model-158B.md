# Modèle de données 158B

Candidat cache, baseA livrée. Les champs ci-dessous sont de domaine ; les noms Rust définitifs restent à vérifier par AuditExisting. Pas de schéma appliqué.

## Cibles et cadence

WakeTarget est un objet fermé : self ; agent avec UUID ; thread avec UUID. self est résolu au daemon depuis l'identité attestée. WakeCadence est fermé : interval avec seconds≥60 ; at avec HH:MM et liste non vide de jours1..7 sans doublons. Le CLI unitaire n'ajoute pas une cadence récurrente au magasin.

## wake_schedules — seule table des programmations

| Champ | Règle |
|---|---|
| schedule_id | UUID stable créé à l'admission. |
| owner | Identité authentifiée, jamais champ client. |
| owner_project | `Option<CommunicationProject>` : snapshot serveur à l’admission, repris dans FrozenWake et revérifié au départ ; jamais un argument d’autorité client. |
| target | Cible canonique résolue, conserve self comme portée déclarée. |
| cadence | Intervalle/ancre, ou heure/jours/fuseau annoncé. |
| label | UTF-8≤2048 octets, jamais tronqué. |
| next_at | Échéance UTC persistée. |
| next_calendar_date | `Option<String>` dans Schedule et CalendarSnapshot : date civile associée au next_at, conservée si cette échéance est déjà due lors d’un changement de fuseau. Les futures échéances non gelées sont calculées dans la nouvelle zone. |
| previous_calendar_date | Dernière date civile figée pour heure fixe ; pas reset au changement deTZ. |
| occurrence | Option d'un FrozenWake ; une seule non soldée par programmation. |
| cancelled_at | Annulation persistante ; empêche nouvelles occurrences. |

Limites :16 lignes par owner,128 au total, annulées comprises. Une annulation ne retire pas une contribution déjà committée. Une remise déjà partie conserve son résultat 099. Aucun historique illimité d'occurrences dans cette table.

freeze_wake_occurrence compare schedule_id/expected_next_at et absence de pending sous transactionImmediate. Elle fige key,due_at,issued_at,canon,operationUUID102 éventuel et prochaine échéance en un commit. Deux ticks concurrents ou une reprise ne créent pas deux occurrences. Une occurrence non terminale bloque la génération suivante. outcome_unknown est terminal sans confirmer la remise ; après clôture, next_at est avancé strictement dans le futur sur son ancre pour coalescer les retards.

## FrozenWake — preuve durable commune

source fermé : scheduled,cli_once,missed_summary. Clé, cible canonique, owner/projet attestés nécessaires, due_at, issued_at du daemon, grace_until, label/body figé, canonversion et bytes, UUID de dépôt 102 éventuel, référence de remise 099 et phase. Ces données ne sont pas des champs d'autorité acceptés du modèle. `FrozenWake.owner_project: Option<CommunicationProject>` conserve le snapshot serveur d’admission ; les gardes de départ le revérifient.

Phases : prepared→deferred→committed_thread ou dispatching_agent→confirmed/outcome_unknown. missed_before_departure seulement quand aucun départ n'a eu lieu et sa grâce a expiré. Une erreur d'écriture ou un pending après crash ne prouve pas cette absence. Les phases publiques distinguent cette incertitude.

Pour le CLI unitaire, la preuve récupérable vit dans canonical_bytes du reçu 099 sous un domaine JSON fermé version1 réservé à B. Rejeu retourne également issued_at du record. Aucun payloadcolumn/table supplémentaire. Le domaine historique canonical_send reste byte-identique. begin_send_delivery conserve les octets BridgetMessage et son ledger existant.

Pour un fil, UUID102 différent de la clé publique wake:schedule:epoch, mais lié et figé dans FrozenWake. Commitpost+intentions 102 est la preuve de remise. Crash après commit et avantsettle B : rejouer exactement operationUUID/canon 102 et enregistrer le reçu déjà présent. Pas transaction tenue entre les deux connexions Store/Idempotency.

## observation_missed — faits et provenance

| Champ | Règle |
|---|---|
| fact_id / fact_seq | UUID stable pour callback ; seq SQLite AUTOINCREMENT durable pour cutoff, jamais réutilisé. |
| owner | Owner attesté avant disparition once. |
| subscription_id | Abonnement d'origine ; ne requiert pas qu'il existe encore. |
| observed_at / retained_until | UnixUTC ; durée7jours depuis fait, pas depuis lecture. |
| fact | Type événement et champs structurés déjà autorisés ; jamais provenance tirée du body. |
| phase | pending_write, missed_before_write, write_unknown. |
| failure_code | Motif fermé connu, pas exception/secret arbitraire. |

Les phases internespré-écriture comptent dans les mêmes8192/64 afin de borner aussi le pipeline. Un succès workerwrite solde l'attente de notification ordinaire ; il ne prouve pas lecture modèle. Une reprise des pending crée write_unknown, pas missed_before_write. Les faits retenus par US7 ne sont pas supprimés par READ/ACKdepage.

## observation_missed_owners — métadonnées nécessaires

Owner row : owner, last_fact_or_gap_at, gap_count et période, return_episode/return_generation, absence/DND précédents, confirmed_fact_seq/owner_gap_version/global_gap_version et compteurs confirmés, summary_pending Option<FrozenWake>, summary_delivery_id éventuel. Maximum8192 owners. TTL7jours après dernier fait/gap, sans prolongation par read. Tant que des faits ou un résumé non terminal restent, le propriétaire est protégé.

Global row unique de type distinct : compteurs et périodes agrégés de couverture évincée, pas UUIDowner/subscription/body. Elle ne reconstruit pas un fait ou une attribution supprimée. Counter d'éviction de faits et perte de couverture ne sont pas additionnés comme deux faits nouveaux.

Entretien mêmeStoreTX : expirefaits, aggregate gapowner, puis expiremetadataowners admissibles. En saturation, retirer seulement le plus ancien propriétaire sans faits et sans résumé non terminal. Si les 8192 propriétaires sont protégés, enregistrer une lacune globale datée dans la même transaction, sans supprimer leurs faits ni créer une 8193e ligne. Faits≤8192,owners≤8192,global=1. Le snapshotabonnement262144octets existant reste borné ; il n'est pas une seconde copie du magasinmissed.

## Transactions et reprise

- Une transactionStore sauve la consommationonce et les pendingfacts avant enqueue ; en panne, pas signal de succès.
- Le callbackworker est corrélé par fact_id/owner ; si sa ligne a déjàétéévincée, ne la recréer ni effacer une ligne réutilisée. La trace d'éviction est déjà durable.
- Retour : choisir les faits/gaps nouveaux, figer cutoff et FrozenWake avant 099. Pendingcanon ne change pas quand arrivent de nouveauxfaits.
- ACK099validé : avancer cutoff sur la preuve de delivery_id/instance/génération ; replayACK n'avance pas deuxfois.
- Crash aprèsACK099avantTXowner : lire la preuve terminale 099 et appliquer la même avancée, jamais renvoyer une nouvelle génération.

Le compteur60CLI/min et INSERTprepared appartiennent à la même transactionImmediate 099 ; aucun count horstransaction ni horloge client. Pas nouvelle table de quota.

## Fermetures finales du contrat ROOT

**Fuseau observé.** Le daemon relève le fingerprint de la zone effectivement rendue par libc (nom de zone et offset) au tick et avant le gel. Un changement observé recalcule seulement les échéances futures non gelées. FrozenWake et previous_calendar_date restent inchangés. Le résultat d'un calcul hors verrou porte son fingerprint ; le CAS refuse un résultat devenu périmé. Aucune promesse de zone figée ou de changement uniquement au restart. Les changements DST du fingerprint n'autorisent pas une seconde occurrence de la même date.

**Retour du worker.** Le canal de résultat utilise try_send, borne64. Si le canal est plein, aucun résultat sans preuve n'est inventé : le pendingfact durable garde sa provenance et sa deadline initiale immuable. Le tick solde un pending dépassant cette deadline en write_unknown, même si l'écriture a pu réussir ; il ne le qualifie jamais de non-écrit. Aucun once recréé. Si Store ne peut pas écrire ce résultat, garder la ligne pending et l'annonce agrégée de panne disponible, puis réessayer à la maintenance ; jamais annoncer une sauvegarde réussie. Le compteur global existant reste une preuve aggregate limitée, pas un reçu durable par fait.

**Cutoff.** Chaque fait porte un fact_seq SQLite AUTOINCREMENT durable et jamais réutilisé, en plus de son UUID de corrélation. Le cutoff du résumé capture ce fact_seq, plus owner_gap_version et global_gap_version séparés. Un fait arrivé après le gel au même observed_at reste au-delà du cutoff. ACK099 valide avance seulement les trois valeurs gelées ; aucune valeur plus récente n'est consommée. Pas triUUID ou horodatage en secondes seul pour ce seuil.

**Création et annulation.** schedule ne prend aucun command_id et n'est pas idempotent à la création. Chaque admission crée un schedule_id UUID neuf côté daemon. Un retry aveugle peut créer un second schedule ; après réponse perdue, relire list au lieu de promettre une création unique. FR103 protège les occurrences du schedule déjà créé, pas cette création. cancel prend seulement ce schedule_id canonique ; sur la même programmation connue déjà annulée, retourne already_cancelled sans mutation ni nouvelle émission. UUID inconnu refuse. Pas second registre de commandes et aucun command_id administratif gratuit. Les --id CLI unitaires conservent séparément la déduplication099 du contrat.

## Borne des programmations — décision ROOT

Limites physiques :128 lignes au total et16 par propriétaire, y compris les programmations annulées. Une admission peut retirer, dans sa transaction, les plus anciennes lignes annulées sans occurrence en attente ou en vol : d'abord celles du propriétaire si sa borne est atteinte, puis celles de la borne globale. Si aucune ligne n'est retirable, schedule_limit. Les preuves099/102 restent intactes. Cancel rend already_cancelled pour une ligne connue ; après retrait borné, son UUID est inconnu et refuse. Aucun historique perpétuel d'annulation.


## Transitions finales158B R1

Les règles finales de pause, départ099 distinct du commit102, unknown terminal, deadlineworker5s/budgetI/O1s, horizonCLI7jours, attribution du label et entretien des résumés sont définies dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/contracts/wake-158B.md. Elles priment les descriptions préparatoires ci-dessus. Aucun nouveau service, worker, registre ou scénario de confort.

Fait structuré sérialisé≤16384octets ; dépassement annoncé comme lacune datée, sans troncature ni copie du corps. Réemploi du circuit-breaker existant au départ agent, propriétaire comme expéditeur logique ; aucune nouvelle quotaB.

Le plafond16KiB est vérifié sur octetsJSON, y compris dansCHECKSQL via length(BLOB), pas nombre de caractères. Résumé agrégé≤16KiB ; gel repoussé jusqu’à clôture des pending_write owner dans leur délai5s initial, pour éviter de consommer un échec tardif sous le cutoff.

Origine native de programmation gelée à l’admission ; lifecycle revérifié au gel/départ par for_child et preuves de mission existantes. FR104self reste OwnState. Résumé : phaseacked relue via send_delivery_phase099, UUIDgelé protège owner recréé, callback aprèsACKOk mêmeanswered_requestNone.

## Bornes de lecture — raccord API final

Le scan des pending CLI traite 128 lignes par page. Le curseur `(issued_at,id)`
est transitoire en RAM, avance aussi sur les Deferred et repart du début à
la fin du parcours. Il ne modifie ni les reçus, leur échéance ou le quota.
Missed prend 20 faits par défaut, 64 au plus, avec after_seq dans le domaine i64.
La réponse JSON complète est bornée à 65536 octets ; la page contient seulement
des faits entiers, chacun ≤16384 octets. Aucun fait n’est tronqué pour remplir
une page. Le seuil de lecture ne consomme ni le fait ni le cutoff d’ACK.
