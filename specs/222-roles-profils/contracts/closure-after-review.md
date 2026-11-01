# Complément de contrat222 — après contre-revue
Date : 2026-11-01. Ce complément tranche les points ouverts du premier plan.

## Identité
Specialist.id EST agent_profiles.agent_id (UUID opaque existant). Aucun second
identifiant de personne. Une définition réutilisable a son propre profile_id,
distinct de la personne et partageable entre plusieurs personnes.
Une conversion reprend l'agent_id de la conversation sélectionnée. Le nom de
flotte, le libellé et les provider_session_id ne deviennent jamais des identités
métier. Les sessions et la présence sont dérivées de provider_bindings et du
roster vivant ; RuntimeBinding n'est pas une nouvelle table.

## MandateGrant et garde de lancement
MandateGrant = grant_id, specialist_agent_id, revision, profile_id,
profile_revision, approved_by, approval_ref, allowed_projects, launch_policy,
issued_at, revoked_at. approval_ref renvoie à une action du client humain
authentifié ou à une réponse humaine corrélée user_input ; jamais à un simple
champ envoyé par un agent. Le mandat d'action lisible reste distinct de cette
autorisation technique. Le port agent peut proposer, pas approuver ce grant.

TOUTES les mutations grant/profil actif/désactivation passent sous DaemonState.
Le même mutex couvre : relecture du grant et de sa révision, pause/révocation/
projet/définition qualifiée, puis réservation FleetSupervisor. Il est interdit
d'appeler request_recovery ou de passer recovering=true pour un réveil métier.
La réservation est corrélée à grant_id/revision dans les métadonnées222.
Avant remise effective/RELEASE au superviseur, relecture sous ce même mutex :
un grant retiré entre réservation et remise ferme le départ, sans remplacer la
mission. Un résultat tardif ne rétablit pas l'autorisation.

Durabilité du spécialiste et persistance d'un processus sont distinctes.
Le réveil utilise une session nommée native, créée sur demande et liée à l'agent_id.
Les sessions créées par ce réveil utilisent SpawnOrder.persistent=false : la durabilité du spécialiste est conservée séparément. Les nouvelles sessions de spécialiste ne sont PAS confiées à la reprise aveugle
des agents persistent au démarrage d'un ancien daemon : leur reprise est toujours
une demande corrélée relisant le grant. Un redémarrage ne fabrique pas de mission.
Un processus ancien de sort inconnu bloque le nouveau départ.
La définition/autorisation durables sont dans la base, les véritables états et
identités de processus restent dans le cycle natif. Aucun champ opaque n'élargit
les permissions d'un SpawnOrder. Le registre natif et la posture restent la borne.

## Un seul magasin d'admission et reprise
Fait vérifié : daemon.rs:4958-4959 ouvre ExecutionStore et DelegationStore avec
LE MÊME config.db_path ; AgentProfileStore utilise st.db_path. Il s'agit du même
fichier SQLite, avec plusieurs connexions métier, pas de bases séparées.
La réservation d'une attribution est écrite UNE fois dans specialist_requests :
UNIQUE(issuer_scope, request_id), canonical_bytes, selected_agent_id,
profile_revision, grant_revision et native_submission_id.
Aucun champ état de mission, PID, phase d'exécution ou copie de résultat.
native_submission_id est déterministe et réservé dans work_submissions comme
prepared dans LA MÊME transaction SQLite que l'attribution. La transaction
utilise la connexion du store de soumissions, pas une transaction imbriquée sur
une seconde connexion. La file/exécution reste celle d'ExecutionStore.

La soumission préparée n'est pas un début d'exécution. La frontière d'admission
native doit promouvoir prepared vers accepted une seule fois, après comparaison
du canon, pour cette opération222 corrélée uniquement. Les admissions historiques
et leurs canons conservent leur comportement. L'envoi/réveil effectif vient après
commit et les recontrôles d'autorité ; les reçus natifs portent la preuve d'effet.

Reprise après coupure :
A. avant commit : rien n'a été réservé ;
B. attribution + prepared sans admission : relire le même canon et grant, reprendre
   la soumission déterministe, jamais refaire la sélection ;
C. admission connue : restituer le reçu natif, aucune nouvelle exécution ;
D. effet/ACK inconnu : rendre unknown et les références ; aucune réassignation.
Ces quatre contre-épreuves sont obligatoires avant validation du raccord.

