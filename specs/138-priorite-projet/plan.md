# Plan 138 — Priorité au projet dans les échanges

Date: 2026-10-06
Branche: session-138-priorite-projet
Statut: Implemented ; 20/20 tâches terminées, contrôles globaux PASS, Converge pass1 CONVERGED et audit A du diff validé. Cinq MED de maintenance non bloquants restent ouverts.

## Contexte technique

Rust 2024, serde, SQLite/rusqlite et SHA-256 existent déjà. Agent Loop est en
Python et possède ses tests unittest. Aucune nouvelle dépendance ni aucun service
n'est requis. Les essais utilisent un BRIDGET_HOME, une socket et un target Cargo
distincts. Le travail non fusionné des sessions 135 et 136 reste préservé.

Le registre ProjectReference/runtime a été retiré. Le refus de lancement est
visible dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:12331.
La CLI produit encore project: None. Cette fonction ne réactive pas ce registre.

## Décision et architecture

1. Ajouter un fait de projet de communication à la connexion vivante. Le pont
   T3 annonce son workspace_root attesté. Le wrapper annonce son répertoire
   constaté. Le daemon canonicalise sur l'hôte attesté la racine commune Git.
   Les worktrees partagent cette racine ; les symlinks convergent. Hors Git,
   seule une racine T3 attestée peut servir de preuve. Absence, conflit, chemin
   inaccessible ou hôte non attesté rendent le fait inconnu. Le domaine mutable
   reste une étiquette. Le fait est réannoncé à la reconnexion et retiré avec elle.
2. Réutiliser live_connection_identity et register_auxiliary. Une connexion
   auxiliaire reçoit le projet de l'instance déjà attestée. Ni from, ni
   issuer_scope, ni un project_id fourni dans un message ne prouvent le projet.
   Annoncer ce fait par CommunicationProjectFact, distinct de Register, après
   l'inscription du transport. Seul le transport propriétaire vivant peut
   l'écrire. Une connexion auxiliaire en hérite sans pouvoir le remplacer.
   Un client CLI de fond établit son contexte sur sa propre connexion Client
   négociée, depuis la racine explicite du run et l'hôte validés par le daemon.
   Il ne crée aucun agent Register et n'emprunte aucune identité T3. Un run
   historique sans racine reste inconnu. Ce contexte ne remplace jamais celui
   d'une identité auxiliaire déjà rattachée.
3. Ajouter une variante d'annuaire de communication scoped. Le contrat
   ListAgents reste global pour les diagnostics existants. who/agents et les
   suggestions utilisent same_project par défaut ; --global est volontaire.
   Un émetteur inconnu reçoit zéro suggestion locale et un avertissement.
   Réponse CommunicationDirectory séparée: ScopedAgentInfo avec AgentInfo aplati
   et métadonnées communication_project/project_relation. AgentInfo ne change pas.
4. Dans communication.rs, partager une prévalidation de portée avec les façades
   et le daemon. Le verdict direct coûte O(1) après lecture des faits vivants.
   La résolution Git a lieu à l'annonce, jamais à chaque message sous verrou.
   Une audience de fil coûte O(M), M limité à 16. L'annuaire coûte O(A), une seule
   passe ; pas de requête Git ni de lecture disque par résultat.
5. Ajouter cross_project_reason: Option<String> à BridgetMessage et ThreadRequest.
   Sa présence exprime le choix interprojets volontaire. La CLI expose
   --cross-project-reason ; MCP expose le même champ. Valider une chaîne trimée
   de 1 à 512 octets UTF-8, sans NUL ni caractères de contrôle. L'absence conserve
   les octets canoniques historiques. La présence ajoute un suffixe canonique
   distinct. null explicite, champ invalide ou trop long sont refusés.
6. Étendre prepare_dispatch avant nouveau dépôt, demande suivie ou notification.
   Même projet: autorisé selon les contrôles actuels. Projets connus différents:
   motif explicite requis. Projet inconnu: compatibilité d'envoi avec warning.
   Le résultat caller reçoit les avertissements établis avant l'effet. Le corps
   du message reste exact. Une réponse n'hérite d'un motif que d'une demande
   réellement suivie, avec sens sender/target inversé et autorisation enregistrée.
