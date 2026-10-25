# Plan 158B — réveils et faits manqués

Candidat d'extension des artefacts canoniques158. ROOT seul applique. Base code A :80b43407936110526b28f2c38ea72e1b34055154 ; livraison documentée :9a94334a109b4d5f939a8ebea464530abc28f4d6. Les22 tâches A restent cochées et inchangées. Tâches B après AuditExisting PASS et contre-revue du plan ; Analyze compare ensuite Spec/Plan/Tasks avant implémentation. US5,US6,US7 sont intégralement couverts. US8 reste reportée.

## Contexte et gates

Rust workspace, rusqlite/SQLite WAL, serde, libc et uuid existent. Aucun ajout de dépendance. Même socket Unix, base du daemon, boucle1s, worker de notification et remise 099. Aucune interface HTTP ni autorité humaine MCP.

Constitution globale et ArticleXX relus. Réemploi avant création : pas scheduler, writer, outbox universelle, registre de quota ou service supplémentaire. Les types de domaine sont nécessaires aux trois chemins programmation/CLI/résumé. La table de métadonnées missed est justifiée par la borne262144octets du snapshot existant et ses8192 owners. Les nouveaux helpers doivent avoir une responsabilité cherchée dans l'existant et trois usages réels, sinon rester inline. Les futurs mainteneurs disposent des transitions, SQL et points de confirmation ci-dessous.

La compétence speckit-plan est appliquée à la conception seulement. Aucun setup-plan.sh ni agent-context updater n'est exécuté : ils muteraient le projet hors du périmètre CACHE autorisé. Aucun hook extensions.yml n'est présent dans le checkout relu. Les artefacts cache sont les propositions à intégrer sousspec158, pas un remplacement du plan A.

## Modules et ownership à affecter après gates

| Fichier existant ou de domaine nécessaire | Responsabilité B |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs | DTO cible/cadence/WakeRequest et résultat fermés ; pas enveloppe réseau neuve. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wake_schedules.rs | Types/calcul civil et orchestrateur du réveil ; pas thread. Nouveau module de domaine justifié par FR101, pas framework. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/wake_schedules.rs | SQL des programmations via Store existant. Migrations additives idempotentes et CAS. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/observation_missed.rs | SQL faits/métadonnées sur même Store et transactions bornées. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs | reserve_cli_wake : rejeu/collision, COUNT et insertion dans une seule TXImmediate ; getter issued_at/canon existants. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs | Provenance typée, once et missed ; filtres/silence volontaire conservés. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs | Hooks tick/Register/DND, départ gardé, feedback du worker et ACKrésumé ; pas nouvelle boucle. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/referent_control.rs | ScheduledWake dans garde autonome unique ; halt A inchangé. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs et mcp_scopes.rs | Outil wake, action missed, classification self et tests exhaustifs A étendus. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs et communication/client.rs | Commandes wake et réemploi négociation/identité/contrôle humains. |

Ne pas rendre threads::post public. threads::handle_with_change est le point pub(crate) à réemployer. Les modules Store relèvent du magasin actuel, pas d'un service distinct.

## APIs et mutations minimales

- freeze_wake_occurrence(schedule_id,expected_next_at,frozen,next_at) : CAS et TXImmediate, une seule occurrence en attente par programmation.
- reserve_cli_wake(key,canon,now) : TXImmediate ; rendre issued_at enregistré au rejeu ; mesurer COUNT réel avant index/cache. OperationKind::Send conservé.
- Le getter des pending CLI ne décode pas canonical_send historique. Domaine JSON fermé FrozenWake B dans canonical_bytes du scope CLI réservé ; validation version/type/provenance à la relecture. Scan par 128 lignes, curseur `(issued_at,id)` transitoire en RAM, avancé aussi pour Deferred et réinitialisé en fin de parcours ; aucun quota ou état durable nouveau.
- Store conserve en une TX le snapshotonce consommé et les pendingfacts. Le changement n'est pas un append du corps modèle dans un journal universel.
- Retour worker : référence fact_id/owner et issue typée written/failed_before_write/write_unknown. Canal de retour borné64, drainé au tick ; aucun callback ne prend le verrou daemon depuis le writer. En saturation, try_send ne bloque pas : pending durable conservé ; tick solde write_unknown à deadline initiale sans preuve. Voir fermeture finale.
- settle_summary_ack(delivery_id,instance,generation) : seulement après acknowledge_send_delivery validé ; même génération/cutoff durable rejouable. Crash entre ACK099 et TXowner : relire phase acked 099, pas réémettre.

## Déroulement complet

