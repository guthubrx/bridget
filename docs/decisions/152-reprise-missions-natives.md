# Décision152 - pause et reprise des missions natives

Date : 2026-10-11. Statut : **Accepté**.
Périmètre : session152. Les validations de compatibilité et de suite complète
restent des gates distincts. Cette décision ne vaut pas reçu de livraison.

## Contexte

Le moteur natif148/149 crée une tâche, un enfant et une mission corrélée.
Le socle150 règle les délais et la rétention. Le contrat151 ajoute l'attente
et la déclaration de lecture, sans effet sur la remise des messages.

Avant152, une interruption de pause pouvait faire échouer la mission.
Une panne faisait échouer une mission engagée sans réponse capturée.
Une annulation pouvait arrêter certains membres avant de découvrir un
membre non géré. Un refus pouvait changer entre deux rejeux.

La session152 corrige ces comportements dans les composants existants.
Elle garde les identifiants de tâche et de mission. Elle réutilise la
chaîne d'exécutions de l'ADR027. Aucun outil, paramètre d'appel, module
Rust ou dépendance nouvelle n'est ajouté.

## Décisions D1 à D22

Repères des réalisations :

- N : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs.
- D : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs.
- E : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs.
- C : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs.
- T : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation.rs.
- F : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/fleet.rs.

Les noms de fonctions sont les repères durables du code. Les captures
de tests gardent leurs lignes et leurs SHA historiques.

