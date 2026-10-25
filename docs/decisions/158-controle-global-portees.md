# ADR158 — Contrôle global humain et portées MCP

Date : 2026-10-12. Statut : **Accepté**. Portée : 158A seulement.
Cette décision normative ne prouve ni implémentation, tests, activation ou
livraison158. 158B reste spécifiée seulement ; aucun nouveau service ou outil
n’est créé par cet ADR.

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/158-controle-global-portees.md#extrait-1).

Base préparée158A : bd37e927eee0b28f058df9d6a6992e38665a6ad0, sources157
8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b livrées. Réemploi avant création :
ControlStateV1, control_client_actor, cmd_control/terminal interactif,
human_cancel/tick, Task.effective_posture/cleanup_done et admission157.
Les reçus d’arrêt ne transforment pas control_events ni ses contraintes CHECK.

## Décisions R1 à R8

- **R1** : vue globale par le plan de contrôle, sans fil d’un client externe. Négociation puis
  capacité ControlStateV1 puis acteur humain attesté. La capacité seule et un
  alias from=humain ne donnent aucune autorité. Vue bornée à 100, sans corps de
  mission/résultat/chemins ; active reprend holds_slot, garde comprise.
- **R2** : états pause et arrêt séparés, une décision commune
  admit_native_delegation dans referent_control. Rejeu avant garde ;arrêt
  prioritaire si pause et arrêt coexistent. Ni pause ni arrêt ne lève l’autre.
  Delegate et Continue relisent cette garde après rejeu. Leur rejeu rend la vue
  courante de la tâche sans mutation ; suspended suit la pause courante. Cette
  vue est distincte du reçu historique de DelegationHaltSet. Les demandes enfant déjà admises gardent la pause seule ; queued et missions en cours restent intacts sans cancel_active.
  Les seules annulations demandées réemploient human_cancel puis tick.
- **R3** : une ligne persistante d’arrêt et 256 reçus portant la réponse historique.
  État et reçu sont committés ensemble. Un command_id connu ne réécrit aucun
  état et n’ouvre aucun parcours d’annulation. Cancelled compte les arbres
  ciblés avant commit, pas les arrêts confirmés. Après commit, annuler au plus
  une fois les seuls arbres retenus par la première application. Crash :
  annulation partielle possible, l’humain utilise delegations cancel. Un refus
  store_unavailable peut survenir avant ou après commit : la CLI signale que
  l’arrêt ou les annulations ont pu être appliqués, sans confirmer le commit.
  Consulter control delegations puis annuler les tâches restantes par
  delegations cancel ; un reçu rejoué n’émet aucune nouvelle annulation.
  Reçus au-delà de 256 purgés ; un geste CLI crée un identifiant neuf. Le rejeu
  d’un identifiant purgé peut être appliqué de nouveau, limite acceptée.
- **R4** : pas de budget de tours ou tokens en158A. Le compteur actuel compte des
  échantillons, sans identifiant de tour dédupliqué ; Codex n’est pas adapté et
  les rondes/reprises ne sont pas agrégées par tâche. Reply_timeout150 reste.
- **R5** : aucun nouveau dépôt de boîte humaine pour un budget absent. Le genre
  budget_reached existant reste réservé au futur besoin, sans nouvelle route.
- **R6** : table unique des six portées, par outil/action/cible, vérifiée par test
  exhaustif CI dans les deux sens et par cohérence readOnlyHint avec la BASE.
  Pas de portée « contrôle humain » dans le type. Journal : base sends,
  readOnlyHint faux, abaissement reads si to est absent ou null ; chaîne vide garde sends. Aucune obligation de compilateur
  n’est prétendue pour les déclarations Vec<Value> existantes.
- **R7** : plafond discovery calculé depuis la tâche de l’appelant, posture
  effective puis demandée et cleanup_done. Seuls reads/writes_own_state permis,
  même terminale avant nettoyage et entre rondes. CallerScope réemploie le RPC natif avec identité inscrite et l’index149 child ; aucune DB MCP. Toute panne RPC/SQL ou réponse invalide refuse fermé chaque appel avec effet
  ou lecture du daemon, sans repli standard. L’aperçu pur local103 de
  bridget_handoff action=preview réemploie le parseur strict et le rendu avant
  RPC : aucune DB, aucun identifiant créé, aucun transport. Reads étant permis
  pour toutes les postures, ce seul chemin ne lit ni tâche ni plafond. Le
  contrat « aperçu sans connexion » existant reste vrai ; aucune autre lecture
  dépendant de la socket n’obtient cette exception. Thread read/ack permis,
  post/create/add_members/close refusés. Journal avec to, publication, envoi,
  annulation de demande et toute délégation refusés sans effet. Lecture
  d’artefact et réponse finale automatique au parent restent permises.
