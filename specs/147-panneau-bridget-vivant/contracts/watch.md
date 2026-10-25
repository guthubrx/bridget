# Contrat du suivi humain — SPEC147

Date : 2026-10-08 ; convergence finale le 2026-10-09. Statut : Implemented — Validé en environnement isolé. Le principal a retenu le signal global sans UUID de fil. NON installé et NON activé.

## Noms exacts et rôle

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-1).

Ce suivi concerne la consultation humaine. Il ne réutilise pas `ObservationSubscribe`, les sollicitations de fils, les abonnements de fin de tour ou les ACK d'agents. Aucun fournisseur de modèle n'est appelé.

## Entrées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-3).

La cible de subscription côté client ajoute un `visitId` local unique par visite. Ce champ isole les atomes de deux visites et impose un seul suivi par visite. Il ne traverse ni RPC ni IPC : l'entrée wire garde exactement les deux champs threadId/projectId.

La CLI s'ouvre avant le chemin d'initialisation d'identité agent. Elle utilise une connexion Client nue. Elle négocie exclusivement `human_thread_watch_v1`. Un daemon absent n'est ni lancé ni remplacé. Un daemon ancien produit `unsupported_version` ; aucun repli vers observations agent ou polling n'est autorisé.

## Succès JSONL et schéma adaptateur de bureau

Une ligne JSON par événement. UTF-8 et schéma fermés :

```json
{"version":1,"generation":"10000000-0000-4000-8000-000000000001","seq":0,"status":"ready"}
```

Champs obligatoires uniquement :

- `version` : exactement1.
- `generation` : UUID canonique opaque, nouveau à chaque connexion acceptée.
- `seq` : entier entre0 et9 007 199 254 740 991 ; `ready` commence à0, les suivants sont strictement croissants dans cette génération.
- `status` : `ready`, `changed` ou `resync`.

Aucun corps, UUID de fil partagé, titre, membre, nom, prompt, raisonnement, secret, verdict ou chaîne libre n'est accepté. Les données de contenu continuent de passer par `bridget.read`146, après vérification des droits.

`ready` exige la revalidation initiale de liste et détail. Il est obligatoire, premier, seq0 et non coalescible. Une rafale, même supérieure à16 avant la première écriture socket, ne peut pas le remplacer par changed/resync. `changed` indique qu'une donnée du périmètre autorisé a changé. `resync` indique qu'une continuité de signaux n'est plus garantie. Les trois déclenchent la même revalidation groupée par lectures146. Le compteur n'est pas un curseur d'historique ; il ne permet aucun ACK et n'avance pas un curseur agent.

L'ordre garanti sur le wire ne garantit pas que le DOM voit ready avant changed : l'atome peut coalescer leurs rendus. Le runtime garde donc un état volatil `BridgetWatchState` : dernier événement, `readyGeneration` mémorisée et `subscriptionId` local neuf. Il enregistre ready avant toute coalescence de rendu. Le panneau accepte un dernier changed/resync seulement si readyGeneration correspond à sa génération et si la visite/subscription sont celles en cours. Ce mémo ne contient aucun corps et n'ajoute aucun champ wire.

Un signal d'une ancienne génération, d'une ancienne visite ou dont la séquence n'est pas supérieure à la dernière acceptée est ignoré. Un saut de séquence exige une revalidation complète comme `resync`, pas un replay inventé. Au dépassement de l'entier sûr, le flux s'arrête ; la reprise négocie une nouvelle génération et `ready` seq0.

## Refus et erreurs

Une erreur CLI est une dernière ligne fermée, suivie de l'arrêt du flux :

```json
{"version":1,"status":"error","code":"binding_unavailable"}
```

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-5).

Un refus ou une révocation ne laisse jamais un cache considéré comme autorisé. La sélection peut survivre à une panne ; le droit ne survit pas à une revalidation refusée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-6).

Pendant l'attente, masquer les corps et conserver seulement le choix UUID. Après ce plafond, afficher l'indisponibilité et proposer refresh manuel ou retour dans la vue. `invalid_output`, version incompatible, projet refusé, binding refusé et demande invalide ne sont jamais réessayés automatiquement. Aucun timer de lecture de corps, nouveau journal, réveil ou modèle n'est ajouté. Un refus métier fermé ne devient pas une panne technique pour contourner ce filtre.

## Autorité et séquencement daemon

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-7).

