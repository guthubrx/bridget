# Feature Specification: Routage de la flotte par modèle, machine et connexion

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 168-routage-flotte
Titre: Routage de la flotte
Statut: In Progress
Priorité: P1
Tâches: 45/56 cochées après T036 ; livraison en cours de préparation
Tests: FULL unique et reprises ciblées exécutées ; limites conservées dans validation/tests168.md

Résumé:
- Contexte: la flotte possède plusieurs moteurs, modèles et comptes, mais le choix ne couvre pas encore leurs combinaisons et leur charge réelle.
- Objectif: choisir une combinaison autorisée modèle/machine/connexion/effort, avec contrôle manuel indépendant et décision automatique désactivable.
- Dépendances: SPEC-154, SPEC-157, SPEC-160, SPEC-162, SPEC-163, SPEC-167.

Fichiers:
- spec.md: ✓
- tasks.md: généré ; 56 tâches, cochage réservé à ROOT
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: session-168-routage-flotte
**Created**: 2026-10-15
**Status**: In Progress
**Priority**: P1
**Source initiale**: 1f69e5d8e651dd27907fd244aee6f90b6b734565

## Point de contrôle avant commit A — 15 octobre 2026

État : **In Progress**, 45 tâches cochées sur 56 après T036. Le code est réalisé ; Converge P3 est CONVERGED, la contre-revue T037 et ses corrections sont closes, l’audit T038 et la FULL unique T036 ont leurs reçus. T039 est en publication ; T040 et les étapes de livraison restent à suivre. Ce checkpoint ne coche aucune tâche et ne vaut ni installation ni livraison.

FULL : 2 820 PASS / 32 FAIL / 71 ignorés. Après reprises ciblées, 29 des 32 échecs initiaux passent ; trois annulations 159 restent non validables sur Linux (observateur kqueue Darwin/BSD). La cause du premier échec d’identité T3 reste inconnue malgré son replay PASS. Aucun succès global vert n’est déduit.

Preuves et état de livraison :

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md

## Problème et valeur attendue

L’utilisateur veut confier des sous-tâches à sa flotte depuis Bridget, sans dépendre de T3.
Le choix doit tenir compte des capacités réellement disponibles, des ressources des machines
et des limites attestées des comptes. Le modèle, sa connexion et sa machine sont trois notions distinctes.
Une machine saturée peut perdre la préférence donnée à son dépôt ou à son cache déjà préparé.

Les choix explicites restent prioritaires. L’automatisation aide à choisir ; elle ne crée aucun droit,
n’authentifie aucun compte et ne transfère pas une mission en cours entre machines.

## Histoires utilisateur et tests indépendants

### US1 — Voir les vraies possibilités de la flotte (P1)

Comme utilisateur, je veux connaître les modèles et connexions disponibles sur chaque machine.
Je peux ainsi choisir un modèle par son vrai nom, sans confondre modèle et abonnement.

**Test indépendant** : deux machines et deux connexions Codex déclarées produisent un catalogue
compréhensible de douze combinaisons admissibles au moins, sans troncature à huit.

1. Étant donné plusieurs versions Terra, Luna, Sol, Haiku, Sonnet ou Opus exposées par les moteurs,
   quand je lis le catalogue, alors seuls leurs identifiants réels attestés sont proposés.
2. Étant donné deux comptes du même fournisseur, quand je lis leurs possibilités,
   alors leurs connexions et faits de quota sont distincts, même si le modèle est identique.
3. Étant donné un modèle absent d’un moteur sur Cartae, quand il existe sur le Mac,
   alors sa disponibilité sur le Mac n’est jamais transposée à Cartae.

### US2 — Imposer chaque axe séparément (P1)

Comme utilisateur, je veux fixer seulement le modèle, la machine, le moteur, la connexion
ou l’effort, puis laisser les autres axes automatiques.

**Test indépendant** : imposer le modèle conserve plusieurs machines admissibles ; imposer ensuite
une machine indisponible produit un refus explicite, sans changement silencieux de modèle ou de compte.

1. Toute combinaison de contraintes indépendantes est appliquée avant une décision.
2. Un axe automatique désactivé conserve sa valeur fixe déclarée ; aucune valeur n’est inventée.
3. Une contrainte impossible refuse avant tout lancement. Une connexion alternative ne contourne pas le refus.

