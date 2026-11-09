# 251 — Séparer conserver une information et solliciter un modèle
Date : 9 novembre 2026. Décision autorisée par François, my-specify-all.
## Cause et résultat attendu
Dix avis de maintenance sans action ont déclenché douze appels modèle dans un fil au repos. « Ne pas répondre » n'empêche pas une inférence ni une compaction. La disponibilité d'un agent n'est pas un abonnement aux bulletins.
## Choix
Le dépôt direct information utilise StoreOnly et la négociation SendInformationV1. L'identité et le scope projet restent vérifiés. Dormance/DND ne changent pas et n'empêchent pas le dépôt. Aucun drapeau d'autorité de mission n'est réutilisé.
Le reçu terminal accepté et le corps ledger sont committés ensemble. Aucun send_delivery, suivi ou exécution n'est créé. Les deux puits de création de remise refusent StoreOnly. Aucun adaptateur fournisseur ne reçoit cette intention. Les messages normaux gardent leurs contrats ; toute corrélation de réponse interdit le mode information avant capture.
La façade sait l'intention demandée et nomme stored seulement sur Accepted. Un Lookup accepted seul ne prouve ni stockage silencieux ni lecture ; conserver l'enveloppe et sa clé. Une panne laisse un résultat inconnu, jamais un repli vers queue. Une collision ledger est refusée sans écraser un autre scope.
## Réutilisation et concurrence
ledger, idempotency_records, prepare_dispatch et journal/search/read restent les sources. Une transaction Immediate borne les changements à la réservation et au corps. Les reçus terminaux ne sont pas drainés comme des remises. Aucun schéma/service/dépendance nouveau.
Les fils notify:[] restent le parcours collectif et permettent pages/remplacement explicite, originaux conservés. Pas de nouveau tableau de bord ni de classification du texte par modèle. Les consignes de maintenance ciblent les travaux réellement affectés ; pas toute la flotte connectée.
## Compatibilité et activation
Les anciennes formes queue gardent leur canon. Nouveau client et ancien serveur peuvent échouer pendant négociation d'enum inconnu, AVANT tout SendIdempotent ; conserver le diagnostic technique, ne pas prétendre avoir envoyé. Après envoi effectif, une réponse perdue reste inconnue avec la même clé.
Le stockage ne produit que les anciens formats ledger et terminal/accepted. Le lecteur de retour doit être qualifié sur une vraie copie : aucun dépôt ne doit devenir une remise. Pas d'ajout de format déclaré sans écrivain effectif.
Les anciennes façades ouvertes ne proposent pas la nouvelle valeur. Leur présence ne permet pas d'annoncer l'activation du mode dans toutes les sessions. La CLI installée et les nouvelles façades l'exposent ; un fil silencieux existant reste utilisable. T3 reste ouvert, sans remplacement ni relance.
## Limites et responsabilité future
Une information doit être explicitement marquée ; aucun filtrage sémantique des anciens textes. Elle ne sera lue que sur consultation volontaire, dans la rétention existante. Un agent absent reste soumis au refus de routage existant. Ni euro ni quota d'abonnement économisé exact ne sont connus. Les mesures de non-réveil comptent les remises réellement créées, pas les réponses vides.
