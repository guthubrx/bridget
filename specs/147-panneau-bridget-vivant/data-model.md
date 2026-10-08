# Modèle de données — SPEC147

Date : 2026-10-08. Modèle proposé. Aucune migration de base.

## Données métier conservées

Fils, membres, messages, séquences, remplacements, opérations idempotentes et dates restent les données Bridget145/146. Create/Post/Close sont les mutations existantes. Aucun nouveau titre modifiable, membres modifiables ou résultat de mission n'est ajouté.

Les noms d'agents affichés viennent de l'annuaire existant. Une modification réelle de nom invalide les vues auxquelles cet agent appartient ; le nom ne traverse pas le flux de suivi.

## Référence de sélection persistée

Table logique : `bridgetSelectionByContextKey`, dans `useRightPanelStore` existant.

- Clé : chaîne JSON du triplet `[environmentId, projectId, threadId]`. Les trois composants viennent du contexte T3 attesté ; aucun concaténat ambigu avec séparateur libre.
- Valeur : UUID canonique du fil Bridget, ou absence d'entrée si aucun choix.
- Validation : valeur UUID valide, structure de clé conforme au triplet attendu. Une donnée persistée invalide n'est pas restaurée.
- Ancien snapshot : champ absent → table vide. Aucun corps n'est migré.
- Suppression : refus d'accès au fil confirmé, fil inexistant confirmé ou suppression du contexte selon la réconciliation native du store.
- Indisponibilité : conserver la référence. Elle n'autorise pas l'affichage d'un ancien corps lors d'une reprise non revalidée.

La table est séparée de `ThreadRightPanelState`. Fermer/reconstruire une surface ne retire pas le choix. Aucun titre, liste de membres, corps, credential, verdict ou curseur agent n'est stocké dans cette table.

## Suivi humain en mémoire

Un suivi possède : connexion Client négociée, conversation T3, projet résolu, agent/instance attestés, génération opaque UUID, ressource de sortie et invalidation en attente.

La génération change à chaque nouvelle connexion. Elle ne constitue pas un cursor métier ou une preuve d'accès.

Le registre est borné à128 suivis par daemon. Chaque suivi a une file de16 signaux au plus et un indicateur de resynchronisation. Tant qu'il n'est pas écrit, ready seq0 occupe une place réservée dans ces16, toujours en tête et jamais coalescée. La saturation fusionne seulement les changements suivants en `resync` au lieu de croître ou de promettre tous les événements. Les socket writers s'exécutent hors verrou.

Cette mémoire n'est pas durable. Elle n'est pas la source de vérité des messages. Après restart, `ready` demande une relecture autorisée de l'état courant.

## Signal de suivi

Champs fermés : `version: 1`, `generation: UUID`, `seq: entier sûr JavaScript`, `status: ready | changed | resync`.

`ready` porte seq0. Les émissions suivantes croissent dans la même génération. `changed` et `resync` invalident liste et détail comme `ready`. Aucun UUID de fil, titre, nom, message ou champ libre n'est admis. Le daemon filtre les mutations en fonction des fils réellement accessibles ; une mutation étrangère ne produit pas de signal. Le compteur n'est pas une séquence de message ni un curseur agent. À `Number.MAX_SAFE_INTEGER`, arrêter le flux ; une reprise crée une nouvelle génération.

La perte d'accès termine le flux avec un code d'erreur fermé existant. Le panneau purge le contenu selon le code, pas selon le texte d'erreur. Un refus confirmé du fil retire sa sélection ; une indisponibilité de liaison ou réseau conserve sa référence mais masque le contenu non vérifié.

## État de lecture volatile

- Clé de contexte : environnement/projet/conversation.
- Génération locale de consultation : différente pour chaque visite, y compris A → B → A.
- Génération de suivi active : celle du dernier `ready` accepté.
- `BridgetWatchState` local : dernier événement, `readyGeneration` mémorisée et `subscriptionId` neuf par subscription. Ce mémo conserve la preuve de ready même si le rendu ne voit que changed.
- `visitId` local : clé distincte de subscription par visite, sans persistance et sans transmission RPC/IPC. Le wire garde threadId/projectId seulement.
- Version de revalidation : incrémentée lors des invalidations ; une réponse dépassée ne devient pas l'état courant.
- `inFlight` et `dirty` : une lecture en cours, une revalidation complémentaire au plus.
- Liste et historique : pages146 déjà autorisées, leur borne de snapshot et leurs curseurs ; aucun stockage durable de contenu ajouté.
- Choix et gestes : UUID référencé dans le store ; dépliages/détails/DOM des lignes inchangées conservés dans la vue.

La revalidation de l'historique acquiert un nouveau snapshot S via la tête `history_recent`. Au-delà de50 nouveautés, parcourir les pages intermédiaires jusqu'à l'ancre ancienne pour ne laisser aucun trou. Reconstruire ensuite les pages du segment consulté avec `to_seq=S` commun. Tête, intervalle et segment restent en staging jusqu'à publication atomique. Une ancienne pagination en cours est invalidée par une nouvelle génération locale. Aucun mélange de tête S et pages d'un ancien snapshot n'est autorisé : leur projection pourrait masquer un remplacement récent. Les clés séquence/UUID maintiennent les messages inchangés et leurs gestes de lecture. Coût O(nouveautés + pages consultées), pas O(base entière).

La reprise CLI technique possède un compteur volatile borné : tentative initiale plus trois reprises maximum avec Schedule existant. Elle concerne seulement unavailable/command_failed/timeout. Elle ne remplace pas le supervisor de transport WebSocket. En attente, corps masqués et référence UUID conservée. Aucun compteur durable ni lecture périodique de contenu ajouté.

