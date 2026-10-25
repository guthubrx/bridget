# Bridget

<p align="center"><img src="assets/branding/bridget-logo.png" alt="Bridget" width="180"></p>

**La communication et la coordination de vos agents, sur ordinateur et Android.**

[English](README.md) · [Français](README.fr.md) · [Documentation](docs/README.fr.md) · [Versions](https://github.com/guthubrx/bridget/releases)

Bridget relie les sessions d'agents entre fournisseurs et entre machines.
Les agents se trouvent, échangent des messages, partagent un fil et délèguent
une mission. Le daemon Rust possède les identités, les reçus et l'état des tâches.
Les clients terminal, MCP et Android utilisent les contrats natifs Bridget.

## Commencer

Installer la dernière version publiée sur macOS ou Linux :

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh
```

La même commande met à jour les fichiers et conserve les comptes et les données.
Sur une installation neuve, elle démarre Bridget, active son démarrage automatique
et vérifie sa réponse. Elle conserve les services et fédérations déjà actifs.
Voir le [guide d'installation](docs/communication-installation.fr.md).

Depuis le dossier de votre projet, lancer un agent :

```sh
bridget codex
# Ou :
bridget claude
```

Lancer directement `codex` ou `claude` ne crée pas à lui seul une identité Bridget.
Un agent doit posséder une identité attestée avant d'utiliser les commandes réservées
aux agents. Un raccordement compatible peut aussi fournir cette identité.

## Communiquer

Dans une session d'agent enregistrée :

```sh
bridget who
bridget agents --json
bridget send --to '<agent_uuid>' --reply --timeout 120 -- 'Relis le contrat et signale les défauts concrets.'
bridget ledger --limit 20
```

Utiliser l'identifiant de l'annuaire. Un nom d'affichage ne donne aucun droit.
La recherche vise le même projet par défaut. Un échange volontaire entre projets
exige `--cross-project-reason`.

Les fils partagés gardent le contexte commun. Une publication silencieuse ne réveille
personne. Une demande cible ses destinataires. Tous les membres autorisés peuvent
lire le fil ; supprimer une notification ne rend pas son contenu privé.

## Déléguer et suivre

```sh
bridget delegate --catalogue --json
bridget delegate 'Analyse les tests et rends les risques principaux.'
bridget task wait '<task_id>'
bridget task status '<task_id>'
bridget task ack '<task_id>'
```

`continue`, `respond` et `cancel` couvrent la suite du cycle. Ajouter `--json`
pour un script. Le catalogue décrit les capacités déclarées ; il ne teste pas
l'authentification. Conserver l'identifiant, la clé de demande et les arguments exacts.
Si la réponse d'admission est perdue, inspecter ou rejouer cette même demande.
Une nouvelle clé peut créer une seconde mission.

La [délégation native](docs/delegation-native.fr.md) couvre les permissions,
les machines distantes, les espaces Git isolés, la continuation et l'annulation.

## Commander depuis Android

Associer l'application à Bridget pour lire les conversations, envoyer un message,
consulter la file et traiter les demandes humaines. Les questions gardent les textes,
choix et descriptions du modèle. Une réponse peut être choisie, écrite ou dictée.
Le texte dicté reste éditable avant validation. Les brouillons restent sur leur
appareil ; la résolution confirmée de la question est partagée.

Le [guide Android](docs/mobile.fr.md) détaille les droits d'association,
les notifications, les aperçus et les mises à jour.

## Comprendre les reçus

| Fait | Ce qu'il prouve |
| --- | --- |
| Demande acceptée | Bridget a enregistré la demande. |
| Remise confirmée | Le transport a confirmé la remise. |
| Lecture déclarée | Le parent a explicitement déclaré avoir lu le résultat. |
| Résultat disponible | Un résultat est enregistré ; son contenu et ses preuves restent à examiner. |
| Issue inconnue | La remise ou l'exécution reste incertaine. Conserver la même clé de demande. |

Une fin de tour ne prouve pas la réussite d'une mission. Une compilation réussie
ne prouve pas que le service actif a chargé le binaire. Un résultat ne prouve pas
que les changements ont été intégrés ou déployés.

## Architecture et confidentialité

Trois crates portent le noyau : `bridget-core` pour les types et gardes communes,
`bridget-transport` pour les protocoles et pilotes, `bridget-daemon` pour le daemon,
la CLI, MCP, le stockage et le cycle des tâches.

L'état est local. Les clients locaux utilisent des sockets Unix ; la fédération
passe par votre configuration SSH. L'accès Android utilise le canal authentifié
configuré lors de l'association. Le trafic fournisseur suit le compte choisi.

Les tailles de trame, pages, files et budgets d'opération sont bornés.
Les refus distinguent un prérequis manquant d'une admission réussie.
Les secrets, bases actives et journaux bruts restent hors Git.

## Documentation

- [Installation et identité](docs/communication-installation.fr.md)
- [Communication, reçus et files](docs/reference-communication.fr.md)
- [Délégation native](docs/delegation-native.fr.md)
- [Fédération SSH](docs/federation-services.fr.md)
- [Application Android](docs/mobile.fr.md)
- [Instructions pour les agents](skills/bridget/SKILL.md)
- [Caches de compilation](docs/build-cache.fr.md)
- [Sécurité](SECURITY.fr.md)
- [Journal des versions](CHANGELOG.fr.md)

Les décisions, spécifications et preuves datées sont accessibles par
l'[index documentaire](docs/README.fr.md). Leurs résultats historiques ne prouvent
pas l'état de la version active.

## Licence

MIT. Voir [LICENSE](LICENSE).
