# Journal des changements

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)


Chaque version publiée s'ajoute à l'historique du dépôt public, avec son tag `vX.Y.Z`. Les numéros
suivent le versionnage sémantique : le dernier chiffre pour les corrections et la documentation, le
deuxième pour les nouvelles capacités, le premier pour les ruptures de contrat. Les dates sont au
format AAAA-MM-JJ.

## 0.8.7 — 2026-11-07

- Préserver le canal distant lorsqu'un refus de préparation initiale connue arrive après l'expiration de son attente. Les tuples inconnus et les époques périmées restent refusés ; aucune tâche n'est créée par ce refus tardif.
- Conserver les causes de capacité ou de liaison pendant la réattestation avant remise, et expliquer un délai de préparation sans annoncer à tort un serveur arrêté ou un changement de droits.

## 0.8.6 — 2026-11-07

- Conserver l'administration des spécialistes pour les appareils associés avec l'ancienne portée administrative globale complète. Les appareils partiels et les autorisations expirées ou révoquées restent refusés.
- Appliquer une décision humaine déjà enregistrée avec sa clé et son opération exactes, sans refaire répondre l'utilisateur. Préciser la résolution d'un refus de portée administrative.

## 0.8.5 — 2026-11-06

- Réutiliser les racines locales d’un placement existant pendant la préparation sur sa machine cible, sans réécrire ce placement ni changer les droits.
- Accepter les observations d’une commande gérée connue sans exiger une fiche de délégation. Garder le contrôle exact de la machine, de l’instance, de la génération et des réponses corrélées.

## 0.8.4 — 2026-11-06

- Autoriser la préparation locale d’une copie du commit lorsque source et cible désignent la même machine provisionnée. Conserver les autres stratégies et les contrôles d’autorité.
- Donner le champ et la cause fermés des refus de validation du relais, avec la limite de taille connue. Un refus générique ne suggère plus une ancienne version du client.

## 0.8.3 — 2026-11-05

- Lire les seules définitions concernées par un état worker, au lieu de décoder tout l’historique conservé pour chaque commande.
- Préserver la connexion après un refus tardif lié à la vue exacte d’une tâche close. Conserver les contrôles d’identité, d’époque et des attentes encore présentes.
- Donner un diagnostic fermé des refus worker, sans corps de commande ni credential.

## 0.8.2 — 2026-11-05

- Budget60secondes pour les inventaires Git explicitement choisis et retrait de la pause entre lectures utiles. Une expiration conserve une commande fermée, durées, volumes et état d’arrêt.
- Correction des lecteurs de réservations de vues64Mio et récupération froide ciblée de cette quarantaine, sans réécriture de reçus ni lancement de processus.
- Cartes publiques de vues256Mio, relecture en flux du seul manifeste des références closes et prise en compte de la carte dans l’espace requis. Les anciens lecteurs sont refusés comme retour.
- Suivi des préparations de vues annulées non publiées jusqu’à confirmation native de leur conservation.

## 0.8.1 — 2026-11-04

- Copies sélectionnées32Gio, inventaires et manifestes64Mio, opérations d’export/audit30minutes. Contrôles du disque, des objets et des échéances métier conservés.
- Conservation d’une admission pendant une interruption de preuve d’origine, avec la même clé et une issue inconnue explicite.

## 0.8.0 — 2026-11-03

- Sommeil natif des agents : historique, réveil explicite et refus des nouvelles sollicitations directes, collectives ou programmées. Les travaux déjà admis et leurs résultats sont conservés.
- Synchronisation des gestes de classement dans les deux sens par défaut pour les clients liés, avec mode propre à Bridget, reçus persistants et reprise sans nouvelle clé tant que l'issue reste inconnue.
- Vues Actifs/Dormants dans l'application, commandes `agents sleep|wake`, filtres CLI et caches respectant le classement serveur. Le format conservé exige un lecteur comprenant le sommeil.

## 0.7.8 — 2026-11-03

- Publication atomique du contexte attesté des projets déjà préparés, y compris les dossiers liés et les racines additionnelles déjà autorisées.

## 0.7.7 — 2026-11-03

- Annoncer le type objet des spécialistes et présenter les schémas composés par des références locales complètes. Un client MCP strict peut ainsi découvrir les37outils au démarrage, sans modifier les actions admises, les permissions ou les gardes.

## 0.7.6 — 2026-11-02

- Expliquer les démarrages distants non confirmés avec les contrôles ciblés et les références de la demande conservées. Une erreur initiale ne masque plus une remise ou un résultat ultérieur. Aucun changement des reprises, admissions ou droits.

## 0.7.5 — 2026-11-02

- Conserver les connexions vivantes du pont pendant les délais de lecture bornés. Suspendre les observations jusqu’au contrôle frais. Nommer les erreurs RPC sans rejouer les remises incertaines.

## 0.7.4 — 2026-11-01