| Décision | Choix retenu et réalisation |
|---|---|
| D1 | La suspension est dérivée de la pause et de l'état retenu. N `current_task_view` ajoute `suspended` ; `task_view` reste pur. Aucun nouvel état métier. |
| D2 | N `tick` vérifie la pause avant l'échéance et le lancement des tâches `queued`, `starting`, `mission_pending` et `working`. L'annulation et les résultats déjà acquis restent autorisés. |
| D3 | C `AutonomousEffect::NativeMissionDispatch` partage la garde de `NativeDelegation`. Admission et remise sont deux effets nommés, avec une seule règle de pause. |
| D4 | N `rejected` rend `Result<bool, String>` et qualifie le seul refus `tour annulé par le pilote` couvert par une interruption de pause (`Ok(true)`). Les autres refus gardent leur effet d'échec (`Ok(false)`). Une erreur de lecture/persistance est propagée ; D ne retire aucun suivi et n'envoie aucun avis pour ce `Err`. |
| D5 | Une exécution `interrupted` due à la pause ne fait pas échouer la tâche native. N `pause_covers` conserve cette distinction. Une erreur SQL de qualification diffère le tick ; elle n'est jamais traitée comme une absence de pause. |
| D6 | La reprise après pause passe par `tick` et N `may_resume`. Elle réutilise `restart_identity` et exige l'instance et la commande du wrapper géré courant ; le checkpoint Task peut encore porter l'ancienne génération. Une interruption de pause doit être `interrupted`, jamais encore `interrupting`. L'ordre des événements de levée ne décide pas seul de la reprise. |
| D7 | E `latest_in_chain` remplace la lecture limitée à l'exécution initiale. La même mission suit ses continuations dans le moteur natif et la reprise ordinaire. |
| D8 | Le `frozen_secs` du suivi est reconstruit depuis la date durable de la demande et C `pause_overlap`. Une reconnexion ne perd ni ne double le gel. |
| D9 | N `spawn_task` sauvegarde `launch_issued_at` avant la première soumission. Une saga déjà soumise garde son enveloppe et son délai de 300 s. Le lancement retenu avant soumission reçoit une date récente à la levée. |
| D10 | N `prepare_restart` conserve `starting`, `mission_pending` ou `working` si identité, droits, délai effectif et borne autorisent la reprise. Seuls les neuf refus métier nommés deviennent un échec ; les autres erreurs sont propagées. Résultats acquis et états protégés restent conservés. |
| D11 | N `native_child_env` réutilise l'environnement natif aux deux chemins de lancement. `native_launch_registry` revérifie révocation, sources et dossier avant une reprise. Extraire l'environnement ne donne aucun droit de lancement. |
| D12 | D `classify_stop` classe sans mutation. `prepare_managed_stop` réutilise cette classification ; N `cancel_tree` peut vérifier l'ensemble avant le premier effet. |
| D13 | T `native_delegation_refusals` conserve seulement cinq codes fixes. Le premier reçu est immuable ; une tâche déjà admise prime. Tout autre code ou texte libre reste sans reçu152. |
| D14 | D `reserve_managed_recoveries` traite Starting et Connected avec filiation attestée. N `current_command` et F `ownership_from_desired` gardent cette filiation. La mission est reconstruite après quarantaine. Si Register précède Fleet, Connected retente le chemin existant après identité Fleet attestée, sur instance courante, connexion primaire et enfant libre, sans nouveau compteur. |
| D15 | `restart_resumes` est incrémenté et sauvegardé dans la décision de reprise avant l'effet. La borne vaut 2. Un crash peut sur-compter, jamais sous-compter. La seule levée de pause ne consomme pas ce compteur. |
| D16 | N `prepare_restart` lit l'échéance effective depuis le journal. Une exécution `interrupting` dont le processus a disparu est réconciliée en `interrupted`. Aucun `settle_suspension` ni décalage relatif durable. |
| D17 | N `rejected` rend `pause_interruption`. Le gestionnaire D `DeliveryRejected` garde le suivi et n'envoie pas d'avis d'échec pour cette seule interruption. |
| D18 | E `latest_in_chain` choisit la feuille de plus grande `generation`, pas la date la plus récente. N `may_resume` refuse une autre exécution de la chaîne. |
| D19 | N `request_frozen_secs` part de la demande durable. `mission_issued_at` garde l'enveloppe et l'origine du suivi, délai explicite ou omis. L'échéance brute se fige uniquement depuis `get_request(mission).created_at`. Une demande absente ou illisible diffère le traitement, sans repli sur une date de tâche ou d'exécution. |
| D20 | N `cancel_tree` classe tout l'arbre avant tout marquage de descendant ou arrêt. `cancellation_root` garde un seul coordinateur. Chaque arrêt confirmé est sauvegardé dans `cancel_stopped` avant l'arrêt suivant. Une erreur ou un résultat terminal reste conservé. |
| D21 | C `pause_overlap` utilise le journal `control_events` et l'index temporel ajouté dans le schéma existant. N `effective_deadline` calcule le délai sans réécrire la date brute. L'admission d'un descendant et `RequestWait` native utilisent ce délai ; le budget d'attente continue. |
| D22 | N `restart_identities` réconcilie le lien et l'instance, puis vérifie le marqueur natif lisible avant de publier l'identité. Une tâche irréconciliable échoue avec `native_restart_identity_mismatch` ; le daemon continue. Les marqueurs corrompus ou illisibles gardent le refus sûr de la flotte. |

## Changements nécessaires révélés par les recettes

La première fixture Starting ne sauvegardait pas encore la flotte désirée.
Son échec prouvait un enfant inaccessible, pas un candidat récupérable.
Le cas prévu réutilise désormais `register_connected_observed` : la flotte,
le bail et le lien sont durables avant le retour d'erreur qui laisse la tâche
Starting. Les gardes sont exercées sur Starting et Connected.

L'E2E de mission imbriquée a observé une mission Working distincte d'une
exécution Starting. La quarantaine de la remise ancienne empêchait sa
reconstruction. La correction minimale de D `schedule_execution_recovery`
réutilise la reprise d'exécution après cette quarantaine.

La fixture privée149 fournit le dépôt, le profil, le PATH et une source
réelle de permissions. Son fournisseur de test est un binaire privé
incorporant le Python déjà utilisé. Aucun snapshot de droits n'est injecté.
Le transport observé est `claude_stream_json`. Le puits natif l'admet avec
les mêmes contrôles d'instance, pilote, modèle, effort, révision et sources.
Le dernier E2E réussi couvre le descendant MCP après chaque reprise.