Create, Post, supersedes via Post et Close déclenchent `changed` sur `Done` seulement. Un renommage d'annuaire réellement modifié déclenche une invalidation pour les vues auxquelles cet agent participe. Une création existante rejouée, une publication rejouée, un refus, un ACK, `history`, `read`, `show`, `list` ou `watch` ne déclenchent pas `changed`.

## Bornes et annulation

-128 suivis humains simultanés maximum par daemon ; dépassement : refus fermé `response_too_large`, aucun suivi partiel.
-16 signaux en attente maximum par suivi. Avant sa première écriture, ready seq0 réserve une place dans ces16. Saturation : coalescer uniquement les changements suivants en `resync`, sans modifier ready. Le marqueur de resync occupe la file bornée, pas une file parallèle non bornée.
-4 Kio maximum par ligne JSONL, y compris une ligne partielle. Le tampon d'une ligne trop longue est refusé avant croissance supplémentaire.
- Handshake/attente initiale `ready` : six secondes maximum.
- Une écriture socket vers un client lent : une seconde maximum, hors verrou. Sur échec ou lenteur persistante, fermer le suivi ; la reprise revalide l'état courant.
- Flux accepté : aucune expiration unary de dix secondes. Fermeture du pipe, interruption de portée, déconnexion RPC ou annulation UI libèrent l'enfant et la connexion.
- Fin stdout sans ligne d'erreur validée : échec immédiat unavailable avant ready ou command_failed après ready ; l'arrêt de portée ferme l'enfant, même s'il est encore vivant.
- Une ligne finale binding_unavailable validée avant la borne de handshake satisfait cette attente initiale. Attendre ensuite son code de sortie au plus500ms pour distinguer exit3 transport et exit2 refus métier. Cette attente séparée ne s'applique pas à tous les EOF ; aucune panne technique n'est convertie en refus de binding.
- Client UI : au plus une revalidation en cours et un dirty complémentaire par contexte. Aucun intervalle de relecture de contenu quand rien n'a changé.

Ces bornes sont des limites techniques du seul contrat147, pas une politique de mission. Leur dépassement ne produit aucun réveil d'agent.

## Cycle de vie UI et sélection

Le contexte de navigation est `[environmentId, projectId, threadId]`. Le choix mémorisé est seulement un UUID de fil. Chaque visite reçoit une génération locale nouvelle ; un retour A → B → A ne valide pas une ancienne réponse de A.

Panneau masqué ou démonté, page masquée ou contexte quitté : libérer immédiatement la consommation du flux. La subscription runtime impose `idleTtlMs: 0`. Au retour, nouvelle connexion et `ready` ; aucune hypothèse de livraison des signaux pendant la pause.

Restaurer le UUID via `show` et `history_recent`, même hors première page. Une indisponibilité ne remplace pas ce UUID par le premier fil visible. Un refus de fil confirmé l'efface.

Les relectures conservent les clés et le DOM des messages inchangés, les dépliages, détails, pages historiques utiles, focus et texte sélectionné. La tête `history_recent` fournit le nouveau snapshot S. Si les nouveautés dépassent sa taille50, parcourir les pages intermédiaires entre la tête nouvelle et l'ancre ancienne ; ne pas sauter cet intervalle. Reconstruire ensuite le segment déjà consulté avec `to_seq=S` commun, dans un staging. Publier atomiquement toutes les pages et leurs relations de remplacement après revalidation ; ne pas mélanger tête récente et pages d'un ancien snapshot qui masquent `superseded_by_seq`. Le coût est O(nouveautés + pages consultées), sans parcourir la base complète. Si une page ancienne est en vol, invalider sa génération puis relire le segment cohérent. Une révocation prime sur ces états visuels.

## Ajout de présentation US5, sans changement du contrat watch

Dans la ligne auteur/date déjà présente, afficher Auteur → Noms pour notify targets/all ou Auteur · Sans sollicitation pour none. Utiliser les targets effectifs lus, auteur exclu selon GO du principal, et les noms detail.members déjà autorisés. Un nom absent affiche Nom indisponible et UUID court, avec détail exact existant. All n'est pas recalculé depuis les membres actuels. Cette indication exprime la demande de sollicitation, pas une preuve de livraison. Aucun champ watch/RPC/IPC, API, annuaire, mutation, notification, réveil ou ligne supplémentaire. La copie demeure le corps original.

## Matrice de tests contractuels

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/contracts/watch.md#extrait-8).

Les reçus partiels sont consignés dans le journal147. Ce contrat de convergence ne déclare pas les nouvelles corrections terminées ni la recette finale réussie.
