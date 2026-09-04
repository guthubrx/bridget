# Feature Specification: Droits lisibles et vérifiables

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 088-droits
Titre: Droits lisibles et vérifiables
Statut: Draft
Priorité: P1
Créée: 2026-09-03
Dépendances: SPEC-080 (centre de contrôle), SPEC-083 (sécurité du contenu, sandbox d'artefacts), SPEC-085 (runtime Docker), SPEC-086 (projet système), SPEC-087 (pause et plafond), ADR-011 (borne de l'agent local)
Résumé:
- Contexte: six couches d'autorisation coexistent (sécurité du contenu locale, sandbox d'artefacts, sandbox et approbations des fournisseurs, runtime des agents, projet système, contrôle référent). Le 2026-09-03, l'interface a affiché « Bloqué par vos réglages locaux » sous des liens alors que rien d'utile n'était bloqué, et au même moment la sandbox d'un agent Codex rendait son shell inutilisable sans qu'aucun écran ne le dise. Le référent ne sait pas quelle couche a refusé, ni où agir.
- Objectif: qu'un refus dise toujours qui refuse, ce qu'il a empêché et le geste pour changer ça ; qu'une seule page « Droits » présente les autorisations par question posée au référent, avec trois profils en mode standard et le détail des mécanismes en mode expert ; et qu'un droit accordé à un agent puisse être vérifié par un essai réel plutôt que supposé.
- Risque principal: une page qui promet un droit que la couche réelle ne donne pas (profil affiché « Confiant », shell mort), ou un serveur relié qui parvient à desserrer la sécurité du navigateur du référent.
- Mitigation: chaque ligne de la page est reliée à un mécanisme nommé et à son lieu de stockage ; les lignes locales ne sont modifiables que localement et la page le dit ; l'action « Tester » constate le comportement réel, daté, au lieu de l'afficher par déduction.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `088-droits`
**Created**: 2026-09-03
**Status**: Draft
**Input**: User description: « Droits : rationaliser et rendre lisibles les autorisations de Bridget. […] Trois étapes : refus explicites ; une page Droits organisée par question avec profils Prudent, Équilibré, Confiant et un mode expert ; une action Tester qui constate le droit réel. »

## Contexte observé

Reconstitué le 2026-09-03 à partir du journal de session d'un agent Codex géré par Bridget sur le serveur, et du code de l'interface :

- Le référent demande « peux tu aller chercher des informations sur internet pour compléter ». L'agent fait quatre recherches web réelles et cite cinq sources publiques. L'interface affiche sous chacune : « Lien HTTPS externe — Bloqué par vos réglages locaux. » Le référent conclut que l'agent a été empêché d'aller sur internet. En réalité, seule l'ouverture du lien au clic est désactivée, par un réglage local du navigateur (« Sécurité du contenu », trois cases toutes désactivées à l'installation). Le message ne nomme ni la couche, ni le réglage, ni le geste.
- Dans la même session, cinq heures plus tôt, chaque commande shell de l'agent a rendu `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`. L'agent a conclu « le shell est inaccessible » et s'est arrêté. Rien dans l'interface n'a signalé qu'une sandbox du fournisseur, en politique lecture seule sur ce serveur, empêchait tout geste. Le référent l'a appris en lisant un journal brut.
- Les couches qui existent aujourd'hui, avec ce qu'elles protègent et où elles vivent :

| Couche | Ce qu'elle décide | Où elle vit | Qui peut la changer |
|---|---|---|---|
| Sécurité du contenu (SPEC-083) | liens, images distantes, aperçus de fichiers affichés dans le fil | le navigateur du référent, ou Bridget Desktop | le référent, localement |
| Sandbox d'artefacts HTML (SPEC-083) | ce qu'une page générée peut faire dans le navigateur | le navigateur | personne, par conception |
| Posture d'agent (registre) | sandbox et approbations du fournisseur : complet, ou découverte en lecture seule | le serveur, définition d'agent | l'exploitant, dans un fichier |
| Runtime des agents (SPEC-085) | image, réseau, exécutables autorisés | le serveur | l'exploitant |
| Projet système (SPEC-086) | les agents peuvent modifier Bridget ou non | le serveur, réglage expert | le référent, avec confirmation |
| Contrôle référent (SPEC-087) | pause, plafond d'objectifs automatiques | le serveur | le référent seul |

