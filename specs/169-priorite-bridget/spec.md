# Feature Specification: Priorité Bridget pour les sous-tâches depuis T3

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 169-priorite-bridget
Titre: Priorité Bridget pour les sous-tâches depuis T3
Statut: Implemented
Priorité: P1
Tâches: 14/14 (100%)
Pipeline: In Progress
Tests: 536/536 ciblés R2 ; R1 conservé, aucun GREEN global

Résumé:
- Contexte: T3 et Bridget proposent chacun un parcours de délégation. L'utilisateur veut privilégier le choix automatique Bridget depuis ses conversations T3.
- Objectif: aligner les consignes de délégation T3 sur une préférence de son intégration Bridget, activable et désactivable dans le fichier de réglages T3 existant.
- Dépendances: SPEC-149, SPEC-163, SPEC-164, SPEC-166, SPEC-168.

Fichiers:
- spec.md: présent — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/spec.md
- tasks.md: 14/14 acceptées par ROOT — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/tasks.md
- plan.md: conception validée par ROOT — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/plan.md
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: session-169-priorite-bridget

**Created**: 2026-10-15

**Status**: Implemented

**Pipeline**: In Progress

**Priority**: P1

**Dependencies**: SPEC-149, SPEC-163, SPEC-164, SPEC-166, SPEC-168

**Source initiale**: 9077b81e

**Autorisation**: invocation explicite de `$my-specify-all` pour le périmètre de cette session.

## Problème et valeur attendue

L'utilisateur veut déléguer des sous-tâches depuis T3 sans choisir à chaque fois leur moteur, leur modèle ou leur machine. Bridget possède déjà ce choix et son moteur d'exécution autonome. Le raccord T3 doit orienter les nouvelles délégations vers ce parcours automatique.

Cette priorité doit être réversible par fichier. Elle ne crée aucun droit. Elle ne déplace aucune tâche déjà lancée. T3 garde la conversation principale et présente les tâches Bridget dans Lineage. Bridget reste responsable du choix, de l'admission, de l'exécution et du résultat.

Le périmètre porte sur des consignes générées cohérentes. Un texte de consigne ne garantit pas l'obéissance absolue d'un modèle. Cette session n'ajoute pas d'intercepteur ni de verrou sur les outils natifs T3.

## Histoires utilisateur et tests indépendants

### US1 — Privilégier la délégation automatique Bridget (P1)

Comme utilisateur, je veux que les consignes de mon agent T3 privilégient Bridget pour une nouvelle sous-tâche. Je laisse Bridget choisir les axes que je n'ai pas imposés.

**Test indépendant** : avec la préférence active, un nouveau tour reçoit une consigne qui désigne le parcours automatique Bridget. Une délégation conforme crée une seule tâche native rattachée au parent. Elle n'ajoute aucune délégation native T3 en parallèle.

1. Étant donné la préférence active, quand T3 prépare les consignes d'un nouveau tour, elles demandent d'utiliser Bridget en priorité pour déléguer une nouvelle sous-tâche.
2. Si aucun modèle, moteur, compte, effort ou machine n'est imposé par l'utilisateur, la consigne laisse ces axes au choix Bridget existant. Le modèle du parent n'est pas transformé en contrainte pour l'enfant.
3. Si l'utilisateur demande explicitement un nouveau fil T3, la consigne conserve cette exception. Les autres outils T3 restent utilisables pour leur fonction habituelle.

### US2 — Activer et désactiver la préférence par fichier (P1)

Comme utilisateur, je veux régler cette préférence dans le fichier de réglages T3 existant. Ce champ concerne l'intégration Bridget. Je ne veux pas de nouveau panneau ni de seconde configuration à maintenir.

**Test indépendant** : la clé absente et la valeur OFF produisent les consignes T3 habituelles. La valeur ON produit la priorité Bridget. Une valeur invalide ou une erreur de lecture est explicite et ne transforme pas ON en OFF.

1. Sans clé de préférence, une installation existante garde le comportement OFF.
2. Avec OFF explicite, la priorité Bridget disparaît des nouvelles consignes. Les demandes explicites de Bridget restent soumises aux capacités et droits existants.
3. Avec ON explicite, les nouvelles consignes portent la priorité Bridget. Le réglage est actualisé avant chaque tour. Un changement ne réécrit pas un tour déjà lancé.
4. Si la configuration ne peut pas être résolue, le raccord signale cette situation. Il ne fabrique pas des consignes OFF comme si la lecture avait réussi.
5. Actualiser cette préférence conserve l'identité attestée du fil. Cette lecture ne provoque pas sa rotation.

