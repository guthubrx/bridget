# 181 — Questions humaines natives Bridget sur Android et terminal

Statut : Implemented (25/25 tâches) ; pipeline Livré avec conservations justifiées. Demande approuvée par invocation my-specify-all le25octobre2026.

## Demande initiale et choix validés

L’utilisateur veut les questions du modèle telles qu’elles existent dans T3, présentées correctement sur téléphone. Il valide le volet compact, son agrandissement, les réponses depuis les notifications et la conservation des réponses. Il demande une réponse libre supplémentaire, éventuellement dictée, pour remplacer ou compléter les choix proposés. Il demande ensuite un émulateur Android et retire son Pixel des essais.
Précision prioritaire : « je ne veux pas être compatible qu’avec T3 […] même […] dans une fenêtre de terminal […] qu’on passe par Bridget ». La continuité ne doit créer aucune dépendance envers T3 ; son abandon est accepté si elle devient complexe.

## Demande reformulée

Permettre de recevoir une question humaine structurée par Bridget, d’y répondre sur Android, dans une notification ou depuis un terminal, puis de reprendre exactement la demande d’origine. Préserver les textes, descriptions et choix produits par le modèle. Proposer une réponse libre écrite ou dictée, seule ou en complément. Conserver le brouillon sur son appareil et montrer la résolution confirmée aux autres clients. T3 reste un adaptateur facultatif ; le parcours complet doit fonctionner sans T3.
Apport du cadrage : la question et sa réponse portent une identité durable native Bridget, au lieu de dépendre du client qui les affiche. Aucun partage de brouillon en cours entre appareils, aucune reformulation automatique des questions ni interprétation des réponses libres.

## Scénarios utilisateur et vérification

### US1 P1 — Répondre à une question native
Un agent terminal lié à Bridget pose une question avec choix et descriptions. Android affiche le même texte dans la conversation et dans Questions. Une question à la fois peut être dépliée, agrandie, parcourue puis validée. Choix simple, multiple et texte libre sont conservés jusqu’à confirmation.
Acceptation : l’agent reçoit une seule réponse liée à la bonne question ; un second clic/rejeu ne produit aucune autre réponse. Un autre agent/appareil hors périmètre ne peut lire ni répondre.

### US2 P1 — Réponse personnelle et dictée
L’utilisateur peut remplacer les propositions ou compléter son choix par du texte. Le micro lance la dictée système. Le texte reconnu reste éditable et n’est envoyé que par validation explicite. Annuler la dictée ne modifie pas la réponse. Si aucun moteur vocal n’est disponible, un message clair laisse la saisie écrite utilisable.
Acceptation : sélection et complément arrivent ensemble, sans interprétation ni suppression. La dictée s’insère dans le bon brouillon et se relit avant envoi.

### US3 P1 — Réponse depuis la notification
Une poussée signale une nouvelle question. Le téléphone relit les faits par le canal authentifié avant d’afficher les choix et une réponse directe. Une question complexe ouvre le bon panneau. Une réponse devenue périmée est signalée ; elle n’est jamais envoyée à une nouvelle question.
Acceptation : application fermée, une réponse directe rejoint la bonne question une seule fois. La notification disparaît ou affiche l’issue confirmée. Aucun contenu sensible n’est placé dans la poussée.

### US4 P1 — Continuité raisonnable
Fermer puis rouvrir Android conserve le brouillon local. Une réponse depuis le terminal ou Android résout la même demande pour tous. Une coupure conserve une commande incertaine et permet son rejeu identique. Les questions annulées, expirées ou résolues ailleurs ne restent pas présentées comme répondables.
Acceptation : remise après coupure sans doublon ; arrêt/reprise du daemon sans perte ; conflit entre réponses visible et sans écrasement. Pas de brouillon partagé en direct entre Android et T3.

