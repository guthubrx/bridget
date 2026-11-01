# Session 222 — Rôles et spécialistes durables
Statut : In Progress. Branche : session-222-roles-profils. Date : 2026-11-01.
Responsable : ce fil. Le programme 221 reste indépendant et In Progress.

## Demande initiale
« Ajouter des rôles aux agents. Trouver un Mainteneur Bridget sans connaître son nom.
Permettre de se déclarer, puis réutiliser des spécialistes avec leurs compétences
et leurs consignes spécifiques. » L'utilisateur a demandé une contre-revue,
examiné des mécanismes de profils réutilisables, puis validé le cadrage ci-dessous
après quinze décisions d'entretien. Le GO de réalisation a été donné après synthèse.

## Demande reformulée et apport
Rendre un spécialiste trouvable par sa fonction, réutilisable d'une mission à
l'autre et capable de terminer le travail autorisé. Séparer sa définition, son
identité durable, ses sessions et les missions reçues. Réutiliser l'admission,
les conversations, le suivi et les contrôles natifs de Bridget. Fournir un
catalogue visuel et les mêmes opérations depuis une conversation.
L'apport principal est la continuité du responsable et de ses acquis sans
dépendance à un nom de conversation, un moteur ou un processus permanent.

## Décisions utilisateur conservées
1. Un spécialiste durable conserve identité, consignes, mémoire et historique
   entre ses sessions ; une session peut rester disponible sans travail utile.
2. Une demande peut réveiller automatiquement un spécialiste durable activé,
   dans son mandat. Aucun GO intermédiaire pour les étapes déjà autorisées.
3. Toute définition créée reste conservée et consultable. Un agent peut créer
   un spécialiste temporaire pour une mission autorisée. Réactiver un profil
   non durable requiert l'accord humain ; le rendre durable autorise ses
   réactivations ultérieures dans son mandat.
4. Aucun plafond de budget ajouté par Bridget n'est actif par défaut. Un plafond
   est configurable et désactivable. Les limites techniques et les refus réels
   du fournisseur restent des faits distincts, jamais masqués.
5. Périmètre explicite : un projet, plusieurs projets ou fonction transversale.
6. Le spécialiste reste joignable pendant son travail. Les missions indépendantes
   peuvent avancer en parallèle ; les conflits de ressources restent coordonnés.
   Aucun nouveau message n'arrête silencieusement une mission.
7. Mémoire durable consultable et synthèse adaptée à chaque mission. Le projet
   courant donne une priorité aux éléments pertinents ; les acquis d'autres
   projets sont réutilisables dans les accès autorisés. Les projets ne sont pas
   des silos exclusifs de mémoire. Chaque élément garde sa provenance.
8. La mémoire s'enrichit automatiquement à partir de l'activité sourcée.
   Faits, décisions et hypothèses sont distincts. Elle reste consultable,
   corrigeable et supprimable. La synthèse ne remplace pas les sources.
9. Les changements de consignes permanentes sont proposés dans une nouvelle
   version ; l'utilisateur valide leur activation. Les missions engagées
   gardent leur version et leurs choix.
10. Identité indépendante du moteur. Préférences et contraintes explicites.
    Un moteur différent peut être choisi pour une nouvelle mission ; une
    mission engagée ne change pas de moteur pour contourner un refus.
11. Attribution automatique à un responsable adapté, avec explication,
    conservation de cette attribution et absence de diffusion du même travail
    à tous les titulaires.
12. Catalogue visuel et conversation sur les mêmes données et contrôles.
13. Désactiver les démarrages automatiques laisse les missions engagées finir.
    Arrêter une mission est une action distincte. Les demandes bloquées restent
    visibles ; profils, mémoire et historique sont conservés.
14. Les mandats par défaut sont adaptés aux rôles, visibles et modifiables.
    Dans son mandat, un spécialiste va au résultat utilisable, livraison comprise
    si prévue. Il peut préparer un outil manquant si cela est autorisé.
15. Export d'une définition seule par défaut. Mémoire et historique sont séparés.
    Accès, connexions aux comptes et désignations officielles ne se transmettent
    jamais par import. L'origine et la version restent visibles.
16. Conversion possible, à la demande humaine, d'une conversation existante :
    conversation et missions conservées, profil et mémoire sourcée proposés.
17. Les compositions d'équipe sont facultatives. Elles décrivent des livrables
    distincts et réutilisent les missions natives ; elles ne créent pas un second
    orchestrateur ni une hiérarchie imposée.

## Scénarios utilisateur et acceptation

### US1 — Déclarer et trouver un rôle (P1)
Un agent déclare plusieurs spécialités avec un périmètre. Un appelant cherche
la fonction « Mainteneur Bridget » sans connaître le nom du titulaire.
Acceptation : la recherche rend les candidats autorisés et leur présence observée ;
un rôle déclaré ne vaut pas désignation officielle. Un changement de libellé ne
change pas l'identité. Un retrait ou une déconnexion ne produit pas une fausse
disponibilité. L'absence de candidat est expliquée et ne lance rien implicitement.