### US3 — Respecter les choix explicites et les droits (P1)

Comme utilisateur, je veux conserver mes contraintes et mes permissions quand je délègue par Bridget. La priorité ne doit pas changer ce que mon agent est autorisé à faire.

**Test indépendant** : une délégation avec un modèle ou une machine imposés conserve ces contraintes. Une contrainte impossible ou un droit insuffisant produit un refus explicite. Aucun autre moteur n'est choisi pour contourner ce refus.

1. Toute contrainte explicite de modèle, moteur, connexion, machine, effort ou espace de travail reste prioritaire sur le choix automatique.
2. Les axes non imposés suivent les règles Bridget existantes. T3 ne recalcule pas un classement de flotte.
3. Une preuve d'identité absente, étrangère, expirée ou révoquée reste refusée. L'activation de la préférence n'active aucun outil désactivé et n'ajoute aucun accord humain implicite.
4. Une politique de droits sans équivalent sûr sur la cible reste refusée. Aucune délégation native T3 de secours n'est lancée par l'hôte après ce refus.

### US4 — Rendre les erreurs explicites sans double lancement (P1)

Comme utilisateur, je veux comprendre un échec Bridget et éviter qu'une même sous-tâche parte aussi dans T3.

**Test indépendant** : l'indisponibilité de Bridget, un outil absent, une désactivation explicite et un refus d'admission donnent chacun une issue explicite. L'hôte ne lance aucun moteur de secours. Un rejeu retrouve la tâche initiale lorsqu'elle existe.

1. Avec ON, les consignes interdisent le basculement silencieux vers la délégation T3 après une absence, une panne ou un refus Bridget. Elles demandent de signaler le blocage.
2. L'hôte ne transforme pas une erreur Bridget en appel automatique à un outil de délégation T3.
3. Une réponse perdue après admission est traitée avec les identifiants et mécanismes de suivi ou de rejeu existants. Elle ne justifie pas une seconde exécution dans un autre moteur.
4. Une panne avant admission ne crée aucune tâche par ce raccord. Une panne après admission conserve la tâche et son état réel, sans annoncer à tort son succès ou son absence.
5. Avec ON sur un raccord qui ne monte pas Bridget, la consigne annonce cette indisponibilité. Elle ne décrit pas un outil absent comme utilisable et ne prescrit pas de bascule silencieuse.

### US5 — Suivre chaque tâche par son moteur initial (P1)

Comme utilisateur, je veux suivre les tâches Bridget dans Lineage et conserver les tâches T3 déjà lancées. Un changement de préférence ne doit pas perturber mon travail.

**Test indépendant** : une tâche Bridget et une tâche native T3 lancées avant un changement de mode gardent leur moteur et leurs références. La tâche Bridget conserve son statut et son résultat dans Lineage. Une lecture ne relance rien.

1. Les tâches Bridget gardent la relation parent/enfant, le journal, le statut et le résultat natifs déjà projetés dans Lineage.
2. Les tâches natives T3 déjà lancées restent suivies par leur parcours T3. Les tâches Bridget déjà admises restent exécutées et suivies par Bridget, même après OFF.
3. Ouvrir Lineage, lire le résultat ou reprendre la lecture après une coupure ne crée ni tâche, ni conversation de premier niveau, ni résultat synthétique supplémentaire.
4. Une panne ou la fermeture de T3 ne retire pas l'autonomie d'une tâche Bridget déjà admise.

## Exigences fonctionnelles