- Ajout de --table avant ou après la commande pour les inventaires, fiches et reçus finis de la CLI. Colonnes par famille, noms d'hôtes, repli Unicode et pipes sans couleur partagent un rendu natif. JSON, portées, confirmations et codes de retour conservés.

## 0.7.3 — 2026-11-01

- Les mises à jour affichent toutes les machines de la fédération, même sans rapport ou avec un état périmé.
- Raccord local du service de mise à jour avec setup ; unités propres à SYS, KS et au worker Mac.
- Un service arrêté reste arrêté. Les dates non observées restent inconnues.
- Paquet Linux construit sur Debian12/glibc2.36 ; APK7003 signé avec le certificat existant.

## 0.7.2 — 2026-11-01

- Le catalogue et les fiches des spécialistes retournent aussi le nom humain courant du profil, distinct du rôle et de l'identifiant. Un renommage est visible à la prochaine lecture, même hors ligne.

## 0.7.1 — 2026-11-01

- Une admission en reprise attend la republication et la preuve du parent, sans inventer un changement de droits. Les révocations restent refusées.
- Le client relit le même reçu pendant une coupure bornée et rend le refus final reçu. Une issue illisible reste explicitement inconnue. Aucun rejeu des demandes historiques17/18.
- L'application Android0.7.0 reste inchangée.

## 0.7.0 — 2026-11-01

Ajoute les rôles déclarés et désignés, profils versionnés, mandats durables, sessions à la demande, mémoire sourcée interprojets, import/export des définitions et compositions facultatives. MCP, CLI et catalogue Android utilisent les mêmes contrôles et reçus natifs. Aucun plafond de travail supplémentaire n'est actif par défaut. Un lecteur compatible des demandes de spécialiste est exigé pour le retour.

## 0.6.5 — 30 octobre 2026

- Le statut explique directement un refus fournisseur de modèle et indique la décision à demander à l’utilisateur.
- Le catalogue relit la configuration actuelle pour ne plus afficher les connexions désactivées.
- Aucune substitution de modèle, modification de compte ou reprise automatique de tâche.

## [0.6.4] - 2026-10-29

- Corriger la transmission du plafond natif attesté entre moteurs, sans exporter les droits de l’application d’origine.
- Proposer les programmes fournisseurs absents à l’installation, avec sélection explicite et comptes existants conservés.
- Distinguer programmes installés, compatibilité, connexion personnelle et accès réel aux modèles.
- Publier les corrections natives livrées depuis0.6.3 : admissions rejouables, diagnostics bornés, préparation économe et catalogue de flotte explicite.
- Reconstruire le paquet Android en0.6.4 pour aligner la version de publication, avec la même interface, le même certificat et la même lignée de signature.

## [0.6.3] - 2026-10-25

- Volet gauche des conversations : favoris, recherche et états d’attention explicites.
- Brouillons et positions propres à chaque agent ; recherche dans l’historique chargé et dictée système éditable.
- Fichiers et photos préparés localement avant envoi explicite ; langage et copie exacte du code.
- Aperçus HTML interactifs isolés, modes statique et code, sans pont privilégié ni accès réseau.
- Compactage du contexte attesté par des événements natifs corrélés, sans progression inventée.

## Non publié

### File de messages Android

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-1).

### Android et accès permanent à la fédération

- Application Android native : association QR, annuaire filtré, messages, fichiers,
  actions de tâche, questions, rejeux durables et stockage local protégé.
- Accès HTTPS avec autorisations d’appareil révocables et travaux bornés.
- Notifications minimales facultatives, période silencieuse et mise à jour signée.
- Attestations et préparation sur la machine d’origine par le worker authentifié ;
  raccordement privé du poste à un maître serveur permanent.

## [0.4.1] - 2026-10-16

### Corrigé

- Une réponse distante acceptée ferme le suivi de sa demande, même sans accusé de transport du parent. Elle ne déclenche plus une fausse alerte de non-réponse.
- Après une panne du stockage, l’entretien reprend cette clôture, même si la tâche est déjà nettoyée. Le résultat reste conservé et le travail ne redémarre pas.
- La corrélation vérifie la mission, la ronde, l’enfant et le parent. Une vraie absence de réponse reste soumise au délai configuré.

### Compatibilité et validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-2).

## [0.4.0] - 2026-10-15

### Ajouté

