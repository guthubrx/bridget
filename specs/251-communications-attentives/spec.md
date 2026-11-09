# 251 — Communications attentives
Statut : Implemented. Pipeline : Livré sur les services autorisés, application T3 conservée. Date : 9 novembre 2026. Responsable : mainteneur Bridget.
## Demande initiale et cadrage montré
François veut rester joignable sans recevoir des avis sans action qui remplissent le contexte ou déclenchent des appels coûteux. Il demande une démarche ambitieuse, le code T3 et des sources concurrentes primaires. Exécution complète autorisée par my-specify-all ; T3 doit rester ouvert.
Demande reformulée : conserver chaque information utile, séparer stockage, consultation et réveil du modèle. Seuls une demande, un résultat attendu ou une action nécessaire doivent solliciter l'agent. Un agent au repos n'a pas à dormir pour éviter le bruit.
Apport : garder la disponibilité sans payer des tours vides. Un reçu de stockage ne vaut jamais lecture.
## Mesure initiale
Exemple privé autorisé : dix avis de maintenance du 5 au 7 novembre, douze appels modèle, aucune action métier, dix réponses finales vides. Entrées cumulées 4 068 798 tokens dont 2 291 456 en cache ; sorties 5 818. La compaction est incluse. Aucun coût en euros ou quota d'abonnement n'est déduit. Aucun contenu métier privé ne sera versionné.
## User stories
### US1 P1 — Informer sans solliciter
L'émetteur choisit explicitement une information sans action. L'agent concerné la retrouve dans le journal sans recevoir de prompt, au repos comme pendant un tour. Un agent connecté en sommeil ou DND n'est pas réveillé. Une mauvaise combinaison demandant une réponse est refusée avant effet.
### US2 P1 — Préserver les communications utiles
Une demande avec réponse, un résultat natif, une coordination en cours et un guidage explicite gardent leurs contrats. Une erreur de stockage ne devient ni un succès ni un envoi ordinaire. Une reconnexion ne transforme pas une information en sollicitation.
### US3 P2 — Prévenir les bons agents
La maintenance publie ses bulletins généraux dans un fil silencieux existant ; les actions avant/après coupure visent seulement les travaux affectés. Les bulletins répétés se lisent ensemble dans le fil et peuvent se remplacer logiquement sans effacer leurs originaux. La présence connectée n'est pas un abonnement. Aucun classement du texte par modèle.
### US4 P2 — Observer le résultat
Le reçu distingue information stockée et remise au modèle. Le journal existant permet lecture ciblée et recherche, avec les permissions habituelles. La mesure de non-réveil repose sur les contrôles de remise, pas sur une réponse vide.
## Exigences
- FR-001 : exposer une intention explicite d'information sans réveil dans les façades CLI/MCP ; défaut historique conservé.
- FR-002 : enregistrer corps et reçu atomiquement dans le stockage durable existant ; zéro remise, file exécutable, contrôle fournisseur ou réveil à l'acceptation et au redémarrage.
- FR-003 : même clé/enveloppe rend le même reçu ; une enveloppe modifiée est refusée. Échec de persistance : aucun faux succès.
- FR-004 : interdire réponse attendue, corrélation de réponse et autorité de résultat sur une information ; aucun résultat ne peut être englouti.
- FR-005 : conserver identité, portée inter-projets et validation du contenu. Le choix silencieux ne donne aucun droit supplémentaire.
- FR-006 : rendre l'information consultable par le journal/recherche existants, sans consultation automatique au prochain tour.
- FR-007 : documenter et appliquer une matrice information/action/urgence ; pas de diffusion générale aux seules présences connectées.
- FR-008 : conserver les fils silencieux, le remplacement explicite et leurs preuves. Corriger les observations qui réveilleraient un dormant si le chemin est confirmé.
- FR-009 : négocier la capacité nouvelle ; un ancien daemon ne doit jamais recevoir un envoi ordinaire en remplacement. Un retour qualifié ne doit pas rejouer les informations stockées.
- FR-010 : activer les services autorisés et vérifier leur comportement ; aucun arrêt/remplacement de l'application T3.
## Réussite
- SC-001 : dix informations successives produisent dix corps consultables et zéro contrôle de remise, zéro demande suivie, zéro exécution ; aucun appel modèle de qualification nécessaire.
- SC-002 : une vraie demande témoin garde une remise et son suivi ; résultats et guidage existants restent verts.
- SC-003 : tests de rejeu, stockage en échec et reprise montrent absence de duplication et de réveil.
- SC-004 : aucune migration de données métier ni nouvelle dépendance/service/table/file de réveil.
- SC-005 : comparaison de trois sources primaires et code T3 consignée ; chaque choix relié au besoin, sans copier une architecture concurrente.
## Cas limites et exclusions
Au destinataire absent : refus de routage conservé, pas de destinataire inventé ; un fil silencieux reste possible selon ses droits. Un dormant connecté accepte le stockage sans changer son état. Une ancienne façade ne sait pas demander le nouveau mode ; le daemon ne devine pas la nature de ses textes. Aucun retrait rétroactif des notifications, aucun changement de compte/moteur, aucune nouvelle page T3, aucun abonnement implicite.
## Dépendances
Sessions102/104 (fils et journal),135 (observations),156/159 (guidage/files),239 (maintenance),241 (admission),244 (conservation),247/248 (coordination),250 (résultat natif).