- **FR001** : L'intégration Bridget DOIT ajouter une unique préférence ON/OFF dans le fichier de réglages T3 existant. Ce champ concerne l'environnement T3, sans politique propre à chaque projet. Aucune nouvelle interface de réglage n'est requise.
- **FR002** : La clé absente DOIT valoir OFF pour préserver les installations existantes. L'activation du lot livré DOIT être un ON explicite, distinct de la valeur par défaut.
- **FR003** : Une valeur invalide ou une erreur de lecture DOIT produire une issue explicite, même si l'ancien mode était OFF. Elle ne DOIT pas être assimilée silencieusement à OFF. Une mise à jour de réglages depuis cet état non fiable DOIT être refusée avant écriture pour conserver le fichier d'origine.
- **FR004** : Avant chaque tour, T3 DOIT actualiser la préférence. Avec ON, les nouvelles consignes DOIVENT privilégier le parcours automatique Bridget pour les nouvelles sous-tâches. Avec OFF, elles DOIVENT retrouver les consignes de délégation T3 habituelles. Le tour déjà lancé n'est pas réécrit.
- **FR005** : Les consignes DOIVENT préserver une demande explicite de nouveau fil T3 et les usages des outils T3 qui ne sont pas une délégation de sous-tâche.
- **FR006** : Les consignes DOIVENT transmettre les contraintes explicites de l'utilisateur sans les assouplir. Les axes non imposés DOIVENT rester automatiques selon les capacités et la politique Bridget existantes.
- **FR007** : Le choix du modèle, du moteur, de la connexion, de la machine et de l'effort, puis l'admission et l'exécution, DOIVENT rester dans Bridget. Le raccord ne DOIT pas créer de second algorithme de flotte.
- **FR008** : Les permissions, identités attestées, révocations, contrôles humains et désactivations existants DOIVENT conserver leur autorité. La préférence ne DOIT créer, étendre ni rétablir aucun droit. Sa simple actualisation ne DOIT pas faire tourner l'identité attestée du fil.
- **FR009** : Avec ON et Bridget indisponible, absent ou refusé, les consignes DOIVENT demander une erreur explicite sans bascule silencieuse vers T3. L'hôte ne DOIT lancer aucun moteur de secours après une erreur de délégation.
- **FR010** : Un même besoin rejoué par le parcours Bridget DOIT conserver les identifiants et garanties de non-duplication existants. Une issue d'admission inconnue ne DOIT pas déclencher une seconde délégation par l'hôte.
- **FR011** : Un changement de préférence DOIT concerner les nouvelles consignes et nouvelles délégations. Il ne DOIT ni migrer, ni relancer, ni retirer le suivi d'une tâche déjà lancée.
- **FR012** : Lineage DOIT conserver sa projection existante des tâches Bridget : lien parent, moteur, machine, statut, journal et résultat disponibles. La consultation DOIT rester sans effet de lancement.
- **FR013** : Une tâche Bridget admise DOIT garder ses capacités de suivi et de résultat hors T3. Le fonctionnement du moteur natif ne DOIT pas dépendre de la préférence du connecteur.
- **FR014** : Les raccords Codex et Claude déjà montés avec Bridget DOIVENT recevoir des consignes équivalentes pour les modes pris en charge. Avec ON, ACP et les autres raccords existants sans montage Bridget, dont OpenCode, Cursor et Pi, DOIVENT recevoir une consigne d'indisponibilité, sans outil inventé ni dispatch silencieux. Aucune couverture ne DOIT être déduite du seul nom du raccord.
- **FR015** : Seule la préférence DOIT être stockée dans le fichier de réglages existant. Les consignes DOIVENT utiliser cette source unique ; leurs textes ne sont pas stockés dans ce fichier. Le lot ne DOIT pas ajouter de second fichier de politique, de copie durable de l'état, de service ni de nouveau protocole Bridget ou public. Le moteur Bridget et ses contrats restent inchangés pour porter cette préférence. La mise à jour privée d'une extension de fournisseur existante reste bornée au mode de consigne.
- **FR016** : La modification T3 DOIT rester limitée au réglage et aux consignes, avec une correction préalable nécessaire à la reprise autorisée du gros fil : pour le seul sujet authentifié Bridget, la réponse complète existante DOIT omettre le contenu redondant de `visibleTurnItems` en rendant ce champ vide, sans toucher `thread`, `runs`, `nodes`, `messages` ou `turnItems`. Les autres sujets DOIVENT garder leur réponse. Aucun intercepteur d'outils natifs, verrou de délégation, droit nouveau, RPC ou contrat public supplémentaire n'est ajouté.
- **FR017** : La livraison DOIT installer puis activer T3 de façon contrôlée, après validation et préparation du retour arrière. Cette activation est explicitement autorisée par l'utilisateur. Elle DOIT préserver les conversations et le suivi des tâches. Les preuves DOIVENT distinguer source validée, paquet installé et code effectivement chargé. Aucune activation persistante ou boucle de relance ne DOIT être créée.