1. AuditExisting sur base A livrée ; comparer les30oracles historiques au seulFULL final B exécuté par ROOT. Après AuditExisting PASS et contre-revue du plan, générer les tâches T101+ ; Analyze compare ensuite Spec/Plan/Tasks et doit passer avant implémentation. Aucun nom de scénario supplémentaire ici.
2. Fondations fermées, migrations additives et gardeScheduledWake. Mesurer réserve/quota et calendrier borné avant toute optimisation.
3. Freeze des occurrences et réemploi 099. SQL et octets canoniques avant transport ; reprise ne refait ni ID, ni horloge, ni UUID102. Une seule attente parschedule ; les intervalles en retard se coalescent.
4. Fil via handle_with_change : owner attesté membre/projet,kindNone,label2048+préfixe,UUID102 persisté. Remise=post+intentions commit 102. Pas expiration10 min sur WakeRow partagée.
5. Agent direct : différé au plus600 s avantdépart, garde/route/DND/capacité fraîches ; System/TriggerTurn produitsdaemon. Après départ, ACK099 ou unknown honnête, jamais fausseabsence.
6. CLI unitaire, quota durable et administration humaine. Filheadless exige BRIDGET_AGENT_ID_FILE et BRIDGET_AGENT_INSTANCE_ID valides, même frontière que CLIthread. Sans identité membre, refus avant réservation. Aucun --from.
7. Outil MCPunique bridget_wake et action missed ; tableScopes exhaustive, self propre/gestionOwnState, autre agent ou fil Sends. readOnlyHint du wake=false. Droits/propriété réévalués côtédaemon.
8. Faits/provenance/worker, entretien8192 faits/8192 owners+1global, résumé1parretour avec ACKcutoff. Pas suppression par lecture ; once disparu n'efface pas owner.
9. Revues/audit/Converge et preuves B, puis livraison ROOT. A garde son reçu et ses22 tasks ; aucune prétention de succès B par les tests historiques.

## Cohérence des transactions et crashes

Freeze/next_at est un commitStore. Réservation 099 et begin_send_delivery ont leurs propres transactions sur autreconnexion du mêmeDB. Ne tenir aucune transaction pendant appel d'une autreconnexion ou socketwrite. Ces frontières sont récupérables par IDs/canons gelés. Le post 102 rejoue son UUID déjàpersisté après crash. Ce n'est pas une garantie d'exécution exactementunique chez un fournisseur.

CLI : lookupd'abord, canon serveur original avant comparaison, puis COUNT+INSERT atomiques. Timestamp client interdit pour quota. Le scope réservé distingue unitairesCLI et autresenvois ; max60 nouveauxID/min. Refus invalides/préadmission ne consomment rien.

Cancel d'une programmation interdit les nouvellesoccurrences ; pending nonparti peut être soldé. Aprèspost 102 ou départ 099, conserver résultat/unknown. Aucun retrait rétroactif de contribution ou faux ACK.

## Dix groupes d'oracles prévus, sans tâches ni noms nouveaux

| Groupe | Obligations exactes |
|---|---|
| 1 | Intervalle≥60s, label≤2048octets,16lignes par owner et128global et objets fermés. |
| 2 | Heurefixe, jours, trou/fold, minimumUTC, changementTZfutur, dateprécédente. |
| 3 | Retardintervalle unique,freeze/restart/idempotence aux frontières de commit. |
| 4 | Pause au vrai départ ; haltA sans effetimplicite réveil. |
| 5 | Agentoffline/DND/capacité et limite600 s avantdépart, unknown aprèsdépart. |
| 6 | Filowner/membre/projet,UUID102,commit et conservation WakeRow ordinaire partagée. |
| 7 | CLI60IDs/min,rejeu/collision/durabilité et identityfilheadless. |
| 8 | Provenanceonce,échecwriter et callbacktyped sans parsingbody. |
| 9 | TTL/overflow8192 faits8192 owners,gapglobal et pertesdatées sans crossowner. |
| 10 | Register/DND mêmeépisode,cutoff immuable,ACK099corrélé,lecture non destructive. |

Réemploi des trente oracles historiques : /Users/moi/.cache/bridget-plan158B-tests-preparation-20261012/oracles-reuse-baseline158B.json. Fixtures existantes seulement ; ne copier ni SIGKILL ni cleanup process-group. ROOT seul exécute les RED/GREEN, compatibilité et mesures nécessaires. Pas nouveau banc long ou scanner.

## Limites et post-design

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

Origine native de programmation gelée à l’admission ; lifecycle revérifié au gel/départ par for_child et preuves de mission existantes. FR104self reste OwnState. Résumé : phaseacked relue via send_delivery_phase099, UUIDgelé protège owner recréé, callback aprèsACKOk mêmeanswered_requestNone.

## Snapshots et pagination — raccord API final

Schedule/FrozenWake : owner_project: Option<CommunicationProject>, snapshot
serveur à l’admission et recontrôle au départ. Schedule/CalendarSnapshot :
next_calendar_date:Option<String>, conserve la date civile du next_at déjà dû
si le fuseau change ; futures échéances non gelées recalculées en zone nouvelle.
Missed : défaut 20, max 64, after_seq en domaine i64 et JSON complet ≤65536 octets,
facts entiers ≤16384 octets sans troncature. Ces bornes complètent les mêmes
oracles et les mêmes tâches ; aucun scénario, module ou helper supplémentaire.