### US2 — Conserver, réemployer et approuver un profil (P1)
Un profil temporaire utile est conservé, proposé au réemploi puis rendu durable.
Acceptation : création, consultation, proposition et activation sont distinguées ;
un agent ne peut pas activer seul la durabilité ni une nouvelle version permanente.
Un réemploi non durable non autorisé est refusé avec l'action humaine précise.
Les versions historiques sont lisibles et une mission garde son instantané.

### US3 — Démarrer, solliciter et suivre un spécialiste (P1)
Une demande conforme trouve un titulaire, ou réveille un spécialiste durable
activé. Elle reste identifiable après perte de réponse, coupure ou redémarrage.
Acceptation : une clé et une enveloppe produisent au plus une mission ; une
enveloppe différente sous la même clé est refusée. L'attribution, les choix du
moteur et les étapes de remise sont observables. Une demande hors mandat ou
aux prérequis inconnus reste explicitement bloquée sans nouveau compte ni droit.
Un spécialiste occupé reçoit l'information sans annulation de son travail.
Deux demandes indépendantes peuvent avancer ; une issue inconnue n'est pas relancée.

### US4 — Réutiliser une mémoire durable interprojets (P1)
Un spécialiste retrouve des acquis du projet courant et une leçon pertinente
provenant d'un autre projet autorisé.
Acceptation : la provenance, le type de connaissance et la validité sont visibles ;
à pertinence comparable le projet courant est prioritaire. Une information
contradictoire n'écrase pas silencieusement une autre. Une source inaccessible
n'est pas révélée. La correction et l'oubli sont respectés au prochain rappel.
Les résumés bornent le contexte chargé, pas la conservation de l'expérience.

### US5 — Administrer depuis le catalogue et la conversation (P1)
L'utilisateur consulte le spécialiste, ses réglages, ses versions, sa mémoire
et ses missions. Il désactive les futurs réveils sans interrompre une mission.
Acceptation : mêmes états et mêmes effets dans les deux parcours, états vide,
occupé, hors ligne, en attente, refusé et erreur lisibles. Mandat et absence
de plafond budgétaire sont affichés. Une annulation exige une action distincte.

### US6 — Importer et convertir sans perdre le travail (P2)
Une définition est exportée puis importée. Une conversation existante devient
un spécialiste avec les décisions pertinentes de son historique.
Acceptation : import sans activation, sans secret ni autorisation transférée ;
conflit de version expliqué. Aucun démarrage provoqué par l'import. L'ancien
historique et les tâches gardent leurs identités ; les connaissances produites
citent leurs sources et restent des propositions rectifiables.

### US7 — Composer et transmettre un travail utile (P2)
Une composition réutilisable associe plusieurs profils à des livrables distincts.
Acceptation : membres, dépendances et critères visibles ; une attribution
partielle garde les missions admises. La reprise sous la même clé n'en crée pas
de doublon. Un membre sans prérequis ne provoque ni repli silencieux ni arrêt
des membres indépendants. Une seule autorité conserve l'état des missions.

## Exigences fonctionnelles
- FR-001 Déclarer, rechercher et retirer plusieurs rôles sans remplacer l'identité.
- FR-002 Distinguer spécialité déclarée et désignation officielle ; seul l'humain
  autorisé active une responsabilité officielle et peut la retirer.
- FR-003 Conserver les périmètres et recontrôler la présence avant attribution.
- FR-004 Persister profils, versions, provenance et définitions temporaires.
- FR-005 Séparer conservation, autorisation de réemploi, durabilité et activation.
- FR-006 Contrôler les activations humaines et les changements de version.
- FR-007 Réveiller un spécialiste durable activé pour une demande conforme.
- FR-008 Conserver une autorisation de réveil vérifiable et révocable ; aucun
  nom de rôle, prompt, ancien message ou booléen client ne vaut autorisation.
- FR-009 Préserver les contraintes explicites du moteur, modèle et machine ;
  les choix non imposés utilisent la sélection native actuelle.
- FR-010 Vérifier les prérequis réels sur la cible, réutiliser les preuves valides
  et nommer ceux qui sont manquants ou non vérifiables.
- FR-011 Fournir des mandats par défaut lisibles et modifiables. Leur description
  ne crée aucun privilège technique supplémentaire.
- FR-012 Désactiver tout plafond de budget supplémentaire par défaut ; rendre
  visibles ses réglages et les limites distinctes du fournisseur.
- FR-013 Attribuer une demande à un seul responsable et expliquer la sélection.
- FR-014 Préserver clé, enveloppe, attribution et références après coupure ;
  interdire réexécution ou réaffectation silencieuse d'une issue inconnue.