Aucune de ces couches ne parle la langue des autres. Le référent doit deviner laquelle a dit non.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Un refus dit qui refuse et quoi faire (Priority: P1)

Le référent voit un refus dans l'interface. Il lit, sans chercher, quelle couche a refusé, ce qu'elle a empêché précisément, et le geste qui change ce comportement. Quand le geste est un réglage local, il peut l'appliquer depuis le refus lui-même.

**Why this priority**: c'est la source directe de la confusion constatée ; c'est aussi la plus petite tranche livrable, sans nouveau mécanisme.

**Independent Test**: publier un message d'agent contenant un lien HTTPS avec les liens désactivés ; publier un tour d'agent dont la sortie contient une erreur de sandbox reconnue ; vérifier les deux rendus.

**Acceptance Scenarios**:

1. **Given** les liens externes désactivés localement, **When** un message contient un lien HTTPS, **Then** la carte du lien affiche « Ouverture au clic désactivée », nomme le réglage « Sécurité du contenu › Liens externes », et propose un geste « Autoriser les liens » qui active ce réglage local et remplace aussitôt la mention par le bouton d'ouverture, sans rechargement.
2. **Given** les liens externes désactivés, **When** un agent ou un serveur relié envoie un message qui prétend activer les liens, **Then** rien ne change et le geste reste réservé au référent dans son navigateur.
3. **Given** un agent géré dont le shell est bloqué par la sandbox du fournisseur (erreur reconnue dans sa sortie), **When** le tour se termine, **Then** le fil affiche, attribué à Bridget et non à l'agent, un refus « Le shell de cet agent est bloqué par la sandbox <fournisseur> sur <serveur> », la posture en cause (par exemple « découverte, lecture seule ») et le lien vers la ligne « Shell » de la page Droits.
4. **Given** un refus d'un mécanisme déjà explicite (pause active, plafond atteint, principal humain requis, capacité non négociée), **When** il est affiché, **Then** il suit la même forme : couche, chose empêchée, geste.
5. **Given** un refus dont Bridget ne connaît pas la couche, **When** il est affiché, **Then** il dit « couche inconnue » et cite la ligne brute, plutôt que d'inventer une cause.

---

### User Story 2 - Une page Droits par question, avec trois profils (Priority: P2)

Le référent ouvre une page « Droits » dans les paramètres. Elle pose trois questions et répond par des lignes en langage courant. En mode standard, il choisit un profil qui règle toutes les lignes d'un coup. En mode expert, chaque ligne se déplie et montre le mécanisme, le lieu de stockage et ce que le serveur ne peut pas changer.

**Why this priority**: c'est ce qui rend les couches lisibles ensemble ; elle dépend de la forme de refus de la story 1 pour que les liens « changer ça » aient une cible.

**Independent Test**: ouvrir la page, choisir chaque profil, vérifier l'état de chaque ligne et de chaque mécanisme sous-jacent ; basculer en expert et vérifier les détails affichés ; tenter de changer une ligne locale depuis le serveur et vérifier le refus.

**Acceptance Scenarios**:

1. **Given** la page Droits, **When** elle s'affiche, **Then** elle contient exactement trois blocs : « Ce que je vois », « Ce que les agents peuvent faire », « Combien ils décident seuls », et chaque ligne affiche son effet en une phrase au présent, par exemple « Les liens s'ouvrent après ton clic ».
2. **Given** le mode standard, **When** le référent choisit Prudent, Équilibré ou Confiant, **Then** toutes les lignes prennent la valeur du profil dans la même seconde, chaque ligne montre sa nouvelle phrase, et une installation neuve ou remise à zéro est en Prudent.
3. **Given** une ligne qui vit dans le navigateur, **When** un profil est choisi depuis un autre appareil ou envoyé par un serveur relié, **Then** cette ligne ne change pas sur cet appareil et la page l'indique : « Réglage local à ce navigateur, non piloté par le serveur ».
4. **Given** le mode expert, **When** le référent déplie une ligne, **Then** il voit le mécanisme piloté, le lieu de stockage, la valeur brute, et, pour une ligne d'agent, la posture du fournisseur qu'elle commande ; il peut modifier la ligne seule, ce qui fait passer le profil affiché à « Personnalisé ».
5. **Given** un profil choisi, **When** un mécanisme sous-jacent ne peut pas prendre la valeur demandée (par exemple un runtime dont le réseau est fermé par l'exploitant), **Then** la ligne l'affiche comme « Demandé : ouvert · Réel : fermé par le runtime du serveur » au lieu d'afficher la valeur demandée.
6. **Given** un agent qui tente de changer un droit par ses outils déclarés, **When** la demande arrive, **Then** elle est refusée avec un motif explicite et le refus est consigné.

---

### User Story 3 - Tester un droit au lieu de le supposer (Priority: P3)

Pour chaque ligne de « Ce que les agents peuvent faire », le référent clique « Tester ». Bridget demande à un agent un geste inoffensif et affiche le résultat réel, daté, avec la couche qui a répondu.

**Why this priority**: c'est ce qui aurait révélé le shell mort avant que le référent le découvre dans un journal ; elle s'appuie sur les lignes de la story 2.

**Independent Test**: pour un agent en posture découverte, tester « Shell » et obtenir un refus daté attribué à la sandbox ; pour un agent en posture complète, tester « Fichiers du projet » et obtenir une réussite datée.

**Acceptance Scenarios**:

1. **Given** la ligne « Shell » et un agent connecté choisi, **When** le référent clique Tester, **Then** l'agent exécute une commande triviale prédéfinie, et la ligne affiche sous trente secondes « Réussi » ou « Refusé par <couche> », l'heure, l'agent et la ligne brute du résultat.
2. **Given** la ligne « Internet », **When** Tester est cliqué, **Then** l'agent tente de joindre une page connue et fixe, et le résultat distingue « joint », « refusé par la sandbox », « refusé par le runtime » et « injoignable » sans pouvoir les confondre.
3. **Given** la ligne « Fichiers du projet », **When** Tester est cliqué, **Then** l'agent lit un fichier connu du projet et le résultat rapporte la lecture ou le refus, sans jamais afficher le contenu du fichier.
4. **Given** un agent en plein tour, **When** Tester est cliqué, **Then** l'essai est refusé « agent occupé » sans interrompre le tour.
5. **Given** un résultat de test, **When** le référent revient plus tard, **Then** le dernier résultat daté reste affiché sur la ligne jusqu'au prochain test, et n'est jamais présenté comme l'état courant s'il a plus d'un jour.

---

### Edge Cases

- Bridget Desktop pilote la sécurité du contenu à la place du navigateur : la page Droits affiche les lignes correspondantes en lecture seule avec le renvoi vers Desktop, et le geste « Autoriser les liens » renvoie de même.
- Deux navigateurs du même référent : les lignes locales diffèrent d'un navigateur à l'autre ; la page le dit, et le profil affiché est « Personnalisé » si les lignes locales ne correspondent pas au profil serveur.
- Un profil est changé pendant la pause : les lignes « Combien ils décident seuls » s'appliquent immédiatement ; changer le profil ne lève jamais la pause.
- Un test est lancé alors qu'aucun agent n'est connecté : la ligne affiche « aucun agent disponible », sans résultat inventé.
- Une erreur de sandbox non reconnue dans la sortie d'un agent : rien n'est attribué ; le refus générique « couche inconnue » s'affiche seulement si l'agent lui-même déclare un blocage.
- Un serveur relié envoie un état de droits qui prétend modifier une ligne locale : la ligne est ignorée, l'événement est consigné, et la page le signale une fois.

## Requirements *(mandatory)*

### Functional Requirements

Refus explicites (US1)

- **FR-001**: Tout refus affiché au référent DOIT porter trois éléments : la couche qui refuse, ce qu'elle a empêché, et le geste qui change ce comportement, ou la mention qu'aucun geste n'existe.
- **FR-002**: Sous un lien dont l'ouverture est désactivée, l'interface DOIT afficher « Ouverture au clic désactivée », nommer le réglage local et proposer un geste qui l'active sur place ; ce geste DOIT rester inaccessible à un message, à un agent et à un serveur relié.
- **FR-003**: Quand la sortie d'un agent géré contient une erreur de sandbox reconnue par Bridget, le fil DOIT afficher un refus attribué à Bridget nommant le fournisseur, le serveur, la posture en cause et le lien vers la ligne concernée de la page Droits. La liste des erreurs reconnues est fermée et documentée.
- **FR-004**: Les refus existants (pause, plafond, principal humain requis, capacité non négociée, origine humaine interdite) DOIVENT être rendus dans la même forme que FR-001.
- **FR-005**: Un refus dont la couche est inconnue DOIT le dire et citer la ligne brute ; Bridget NE DOIT PAS attribuer une cause par déduction.

Page Droits (US2)

- **FR-010**: Le centre de contrôle DOIT proposer une page « Droits » à trois blocs, « Ce que je vois », « Ce que les agents peuvent faire », « Combien ils décident seuls », chaque ligne portant une phrase d'effet au présent.
- **FR-011**: Les lignes sont un inventaire fermé : liens externes, images distantes, fichiers du projet en aperçu, artefacts HTML (bloc 1) ; internet, fichiers du projet, shell, modifier Bridget (bloc 2) ; pause, plafond d'objectifs automatiques, réassignation automatique (bloc 3). Ajouter une ligne est une décision de spécification.
- **FR-012**: Trois profils fermés, Prudent, Équilibré, Confiant, DOIVENT régler toutes les lignes d'un coup ; leur matrice est documentée dans la spec et affichée en mode expert ; le profil à l'installation et après remise à zéro est Prudent.
- **FR-013**: Une ligne qui vit dans le navigateur NE DOIT être modifiable que depuis ce navigateur ; un profil choisi ailleurs ou reçu d'un serveur relié NE DOIT PAS la changer, et la page DOIT le dire sur la ligne.
- **FR-014**: En mode expert, chaque ligne DOIT montrer le mécanisme piloté, le lieu de stockage, la valeur brute et, pour une ligne d'agent, la posture de fournisseur commandée ; modifier une ligne seule DOIT faire passer le profil affiché à « Personnalisé ».
- **FR-015**: Quand un mécanisme ne peut pas prendre la valeur demandée par le profil, la ligne DOIT afficher la valeur demandée ET la valeur réelle, jamais la seule valeur demandée.
- **FR-016**: Les autorisations internes des fournisseurs (Codex, Claude, autres) NE DOIVENT PAS être recopiées ligne par ligne ; le profil choisit une posture de fournisseur parmi celles que Bridget sait lancer.
- **FR-017**: Un droit serveur NE DOIT être modifiable que par le principal humain reconnu (interface ou terminal interactif) ; une tentative par un agent ou un serveur relié DOIT être refusée avec motif et consignée.
- **FR-018**: Le profil et les lignes serveur DOIVENT survivre à un redémarrage du service.

Tester un droit (US3)

- **FR-020**: Chaque ligne du bloc « Ce que les agents peuvent faire » DOIT proposer une action « Tester » qui demande à un agent connecté et libre un geste inoffensif prédéfini, et affiche le résultat réel, daté, avec l'agent et la couche qui a répondu.
- **FR-021**: Les gestes de test sont fermés et documentés : commande triviale pour « Shell », lecture d'un fichier connu du projet pour « Fichiers », requête vers une page connue et fixe pour « Internet », lecture d'un fichier du checkout Bridget pour « Modifier Bridget ». Aucun geste n'écrit, n'installe, ni ne dépense.
- **FR-022**: Le résultat DOIT distinguer réussite, refus par la sandbox du fournisseur, refus par le runtime du serveur, refus par Bridget, et injoignable ; il DOIT conserver la ligne brute qui permet de le contredire.
- **FR-023**: Un test NE DOIT PAS interrompre un tour en cours ; un agent occupé rend « occupé » ; aucun agent disponible rend « aucun agent ».
- **FR-024**: Le dernier résultat daté DOIT rester affiché sur la ligne ; au-delà d'un jour il DOIT être présenté comme ancien, jamais comme l'état courant.
- **FR-025**: Un résultat de test DOIT être consigné avec l'agent, la date, le geste et la ligne brute, pour être relu plus tard.

### Key Entities

- **Ligne de droit** : question posée (bloc), nom, phrase d'effet, mécanisme piloté, lieu de stockage (navigateur, serveur, fournisseur), valeur demandée, valeur réelle, modifiable localement seulement ou non.
- **Profil** : Prudent, Équilibré, Confiant, Personnalisé ; matrice profil → valeur par ligne.
- **Refus** : couche, chose empêchée, geste, ligne brute, horodatage, attribution (Bridget ou agent).
- **Résultat de test** : ligne testée, agent, geste, issue (réussi, refusé par sandbox, refusé par runtime, refusé par Bridget, injoignable, occupé, aucun agent), ligne brute, date.
- **Posture de fournisseur** : ensemble fermé des façons dont Bridget lance un agent (aujourd'hui complet ou découverte en lecture seule), commandé par le profil.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sur le scénario du 2026-09-03 rejoué (liens désactivés, sandbox bloquée), 100 % des refus affichés nomment la couche et le geste ; le référent active les liens en un clic sans quitter le fil.
- **SC-002**: Un blocage de shell par la sandbox d'un fournisseur est visible dans le fil dans la minute qui suit la fin du tour, sans lecture de journal.
- **SC-003**: Depuis la page Droits, choisir un profil met toutes les lignes serveur à jour en moins d'une seconde, et chaque ligne locale indique qu'elle n'a pas bougé quand c'est le cas.
- **SC-004**: Aucune ligne locale ne peut être modifiée par un serveur relié, un agent ou un message : 0 réussite sur un jeu d'essais adverses documenté.
- **SC-005**: Un test de droit rend un résultat daté et attribué en moins de trente secondes, et un refus de sandbox y est reconnu comme tel dans 100 % des cas du jeu d'essais.
- **SC-006**: Un référent qui n'a jamais vu les couches retrouve, à partir d'un refus, la ligne à changer en moins d'une minute, sur les cinq refus du jeu d'essais.

## Assumptions

- Les postures de fournisseur disponibles sont celles que Bridget lance déjà : complète (sandbox et approbations contournées) et découverte (lecture seule, sans approbation). Le profil choisit entre elles ; une posture intermédiaire n'est pas dans le périmètre.
- « Modifier Bridget » réutilise le réglage expert du projet système (SPEC-086) ; « Pause » et « Plafond » réutilisent l'état de contrôle (SPEC-087) ; « Réassignation automatique » réutilise la conduite de réassignation de Maicie ; les lignes du bloc 1 réutilisent la sécurité du contenu (SPEC-083).
- La matrice des profils, à valider en revue : Prudent = tout fermé sauf pause inactive, plafond 5, posture découverte, réassignation différée ; Équilibré = liens et aperçus au clic, images fermées, artefacts inline, posture complète, plafond 5, réassignation active ; Confiant = tout ouvert, plafond 20, posture complète, réassignation active. Modifier Bridget reste fermé dans les trois profils et ne s'ouvre qu'en expert avec confirmation.
- Le principal humain est reconnu comme dans SPEC-087 : connexion de l'interface sous l'identité humaine, ou terminal interactif.
- Les tests de droit passent par un agent existant ; la feature ne lance pas d'agent dédié.
- Bridget Desktop, quand il pilote la sécurité du contenu, reste la source de vérité pour ces lignes ; la page les affiche en lecture seule.

## Hors périmètre

- Éditer les règles internes de sandbox de Codex ou Claude, ou en créer une posture nouvelle.
- Piloter le runtime Docker de SPEC-085 ligne par ligne ; la page affiche sa valeur réelle, elle ne la change pas.
- Un registre de droits par agent : les profils sont globaux à l'installation.
