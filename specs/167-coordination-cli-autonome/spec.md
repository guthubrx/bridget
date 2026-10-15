# Feature Specification: Coordination CLI autonome

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 167-coordination-cli-autonome
Titre: Coordination CLI autonome
Statut: In Progress
Priorité: P1
Tâches: 27/28 (96%)
Tests: 87/87 (100%)

Résumé:
- Contexte: Bridget sait déjà lancer et suivre des agents, mais le parcours complet reste surtout accessible par MCP ou par plusieurs commandes techniques.
- Objectif: permettre depuis un terminal de créer un agent, lui confier une mission et suivre son résultat avec une interface simple et autonome.
- Dépendances: SPEC-148, SPEC-149, SPEC-151, SPEC-153, SPEC-155, SPEC-157, SPEC-163

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-167-coordination-cli-autonome`
**Created**: 2026-10-15
**Status**: In Progress
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-151, SPEC-153, SPEC-155, SPEC-157, SPEC-163

## Problème

Bridget peut communiquer avec des agents, lancer des équipiers et gérer des délégations durables. Cependant, un agent coordinateur lancé depuis le terminal ne dispose pas d'un parcours simple et homogène pour confier une mission complète.

La création d'un agent et l'envoi de sa mission sont séparés. La délégation automatique exige plusieurs paramètres techniques. Le suivi des tâches natives est riche par MCP, mais il n'est pas exposé avec la même simplicité dans la ligne de commande.

L'utilisateur doit pouvoir employer un agent Bridget comme coordinateur autonome depuis son terminal. T3 peut enrichir cette expérience, mais il ne doit pas être nécessaire pour créer, suivre, continuer ou arrêter une tâche.

## Histoires utilisateur

### US1 — Confier une mission en une commande (P1)

Comme utilisateur d'un agent Bridget lancé dans un terminal, je veux décrire une mission avec une seule commande afin que le coordinateur choisisse ou crée un agent et démarre son travail sans configuration répétitive.

**Test indépendant** : depuis un projet reconnu, une mission simple est acceptée avec la seule description de la tâche. La commande rend immédiatement un identifiant durable et l'agent commence son travail.

**Scénarios d'acceptation** :

1. Étant donné un projet reconnu et au moins un type d'agent admissible, quand l'utilisateur transmet seulement une mission, alors Bridget choisit un candidat autorisé et démarre une tâche.
2. Étant donné qu'un profil est admissible, quand la politique autorise un lancement, alors Bridget crée l'agent puis lui remet la mission.
3. Étant donné qu'aucun candidat n'est admissible, quand l'utilisateur transmet la mission, alors Bridget rend un refus explicite sans créer de tâche partielle.
4. Étant donné une commande acceptée, quand le terminal rend la main, alors il affiche au minimum l'identifiant de tâche, l'agent choisi et l'état courant.

### US2 — Garder le contrôle des choix avancés (P1)

Comme utilisateur expérimenté, je veux pouvoir choisir un type d'agent, un modèle, un effort, une posture ou un dossier de travail sans perdre le parcours simple par défaut.

**Test indépendant** : chaque sélection explicite est soit respectée exactement, soit refusée avec sa contrainte. Aucune sélection silencieuse ne remplace un choix demandé.

**Scénarios d'acceptation** :

1. Étant donné un type d'agent disponible, quand l'utilisateur le demande, alors la tâche utilise ce type.
2. Étant donné un modèle indisponible, quand l'utilisateur le demande, alors la commande refuse avec la raison réelle et ne choisit pas un autre modèle.
3. Étant donné un dossier de travail explicite, quand la tâche démarre, alors l'admission et l'exécution utilisent ce dossier.
4. Étant donné aucun choix avancé, quand la tâche démarre, alors les valeurs par défaut viennent de la politique Bridget existante.

### US3 — Piloter le cycle de vie depuis le terminal (P1)

Comme utilisateur, je veux lister, lire, attendre, continuer et annuler mes tâches afin de coordonner des agents sans interface graphique.

**Test indépendant** : une tâche créée depuis le terminal peut être suivie jusqu'à son résultat, continuée quand sa garde le permet, puis clôturée ou annulée avec les mêmes états que par MCP.

**Scénarios d'acceptation** :

1. Étant donné une tâche active, quand l'utilisateur demande son état, alors la commande affiche son état durable et les actions possibles.
2. Étant donné une ou plusieurs tâches, quand l'utilisateur attend leur résultat, alors la commande se réveille sur une issue utile sans interrogation répétée.
3. Étant donné une tâche terminée et encore continuable, quand l'utilisateur transmet une nouvelle mission, alors une nouvelle ronde s'ouvre sur le même agent.
4. Étant donné un enfant qui attend une réponse, quand l'utilisateur répond depuis le terminal, alors la réponse est liée à la bonne demande.
5. Étant donné une tâche annulable, quand l'utilisateur l'annule, alors sa descendance suit les règles de sécurité existantes.

### US4 — Automatiser sans dépendre de T3 (P2)

Comme utilisateur de scripts et d'autres interfaces, je veux une sortie stable et exploitable par machine afin d'intégrer Bridget à mon environnement sans dépendre de T3.

**Test indépendant** : chaque commande du nouveau parcours fournit un format machine stable, conserve les identifiants durables et fonctionne lorsque T3 est arrêté.

**Scénarios d'acceptation** :

1. Étant donné une commande de coordination, quand le format machine est demandé, alors la sortie contient des champs stables et aucun texte décoratif parasite.
2. Étant donné une nouvelle tentative avec la même clé et les mêmes données, quand l'issue précédente était incertaine, alors Bridget ne crée pas une seconde tâche.
3. Étant donné que T3 est absent, quand le daemon Bridget et le pilote requis sont disponibles, alors le parcours complet reste utilisable.
4. Étant donné une tâche créée par MCP, quand le terminal la consulte, alors les identifiants et les états ont le même sens.

## Exigences fonctionnelles

- **FR-001.** Un agent coordinateur Bridget doit pouvoir confier une mission en ligne de commande en fournissant seulement son texte.
- **FR-002.** Le parcours simple doit détecter le projet courant et appliquer la politique d'admission existante.
- **FR-003.** Le système doit choisir un profil admissible puis créer l'agent selon la décision durable de la politique existante.
- **FR-004.** Une acceptation doit rendre l'identifiant de tâche, l'identité de l'agent choisi et l'état courant avant que le travail long soit terminé.
- **FR-005.** L'utilisateur doit pouvoir sélectionner explicitement le type d'agent, le modèle, l'effort, la posture, le dossier de travail, le délai de réponse et la conservation pour une ronde suivante.
- **FR-006.** Une sélection explicite indisponible doit être refusée. Le système ne doit pas lui substituer silencieusement une autre valeur.
- **FR-007.** Les valeurs non précisées doivent venir des politiques et valeurs par défaut existantes. La nouvelle interface ne doit pas créer une seconde politique de sélection.
- **FR-008.** Le terminal doit proposer une liste humaine globale et bornée des tâches. La lecture détaillée et les mutations d'une tâche doivent rester limitées à son coordinateur propriétaire.
- **FR-009.** Le terminal doit permettre d'attendre une ou plusieurs tâches sans interrogation répétée et sans annuler le travail à l'expiration de l'attente.
- **FR-010.** Le terminal doit permettre de continuer une tâche encore gardée avec une nouvelle mission et une nouvelle clé de demande.
- **FR-011.** Le terminal doit permettre de répondre à une question actionnable de l'enfant.
- **FR-012.** Le terminal doit permettre d'annuler une tâche et sa descendance selon les règles existantes.
- **FR-013.** Le terminal doit permettre de déclarer la lecture d'un résultat sans confondre lecture, remise et purge.
- **FR-014.** Chaque commande du nouveau parcours doit proposer une sortie lisible par humain et une sortie structurée pour l'automatisation.
- **FR-015.** Les nouvelles commandes doivent utiliser les mêmes tâches, états, droits, quotas, reçus, refus et règles de rejeu que les outils MCP correspondants.
- **FR-016.** Une nouvelle tentative portant la même clé et la même enveloppe doit rendre l'état durable existant sans créer de doublon.
- **FR-017.** Une nouvelle tentative portant la même clé et une enveloppe différente doit être refusée explicitement.
- **FR-018.** Le parcours complet doit fonctionner sans processus T3 actif.
- **FR-018a.** Une commande qui exige un coordinateur doit refuser une invocation sans identité Bridget. Le refus doit indiquer comment lancer ou rattacher un agent. La commande ne doit pas fabriquer une identité d'agent pour un processus humain.
- **FR-019.** Les commandes historiques de lancement, délégation automatique, envoi et contrôle doivent rester compatibles.
- **FR-020.** La solution ne doit ajouter ni nouveau fournisseur, ni nouveau moteur de tâche, ni nouvelle base, ni nouveau service permanent.
- **FR-021.** Les erreurs doivent indiquer une cause exploitable et la prochaine action sûre, sans exposer de secret.
- **FR-022.** Le parcours simple et chaque action de cycle de vie doivent avoir un test déterministe couvrant succès, refus et rejeu pertinent.

## Cas limites

- Le terminal n'est pas attaché à une identité d'agent, mais le projet courant est connu : la commande refuse avant tout effet et explique le rattachement requis.
- Plusieurs agents admissibles ont le même niveau de préférence.
- Le candidat choisi devient indisponible entre l'admission et le lancement.
- La commande est interrompue après le commit durable, avant l'affichage du reçu.
- L'attente expire pendant que l'enfant travaille encore.
- Une tâche se termine pendant l'ouverture d'une nouvelle ronde.
- L'enfant pose une question au même moment que son annulation.
- Le daemon redémarre pendant une attente.
- La sortie humaine est affichée dans un terminal sans couleur.
- La sortie structurée est redirigée vers un script.

## Critères de succès

- **SC-001.** Un coordinateur authentifié peut démarrer le parcours nominal avec une seule commande et un seul argument métier : la mission.
- **SC-002.** Dans 100 % des scénarios d'acceptation, une commande acceptée affiche un identifiant durable avant la fin du travail délégué.
- **SC-003.** Dans 100 % des tests de sélection explicite, le choix est respecté exactement ou refusé sans substitution.
- **SC-004.** Les états observés depuis le terminal et depuis MCP sont identiques dans 100 % des scénarios croisés.
- **SC-005.** Une reprise avec la même clé ne crée aucun doublon dans 100 % des scénarios de rejeu.
- **SC-006.** Le parcours créer, suivre, attendre, continuer, répondre, accuser lecture et annuler réussit sans processus T3 dans une recette isolée.
- **SC-007.** Les commandes historiques couvertes par la régression conservent leur comportement et leur format documenté.
- **SC-008.** Aucun service permanent, stockage ou dépendance externe supplémentaire n'est nécessaire.
- **SC-009.** Un nouvel utilisateur peut exécuter le parcours nominal à partir de l'aide intégrée sans consulter le code source.

## Hypothèses

- Le daemon Bridget et au moins un pilote d'agent sont installés et accessibles.
- La politique d'admission existante reste l'autorité de sélection et de quota.
- Le dossier courant représente le projet par défaut, sauf sélection explicite.
- Le client en ligne de commande peut générer automatiquement une clé de demande sûre et l'afficher dans le reçu.
- Le parent d'une tâche native reste une identité d'agent Bridget vivante. Une session humaine seule ne devient pas un parent d'agent.
- Les tâches natives actuelles restent la source de vérité du cycle de vie.

## Hors périmètre

- Reproduire l'interface graphique, les worktrees ou le graphe d'exécution de T3.
- Ajouter un planificateur de graphes ou un langage de workflows.
- Ajouter un nouveau fournisseur ou modifier l'authentification d'un fournisseur.
- Modifier les décisions d'admission, les quotas ou les droits existants.
- Créer une interface plein écran dans le terminal.
- Déduire qu'une fin de tour constitue une réussite de mission.
- Déployer ou configurer automatiquement T3.

## Dépendances

- **SPEC-148** : identité de délégation et résultat corrélé.
- **SPEC-149** : filiation, identité durable et visibilité Lineage.
- **SPEC-151** : attente durable des résultats et réponses.
- **SPEC-153** : continuation du même enfant sur plusieurs rondes.
- **SPEC-155** : demandes de l'enfant et réponse du parent.
- **SPEC-157** : catalogue des capacités et quotas de lancement.
- **SPEC-163** : décision d'admission et sélection des fournisseurs.
