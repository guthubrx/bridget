# Journal des changements

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)


Chaque version publiée s'ajoute à l'historique du dépôt public, avec son tag `vX.Y.Z`. Les numéros
suivent le versionnage sémantique : le dernier chiffre pour les corrections et la documentation, le
deuxième pour les nouvelles capacités, le premier pour les ruptures de contrat. Les dates sont au
format AAAA-MM-JJ.

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