### US3 — Distribuer des sous-tâches selon les faits frais (P1)

Comme coordinateur, je veux distribuer plusieurs sous-tâches indépendantes sur plusieurs
machines et espaces de travail, sans demander une confirmation humaine à chaque tâche.

**Test indépendant** : deux sous-tâches indépendantes sont admises sur deux espaces distincts ;
la saturation attestée d’une machine peut déplacer la nouvelle sous-tâche malgré un cache favorable.

1. Les faits frais de processeur, mémoire, charge et capacité de compilation déclarée influencent le classement.
   Le type de travail général ou compilation est déclaré explicitement ; il ne vient pas d’un mot-clé de mission.
2. Une mesure inconnue ou périmée reste inconnue ; elle ne devient ni capacité libre ni saturation prouvée.
3. La présence du dépôt et la continuité du cache donnent un bonus borné, jamais une autorisation
   de démarrer sur une autre base ou de contourner les contraintes de droits et de ressources.
4. Une tâche déjà en cours reste sur sa machine ; une reprise utilise son point de contrôle existant.

### US4 — Choisir avec des décideurs facultatifs et bornés (P1)

Comme utilisateur, je veux un premier classement déterministe, puis un service de décision
disponible, puis un petit modèle de secours, puis le classement initial si les deux sont indisponibles.

**Test indépendant** : sur le même ensemble admissible, couper le service fait appeler uniquement
le petit décideur ; couper les deux conserve une sélection déterministe reproductible. Les efforts compatibles restent des choix du modèle,
sans multiplier les combinaisons par chaque effort.

1. Le classement initial filtre les interdits et prépare un ordre explicable.
2. Avec plusieurs possibilités et une décision autorisée, le service configuré est consulté en priorité.
3. Si ce service est indisponible, désactivé, trop lent ou invalide, le petit décideur autorisé
   peut choisir une seule combinaison parmi celles préparées.
4. Ce petit modèle décide du routage ; il n’exécute pas la tâche et n’a ni outil, ni droit Git,
   ni accès aux secrets, ni possibilité d’ajouter un candidat.
5. Si les deux décideurs ne donnent pas de réponse valide à temps, le classement déterministe décide.
6. Une préférence de type « présentation = Claude » n’est jamais codée comme règle universelle.

### US5 — Déclarer des connexions déjà authentifiées (P1)

Comme utilisateur, je veux déclarer les comptes utilisables sans que Bridget gère mes identifiants,
ma facturation ou des connexions interactives répétées.

**Test indépendant** : deux environnements Codex déjà authentifiés sont utilisables séparément ;
une connexion non déclarée ou indisponible n’est jamais créée automatiquement.

1. La déclaration associe une identité de connexion à un moteur et une machine compatibles.
2. L’authentification initiale se fait hors Bridget, une fois selon la procédure du moteur.
3. Le socle fonctionne pour Codex et représente aussi les connexions Claude sans confusion
   entre le moteur Claude Code et le fournisseur commercial qu’il peut employer.
4. Aucun contenu de fichier d’authentification, jeton ou solde opaque n’apparaît dans le catalogue
   ni dans une requête au décideur.

### US6 — Rejouer, reprendre et expliquer sans nouvelle décision (P1)

Comme coordinateur, je veux retrouver la décision et le résultat exacts après une coupure,
sans nouvelle consultation payante ni lancement concurrent.

**Test indépendant** : deux demandes concurrentes de même clé, puis un redémarrage et un rejeu,
conservent une seule décision durable et au plus une admission métier.

1. Le choix, ses contraintes et la révision de politique sont conservés avant lancement.
2. Rejouer la même enveloppe ne recharge pas une politique nouvelle pour remplacer le choix.
   Le SHA de départ appartient à cette enveloppe : après un nouveau commit source, la CLI produit
   un canon différent et la même clé reçoit envelope_mismatch ; consulter alors bridget task status.
3. Une panne entre niveaux de décision ne déclenche pas une nouvelle chaîne au redémarrage ;
   la reprise emploie les faits figés et son repli déjà défini.
