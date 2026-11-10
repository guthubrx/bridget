# Session 258 — Conversation complète des sous-agents

Statut : In Progress. Autorisation : demande my-specify-all du 10 novembre 2026.
Dépendances : 149 (lignée native), 154 (workspace), 155 (questions), 174/196
(lectures), 241 (fiabilité), 246 (filiation), 251 (présentation commune).

## Demande initiale et cadrage

François demande les pages normales des agents pour les sous-agents : défilement
automatique, qualité de regroupement des outils, et la même boîte inférieure
pour parler, changer le modèle et l'effort. Il relancera lui-même l'application.

Demande reformulée : réutiliser l'interface de conversation existante pour les
pages Bridget et raccorder ses actions à l'enfant natif existant. Le parent,
la mission, la ronde et le résultat restent sous l'autorité de Bridget. Les
confirmations reflètent les effets réellement observés, localement et à distance.
Apport : une seule expérience et une seule implémentation des fonctions communes.

## US1 — Lire et suivre une conversation (P1)

Le lecteur retrouve les messages, le Markdown, les cadres de code, la copie,
les outils regroupés/repliables, leurs durées connues et leurs erreurs dans les
composants normaux. Les lacunes restent visibles, sans succès ou durée inventés.
Le journal suit la fin quand le lecteur est en bas ; remonter suspend le suivi.
Le bouton de retour en bas le réactive. Changer de page n'altère aucune mission.

## US2 — Parler à l'enfant (P1)

La boîte commune accepte la saisie multiligne et les raccourcis habituels,
conserve son brouillon et l'historique de saisie, et remet une intervention à
l'enfant affiché. L'état de remise distingue accepté, refusé et issue inconnue.
Une répétition avec la même clé ne produit aucun doublon. Une erreur conserve
le texte. Un agent occupé suit la politique native ; aucun tour T3 indépendant
n'est créé sur la projection. Une interruption ne vaut pas annulation de mission.
Les capacités de contexte et de pièces jointes sont celles réellement
transportables sur la cible ; un fichier distant n'est jamais ouvert localement
par supposition. Toute capacité absente est explicitement expliquée dans la page.

## US3 — Choisir modèle et effort (P1)

Les contrôles visuels communs montrent la sélection constatée et les choix
admis par la cible. La validation fournisseur confirme ou refuse une demande.
Le tour courant n'est pas remplacé ; le prochain tour emploie le choix confirmé.
Une coupure conserve une issue inconnue et ne déclenche aucune substitution.
Les limites du pilote sont explicites, sans élargissement de droits.

## US4 — États et livraison (P1)

Un enfant terminé, libéré, révoqué ou déconnecté reste lisible selon les preuves
conservées. Les commandes impossibles sont désactivées avec une cause utile.
Les pages normale et Bridget gardent la même présentation et la même boîte.
La distinction local/distant et fournisseur reste lisible dans la lignée.
Le paquet local T3 est construit, vérifié, signé et préparé pour la relance de
François. Aucun arrêt de T3 par ce travail. Version installée et version chargée
sont distinguées. Les services Bridget modifiés suivent leur livraison native.

## Exigences et critères de réussite

- FR01 : partager le rendu et les contrôles ; aucun second éditeur/rendu.
- FR02 : suivre/suspendre/rétablir le défilement sur un journal vivant.
- FR03 : conserver les regroupements et états des outils, dont les sorties partielles.
- FR04 : envoyer de manière corrélée et idempotente à la bonne tâche/instance/ronde.
- FR05 : préserver brouillon et issue inconnue ; aucun rejeu sous une nouvelle clé.
- FR06 : garder les commandes métier de la projection T3 ordinaires interdites.
- FR07 : confirmer modèle/effort auprès du pilote réel et préserver le parent.
- FR08 : vérifier propriétaire, projet, autorité, présence et révocations avant effets.
- FR09 : prendre en charge local/distant et signaler fin/perte du lien sans relance.
- FR10 : conserver les fonctions de boîte communes selon capacités attestées.
- FR11 : vérifier l'usage visible avec clics réels et diaporama léger.
- FR12 : intégrer/pousser/nettoyer et préparer l'installation sans relancer T3.

La recette utilise des données fictives et des transports de test. Aucun rejeu
des missions Parix ou essais fournisseur sans nécessité. Les droits et plafonds
existants sont conservés. Pas de fusion des 105 commits amont T3 par opportunité.
La parité couvre les pages web/desktop étudiées ; le mobile conserve ses contrats
et reçoit les adaptations de compatibilité nécessaires, sans seconde refonte UI.

## Qualité de la spécification

- [x] Demande, reformulation, exclusions et bénéfice conservés.
- [x] Chaque histoire comporte un résultat observable.
- [x] Aucun changement de fournisseur, compte ou droit implicite.
- [x] Résultat natif, état de remise et fin de tour distingués.
- [x] Activation T3 par l'utilisateur explicitement exclue du travail automatique.

## Extension de correction — incident du10novembre

FR13 : pour le refus OAuth observé dans la revue du fil COPIL Thémys et celle du lot258, conserver le diagnostic attesté dans le refus et son guidage. Seul un diagnostic exact corrélé au terminal fournisseur api_error établit une session expirée. Un texte cité dans un tour réussi ou un ancien api_error reste distinct. Expliquer la reconnexion du compte existant sur la machine concernée, conserver clé/tâche/fichiers et interdire toute relance implicite. Aucun document métier ni compte modifié par ce correctif.