## Entités principales

- **Préférence du connecteur** : nouveau champ ON/OFF dans le fichier de réglages T3 existant ; une absence vaut OFF, une erreur reste une erreur.
- **Consigne de délégation** : texte transmis au nouveau tour pour expliquer la priorité, ses exceptions et les issues de refus.
- **Demande de sous-tâche** : besoin avec ses contraintes explicites, son parent attesté et ses références de suivi.
- **Tâche native Bridget** : admission, choix, exécution, journal et résultat dont Bridget reste l'autorité.
- **Projection Lineage** : présentation existante d'une tâche et de son parent ; elle ne devient pas un second moteur.

## Cas limites

La clé peut être absente, mal typée ou illisible. Le réglage peut changer alors qu'un tour ou une tâche est en cours. Un ancien processus peut garder un code chargé différent du paquet installé. Un raccord peut ne pas exposer Bridget ou ne pas prendre en charge son parcours automatique.

Bridget peut être arrêté, son outil peut être désactivé ou sa preuve de fil peut avoir expiré. Une contrainte peut ne correspondre à aucune cible autorisée. Une réponse peut être perdue après admission. Le même besoin peut être rejoué après une coupure. Lineage peut être indisponible alors que la tâche continue.

Une demande peut viser explicitement un nouveau fil T3. L'utilisation d'un outil T3 sans délégation de sous-tâche ne doit pas être reclassée comme délégation. Un modèle peut ne pas suivre une consigne ; ce lot ne prétend pas empêcher ce comportement par une garde technique nouvelle.

## Critères de succès mesurables

- **SC001** : Pour Codex, Claude, ACP, OpenCode, Cursor et Pi, les trois cas clé absente, OFF explicite et ON explicite produisent les consignes attendues. Avec ON, Codex et Claude désignent Bridget, tandis que les quatre raccords sans montage Bridget indiquent son indisponibilité. Zéro consigne contradictoire active ne subsiste dans cette matrice.
- **SC002** : Chaque cas prévu de configuration invalide ou de lecture en erreur produit un diagnostic explicite. Aucun de ces cas n'est présenté comme un OFF lu avec succès. Une tentative de mise à jour dans cet état conserve le fichier byte-identique et n'effectue aucune écriture de secret.
- **SC003** : Dans une recette du raccord validé avec ON, une sous-tâche conforme passe par le choix automatique Bridget, produit une tâche native et retrouve son résultat corrélé. Les traces distinguent l'appel Bridget, l'outil T3 `delegate_task` et les outils natifs de sous-agent du fournisseur. Pour cette même sous-tâche, les deux derniers comptes observés valent zéro.
- **SC004** : Chaque scénario de contrainte explicite conserve les axes imposés ou obtient un refus nommé. Aucun scénario ne gagne de droit ni de cible par l'activation de la préférence. L'identité attestée est identique avant et après sa simple actualisation.
- **SC005** : Les cas Bridget absent, outil désactivé, preuve invalide et admission refusée produisent une issue explicite. L'hôte effectue zéro lancement natif T3 de secours. Un rejeu admis retrouve au plus une tâche Bridget.
- **SC006** : Après ON vers OFF et OFF vers ON, le tour suivant reçoit le nouveau mode. Les tâches déjà lancées conservent leur moteur initial et leurs références. Les lectures de suivi ajoutent zéro tâche et zéro conversation de premier niveau.
- **SC007** : Les scénarios de nouveau fil T3 explicitement demandé et d'outil T3 hors délégation conservent leur parcours. Les consignes ne prescrivent pas leur remplacement par Bridget.
- **SC008** : Une inspection du périmètre livré constate zéro nouveau réglage visuel, fichier de politique supplémentaire, stockage durable distinct, service ou protocole Bridget/public pour cette préférence.
- **SC009** : Après installation et activation contrôlée, le reçu identifie le code chargé, la préférence ON et les nouveaux contextes. Il prouve la reprise et le suivi des conversations conservées, sans boucle de relance ni activation persistante. Une recette sur une autre instance est identifiée comme telle ; elle ne remplace pas la preuve de l'activation cible.

## Hypothèses et dépendances