La recette T025 a corrigé trois erreurs de fixture : mauvaise instance,
contrôle après Registered non drainé, puis entretien omis après l'ACK SQLite.
Le dernier passage couvre le vrai résultat corrélé et sa remise unique.
Ces échecs restent dans les captures historiques.

## Compatibilité et oracles

Les formes d'appel et les octets canoniques148/149/150/151 restent identiques.
Aucun nouveau paramètre n'est requis. Les anciens payloads de tâche lisent
les champs nouveaux avec leur valeur par défaut.

La règle149 d'échec systématique après panne change volontairement
(FR-013/FR-014/FR-015/FR-027). Les tests concernés doivent garder leurs
contrôles de droits, sources, identité, filiation et états protégés.
Une incompatibilité attendue doit être expliquée par son FR ; elle n'est
pas un échec hérité. Un défaut de production reste un défaut à corriger.

Les cinq oracles149 F2 de fermeture d'exécution et de remise restent stricts.
La correction ferme l'exécution native perdue sans bloquer le résultat acquis
du parent. Deux gardes restent aussi strictes : `may_resume` exige l'identité
réconciliée et le wrapper courant ; `prepare_restart` propage l'erreur nommée
du propriétaire d'un queued sans changer cette tâche ni fermer son enfant.
Les cinq autres cas149 sont les seules adaptations de contrat autorisées.

La vue courante ajoute `suspended` et `resume_attempts` (FR-024).
L'oracle148 compare strictement cette vue avant et après admission, puis
après réouverture. Il garde les identifiants, le modèle gelé et les droits.
Il ne la compare pas à la projection pure qui n'a pas ces deux champs.

Les quatre adaptations150 restent dans les cas existants : origine du suivi
depuis `mission_issued_at` avec délai explicite ou omis (FR-010), trigger
d'écriture conservé sur un suivi divergent, lancement seulement après la
levée (FR-008), égalité stricte avec `suspended: true` pendant la pause (FR-024).

Le contrat151 reste une lecture pure. Le budget `wait_timeout`, les limites
d'attente, l'annulation/EOF, le résumé figé et l'accusé de lecture ne changent
pas. L'accusé de lecture ne prouve pas la remise et ne débloque pas la purge.
Lineage garde sa forme, ses clés et ses états.

## Conséquences et limites

- Les dates brutes restent stables. Le délai effectif exclut les pauses.
  Il est reconstruit depuis le journal, y compris après un crash avant tick.
- Un lancement déjà soumis n'a pas d'horloge gelée : ses 300 s continuent.
  Une pause ne réécrit pas une saga de lancement.
- La reprise reste bornée. `native_resume_exhausted`,
  `mission_reply_timeout`, `unreachable` et les refus de droits donnent
  des causes explicites. Une mission annulée ne ressuscite pas.
- Un membre non arrêtable au précontrôle produit zéro arrêt et zéro marquage
  de descendant. Un arrêt peut encore échouer après ce contrôle. Les
  confirmations acquises restent durables ; seuls les arrêts non confirmés
  sont rejoués. Ce contrat ne promet pas des arrêts de processus atomiques.
- Un marqueur illisible n'est ni réparé ni supprimé par152. Les protections
  de groupe et de naissance du processus restent en place.
- FR-029 couvre commande, instance et lien natifs dans un marqueur lisible.
  Une cible d'exécution étrangère, une erreur de CAS ou de stockage peut
  bloquer le démarrage. Les contrôles149 restent stricts.
- Après les arrêts confirmés, les clôtures métier peuvent rester partielles
  sur erreur de CAS. La boucle d'arrêts est alors déjà terminée.
- Les cinq reçus de refus restent à exactement 30 jours. L'entretien
  supprime ceux qui sont plus anciens. Aucun réveil d'attente ou changement
  Lineage n'est produit par ces seuls reçus.
