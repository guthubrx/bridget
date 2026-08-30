# Spécification - SPEC-074 Bridget Desktop

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 074-bridget-desktop
Titre: Bridget Desktop multi-serveurs via SSH
Statut: Implémentée - validation opérateur restante
Priorité: P1
Résumé: Une application de bureau permet de piloter des relais Bridget atteignables depuis le Mac ou joints via SSH, sans exposer de port sur les serveurs. Les agents continuent de s'exécuter sur leurs serveurs respectifs.
Déclarations de périmètre: Le navigateur distant est une capacité future documentée, pas une fonction de cette SPEC.
<!-- SPEC-FORMALISM:END -->

**Branche**: `session-074-bridget-desktop`
**Créée**: 2026-08-30
**Dépendances**: SPEC-002, SPEC-063, SPEC-064, SPEC-069, SPEC-070, SPEC-071, SPEC-072, SPEC-073

**Preuves d'implémentation**: `evidence/validation.md` et `evidence/macos-package.md`.

## Contexte

Bridget est aujourd'hui contrôlé depuis une interface web locale au serveur. Le relais UI écoute volontairement sur la boucle locale et les agents s'exécutent sur le serveur. L'opérateur veut une application installée sur son Mac, capable de joindre un ou plusieurs endpoints Bridget déjà atteignables depuis le Mac, ou de créer elle-même un tunnel SSH sans rendre leurs relais accessibles depuis Internet. Le chemin réseau déjà créé par l'opérateur ne définit pas une seconde sorte de serveur.

## Objectifs

1. Permettre à l'opérateur d'ajouter et de nommer un serveur Bridget à partir d'un endpoint loopback déjà accessible, ou de son hôte SSH, son port, son compte et une identité SSH déjà disponible sur le Mac.
2. Permettre à l'opérateur de valider l'identité SSH d'un serveur lors de la première connexion et de refuser clairement un changement inattendu d'empreinte.
3. Ouvrir et superviser depuis l'application un canal SSH local quand il est demandé, ou contrôler un endpoint déjà accessible, sans modifier l'écoute locale du serveur.
4. Présenter l'interface Bridget réellement servie par chaque serveur connecté, ses erreurs et son activité sans promettre une remise ou une exécution non prouvée.
5. Permettre la bascule entre plusieurs serveurs et l'affichage volontaire de deux serveurs simultanément, sans mélanger leurs agents, messages ou notifications.
6. Permettre un endpoint loopback déjà accessible depuis le Mac, quel que soit le chemin qui le rend accessible, sans créer de tunnel SSH supplémentaire.
7. Poser une frontière de sécurité qui interdit au contenu venant d'un serveur d'obtenir des privilèges locaux de l'application.
8. Préserver la possibilité future d'afficher et contrôler une session navigateur exécutée sur un serveur, sans démarrer ni diffuser de navigateur dans cette SPEC.

## Hors périmètre

- Lancer, diffuser, observer ou contrôler un navigateur distant.
- Exposer le relais UI, un port VNC, un port de débogage ou un autre port Bridget sur le réseau public.
- Créer un nouveau système d'authentification serveur, un reverse proxy ou un VPN.
- Importer ou recopier une clé privée SSH dans l'application par défaut.
- Prendre en charge l'authentification SSH par mot de passe dans la première version.
- Modifier la sémantique des agents, de Maicie, des exécutions, des interruptions ou des fournisseurs existants.
- Construire une copie indépendante de la messagerie Bridget.

## Récits utilisateur et critères d'acceptation

### US1 - Ajouter un serveur distant

En tant qu'opérateur, je peux créer un profil pour un serveur Bridget et l'ouvrir dans Bridget Desktop sans exposer le relais du serveur sur Internet.

Critères d'acceptation :

- Le formulaire demande uniquement les informations nécessaires : nom lisible, hôte, port, compte SSH et identité SSH existante.
- Le profil n'enregistre ni mot de passe SSH, ni contenu de clé privée, ni jeton UI.
- La première connexion affiche l'empreinte SSH à accepter ou refuser ; une empreinte différente bloque la connexion avec une explication actionnable.
- Après connexion, le serveur continue d'écouter son relais UI uniquement sur sa boucle locale.
- Un échec SSH, d'empreinte, de tunnel ou de relais est distingué et lisible.

### US2 - Voir l'état réel d'une connexion

En tant qu'opérateur, je sais si l'application cherche le serveur, si le tunnel est établi, si le relais répond, si le flux est actif ou si une reconnexion est en cours.

Critères d'acceptation :

- L'application ne présente jamais le tunnel comme une preuve que le relais ou les agents sont disponibles.
- Une coupure est signalée sans masquer le dernier état connu ; une reconnexion est tentée de façon bornée et visible.
- Les actions utilisateur existantes restent attachées à leur résultat réel fourni par Bridget.
- Les diagnostics excluent clés privées, jetons, corps de messages et chemins sensibles.

### US3 - Passer d'un serveur à l'autre ou les comparer

En tant qu'opérateur, je peux basculer entre des profils et ouvrir explicitement deux panneaux, afin de suivre deux serveurs sans ambiguïté.

Critères d'acceptation :

- Chaque panneau affiche le nom du serveur et conserve ses propres agents, messages, flux et indicateurs.
- Une notification, un agent ou un message ne peut pas apparaître sous le mauvais serveur.
- Fermer un panneau ne supprime pas son profil ; fermer l'application ferme proprement ses canaux actifs.
- La navigation clavier permet de choisir un profil, changer de panneau et revenir à la zone de travail sans piège de focus.