7. Pour create/post, contrôler tous les membres qui peuvent lire, y compris
   notify=[]. Réutiliser l'audience immuable de 102 et la transaction actuelle.
   Chaque opération mixte porte son motif structuré. Le caller peut réutiliser
   le motif d'un mandat explicite pour les mêmes membres. Ne pas ajouter de
   colonne, table ou consentement implicite au fil.
   Un ancien fil mixte sans motif exige un motif lors du prochain dépôt, sans
   changer ni rejouer son historique. Le motif participe au canon de l'opération.
   Un refus ne change rien.
8. Réutiliser le client Unix partagé, les négociations de capacités et les
   résultats existants pour CLI/MCP/fédération. Les pairs portent le motif sans
   redéduire le projet depuis une étiquette. Un pair ancien ne doit pas ignorer
   un motif nouveau : le client exige la capacité138 avant de transmettre le
   message ; la capacité absente produit un refus explicite sans envoi. Une route
   extérieure sans fait attesté reste inconnue et avertie, pas supposée locale.
   Étendre aussi les façades handoff.rs (parse_request/HandoffTransport) et
   attach.rs (JournalRequest). Elles transmettent le même motif et résultat à
   la garde daemon existante. Aucun helper ni moteur de portée parallèle.
9. Préserver les opérations déjà acceptées et leur résultat durable. Consulter
   le reçu idempotent existant avant tout nouvel effet ou nouvelle validation de
   portée. Une opération ancienne déjà engagée termine selon son contrat initial.
   Aucun scan ni rejeu de file existante. Un nouveau corps ou motif sous la même
   clé constitue un conflit, même après redémarrage.
   L'outbox Agent Loop fige la racine de contexte, le motif et le destinataire
   avec le corps avant la première tentative. Toute reprise utilise cette
   enveloppe, même si le mandat du run change ensuite.
10. Agent Loop enregistre la racine de travail du run, distincte de run.domain.
    Il utilise l'annuaire scoped pour proposer et recruter. Vérifier le projet à
    attach-agent et avant dispatch. Aucun repli extérieur si les locaux sont
    absents ou busy. Les attributions explicites possèdent un motif borné par
    agent et rôle. Les envois de fond réutilisent cette preuve de mandat dans la
    demande structurée. Worker, coordinator et ROOT déjà mandatés gardent leurs
    rappels. ROOT configuré sans mandat crée une décision à prendre.
    La publication d'un résultat et son archivage restent sous le même verrou
    existant. T020 impose l'interleave entre CAS et archive pour protéger le
    chemin canonique d'une publication concurrente, sans second mécanisme de verrou.
    Borne T020 : remise initiale existing_bridget et publication write_task_result.
    Les pannes E/S ordinaires avant commit restaurent le document précédent ;
    une panne de journal après commit reste un warning fermé. Aucun writer externe
    sans verrou, crash multi-fichiers ou rollback défaillant n'est couvert.
11. Compléter la règle de la skill Bridget et les aides des outils. Encourager
    le même projet, montrer --global, expliquer le choix interprojets, le motif
    réutilisable et la compatibilité inconnue. Aucun changement de droits 133.
12. SteerCurrent.message injecte aussi un contenu dans le contexte d'une cible.
    Réutiliser la même garde avant cette injection dans handle_execution_control.
    Une reprise acceptée est rendue avant la garde mutable. Conserver ses warnings
    dans une seule colonne project_warnings de execution_control_commands,
    TEXT JSON NOT NULL DEFAULT '[]'. init_schema et le pattern d'introspection
    existant portent la migration compatible. Aucun nouveau registre ou table.
    Cette colonne est l'exception explicite à l'absence de migration annoncée
    initialement. canonical_bytes reste comparé exactement ; refusal_reason
    reste un refus, jamais un conteneur de warnings. Anciennes commandes: [].
    Interrupt sans message et canon sans motif conservent leur contrat.

## Points d'extension et responsabilité

Les chemins sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet.
Chaque changement doit préserver les éditions simultanées de ses voisins.

| Item | Propriétaire fonctionnel | Point à étendre |
|---|---|---|
| Fait projet et comparaison | noyau de communication | communication.rs et types partagés existants |
| Annonce et autorité | daemon/ponts | daemon.rs, wrapper.rs, t3code.rs, t3code_contract.rs |
| Annuaire scoped | daemon/transport | protocol.rs ; ListAgents global conservé |
| Garde directe et réponses | daemon | prepare_dispatch et demande suivie existante |
| Consigne injectée et résultat durable | noyau daemon/protocol et propriétaire execution_store | handle_execution_control, reserve_control_command, init_schema et résultat existants ; T019 |
| Motif de message et requête | core/transport | BridgetMessage, ThreadRequest, canons existants |
| Mandat de fil | service/store fils | threads.rs ; canon et membres existants |
| Warning émetteur | transport/façades | résultats send/thread existants |
| CLI/MCP, passation, journal et fédération | façades/client | cli.rs, mcp.rs, communication/client.rs, handoff.rs, attach.rs |
| Projet du run et recrutement | Agent Loop | /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py |
| Mandats et rappels | Agent Loop | fonctions d'attribution, dispatch et contrôle 135 |
| Règle d'usage | documentation | skill Bridget du worktree et aides publiques |
| Acceptation | tests existants | harnais fils 102/136 et tests Agent Loop |