- **R8** : réveils158B seulement préparés. Les primitives de boucle/notification
  seront réemployées après un plan propre. Un réveil reste un effet autonome ;
  droits de projet et de fil devront précéder admission. Aucun planificateur,
  commande wake, rejeu de faits ou consommation de secret n’est livré en158A.

## Amendement explicite de l’ADR027 et de la spec150

L’ADR027 conserve son état de pause et ses gardes. 158A étend sa garde au puits
NativeDelegation par une composition unique avec l’arrêt séparé. L’arrêt ne
suspend ni ne reprend une mission ;l’annulation active est opt-in, par geste
humain. Les options154/155 et la sélection/pinning/rejeu157 restent conservés.
Aucune CLI de contrôle n’est exposée au modèle par MCP.

La mention « budgets de tokens ou de coût (session158) » de la spec150, ligne77,
est levée. Elle ne vaut ni livraison de budget158A ni promesse158B. Reprendre
un budget de tours seulement avec : identifiant provider-neutre de tour ;
adaptateur Codex ; agrégation de toutes les exécutions/rondes/reprises ; succès
au tour N accepté et refus d’ouvrir N+1 ; évaluation à l’événement d’usage plutôt
que balayage récursif périodique. À défaut de ce dernier point, CTE avec LIMIT
et test de charge prévu dans la future spec prouvant la borne. Aucun compteur
ou test futur n’est ajouté pendant158A.

## Alternatives rejetées et conséquences

Rejetés : drapeau lineage pour ouvrir toute la vue ; capacité seule comme preuve
humaine ; réutiliser paused pour l’arrêt ; last_command_id seul ou rejeu qui
réannule une tâche C nouvelle ; nouveau genre control_events avec migration
CHECK ; réécrire toutes les déclarations MCP pour une garantie de compilateur ;
posture demandée seule ou terminal() pour le plafond ; tout fil interdit sans
read/ack ; compteur de tours actuel présenté comme budget fiable.

La vue est humaine et bornée. La garde unique réduit la dérive entre pause et
arrêt. Le test de CI détecte un outil sans portée, une entrée orpheline ou une
annotation incohérente. Les ruptures discovery sont explicites ; un besoin de
publication impose une nouvelle spec, pas une exception ajoutée en cours.

Limites : borne ADR011 inchangée, issuer_scope déclaré par le client, processus
hostile du même compte hors garantie ; test CI plutôt que compilateur ; reçus256
non assimilés à un audit historique ; annulation après commit au plus une fois,
partielle après crash ou refus de stockage/arrêt ; plafond4096 des reçus human_cancel pouvant refuser une cible ; cancelled n’est pas une confirmation d’arrêt.
Un magasin corrompu ou une lignée incohérente ferme le snapshot cancel_active
avant commit ; l’arrêt simple sans annulation reste possible si sa propre
lecture/écriture est disponible. La purge normale conserve les arbres jusqu’à
leur solde complet ; aucune perte normale d’ancêtre n’est établie par la revue.
Control status sort avec le code 1 si les données d’arrêt sont indisponibles, même après un
affichage partiel du contrôle087 : cet affichage ne vaut pas succès. Le coût
lookup est mesuré par T017 avant choix, aucune latence ou cache nouveau garanti
par cet ADR. Aucun succès global, fournisseur, modèle ou runtime158 prouvé ici.

## Références

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/spec.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/research.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/contracts/control158.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/150-socle-delegation/spec.md:77
- /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/011-approbation-exposee-puis-second-facteur.md

## Complément 158B — réveils et faits manqués

Date : 2026-10-12. Statut : **Accepté** pour la décision. Ce complément étend
la portée normative indiquée en tête. Les décisions et le reçu 158A restent
inchangés. Aucun code livré, test vert ou activation 158B n’est prouvé ici.
US5–US7 et FR101–107 sont couverts. US8 reste reportée faute de consommateur.

La décision R8 préparatoire est complétée par une greffe sur la boucle 1s,
Store, worker et registres 099/102 existants. Les trois modules de domaine/SQL
et tables wake_schedules, observation_missed, observation_missed_owners portent
les responsabilités auditées. Aucun service, thread, dépendance, second writer,
outbox universelle, quota RAM ou registre administratif supplémentaire.