- FR-015 Distinguer conservation, mise en file, remise, prise en charge, résultat
  et validation ; une remise ne prouve pas la réussite du travail.
- FR-016 Garder un spécialiste joignable occupé ; paralléliser les travaux
  indépendants sans interrompre ni masquer les conflits.
- FR-017 Enrichir automatiquement la mémoire avec sources, projet, portée,
  type et date ; conserver les incertitudes sans fausse preuve.
- FR-018 Prioriser les acquis pertinents du projet courant et rappeler les
  autres projets autorisés ; ne pas rendre la mémoire exclusive à un projet.
- FR-019 Produire une synthèse adaptée avec accès aux détails conservés.
- FR-020 Permettre consultation, correction, oubli et historique des changements
  de mémoire ; ne pas confondre oubli et suppression des produits métier.
- FR-021 Garder les consignes actives distinctes des leçons ; ne pas activer une
  évolution de profil par une simple écriture de mémoire.
- FR-022 Rendre toutes les opérations utilisateur nécessaires dans le catalogue
  et par conversation, avec états, progression et diagnostics concordants.
- FR-023 Désactiver les réveils sans annuler les missions ni perdre l'attente.
- FR-024 Arrêter une mission par le parcours natif explicite, avec preuve d'arrêt.
- FR-025 Exporter la définition seule ; séparer explicitement les transferts
  de connaissances et conserver leur provenance et leur périmètre autorisé.
- FR-026 Importer sans activer ni transférer des comptes, accès ou désignations.
- FR-027 Convertir une conversation existante sans réécrire ses missions ;
  extraire des connaissances sourcées et rectifiables depuis son historique.
- FR-028 Réutiliser des compositions d'équipe avec dépendances explicites,
  reçus individuels et reprise idempotente, sans second magasin de missions.
- FR-029 Rendre chaque refus actionnable, limité à sa portée réelle, avec la
  vérification manquante et la décision humaine requise ; autres travaux libres.
- FR-030 Garder les parcours existants compatibles et leurs données intactes ;
  un client ancien reçoit un refus de capacité nommé pour une fonction nouvelle.
- FR-031 Livrer sur les composants concernés, vérifier les images chargées et le
  retour compatible ; préserver les missions et nettoyer seulement le périmètre sûr.

## Entités et responsabilités
Rôle ; profil versionné ; spécialiste durable ; session active ; mandat et
autorisation ; demande attribuée ; mission native ; connaissance sourcée ;
composition d'équipe. Le moteur Bridget porte état et contrôles. Les interfaces
affichent les mêmes faits et demandent les mêmes actions, sans autorité parallèle.

## Hypothèses et limites
Périmètre initial : clients Bridget existants (conversation/MCP, CLI et interface
Android), serveurs Mac/Linux de la fédération déjà autorisée. Les nouveaux clients
du programme 221 ne sont pas reconstruits dans cette session.
Les sources tierces alimentent une étude de mécanismes, pas une dépendance runtime.
Aucune dépense nouvelle ni copie de secret. Aucune garantie d'accès à tous les
modèles, aucune auto-promotion d'expertise et aucune promesse de vérité d'un souvenir
du seul fait qu'une source existe. Les plafonds de sécurité des transports restent
distincts d'un budget de travail configuré. Les installations restent dans le mandat.
Les agents et historiques actuels sont conservés ; aucune conversion massive
d'étiquettes ou d'historiques n'est déclenchée sans sélection humaine.

## Critères de réussite
- SC-001 Retrouver le mainteneur par fonction et lui remettre une demande sourcée,
  sans nom préalable, avec responsable et états de remise vérifiables.
- SC-002 Après arrêt puis reprise, retrouver le même spécialiste et ses acquis ;
  un réveil autorisé démarre une seule mission, une autorisation retirée la refuse.
- SC-003 Démontrer un rappel interprojets pertinent et la préférence du projet
  courant à pertinence comparable, ainsi que la non-divulgation d'une source refusée.
- SC-004 Démontrer les mêmes opérations dans le catalogue et la conversation,
  y compris les états vide, hors ligne, occupé, désactivé et refusé.
- SC-005 Démontrer import, changement de version, oubli et conversion sans perdre
  tâche, source ni produit préexistant ; aucun effet automatique à l'import.
- SC-006 Répéter une demande après perte de réponse et une composition partielle :
  références inchangées, aucun second travail, contraintes respectées.
- SC-007 Toutes les exigences ont un contrôle ciblé ou une preuve de circuit réel ;
  aucun statut de fin globale si une tâche requise reste ouverte.
- SC-008 Sources intégrées et poussées, composants concernés réellement chargés,
  retour compatible vérifié et conservations de nettoyage motivées.

## Dépendances
Profils109/110, communications et suivi natifs, admission168/171, interface177,
file180, catalogue220, capacités et corrections221. La session222 réutilise leurs
contrats au lieu de refaire un annuaire, une admission ou un orchestrateur.