Le script de skill est modifié dans le worktree dotfiles isolé, base 5fc64e37.
La copie Claude du même dépôt est vérifiée sans écraser ses différences propres.
La skill vivante reste inchangée. Aucune installation globale ou modification
des lanceurs de production n'entre dans cette session.

Découpage: le noyau possède protocol/message, communication, la garde daemon
et les canons des fils. Les façades possèdent cli/mcp/client et l'annonce
wrapper/T3, handoff et attach. Elles consomment le contrat du noyau sans recopier sa règle.
Agent Loop possède son run et ses mandats.

## Constitution Check

PASS de conception: session autorisée, worktree dédié, aucun fournisseur payant,
aucun restart production, aucun nouveau registre ou package. ADR 046 documente
le choix. Les accès membre, preuves auxiliaires, contrats fermés et reçus durables
restent les autorités existantes. Les standards de tests sont adaptés à Rust
et unittest ; les commandes frontend ne s'appliquent pas à ce périmètre.

Article XVIII: comparaison O(1), filtrage O(A), fil O(M) borné. Article XIX:
un champ de motif partagé et une variante d'annuaire ; une seule colonne de
warnings durable pour le contrôle existant. Aucun stockage de consentement de fil.
Article XX: une règle lisible et une garde commune réduisent les décisions
répétées. Le mainteneur peut lire le contrat, observer le verdict et exécuter
la recette sans contexte caché. Aucun moteur de politique général n'est ajouté.

Les ponts constitution et standards manquent dans le worktree. Ceux du dépôt
principal ont été lus. La constitution utilisateur et ses références ont été
appliquées. La synchronisation éventuelle relève du principal ; cet artefact
ne copie pas les fichiers générés.

## Séquence de vérification

Créer d'abord le Gherkin 138. Exercer les comportements avant modification.
Étendre le fait de connexion et le contrat, puis la garde directe, les fils,
les façades et enfin Agent Loop. Chaque tranche reçoit des tests ciblés.
La matrice couvre local/global/inconnu, identité forgée, worktree/symlink,
cross connu avec et sans motif, réponses corrélées, fil mixte silencieux,
ancien client, ancien reçu et opération acceptée en cours de remise.
Un fil A/B/U conserve la garde A/B malgré U inconnu. Une reprise de notification
après échec transporte le motif figé, pas le mandat courant modifié.

Les essais utilisent un vrai daemon temporaire et des transports de test locaux.
Les tests ne lancent aucun fournisseur. Réutiliser les assertions SQLite et
les échanges CLI/MCP de 102/136, plutôt que créer un second harnais.
Tests Agent Loop: recrutement local, busy, ROOT extérieur sans mandat, mandat
worker/coordinator/ROOT et plusieurs ticks sans nouvelle confirmation humaine.

Après les contrôles ciblés: cargo test --workspace, fmt --check, clippy avec
-D warnings, build release isolé et unittest de la skill. Puis lecture adverse,
Analyze, Converge et audit demandés par le pipeline principal. Les commandes
et résultats réels seront consignés ; ce plan ne déclare aucun test passé.

## Divergences volontaires et limites

ListAgents reste global pour préserver les diagnostics. Un nouvel annuaire scoped
répond à l'intention de communication sans casser cette observation existante.
Les inconnus ne sont pas proposés automatiquement, mais les anciens envois
restent possibles avec avertissement. Cette compatibilité limite la portée
de la règle ; elle ne constitue pas un contrôle d'accès de sécurité.

L'avertissement est établi avant l'effet et retourné au caller. Il n'impose pas
un dialogue humain en deux temps. La comparaison est un fait de communication,
pas une permission sur les fichiers ni une preuve de confidentialité fournisseur.

Livraison de session: code et preuves isolés, sans commit, installation globale,
fusion, API fournisseur ou redémarrage production.
