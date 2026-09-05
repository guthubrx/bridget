# Contrat d'extraction — communication et fédération

Ce contrat conserve la sémantique de la référence Git dfa2134dcfe2a2522e3ae77d93561e6ae72556b3. Il ne prétend pas encore en avoir gelé toutes les fixtures : c'est le livrable testable T002.

## Surfaces conservées

CLI/MCP pour send/reply/who/ledger, lifecycle spawn/stop, observation attach, découverte et négociation de capacités, interfaces publiques guichet/événements pour service externe. Les détails de signature restent ceux du corpus source ; `in_reply_to`, `id`, `issued_at` et le scope stable ne sont pas optionnels au sens du retry : tous les éléments du canon doivent être rejoués sans modification.

Les appels CLI/MCP utilisent une même construction canonique et une même lecture du ledger. Aucune façade ne lit une DB distante présumée locale. Les outils MCP n'écrivent sur stdout que leur protocole négocié. Une opération de GUI/projet retirée est refusée explicitement ; elle n'est pas transformée en appel métier implicite.

## Tableau de vérité minimal

| Fait observé | Ce qui peut être affirmé | Ce qui ne peut pas être affirmé |
|---|---|---|
| Connexion/authentification valide | Identité et capacités de cette connexion | Travail effectué |
| Dépôt durable / in_flight attesté | La demande est enregistrée et en cours de remise | Le fournisseur a traité la mission |
| ACK de remise valide | L'injection reconnue par le transport est durablement attestée | Le résultat est juste ou complet |
| Réponse liée enregistrée | Demande answered ; rappels de cette demande arrêtés | Objectif Maicie réussi |
| Transmission commencée, pas d'ACK valide | outcome_unknown suivant le contrat | Échec certain, donc droit de renvoyer sous un ID neuf automatiquement |
| Gap | Séquence manquante attestée | Flux frais parce qu'un fragment ultérieur arrive |
| Unavailable | Source indisponible | Existence d'une lacune de données déjà prouvée |

Le tableau ne remplace pas les issues fermées existantes : le corpus couvrira aussi les refus et terminaux historiques, dont expiration et orphaned. La skill doit expliquer leur conduite sans uniformiser tous les cas en « réessayer ».

## Limite de l'idempotence

L'identité du retry est stable avant connexion. Même clé + même canon retrouve la même histoire ; canon divergent est refusé sans mutation. Un horizon expiré n'autorise aucune réémission automatique sous une nouvelle clé.

La preuve d'injection unique doit nommer les processus qui survivent au crash. Une panne du wrapper entre un effet fournisseur et son accusé peut laisser une ambiguïté irréductible ; aucun contrat externe ne la résout par magie. Conserver le refus/indétermination prévus par le socle plutôt que promettre une exécution exactement une fois sans transaction fournisseur.

## SSH : même protocole, un maître

Le tunnel transporte la socket Unix du même daemon maître. La connexion réseau n'est pas le mode d'agent. Les sockets, répertoires et identités d'instance test sont explicites ; aucun remplacement d'une socket existante non possédée. Vérification de clé d'hôte SSH conservée, échec de forwarding fatal au tunnel, keepalive borné, aucune ouverture d'un port applicatif public.

Reconnexion : même identité logique, nouveau contexte de connexion, nouvelles autorisations négociées ; aucune conservation accidentelle de capacité d'une connexion morte. Retry et curseur proviennent de l'état durable, jamais du dernier nom affiché. Un curseur hors rétention annonce Gap ; une coupure annonce Unavailable, sans effacer le fait de Gap antérieur.

## Sécurité : modèle de confiance déclaré

Socket accessible seulement au compte/aux accès SSH autorisés. Répertoires et fichiers sensibles privés dès création ; validation du propriétaire, du type et des symlinks. Bornes de trames incluant le délimiteur, de mémoire/file et de temps total sur connect/handshake/write/read. Identité stable vérifiée, scopes isolés et capacité retirée lors de toute sortie de connexion.

Un contenu d'agent est une donnée non fiable, pas une autorité d'administration. Les changements de registre/permissions et les approbations humaines restent hors des messages libres. Il n'existe pas d'isolation forte contre un adversaire exécutant déjà du code sous le même UID : SSH n'est pas une sandbox d'agent. Les protections fournisseur demeurent nécessaires.

## Corpus à épingler avant modification

T002 doit matérialiser les bytes réels, leur SHA-256 et leur origine Git pour : négociations, envois et replies, lookup et issues, annuaire/ledger, attach snapshot/live/Gap/End, lifecycle de processus, services/claims/replies/événements. Chaque famille conservée reçoit un test de lecture dans le daemon et un consommateur ; une mutation d'un octet doit être détectée. Ne pas geler seulement le handshake.