4. Les raisons exposent le mode, la fraîcheur et les inconnues utiles, sans publier les secrets.
5. Une erreur technique ou de stockage n’est pas présentée comme un refus métier définitif.
6. Le choix durable reste identique. La revalidation des droits, capacités et de la base peut
   refuser son lancement, notamment si sa machine est devenue saturée ; aucune autre cible
   n’est choisie et aucun décideur n’est rappelé pour cette clé.

### US7 — Partir du bon code et livrer sans synchronisation cachée (P1)

Comme coordinateur, je veux préparer le commit exact sur la machine choisie, puis récupérer
et intégrer le travail distant avant de nettoyer son espace.

**Test indépendant** : une base absente interdit le lancement ; après préparation autorisée du commit,
le travail distant rapporte ses commits et peut être intégré sur une autre machine.

1. Le commit de départ exact doit être disponible sur la cible avant tout lancement de code.
2. La continuité d’un dépôt ne force pas la machine ; le coordinateur peut préparer la même base ailleurs.
3. Le résultat indique la base, la branche, la tête et les modifications non enregistrées.
4. Un résultat disponible ne signifie pas que le travail est intégré.
5. Un intégrateur autorisé récupère et vérifie le résultat avant nettoyage ; le daemon ne fait
   aucun fetch, pull, push ou merge de livraison.
6. T3 peut rester fermé pendant tout le parcours.

## Exigences fonctionnelles

- **FR-001** : Le parcours de création, choix, suivi, reprise et arrêt DOIT fonctionner hors T3.
- **FR-002** : Le catalogue DOIT séparer modèle réel, moteur, machine, connexion et effort.
- **FR-003** : Les versions proposées DOIVENT venir des capacités et déclarations attestées de chaque machine ;
  aucun alias de famille ne DOIT être transformé en version imaginée.
- **FR-004** : Le catalogue DOIT accepter au moins douze combinaisons et ne DOIT pas tronquer une liste à huit.
  La borne globale est de128options déclarées, avant les filtres de demande ; un dépassement du registre
  DOIT être refusé sans catalogue partiel ni perte silencieuse.
- **FR-005** : L’utilisateur DOIT pouvoir fixer chacun des cinq axes indépendamment.
- **FR-006** : Une contrainte explicite et les droits hérités DOIVENT primer sur tout score ou décideur.
- **FR-007** : La déclaration d’une connexion DOIT désigner un environnement déjà authentifié ;
  Bridget ne DOIT ni lancer un login interactif ni recopier les secrets.
- **FR-008** : Deux connexions du même fournisseur DOIVENT rester isolées pour les modèles, quotas et crédits observés.
- **FR-009** : Les crédits attestés utilisables DOIVENT rester pris en compte quand le quota inclus est épuisé ;
  aucun montant ne DOIT être interprété comme une autorisation. À pertinence et charge proches,
  le quota restant et une remise à zéro proche attestés DOIVENT pouvoir donner une préférence bornée,
  sans sommer les comptes ou estimer une dépense ; l’inconnu reste neutre.
- **FR-010** : Les capacités de connexion Claude DOIVENT être représentables par le même contrat, sans gestion de facturation.
- **FR-011** : Chaque mesure de machine DOIT porter sa provenance et son âge ; une mesure absente ou périmée DOIT rester inconnue.
- **FR-012** : Le classement DOIT considérer processeur, mémoire, charge et capacité de compilation déclarée,
  à partir d’un type de travail général ou compilation explicitement fourni, général par défaut.
  Ce signal reste séparé du brief hérité ; aucun mot-clé, framework de benchmark ou commande de mission.
- **FR-013** : La continuité du dépôt et du cache DOIT rester un bonus borné ; une saturation fraîche franche
  DOIT exclure une machine lorsque son alternative admissible est prouvée disponible, avant tout décideur.
  Si la machine est imposée, refuser sans bascule plutôt que masquer sa saturation.
- **FR-014** : Les sous-tâches indépendantes DOIVENT pouvoir partir dans plusieurs espaces isolés et machines.
- **FR-015** : Les missions en cours ne DOIVENT pas être migrées à chaud ; la reprise DOIT conserver les invariants existants.
- **FR-016** : La chaîne de choix DOIT appliquer dans cet ordre : admissibilité et classement déterministes,
  service de décision, petit décideur de secours, ordre déterministe final.