### US4 - Employer un endpoint Bridget déjà accessible

En tant qu'opérateur, je peux ajouter un endpoint loopback déjà accessible depuis mon Mac, y compris s'il provient d'un tunnel SSH que j'ai ouvert, sans configurer de second SSH dans l'application.

Critères d'acceptation :

- Un endpoint existant ne lance aucun tunnel et n'emploie aucun identifiant SSH.
- Son indisponibilité est affichée comme un endpoint inaccessible, sans être confondue avec une erreur SSH.
- Le port est librement choisi parmi les ports loopback valides et le relais est vérifié par un contrôle authentifié.

### US5 - Ne pas fermer la porte au navigateur distant futur

En tant qu'opérateur, je pourrai ultérieurement associer une session navigateur serveur à un profil déjà connecté, sans réécrire la gestion des serveurs.

Critères d'acceptation :

- Le modèle de connexion distingue une capacité UI actuelle d'une capacité navigateur future.
- Aucun tunnel générique, navigateur, affichage d'écran ou contrôle humain-agent n'est lancé par cette SPEC.
- La documentation identifie qu'une future session navigateur doit être isolée par exécution, rester sur boucle locale et arbitrer explicitement la reprise de main humaine.

## Exigences fonctionnelles

- FR-001 : Bridget Desktop gère au moins un profil SSH géré et un endpoint loopback existant.
- FR-002 : Un profil distant utilise uniquement un canal SSH initié par le Mac vers le serveur déclaré.
- FR-003 : Le relais distant reste lié à la boucle locale du serveur.
- FR-004 : L'application n'accorde aucun privilège local au contenu rendu depuis un relais connecté.
- FR-005 : Les profils mémorisent leurs métadonnées non secrètes et ne mémorisent pas les jetons de relais ou les clés privées.
- FR-006 : L'application présente le nom et l'état de chaque origine de manière persistante dans chaque panneau.
- FR-007 : Un endpoint déjà accessible n'utilise pas SSH et ne prétend pas savoir quel tunnel, proxy ou autre chemin le rend joignable.
- FR-008 : L'application permet de fermer, reconnecter et retirer un profil avec confirmation pour le retrait persistant.
- FR-009 : Le contrat serveur permettant de découvrir un endpoint UI est minimal, authentifié par SSH et ne journalise pas le jeton retourné.
- FR-010 : Les échecs sont catégorisés en identité SSH, authentification SSH, tunnel, endpoint UI, relais indisponible ou version incompatible.

## Exigences non fonctionnelles

- Sécurité : aucune commande shell libre issue de l'interface, aucune clé ou jeton dans les journaux, les captures de test, Git ou les diagnostics exportés.
- Accessibilité : l'ajout de serveur, les onglets et les panneaux sont entièrement utilisables au clavier avec ordre de focus visible et cohérent.
- Fiabilité : la fermeture d'une fenêtre ne laisse pas de tunnel orphelin ; les pertes réseau n'entraînent ni envoi dupliqué ni état fictif.
- Compatibilité : le client détecte explicitement un relais incompatible et indique la mise à jour requise.
- Performance : deux panneaux actifs restent indépendants et n'empêchent pas l'usage interactif de l'un si l'autre se reconnecte.
- Exploitation : un diagnostic redacted permet de distinguer une panne du Mac, du SSH, du tunnel ou du relais sans révéler de secret.

## Entités clés

- Profil de connexion : configuration non secrète qui décrit une origine nommée et son accès, endpoint déjà accessible ou tunnel SSH géré.
- Identité de serveur : empreinte SSH approuvée pour un profil distant.
- Session de connexion : état temporaire d'un profil ouvert, incluant son canal et son endpoint UI courant.
- Panneau : vue associée à une seule session de connexion et à une seule origine.
- Capacité : service explicitement disponible pour une session, UI maintenant et navigateur dans une SPEC future.

## Hypothèses

- Le Mac dispose du client SSH système et peut utiliser son agent ou une identité déjà autorisée.
- Chaque serveur Bridget conserve le relais UI local et peut répondre à une découverte d'endpoint via une commande restreinte.
- Le premier binaire cible macOS ; sa compilation et sa validation finale nécessitent un environnement macOS, distinct du développement serveur Linux.

## Critères de succès

- Un nouvel opérateur peut ajouter un serveur configuré en moins de trois minutes sans copier de secret dans l'application.
- Dans 100 % des essais de perte de réseau contrôlée, l'application affiche un état de reconnexion ou une erreur explicite plutôt qu'un faux état connecté.
- Deux serveurs distincts peuvent être ouverts simultanément sans qu'un message ou un agent soit présenté sous la mauvaise origine.
- Un endpoint existant indisponible produit une explication distincte d'une erreur SSH.
- Les tests de sécurité ne trouvent ni clé privée ni jeton de relais dans la configuration persistée, les journaux applicatifs ou les diagnostics.

## Checklist de validation de spécification

- [x] Le périmètre utilisateur, les limites et la future capacité navigateur sont explicitement séparés.
- [x] Les critères couvrent les profils distants, le local, la double vue, les erreurs et la sécurité.
- [x] Les exigences sont vérifiables et ne dépendent pas d'une implémentation cachée.
- [x] Aucun besoin de clarification critique ne subsiste : la première version est macOS, à clés SSH existantes et sans mot de passe.