L'évolution additive nécessaire à l'isolation est versionnée et testée avec le client précédent. Une inconnue n'est ni ignorée silencieusement ni déclarée disponible. Le corpus historique épinglé n'est pas réécrit.

## Clôture d'un service externe — clarification T018

La clôture d'un rapport de service utilise les identités de la demande suivie
durable, jamais le libellé historique du service : le déposant doit être le
destinataire de cette demande. La capacité et le claim courant autorisent le
traitement du rapport, pas la clôture d'une demande tierce. Le helper partagé
conserve son couple sender/target et state=open, dans la transaction de replied
et de l'événement. Un rapport accepté peut rester traçable sans rouvrir un
terminal (D-208) ; l'état métier du rapport n'est pas une identité de session.

## Nom affiché sans interface — extension additive T015, version 1

`display_name_set` porte uniquement `request: {version:1, display_name}`. Aucun
agent cible, chemin, scope, instruction ou label n'est accepté. Le principal
et l'instance sont ceux de la connexion wrapper/auxiliaire vivante, validés par
le même helper que la lecture de contenu ; les droits historiques de contenu
projet restent un contrôle supplémentaire propre au contenu, pas au renommage.
Les rôles service et client idempotent n'acquièrent pas ce droit. La CLI utilise
l'inscription auxiliaire déjà partagée avec MCP, jamais une ouverture de SQLite.

Canon fermé et fixture additive display-name-v1.jsonl (les 17 fixtures amont
restent inchangées). Version inconnue, nom vide/trop long (80 caractères),
caractère de contrôle, champ supplémentaire ou encodage non canonique :
invalid_request sans mutation. Les autres refus sont identity_unavailable,
name_conflict, revision_conflict et storage_unavailable, non fusionnés.

La transaction IMMEDIATE ne modifie que le nom affiché, son index normalisé,
sa révision et son horodatage ; elle contrôle rows_affected et réutilise la
garde d'unicité des profils. UUID, instance, canon, instructions/labels et
historique restent intacts. Même nom courant : aucune écriture. Le résultat
Applied contient l'UUID, le nom normalisé et la révision constatés AVANT commit,
renvoyés APRÈS succès du commit. Une lecture concurrente post-commit ne peut
substituer le résultat d'une autre opération. Une erreur de transport n'est
jamais annoncée comme un succès ; répéter la commande identique est possible.

## Lecture de contenu sans interface — extension additive T010, version 1

Le renderer retiré n'est pas remplacé par un serveur web. `artifact_read` sur
la socket existante porte un ArtifactReadRequest fermé : version=1,
artifact_ref, version_ref (256 octets maximum), kind={kind:manifest} ou
{kind:blob,digest:SHA256}, offset et limit (1..16384). Aucun chemin, projet ou
principal n'est fourni par le client. L'enveloppe est l'encodage canonique
du protocole, sans champs supplémentaires.

La portée vient de l'inscription attestée : propriétaire vivant, ou auxiliaire
reconnu du même principal ET de la même instance vivante. Un projet historique
exige sa liaison active et la même génération. Le digest seul n'autorise rien :
le blob doit appartenir à cette version autorisée. Aucune visibilité globale.

ArtifactReadResult retourne version/références et une issue fermée : Chunk
(bytes, digest du contenu ENTIER, total_len, offset, next_offset) ou Rejected
(motif typé). Manifestes stockés et blobs sont vérifiés par SHA-256, jamais
reconstruits. Corruption, absence et défaut d'autorité restent des refus.
Offset à la fin rend une page vide terminale ; au-delà, refus. Le blob est
vérifié en flux sur le même descripteur, au plus 16K retournés par page.
Le manifeste historique enrichi est plafonné avant allocation à 512K+256 :
la réserve couvre les seules métadonnées HTML ajoutées après validation.

CLI : `bridget artifact read --artifact-ref REF --version-ref REF [--blob SHA256]
[--offset N] [--limit N]`. MCP : `bridget_read_artifact`, arguments identiques
au DTO. Même client Unix, résolution d'identité et inscription auxiliaire.
Le JSON CLI est celui du protocole ; MCP le duplique en structuredContent et
TextContent. Rejet métier : issue déclarée et exit CLI non nul ; panne : erreur
technique. Publication/références/provenance/quota antérieurs sont conservés.
HTML est inerte : runtime_policy reste dans son canon historique, sans moteur
pour l'exécuter. Aucun upload, fetch HTTP ou filesystem partagé nouveau.

Preuves : core_089_content_test (daemon/CLI/MCP réels, portée, bytes, bornes),
core_089_removed_surface_test (UI refusée avant effet), tests historiques
de publication/store/policy/service. La dette historique des lectures de
lignes et de leurs budgets IO reste explicitement à fermer en T029/T030.