- **FR-017** : Un candidat unique ou des axes tous fixés DOIVENT permettre un choix sans consultation externe inutile.
- **FR-018** : Le routage automatique et chaque niveau de décision DOIVENT être configurables et désactivables par machine.
- **FR-019** : Un niveau désactivé ou déclaré inaccessible depuis la machine de décision ne DOIT émettre aucune tentative réseau.
- **FR-020** : Le service de décision DOIT être défini par un contrat indépendant du nom Laya, avec adresse et modèle configurés.
- **FR-021** : Le petit décideur DOIT choisir une combinaison complète parmi les candidats admissibles,
  sans exécuter la mission ni déléguer une autre tâche. Chaque décideur DOIT recevoir les vrais noms
  de modèles, leurs efforts compatibles et les faits non secrets utiles : ressources, fraîcheur,
  continuité, quotas et crédits attestés ; un ID opaque reste la seule référence d’autorité.
- **FR-022** : Sa sortie DOIT être fermée et utiliser un identifiant opaque ; tout ajout, cible inconnue ou réponse tardive DOIT être rejeté.
- **FR-023** : Le décideur ne DOIT recevoir aucun outil métier, système, réseau ou MCP, ni droit
  d’exécution. Seul le formateur natif StructuredOutput imposé par json-schema est autorisé, sans
  droit d’exécution ; toute autre capacité ou tool_use est rejeté. La sortie est validée localement
  parmi les ID et efforts fermés. Bridget ne DOIT injecter
  automatiquement ni fichier privé, ni secret de configuration ou d’authentification, ni commande à exécuter.
  Le brief fourni par l’utilisateur reste une donnée inerte bornée ; aucun détecteur parfait de secrets n’est promis.
- **FR-024** : Chaque consultation et la chaîne entière DOIVENT avoir un délai maximal configuré et borné ;
  une indisponibilité connue DOIT déclencher le niveau suivant sans attente inutile.
- **FR-025** : Le système DOIT réserver durablement la clé avant consultation et conserver le choix avant admission.
- **FR-026** : Concurrence, coupure et rejeu ne DOIVENT produire ni redécision d’une demande déjà choisie ni double admission.
- **FR-027** : Une reprise après panne ne DOIT appeler à nouveau ni le service ni le petit décideur pour la même demande.
- **FR-028** : Les limites de tâches, contrôles humains, permissions, rétention et annulation existants DOIVENT rester applicables.
- **FR-029** : Tout espace Git168, partagé ou worktree, DOIT recevoir le SHA exact de départ fourni
  par le coordinateur, préparé et vérifié sur la cible avant lancement. Sans SHA, refuser avant fournisseur.
  Hors dépôt Git, un espace partagé pour travail général peut omettre cette base.
- **FR-030** : La base, la branche, la tête et les commits du résultat DOIVENT rester contrôlables par le coordinateur.
- **FR-031** : Le daemon ne DOIT pas synchroniser Git ; la récupération et l’intégration relèvent du coordinateur autorisé.
- **FR-032** : Le nettoyage ne DOIT retirer aucun travail non récupéré, vivant, modifié ou étranger.
- **FR-033** : La décision DOIT exposer son mode, la connexion choisie, les faits utiles et leurs inconnues, sans secret.
- **FR-034** : Le comportement explicite antérieur DOIT rester compatible quand les nouvelles options sont absentes.
- **FR-035** : L’activation réelle DOIT vérifier les modèles, connexions, machines et replis déclarés ;
  une entrée non provisionnée ne DOIT pas être annoncée comme opérationnelle.

## Entités principales

- **Connexion** : environnement authentifié déclaré ; moteur et machine autorisés, identité opaque et état observé.
- **Option de routage** : combinaison complète modèle/moteur/machine/connexion/effort.
- **État machine** : mesures fraîches, capacités déclarées, présence et continuité utile.
- **Politique** : axes automatiques, contraintes, critères, services et niveaux autorisés par machine.
- **Décision durable** : demande figée, candidats, niveau employé, choix, repli et preuve d’admission.
- **Résultat de travail** : base exacte, branche, tête, commits, état du dossier et résultat métier.

## Cas limites