L’unique outil bridget_wake expose schedule/list/cancel. Missed est une action
de bridget_events. Self et gestion propre sont WritesOwnState ; agent et fil
sont Sends. Base wake Sends/hintfalse, missed Reads. Identité, propriétaire,
projet et membre sont contrôlés aux puits. La CLI globale humaine réemploie
ControlStateV1 et son acteur attesté ; mutation avec TTY, aucun admin MCP.
La CLI unitaire headless n’est pas anonyme vers un fil et ne lance aucun modèle.
La borne ADR011 du même compte reste.

Schedule n’est pas idempotent à la création : réponse perdue→list. Limites
physiques 16/owner et 128 global, annulées comprises ; retrait transactionnel des
anciennes annulées sans pending/inflight seulement. Un UUID retiré refuse,
sans prouver que l’ancien cancel a échoué. Quota CLI 60 nouveaux IDs/min sous
TX099, replay/collision avant compte, horizon 7 jours depuis issued_at initial.
UUID102 et canon sont gelés avant réserve 099.

Pause interdit le gel d’une nouvelle occurrence. L’intervalle rattrape la
dernière échéance due une fois, avec prochaine strictement future ; une heure
fixe vieille de plus de 600s est sautée. Prepared conserve sa grâce 600s initiale.
Fuseau observé du daemon ; domaine offsets en minutes entières±24h, premier
UTC d’une heure double DST ou non-DST, heure inexistante sautée. La date civile
précédente et les occurrences gelées survivent au changement TZ. Aucune garantie
exactly-once fournisseur illimitée ni comparaison de produits mesurée.

L’origine native Task/instance est gelée et revérifiée au gel/départ avec
for_child et les preuves existantes. Fin connue : annulation des futures
occurrences, prepared non parti soldé owner_unavailable. Absence ou erreur SQL
ferme le départ, sans repli ni processus ou ronde ressuscités. La notification
System attribue le propriétaire logique et conserve le label comme donnée.
Circuit-breaker existant 8/180s par couple au départ, aucun nouveau quota B.

Frontières distinctes : agent dispatching099 avant écriture, puis ACK099 ou
unknown terminal sans renvoi ; fil commit contribution+intentions102 sous UUID
gelé, pas lecture et pas expiration 600s des alertes partagées. Aucune transaction
ne traverse plusieurs connexions ou I/O. La reprise conserve canon/ID/issued_at.

Once est consommé avec le fait/gap dans une transaction. Fait structuré≤16 KiB,
8192 globaux pending inclus,64 par abonnement,7 jours ; trop gros→gap daté sans
copie du corps. Provenance typée avant once, jamais déduite du body. Worker
unique : I/O 1s, deadline 5s non renouvelée, retour 64 ; unknown ne prouve pas
une non-écriture. Journal_only/throttle ne deviennent pas des pertes.
Métadonnées 8192 owners+1global fixe ; faits/pending non terminal protégés.
8192 protégés→gap global, aucune 8193e ligne. Évictions/TTL conservent des périodes
agrégées sans UUID ou corps étrangers ; pas une histoire attribuable illimitée.

Un retour Register/DND est un épisode. Le gel attend tous ses pending_write
soldés, puis fige UUID/canon/issued_at/génération/cutoffs. Résumé agrégé≤16 KiB,
jamais concaténation de 8192 faits ; détails par missed. Seul ACK099 validé avance
les seuils ; ni lecture ni unknown. Reprise par send_delivery_phase car
send_delivery masque acked ; callback après Ok ACK même answered_request=None.
CAS owner/génération/delivery_id/cutoffs et UUID gelé protègent un owner recréé.
Unknown terminal : prochain résumé seulement au nouveau retour réel, avec
incertitude précédente, jamais retry automatique de l’ancien canon.

Rejetés : scheduler séparé, second writer, quota RAM, auteur System non attribué,
TTL B sur WakeRow102, horaire client d’autorité, concaténation de 8192 faits, cutoff
avancé par lecture, parse de body, owner arbitraire ou secret fictif. Les
transitions exposent les coûts et l’incertitude. Portées garanties par CI,
pas par le compilateur ou un confinement OS. Aucun runtime, fournisseur,
performance ou recette 158B n’est attesté par cette décision.

Références :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/spec.md (partieB)
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/plan-158B.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/contracts/wake-158B.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/reuse-audit-158B.md