## Mémoire et oubli
Clé de source = type + identité source + révision/ronde/intervalle + digest.
Une entrée oubliée reste comme tombstone de cette source, sous UNIQUE
(specialist_agent_id, source_key), même si sa projection de rappel est supprimée.
La capture et la conversion consultent cette clé avant insertion. Oubli et
suppression d'un produit/journal sont deux actions distinctes.
Source d'accroche : réception du résultat corrélé de Task native (pas un texte
assistant quelconque), et ObservedActivity attesté du wrapper pour une conversation
convertie. Le second enregistre d'abord la référence journal/seq, non un fait
inventé. La lecture ciblée de la source remplit un résumé rapporté ; les corrections
ou le spécialiste peuvent ensuite produire des connaissances typées et sourcées.
L'indisponibilité de la source reste visible et ne déclenche pas de scan global.
La collecte utilise les canaux/journaux existants et ne lance pas un modèle de plus.

Rappel : normalisation fold_for_search déjà existante, termes de requête
paramétrés, classement par correspondances lexicales, préférence projet à score
égal, puis date et identifiant. Réponse paginée avec sources et indicateur de
fenêtre. Pas de FTS5 ni de recherche vectorielle prétendue. Les requêtes et leur
fenêtre sont bornées ; aucune limite globale de conservation de la mémoire.

## Versions et compatibilité
agent_profile_applications suit l'application d'une instructions_revision ; il
ne stocke pas les anciens textes. Il reste réutilisé pour l'état appliqué d'une
session. DefinitionRevision conserve les textes immuables réutilisables ; ce n'est
pas une seconde table d'applications. Deux responsabilités distinctes.
ControlStateV1 et MobileAdministrationV1 SONT des variantes ClientCapability
(protocol.rs:224 et200), pas les noms de nouveaux services. Les requêtes réelles
restent ControlStateFrame/MobileAdministrationRequest.
SpecialistsV1 est une capacité client additive négociée via ClientHello/Welcome ;
nouveaux ports refusés si absente. Les données222 n'ordonnent aucun lancement par
un lecteur ancien ; une image de retour qualifiée reste requise.

## Formes fermées et refus
Toutes les opérations : version=1, action, request_id pour mutation, expected_revision
pour remplacement, identifiants UUID ; aucun actor/is_human/credential dans les
documents de profil. Paramètres inconnus refusés.
Exemples :
- search {version:1,action:search,role_id,project_id?,cursor?,limit?}
- declare {version:1,action:declare,roles:[{role_id,label,scope}],expected_revision}
- propose {version:1,action:propose,request_id,definition}
- request_role {version:1,action:request_role,request_id,role_id,project_id,task}
- recall {version:1,action:recall,specialist_agent_id,project_id,query,cursor?,limit?}
- activate/grant/revoke : canal humain natif, définition/version et scope exacts ;
  commande corrélée et révision attendue ; résultat nommé avec nouvelle révision.

request_role renvoie un de ces codes fermés :
invalid_request, role_not_found, role_scope_denied, specialist_disabled,
specialist_not_durable, human_reuse_approval_required, mandate_required,
mandate_revoked, mandate_changed, profile_version_conflict,
requirement_missing, requirement_unverified, specialist_execution_unknown,
native_capacity_unavailable, native_peer_unverified, envelope_mismatch,
storage_unavailable, native_submission_unknown.
Les refus des frontières natives conservent aussi leur code original dans un
champ borné autorisé, sans recopier un texte fournisseur/chemin privé.
Le détail nomme la prochaine action, la portée et la décision humaine requise.

## Dérivation et garde prepared
native_submission_id = UUIDv5 avec namespace UUID NAMESPACE_URL et nom UTF-8 « bridget:specialist:v1: » suivi du SHA256 hexadécimal de JSON canonique [issuer_scope,request_id,canonical_digest]. Le canon conserve les champs métiers validés, jamais le temps du rejeu. Si UUIDv5 indisponible, UUID à partir des16premiers octets SHA256 du même nom, bits version8 et variant RFC4122, documenté et testé avant emploi.
Aucune ligne prepared sans exécution ni entrée de file ne doit devenir du travail par un drain, recovery, lecture ou redémarrage existant. Seul le port222 corrélé peut la promouvoir, même canon et grant actuel. Tests négatifs indispensables sur take_next_submission et parcours de reprise.