L'exploration confirme un service et un fichier de réglages T3 existants, sans champ Bridget à ce jour. Cette session y ajoute une seule préférence pour l'intégration. Le plan précisera son point de lecture. La préférence est actualisée avant chaque tour, sans rotation de l'identité attestée. Elle ne modifie pas les consignes d'un tour déjà lancé.

L'activation souhaitée pour cette livraison est ON explicite. La clé absente reste OFF pour les installations qui n'ont pas choisi cette préférence. Une consigne donne une priorité au modèle ; elle ne constitue pas une preuve d'exécution ni une autorisation.

Les dépendances existantes apportent les capacités à réutiliser :

- **SPEC-149** : sous-agents natifs, droits hérités, non-duplication et projection Lineage.
- **SPEC-163** : choix configurable, contraintes explicites et admission protégée.
- **SPEC-164** : lecture des tâches Bridget distantes et résultats dans les clients T3.
- **SPEC-166** : identité du fil réattestée après reprise du fournisseur.
- **SPEC-168** : choix automatique de la combinaison modèle, moteur, connexion, machine et effort.

Les montages Bridget existants sont présents pour Codex et Claude. ACP, OpenCode, Cursor et Pi ne montent pas Bridget ; ON doit y annoncer cette limite. Une présence dans la configuration ne suffit pas à annoncer une capacité active. Les raccords réellement exercés et leurs limites seront tracés. Les tests de consignes et les recettes avec un fournisseur réel sont des preuves distinctes. Toute recette emploie uniquement les raccords et identités existants, déclarés et vérifiés ; aucun agent ou modèle étranger n'est lancé pour combler une preuve manquante.

L'utilisateur a explicitement autorisé l'activation finale de T3 pendant cette session. Cette instruction remplace la restriction initiale qui prévoyait une installation seule. La procédure doit conserver les conversations et un retour arrière prêt. L'incident de relances répétées de SPEC-166 interdit de réintroduire un mécanisme d'activation persistant.

## Hors périmètre

Nouvelle interface de réglage, second moteur de politiques, blocage technique des outils natifs T3, nouveau sélecteur de modèle ou de machine, nouvel algorithme de flotte, compte ou authentification supplémentaires, gestion de facture, migration des tâches en cours, nouvelle conversation T3 en doublon, refonte de Lineage, activation persistante ou relances en boucle.

## État de préparation

ROOT accepte les corrections T013/T014 après lecture des9 deltas et des preuves R2 :14/14 tâches, statut Implemented pour l'implémentation, pipeline In Progress. Les536 cas distincts affectés R2 sont finalement PASS après reprises ; ils ne sont pas ajoutés au bilan R1 recouvrant. Le passage1 reste historique ; ROOT a ensuite clos le passage2 CONVERGED, sans tâche ajoutée. La contre-revue R2 est APPROVE_WITH_CHANGES ; ROOT ne confirme pas C1 et conserve C2 comme exception d'enveloppe OFF documentée. L'audit source est clos pass, J1 clos, sans code nouveau ni passage3. Les preuves sont décrites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/tests169.md. FR017, SC003 réel, SC009 et les jalonsJ2–J6 restent ouverts. Aucun paquet installé, activation ou recette réelle169 n'est déduit de cette acceptation d'implémentation.

Le template projet et le template global annoncé étant absents lors du cadrage, la rédaction reprend les sections obligatoires du protocole Specify et le format de SPEC-168. Cette dégradation documentaire ne change pas le besoin.

## Précisions après contre-revue R2

« OFF habituel » désigne la politique de délégation. Sans MCP, une enveloppe de reset OFF est ajoutée même au premier tour quand l'état précédent est inconnu. runOrdinal1 n'exclut pas import/reprise d'une conversation native. L'enveloppe sans MCP n'est plus byte-identique ; seuls les6 986 octets de la constante OFF avec MCP le sont. Codex déduplique le contexte identique ; coût restant non mesuré.

C1 n'est pas confirmé dans le SDK installé. Une query Claude blocked peut rester ainsi jusqu'à réouverture ou changement de clé/processus. Absence physique d'un processus figé, reprise Claude existante positive et visibilité UI du message de divergence ne sont pas prouvées. Les qualifications natives couvrent seulement les sorties normales des deux paires installées. Pi réel absent, serveur types non GREEN et gates FR017/SC003 réel/SC009/J2–J6 restent des limites ; P2 reste clos.

## Checkpoint documentaire après clôture J1

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