- `latest_in_chain` annonce O(c × (log n + log c)), espace O(c),
  pour c maillons sur n exécutions/liens. ORDER BY/LIMIT 1 garde une seule
  feuille, coût O(c). `may_resume` inclut le coût de réconciliation d'identité.
  `pause_overlap` annonce O(log N + P) avec son index temporel.
  Le précontrôle local de l'annulation
  annonce O(m + f + p + n log n), hors lectures de descendants/liens et
  marqueurs. Les sauvegardes cumulées représentent O(n²) octets. Ce sont
  des analyses du code, pas des mesures de performance.

## Preuves et état de validation

Les tranches restent séparées. Aucun total de workspace n'est déduit de
leur addition.

| Domaine | Preuve actuelle |
|---|---|
| US1/US2 | 17 PASS R2 puis 1 PASS T019 R3 ; oracles de pause, délais, lancement et résultat conservé. |
| US3 | R6 E2E : 5 PASS. Deux redémarrages réels de daemon privé, filiation et descendant MCP après chacun. Gardes R3 : 2 PASS contenant sept sous-cas Starting/Connected. T025 R4 : 1 PASS avec résultat corrélé remis une fois. |
| US4 | 4 PASS annulation, 2 PASS précontrôle et témoin148 distinct de 1 PASS. Le crash de la fixture est un vrai échec SQL après confirmation, puis réouverture et Desired Stopped sans superviseur. Les arrêts sont simulés ; aucun PID ou SIG réel n'est prouvé par ces unitaires. |
| US5 | 15 PASS : magasin, admission, normalisation du registre et entretien. Le reçu reste à 30 jours et disparaît à 30 jours + 1 s. |
| Compatibilité T032 | R1 : 217 PASS/16 FAIL/1 ignored, sans waiver. R2 : native149 65 PASS/7 FAIL et native150 51 PASS/0 FAIL. R3 : six filtres de référence, 232 PASS/1 FAIL/1 ignored ; les six intégrations149 ne sont pas exécutées après l'échec du lib. R4 : 78 PASS149. L'agrégation compatible R3 avec149 remplacé par R4 donne239 PASS/0 FAIL/1 ignored, en plusieurs commandes. T032 reste ouverte jusqu'aux sources finales et à leur comparaison. |
| Complément native152 | R3 : 51 unitaires PASS puis E2E 4 PASS/1 FAIL. R4 : 51 unitaires PASS avec SC-002/SC-003 renforcés. La reprise imbriquée E2E reste en diagnostic ; aucun GREEN final n'est revendiqué. |
| R5 final ciblé, avant patch de demande absente | Sept filtres : 295 PASS/0 FAIL/1 ignored, RC0 chacun. Les six filtres compatibles donnent239 PASS/0 FAIL/1 ignored comme la référence ; native152 donne51 unitaires +5 E2E PASS. T032 est clôturée. T034 est clôturée après relecture des coûts et du minimalisme. |
| Suite complète R1, avant patch de demande absente | 94 cibles extérieures, 2103 PASS/8 FAIL/72 ignored, RC101 ; mêmes cinq cibles et huit noms que W0041512046/8/72. Aucun succès global. T036/T037 restent ouvertes avant les preuves finales. |
| Validation finale | Suite complète, relecture, Converge, audit et livraison restent des gates distincts ; aucun runtime de production ou modèle réel n'est validé par cet ADR. |

Les E2E utilisent un fournisseur de test privé. Ils exécutent les vrais
chemins de daemon, flotte, protocole et MCP ; ils ne constituent pas une
recette de modèle réel ni une activation de production.

Contrat et plan :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/contracts/reprise152.md.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

Captures de référence :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r3-t019-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-gardes-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r6-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r4-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us4-152-r1-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us5-152-r1-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r1-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r2-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r3-results.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r4-results.json.

Les anciennes captures restent conservées. Git, installation sur disque
et processus chargés doivent être prouvés séparément à la livraison.
L'activation est coordonnée par le principal sans interrompre les autres
fils T3 ; aucun redémarrage T3 n'est autorisé par cet ADR.

