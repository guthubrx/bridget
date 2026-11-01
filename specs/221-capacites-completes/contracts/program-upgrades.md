# Mise à jour d’un programme déjà qualifié — correction221

Demande validée : une empreinte différente déclenche un contrôle automatique,
plutôt qu’une obligation de modifier manuellement les profils. Version et SHA
restent des faits de diagnostic. Périmètre réel : profils Codex portant déjà
une observation ; les autres protocoles gardent leur contrôle existant tant
qu’un mécanisme de contrat natif n’est pas qualifié pour eux.

## Réutilisation et décision

- lifecycle::launch_preflight reste sans spawn ni écriture. Une nouvelle image
  est acceptée seulement avec une preuve locale du contrat utilisée par ce code.
- Le collecteur provider_catalogue220, son créneau physique unique, son bootstrap
  ManagedChild et son marqueur probe portent le contrôle. Pas de second service,
  de file de missions, d’autorité d’admission ou de table.
- Le schéma versionné vient du CLI existant app-server generate-json-schema --experimental.
  argv fixe, HOME temporaire privé, environnement minimal sans credentials,
  aucun thread/start, turn/start ou login envoyé. Délai5s après RELEASE,
  ready2s, sortie fichiers limitée1024entrées/32Mio et lecture4Mio par schéma.
- Le SHA est lié au chemin et à l’image, recontrôlé après génération et après
  handshake. Cache de hachage32images invalidé par inode/taille/mtime/ctime ;
  lecture par blocs, aucun chargement du binaire entier à chaque candidat.
- Le contrat de base porte initialize, thread/start et turn/start. Les formes
  interruption/pilotage/reprise/fork sont indépendantes. Une nouvelle contrainte
  de schéma non comprise n’est jamais une preuve positive. Le contrôle structurel
  ne promet pas une équivalence sémantique de toute la version fournisseur.
- Le handshake natif et les contrôles de modèle, droits et compte continuent
  lors du lancement réel. Le seul initialize ne prouve pas toutes les méthodes.
- Les preuves sont des faits privés bornés, un fichier par chemin de programme,
  sous .cache/bridget-program-compat, contrat bridget-codex-program-v1.
  Elles ne remplacent ni l’autorité des profils, ni une authentification, ni
  un accès à un modèle. Aucune réécriture automatique des profils ou tâches.
- Le wrapper intersecte les opérations déclarées avec celles de l’image relue ;
  aucune nouvelle capacité de contrôle n’est accordée. Il ne rapporte pas la
  version historique comme version réelle d’une image différente. Une version
  absente du userAgent reste not_reported.
- Une preuve positive n’est publiée qu’après arrêt confirmé du groupe métadonnée.
  Un arrêt incertain garde le Child et le marqueur du collecteur ; aucune seconde
  sonde n’est lancée et aucune preuve positive n’est fabriquée.

## Refus et compatibilité des pairs

ProgramCompatibilityV1 négocie seulement trois motifs de sonde précis :
program_upgrade_check_pending, program_protocol_incompatible et
program_check_unavailable. Un ancien maître reçoit DefinitionUnavailable,
sans nouveau champ ni nouveau motif inconnu. Une capacité absente ne donne
pas de droit et n’affirme pas que le contrôle est exécuté actuellement.
Le conseil distingue absence de preuve, incompatibilité constatée du contrat
et contrôle local indisponible. Il conserve la clé, la tâche et detail.admission,
la portée de la demande et les autres travaux. Aucun replay automatique.

## Invariants et sortie du lot

Aucun changement de compte, droit, modèle, effort, mapping, reçu admis ni profil.
Aucune installation ni mise à jour du fournisseur par ce correctif.
Une version antérieure de Bridget ignore les preuves ; elle peut refuser une
image fournisseur nouvelle, sans perte d’état. Le retour exécutable est qualifié
contre les programmes effectivement présents avant livraison.

Contrôles utiles : image changée compatible ; changement pendant le contrôle ;
contrat de base incompatible ; option seule absente ; preuve privée/périmée pour
une autre image ; causes non mélangées entre candidats ; ancien pair ; bootstrap
réel et arrêt retenu. Puis assemblages Linux/Mac, revue du delta, activation et
contrôle des sept images chargées. Aucun essai métier ni campagne générale.

## Clarifications de contre-revue et responsabilité

La preuve distingue compatible, incompatible structurellement constaté et non
vérifié. Un vocabulaire de schéma inconnu, y compris sur une opération facultative,
donne non vérifié. Une opération comprise mais refusée reste simplement absente.
Les requêtes témoins partagent leurs constructeurs avec le transport réel ;
ClientRequest et les paramètres de chaque méthode sont contrôlés.
L'identité du vérificateur est le SHA calculé automatiquement à la compilation
sur ses sources, le collecteur et les constructeurs de requêtes, plus la version
du paquet. Changer le vérificateur invalide les preuves de l'ancien code.
La révision de capacité runtime contient cette identité et le SHA de l'image.

Le collecteur vise 120 s après succès, 30 s après échec ; un changement de signal
respecte MIN_REFRESH=5 s. Les connexions restent traitées séquentiellement.
Une preuve incompatible est conservée tant que l'image et le vérificateur ne
changent pas. Le retour vers l'image historique ou une adaptation du pilote
reste une décision de l'utilisateur. Aucun essai automatique de modèle.

Minimalisme : réutilisation du collecteur, du bootstrap et de sha2 déjà présent
dans le workspace ; sha2 est aussi déclaré pour build.rs, sans nouvelle version.
Le module porte des règles de compatibilité, de confidentialité et de bornage ;
aucun second orchestrateur ni magasin de tâches. La grammaire partielle est une
limite explicite et ferme en non vérifié. Aucun code supprimable identifié à
comportement constant dans ce delta. Coût futur : maintenir la liste fermée de
vocabulaire et les constructeurs partagés lors d'une évolution du protocole.