- Le catalogue de routage présente les modèles déclarés sur chaque machine avec leur moteur, leur connexion et leurs efforts pris en charge. `bridget delegate --catalogue --json` permet de le consulter par le parcours natif Bridget.
- Une tâche peut imposer séparément le modèle, le moteur, la machine, la connexion ou l’effort. Les axes non imposés restent libres dans les limites du registre. Un effort absent conserve le comportement natif du fournisseur.
- Le choix tient compte des faits de charge frais, de la continuité du travail Git, des capacités de compilation déclarées et de l’état attesté des quotas ou crédits de la connexion cible. La continuité reste bornée : elle ne bloque pas le déplacement vers une autre machine disponible.
- La chaîne de décision est configurable et désactivable par fichier de configuration. Elle applique les filtres déterministes, puis peut consulter un serveur de décision compatible, tel que Laya ou Jev, puis un modèle léger de secours. Le classement déterministe reste le dernier recours. Le modèle de secours choisit la combinaison d’exécution ; il n’effectue pas la tâche à sa place.
- L’accès au service de décision est déclaré par machine. Un service désactivé ou indisponible n’empêche pas les niveaux de secours autorisés. Les décideurs ne peuvent choisir que parmi les combinaisons déjà admissibles.
- L’état durable et la projection Lineage v4 conservent le choix, sa provenance et les faits bornés utilisés. Un rejeu exact retrouve le reçu enregistré sans nouvelle décision.

### Git et exécution distante

- Le démarrage distant vérifie le commit de départ attendu avant la préparation du travail. Une divergence Git ne devient pas une tâche exécutée sur une autre révision.
- Des travaux distincts peuvent utiliser leurs propres worktrees. Le moteur de routage ne lance aucun `git fetch`, `git push` ou merge : la synchronisation et la livraison Git restent des actions explicites de coordination.
- Le catalogue reste distinct des droits réels. Un modèle déclaré ne prouve ni l’accès du compte, ni un quota ou des crédits inconnus. Les connexions restent celles autorisées et configurées sur chaque machine.

### Compatibilité et exploitation

- Les contrats existants, dont les projections Lineage v1 à v3, restent conservés. Lineage v4 ajoute les métadonnées de décision bornées ; les lectures anciennes gardent leur forme et n’obtiennent pas ces nouveaux détails.
- L’admission v2 et les nouvelles capacités de probe sont négociées explicitement. Une capacité absente ne déclenche pas de conversion silencieuse vers une ancienne admission.
- La migration du stockage conserve les reçus antérieurs. Après l’écriture d’un reçu d’admission v2, ne pas revenir à un binaire 0.3.0, même si le routage est ensuite désactivé. Utiliser un binaire compatible avec cette migration, ou une restauration cohérente validée qui ne perd aucun travail.
- Bridget est utilisable depuis le terminal avec son moteur natif.

### Validation et limites connues

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-3).

## [0.3.0] - 2026-10-15

### Ajouté

- `bridget delegate <MISSION>` donne à un agent coordinateur un parcours court pour choisir un profil, créer un enfant et lui confier une mission. Les options avancées restent facultatives.
- `bridget task` expose la liste, l’état, l’attente, la continuation, les réponses, l’annulation et l’accusé de lecture. Ces commandes réutilisent le moteur natif et fonctionnent par le parcours natif Bridget.
- La sortie `--json` reste exploitable par script. Un terminal sans identité d’agent reçoit un refus avant tout effet et une instruction de rattachement.

### Limites de validation

- La coordination directe exige une identité d’agent Bridget vivante. Elle ne transforme pas un terminal humain en agent.
- `task list` est une projection humaine globale. Les autres opérations restent limitées au coordinateur propriétaire.

## [0.2.1] - 2026-10-15

### Corrigé

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-4).

### Exploitation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-5).

### Limites de validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-6).

## [0.2.0] - 2026-10-14

### Ajouté

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-7).

### Corrigé

- Le calcul de quota intègre les crédits Codex du compte cible (5872f48e).

### Limites de validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-8).

Voir la référence de communication et le guide d'installation. Toutes les entrées publiées restent
conservées.

## [0.1.3] - 2026-10-02

### Corrigé

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-9).

### Documentation

- Guide d'installation : déclarer un fournisseur compatible Claude (exemple GLM) ou un second
  compte Codex dans le registre, avec sa variante lecture seule automatique.

### Modifié

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-10).

## [0.1.2] - 2026-10-02

### Documentation

- Skill : lancer des équipiers en lecture seule est autorisé d'office pour une tâche de lecture,
  sans terminal ; seul le droit d'écrire (`--posture development`, Codex) exige un terminal
  humain ; un refus « posture_decouverte » signale un type non déclaré dans le registre.

### Corrigé

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-11).

## [0.1.1] - 2026-09-25

### Modifié

- `bridget --help` affiche la version réelle du paquet ; elle était figée à 0.1.0 dans le texte.

### Documentation

- README à jour des capacités récentes, et ce journal des changements.
- La publication est désormais versionnée : une nouvelle version s'ajoute à l'historique au lieu de
  le remplacer, ce qui permet de comparer deux versions.

## [0.1.0] - 2026-09-25

Première version publique versionnée. Les publications précédentes remplaçaient l'historique ;
celle-ci en est le point de départ. Elle contient notamment les changements suivants.

### Ajouté

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-12).

### Corrigé

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.fr.md#extrait-13).