## Correctifs terminalement retenus avant la validation finale

La contre-analyse rejette H1 dans sa lecture trop large de FR-029.
Les contrôles149 d’autorité et de cible étrangère restent stricts.
M2/M3/M4 sont corrigés dans les chemins existants : erreur de pause
propagée ou différée, neuf refus métier nommés seulement, origine
d’échéance dans la demande durable. M5 reste une réserve LOW sans patch :
le précontrôle sans effet et l’échec partiel après celui-ci sont distincts.

Connected retente la reconstruction après identité Fleet attestée si
Register est arrivé avant celle-ci. Les gardes portent sur l’instance
courante, la commande gérée, la connexion primaire, la route et l’état libre.
La même fonction de reprise est réutilisée ; aucun nouveau compteur.

R5 est en cours sur ces sources gelées, après les captures historiques
R3/R4. Aucun GREEN final, APPROVE de revue, audit, modèle réel ou activation
de production n’est déduit de cette seule relecture.

## Mise à jour après R5, suite R1 et patch de demande absente

R5 termine la tranche ciblée :295 PASS/0 FAIL/1 ignored, dont239/0/1 pour
la comparaison des six filtres et51 unitaires +5 E2E152 PASS. T032/T034 sont
clôturées après lecture des coûts et du minimalisme :48/50. Quatre oracles149
ont réellement changé ; le cinquième témoin reste byte-identique et strict.

La suite complète R1 donne94 cibles,2103 PASS/8 FAIL/72 ignored, RC101.
Les mêmes cinq cibles et huit noms sont en échec dans W004151. Aucun
succès global. R5 et la suite R1 précèdent le correctif None.

La revue R2 reste REQUEST_CHANGES. La limite H1 est explicitée sans modifier
FR-029 ni les gardes149 : cible étrangère, erreur CAS ou stockage peuvent
bloquer le démarrage. L5 est réfutée dans sa causalité : le continue de
clôture métier intervient après les arrêts confirmés. Une clôture CAS
partielle reste possible ; les confirmations d'arrêt restent durables.

Le cas Ok(None) est désormais traité dans les deux branches : clôture liée
par close_lost_execution avec gardes existantes avant la sauvegarde
failed/request_unavailable. SQL Err ou clôture refusée reportent sans
mutation de tâche dans ces branches. Le chemin avant admission reste intact.
Aucune date n'est fabriquée. Aucun nouvel outil, helper, test autonome ou ID.

R6 après ce patch :48 PASS/3 FAIL, RC101, E2E non exécutés. Trois oracles
US4 observent failed au lieu de working ; diagnostic fixture/produit en
cours, sans adaptation ni défaut produit déduit. T036/T037 restent ouvertes.
Aucun GREEN après patch, APPROVE final, Converge, audit ou activation.

Preuves actuelles :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.

Diagnostic R6 confirmé par les trois DB privées : aucune demande ou
exécution, échéances NULL ; Working était forcé sans admission dans les
fixtures. Les arrêts sont conformes et aucun défaut FR-018/FR-019 n'est
établi. La correction test-only autorisée réutilise create_request pour rendre
leurs données valides, sans changer oracle ou préfixe produit ni fabriquer
une échéance.
R7 puis098 constituent les contrôles finaux prévus après R1, chacun sur son
hash. Aucun GREEN R7 ni succès global déduit ; T036/T037 restent ouvertes.

R3 reconnaît H1/M2/M3/M4/L5 résolus. Son unique défaut retenu, pause ouverte
après request_unavailable, est corrigé par une condition dans le chemin
abandon_pause_interruption existant après les gardes cible/CAS. Aucune
continuation créée. R7 garde deux FAIL de données synthétiques sans Request ;
les deux fixtures reçoivent une admission durable réelle, oracles inchangés.
Le comptage antérieur de quatre adaptations149 est rectifié : cinq, O4 incluse,
plus le témoin supplémentaire byte-identique. R8/098 et T036/T037 restent ouverts.