### US5 P1 — Adaptateurs et indépendance
Les questions natives Bridget fonctionnent avec T3 absent. Les modes terminal capables d’un échange structuré utilisent ce parcours. Les questions natives T3 sont présentées par un adaptateur qui conserve leur demande d’origine. Les terminaux purement interactifs sans callback ni montage Bridget ne sont pas pilotés par simulation de touches : leur limite est explicite et un lancement compatible est indiqué.
Acceptation : une recette native complète dans un environnement sans connexion ni composant T3, puis une recette de l’adaptateur T3. Aucun changement du contenu des questions, aucun redémarrage de fournisseur pour répondre.

## Exigences fonctionnelles

- FR001 Chaque demande porte un identifiant durable, son agent source, ses questions/options d’origine et sa capacité réelle de réponse.
- FR002 Les identités et périmètres autorisés proviennent de Bridget ; les identités fournies dans un texte de question ne font pas autorité.
- FR003 Afficher le texte complet et les descriptions ; choix simples/multiples et réponse libre supplémentaire ou de remplacement. Aucune option présélectionnée n’est envoyée sans geste explicite.
- FR004 Panneau repliable au-dessus de la saisie, compteur, navigation, agrandissement et reprise sans effacer les choix.
- FR005 Distinguer brouillon, envoi en cours, résultat inconnu, réponse reçue, prise en compte confirmée et résolution ailleurs.
- FR006 Réponse écrite et dictée éditable, avec conservation des sélections et du complément.
- FR007 Notifications authentifiées avec choix courts ou réponse directe ; questions complexes ouvertes dans leur conversation. Réévaluer le périmètre et l’état avant toute réponse.
- FR008 Même opération rejouée = même réponse/reçu. Même clé avec un contenu différent = refus. Deux réponses concurrentes ne se remplacent pas.
- FR009 Déconnexion, révocation, changement de projet, fin de session ou question devenue non reprenable ferment la possibilité de répondre sans réinjecter un prompt ordinaire.
- FR010 Les autres clients voient une résolution confirmée ; les brouillons restent locaux à leur appareil.
- FR011 Parcours natif indépendant de T3, accessible depuis terminal et Android. Les adaptateurs ne définissent pas le stockage ou le protocole central.
- FR012 Conserver les anciens parcours de décisions et de file Android. Ne pas transformer une autorisation d’exécution en question libre.
- FR013 Vérifier les parcours sur émulateur Android ; conserver les preuves réelles, tests, limites et reçus de livraison.

## Entités et limites

Demande humaine, questions/choix, réponse structurée (sélections et texte), opération de réponse, reçu du producteur, brouillon local. Données bornées, conservation7jours par défaut conformément aux échanges Bridget. Aucun secret demandé par ce parcours. Les limites de capacité doivent être visibles, sans troncature silencieuse.

## Critères de réussite

- SC001 Les trois modes de réponse (choix, texte, sélection+complément) sont reçus une seule fois par leur demande native, T3 absent.
- SC002 Une fermeture/réouverture conserve tous les champs non envoyés ; une réponse depuis l’autre client ferme la demande sous10s en connexion saine.
- SC003 Notification en arrière-plan : réponse courte et texte libre atteignent le bon agent ; une question déjà résolue ne peut déclencher une seconde remise.
- SC004 Aucun écart de texte/description/choix entre source et téléphone sur les fixtures ; aucune perte de sélection lors de la dictée ou de l’agrandissement.
- SC005 Courses, replays, coupures et reprise daemon sont testés. Contrôles d’accès négatifs vérifiés.
- SC006 Émulateur installé et démarré ; diaporama des circuits réels vérifié ; sources intégrées/poussées et binaires actifs attestés, avec conservations justifiées.

## Périmètre et dépendances

Réutiliser sessions087/155 (décisions et callbacks),177 (mobile/auth/notifications),178 (rendu/sécurité),180 (file/brouillons locaux). T3 sert de référence UX et d’adaptateur optionnel. Pas de synchronisation de brouillons non envoyés, modification des consignes sur la qualité des questions, nouveau service cloud vocal, dépense externe ou simulation de touches dans une TUI.