## Garde effective de RELEASE
Le superviseur de processus est volontairement séparé de DaemonState. Après BootstrapReady et avant RELEASE, SpecialistLaunchAuthority ouvre BEGIN IMMEDIATE sur le même SQLite, relit enabled/durable/auto_start/profil/grant/projet et pause. Il garde ce verrou jusqu’à RELEASE. Les mutations humaines écrivent dans cette même base ; elles ne peuvent plus passer entre la preuve et RELEASE. Cette garde SQL complète le mutex de réservation, sans le copier dans le superviseur ni démarrer un fournisseur pour vérifier des droits. La session reste persistent=false.

## Contact en ligne et réemploi distincts
Une déclaration de rôle rend une identité déjà en ligne joignable via le port de message natif, sans activation durable ni application de profil non approuvé. Cela ne crée aucun processus et n’accorde aucun droit. Le réveil d’une identité hors ligne exige toujours durabilité, auto_start et grant humain courant. La demande et sa sélection sont réservées avant remise ; le réveil reprend cette même soumission à l’enregistrement. Le canon du lancement reprend les dates figées de la réservation, pas l’heure du rejeu.

## Résultats de contact par rôle
Les contacts entre agents utilisent le suivi de réponse natif (fenêtre1h, distincte du budget et sans arrêt de mission). Une réponse reçue sous identité attestée et corrélée à la soumission222 est capturée comme reported_result, pour les spécialistes participants. Ce troisième hook complète Task et ObservedActivity : il ne fait pas passer un message pour un résultat Task ni pour une preuve métier. Source = message_id/in_reply_to/digest/fragment ; répétition idempotente et tombstones identiques aux autres sources.


## Approbation conversationnelle native
request_approval porte une opération permanente fermée et un UUID de question.
La frontière produit la question avec le canon exact. user_input conserve owner,
instance, revision, réponse de l'humain authentifié et command_id. Le même agent
reprend la même opération après réponse ; une réponse libre ou un champ MCP ne
confère aucune autorité. Seul selected=[approve] sur cette question exacte permet
l'application, avec portée initiale conservée et révision recontrôlée. L'ACK natif
devient applied après succès. Une opération changée conserve envelope_mismatch.

## Réemploi humain ponctuel
Une commande humaine peut réveiller une identité non durable sous son mandat
courant, pour cette seule soumission. La preuve est liée au canon et à la
soumission ; elle ne donne aucun auto_start futur. Une demande d'agent reste
refusée sans durabilité autorisée. Les préférences explicites sont des contraintes
vérifiées ; si le modèle vivant n'est pas attestable, le refus dit unverified.


## Budget facultatif et prérequis
Aucun budget au repos ni plafond par défaut. Les limites explicites figées dans
la soumission sont contrôlées aux frontières de nouvelle admission/continuation.
La durée est celle de l'exécution, pas celle de conservation du spécialiste.
Les descendants natifs et continuations connus sont comptés ; une limite de
jetons avec usage absent rend budget_unavailable, pas une estimation de zéro.
Ces contrôles ne promettent pas l'interruption instantanée d'un tour fournisseur
à un nombre exact de jetons. Les droits et quotas techniques demeurent distincts.
prepare_requirements=false par défaut dans launch_policy humaine. Si vrai,
la demande confie une phase préalable de vérification/préparation au spécialiste
dans les droits existants ; aucune chaîne de profil n'est exécutée comme commande.
Si faux et prérequis non attesté, l'admission rend requirement_unverified.


## Premier lancement temporaire
request_temporary fige une définition neuve du parent dans la demande native
AutoDelegateV2, avec ses consignes, procédures, contraintes et besoins. Le parent
garde ses droits réels ; aucune définition n'élargit la posture ou l'admission.
La première utilisation est réservée sous clé/canon avant la préparation
asynchrone. La même clé reprend cette admission, une autre clé ne consomme pas
une seconde fois la version non durable. Une tâche déjà admise est conservée
même si son rattachement documentaire doit être réparé. Le port ordinaire reste
autorité de tâche ; aucun appel fournisseur ni cycle de mission222 concurrent.