## Présentation dérivée de la sollicitation — US5

Source existante : notify.mode et targets effectifs du message autorisé, plus noms disponibles dans detail.members. Aucune nouvelle donnée métier ou persistée. none produit Auteur · Sans sollicitation ; targets/all produisent Auteur → Noms pour les seuls targets effectifs du message, auteur exclu selon GO du principal. All n'est pas recalculé à partir de la liste actuelle des membres. L'exclusion de l'auteur n'est pas une preuve de livraison.

Un nom absent produit Nom indisponible + UUID court ; l'identifiant exact reste accessible dans les détails. La ligne auteur/date existante porte cette indication, sans ligne supplémentaire. Cette projection ne modifie pas les corps, copies, notifications, droits ou flux de suivi. Elle ne constitue pas un reçu de livraison.

## Transitions

1. `inactive` → `connecting` : panneau visible dans un contexte valide ; lire la référence mémorisée, démarrer un suivi scoped.
2. `connecting` → `revalidating` : `ready` nouvelle génération ; revalider liste et fil choisi par read146.
3. `revalidating` → `current` : réponses autorisées correspondant à la visite et à la version actuelle ; conserver les gestes encore valides.
4. `current` → `revalidating` : `changed`, `resync` ou refresh manuel ; coalescer les demandes.
5. État actif → `temporarily_unavailable` : rupture technique unavailable/command_failed/timeout ; garder le UUID, masquer les corps et invalider toute garantie de fraîcheur. Le supervisor reprend le WebSocket ; un stream CLI rompu sous WebSocket sain possède au plus trois reprises supplémentaires.
6. `temporarily_unavailable` → `revalidating` : tentative technique admise ou retour visible, puis nouveau `ready` ; aucun intervalle manqué supposé vide. Plafond atteint → arrêt avec message et refresh manuel. Invalid_output/version/projet/binding/demande invalide → arrêt sans reprise automatique.
7. État actif → `denied` : code autoritaire du contexte ou du fil ; arrêter la ressource et purger/masquer le contenu. Seul `thread_unavailable` confirmé par la lecture du fil choisi retire son UUID. Un refus de binding/projet n'est pas une preuve de suppression du fil ; la référence reste sans autorité. Un refus métier fermé n'est pas autoreconnecté en boucle ; refresh manuel ou nouvelle visite peuvent revalider.
8. État actif → `inactive` : fermeture, démontage, navigation ou page masquée ; cancel, libérer IPC/enfant/RPC. Les références de choix non refusées restent.

## Invariants

- Référence mémorisée ≠ autorisation.
- Signal ≠ contenu, mission ou ACK.
- Mutation Done après commit → invalidation ; replay/refus/lecture/ACK → aucun changement.
- Inscription précède `ready` sous le verrou qui ordonne les mutations.
- Pas de génération de consultation réutilisée après navigation.
- Panne conserve le choix ; refus confirmé du fil choisi retire son UUID et ses corps. Refus du contexte masque/purge les corps et arrête le suivi, sans déduire la suppression du fil.
- Flux fermé → ressources libérées sans rétention cinq minutes.
- Wire ready protégé ≠ ready nécessairement rendu : readyGeneration reste mémorisée avant coalescence, sous la visite/subscription active.
- Reprise CLI bornée ≠ polling de corps ; aucun refus métier fermé n'est converti en panne retryable.
- Les auteurs, corps et verdicts métier ne sont pas inventés ou modifiés par la vue.

## Extension UUID US6 — référence agent, pas nouvelle donnée durable

require_uuid normalise les champs UUID des actions de fils partagés déjà existants en forme hyphénée minuscule avant clé d'idempotence/SQL. UUID majuscule/minuscule/mixte admissible représente la même valeur ; aucune table ou migration. Membres/targets/reçus conservent leurs relations. Le corps et les noms/préfixes CLI ne sont pas réécrits.

canonical_uuid du contrat humain145/146/147 et les curseurs restent stricts. La normalisation ne vaut aucune attestation d'identité ni permission. ACK garde son refus ReceiptInvalid. Le montage MCP US7 suit le plan natif validé, pas une identité injectée par un outil T3. Aucun champ d'identité dans stdio/env MCP ; les preuves OS privées restent distinctes des fixtures de configuration SDK.

## US9 — projection visuelle sans nouveau modèle

Le choix UUID par contexte reste celui de US2 ; aucun champ persistant ajouté. La ligne épinglée est une projection unique du détail autorisé lorsque le choix n'appartient pas à la page/liste filtrée ; aucune projection en cas de masque/refus. Réutiliser mêmeparent/key, état accessible et details frère du contrôle de choix. Les messages gardent leurs keys séquence/ScrollArea/copie ; retirer seulement le bloc répété inférieur. Aucun corps/titre/membres nouvellement persisté ni API supplémentaire.

## US8 — membres ajoutés, modèle existant étendu

Réutiliser discussion_members : nouveau agent_id canonique avec acked_seq0 et aucune ancienne lecture/reçu copiée. Discussion thread garde créateur, corps, séquences, kinds et activité. Thread operations accepte add_members après migration cibléev2→v3 ; ancien résultat/reçu/index conservé. NoChange n'écrit pas d'opération et ne consomme pas sa clé ; changement réel écrit un résultat idempotent. La transaction reçoit l'union autorisée et refuse toute divergence après relecture ; maximum16, entrants bornés entries+16. Membership n'est pas une entry ni un wake. Targets anciens immuables ; all futur utilise la nouvelle audience. Historique antérieur complet accessible sous les contrôles existants, jamais rejoué comme mission.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.