Un compte peut exposer un modèle sur une machine et pas sur l’autre.
Un quota absent n’est pas un quota libre ; un crédit d’un autre compte n’est jamais transféré.
Une mesure peut expirer pendant la décision. Une machine peut se saturer après son classement.
Un dépôt ou son commit peut manquer sur la machine préférée. Un droit peut être révoqué avant préparation.
Un décideur peut répondre après son délai, choisir une cible exclue, ou demander une commande.
Un processus peut mourir après la réponse du décideur et avant l’écriture du choix.
Le même request_id peut arriver deux fois, ou avec une enveloppe différente.
Un ancien worker peut ne pas annoncer les nouvelles capacités.
Une connexion peut être désactivée après classement ; l’admission finale recontrôle sans relancer la chaîne.
Une erreur technique ne doit pas libérer à tort un choix encore récupérable.
Deux tâches qui écrivent les mêmes fichiers ne sont pas des sous-tâches indépendantes par simple déclaration.

## Critères de succès mesurables

- **SC-001** : Dans un namespace isolé sans serveur, pont ni configuration T3, une mission automatique locale
  et une mission distante terminent avec résultat corrélé, sans arrêter le T3 de l’utilisateur.
- **SC-002** : Un catalogue de douze combinaisons au moins reste complet et sélectionnable sans troncature à huit.
- **SC-003** : Dans chaque scénario de contrainte explicite testé, aucune sélection ne viole un axe imposé.
- **SC-004** : Deux comptes distincts du même fournisseur conservent des faits de quotas séparés dans tous les scénarios prévus.
- **SC-005** : Dans la chaîne réelle, une machine franchement saturée ne peut pas être choisie par le service
  ou le petit décideur lorsqu’une autre machine admissible est prouvée disponible, malgré le bonus de continuité.
  Une machine imposée saturée refuse sans bascule ; le rejeu conserve le choix déjà durable.
- **SC-006** : Les trois issues service valide, petit décideur valide et dernier repli sont chacune prouvées.
  Le service choisit sur un ensemble entier qui tient dans ses bornes attestées, huit options au plus par
  prudence initiale ; la matrice12+est prouvée par le petit décideur réel et le score, avec indisponibilité
  de capacité explicite si le service ne peut tout représenter. Les niveaux désactivés émettent zéro consultation.
- **SC-007** : Toute chaîne rend un choix ou une issue explicite avant le budget configuré,
  hors préparation Git dont le délai reste séparé et borné.
- **SC-008** : Un rejeu et deux demandes concurrentes donnent au plus une admission ;
  après redémarrage, zéro nouvelle consultation de décision pour cette clé.
- **SC-009** : Aucun secret de configuration ou d’authentification, chemin d’authentification ou outil
  métier/système/réseau/MCP n’est automatiquement injecté. Seul StructuredOutput formate le JSON
  sans droit d’exécution ; les autres outils/tool_use sont rejetés et la sortie est validée localement. Le brief utilisateur est borné et inerte,
  sans garantie de détection universelle de secrets ; une réponse invalide ne déclenche aucun effet.
- **SC-010** : Chaque lancement de code prouvé part du commit demandé ; chaque nettoyage final
  dispose d’une preuve de récupération et d’absence de travail vivant ou étranger.
- **SC-011** : Les noms réels et capacités de chaque modèle/connexion activés sont vérifiés sur leur machine ;
  la documentation distingue clairement les possibilités déclarées et l’activation observée.

## Hypothèses et limites explicites

Les connexions sont déjà authentifiées par l’utilisateur ou l’opérateur hors Bridget.
Le chemin d’authentification reste privé à la machine ; aucune connexion ne promet un crédit disponible sans preuve fraîche.
La configuration initiale doit refléter les capacités réellement exposées, sans promettre une version future de modèle.
La continuité Git améliore le choix mais ne remplace pas la préparation du commit exact.
Le petit décideur est un rôle séparé de l’exécutant, même si son modèle appartient à la même famille.
Le détail des bornes, de la négociation et des commandes d’activation relève du plan.
Aucune gestion de facturation, aucun login automatique, aucune migration à chaud, aucun produit cartésien illimité,
aucun framework de benchmark ni synchronisation Git périodique n’entre dans ce périmètre.
