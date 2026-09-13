# Fédération SSH permanente — services 095

Une fédération Bridget est un **daemon maître unique** dont une socket cliente est publiée sur une autre machine par SSH inverse. Les clients voient le même annuaire et le même ledger. Ce n'est pas une réplication de bases ni une sélection automatique de secours.

## Installer depuis le binaire

La commande `bridget federate` de la version 096 embarque le script canonique.
Elle le matérialise dans une copie privée propre à l'invocation, puis le retire ;
le script `scripts/federate-ssh.sh` reste l'unique implémentation. Lors d'une
installation, il copie son lanceur dans un emplacement utilisateur stable : le
service ne dépend ensuite ni du binaire appelant, ni du paquet source, de Git,
d'un worktree ou de l'ancien Bridget.

```sh
bridget federate ssh://cartae.app -p 2222
bridget federate status
```

La destination accepte `ssh://[utilisateur@]hôte[:port]`. Le port vaut 22 par
défaut ; un port dans l'URL et `-p` doivent être identiques. Pour une liaison
existante, le nom exact est comparé avant toute résolution. Un alias DNS peut
retrouver une IP enregistrée pour une réutilisation non mutante, sans réécrire
la cible SSH ni contourner `known_hosts`. Une résolution dure au plus cinq
secondes et 64 adresses ; l'inventaire est limité à 128 installations. Tout
dépassement ou toute ambiguïté est refusé, `--label` permettant de départager.

Pour une nouvelle destination, les options avancées sont `--label`, `--user`,
`--identity`, `--known-hosts`, `--root`, `--socket`, `--remote-root` et
`--remote-socket`. Les valeurs absentes ne sont demandées que si stdin et stdout
sont tous deux des terminaux. En automatisation, la commande échoue en listant
les options manquantes ; elle ne lit jamais le pipeline comme des réponses.

Les options de liaison sont identiques pour `run` et `install` : `--label`, `--host`, `--user`, `--port`, `--identity`, `--known-hosts`, `--root`, `--socket`, `--remote-root`, `--remote-socket`. Tous les chemins sont absolus et validés. La clé et le fichier de clés d'hôtes doivent déjà exister et être privés ; aucune acceptation automatique d'une nouvelle clé d'hôte.

- Sur macOS, `install` crée un LaunchAgent utilisateur ; sur Linux, une unité `systemd --user`.
- Le service supervise le processus SSH au premier plan. Une déconnexion déclenche une relance à cadence bornée, pas une boucle applicative.
- `status --label NOM` rapporte l'état natif du service, qui ne suffit pas à prouver une livraison : vérifier aussi `bridget who` depuis le client distant.
- `remove --label NOM` arrête et retire uniquement l'installation reconnue. Après arrêt confirmé, la même sonde nettoie son endpoint périmé pour permettre une réinstallation. Si l'arrêt ou le nettoyage reste incertain, les fichiers sont conservés pour une nouvelle tentative. Les clés SSH et les données Bridget sont conservées.
- La reprise d'une socket périmée est réservée au service installé, après contrôle de type, propriétaire, droits et refus de connexion explicite. Une socket vivante ou une erreur ambiguë reste un refus.

Sous Linux, pour survivre à la dernière déconnexion utilisateur, le gestionnaire utilisateur doit rester actif (`loginctl show-user NOM -p Linger`). Le script ne réclame pas sudo et ne modifie pas cette politique implicitement. Sur macOS, un LaunchAgent démarre à l'ouverture de session, pas avant login ; la machine maître doit rester allumée.

## Cartae client, Mac maître

Le namespace par défaut du nouveau binaire est `.cache/bridget-core` dans le HOME fournisseur, et sa socket `bridget.sock`. Le client peut donc employer le même binaire sans wrapper de shell ni modification globale du HOME. Un `federation.env` privé dans ce namespace contenant `channel=ssh-unix` atteste le canal.

Un client fédéré ne doit pas démarrer un daemon concurrent. Les agents lancés localement sur Cartae rejoignent le maître ; `spawn` adressé au daemon Mac lance un processus sur le **Mac**, pas sur Cartae. SSH ne crée pas un ordonnanceur distant.

La migration historique doit sauvegarder les unités, alias et bases, arrêter les anciennes autorités autorisées puis activer le nouveau client. Ne jamais réutiliser le script historique qui supprimait la socket avant le bind. Une coupure du tunnel ne justifie pas un fallback vers l'ancienne base.

## Commandes avancées et compatibilité du script

Chemin de distribution Mac : /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/federate-ssh.sh.
Chemin de distribution Linux : /home/moi/.local/lib/bridget-communication/scripts/federate-ssh.sh.
La copie du runner utilisée par le service est distincte et stable sous le répertoire utilisateur d'installation.

L'interface recommandée pour la liaison Cartae déjà installée est :

```sh
bridget federate ssh://cartae.app -p 2222
bridget federate status
```

La première commande reconnaît `cartae-core` enregistré avec l'IP
`37.59.185.67` sans modifier ses fichiers ni relancer son service. L'état actif
du service ne suffit toujours pas à prouver la connectivité : vérifier aussi
`bridget who` depuis le client distant.

Le script reste disponible pour le diagnostic avancé et la compatibilité 095.
Installation explicite du tunnel sur le Mac maître :

```sh
bash /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/federate-ssh.sh install \
  --label cartae-core --host 37.59.185.67 --user moi --port 2222 \
  --identity /Users/moi/.ssh/id_ed25519 --known-hosts /Users/moi/.ssh/known_hosts \
  --root /Users/moi/.cache/bridget-core --socket /Users/moi/.cache/bridget-core/bridget.sock \
  --remote-root /home/moi/.cache/bridget-core --remote-socket /home/moi/.cache/bridget-core/bridget.sock
```

L'IP est celle de cartae.app, déjà attestée dans le fichier de clés d'hôtes ; le script ignore volontairement la configuration SSH implicite. La machine maître ne change pas avec le choix du nom de service.

```sh
bridget federate status --label cartae-core
ssh -p 2222 moi@cartae.app /home/moi/.local/bin/bridget who
```

Le retrait est volontaire et coupe les communications distantes. La forme la
plus sûre nomme explicitement l'installation :

```sh
bridget federate remove ssh://cartae.app -p 2222 --label cartae-core
```

Une correspondance textuelle exacte avec l'hôte enregistré peut être retirée
sans confirmation supplémentaire. Si la destination ne correspond que par DNS,
un appel non interactif exige `--label`. Dans un double terminal, Bridget affiche
le label, l'hôte réellement enregistré et le port, puis exige la réponse littérale
`oui`. Une destination et un label divergents sont refusés sans mutation.

## Recette et limites

Les tests de scripts couvrent les contrats des deux gestionnaires aux frontières système. La recette réseau `scripts/tests/federation_095_roundtrip.py` utilise deux clients de test distincts, pas deux modèles : elle vérifie inscription, annuaire et messages dans les deux sens. Les résultats de service réels, chemins d'installation et procédure de retour arrière sont consignés dans `specs/095-federation-services/implementation.md` lors du déploiement.

Installation du 7 septembre 2026 : launchd Mac actif, annuaire commun et échanges
réels dans les deux sens validés avant et après interruption du tunnel. Les 22
tests de gestionnaires doublés passent sur macOS et Linux ; aucun maître Linux
avec tunnel sortant réel n'est revendiqué, faute d'authentification SSH déjà
autorisée pour cette recette. Aucun service vide n'est installé sur Cartae :
Cartae est le client du tunnel, le Mac en supervise la connexion.

## Retour arrière de la bascule Cartae

Sauvegarde : /home/moi/.local/state/bridget-migration-095-20260907.
Les bases historiques n'ont pas été déplacées ni modifiées par une migration.
Les unités historiques restent présentes mais désactivées ; les anciens skills
se trouvent sous `retired-skills` dans cette sauvegarde, avec les copies et
empreintes initiales sous `skills` et `MANIFEST.sha256`.

Pour revenir volontairement à l'ancien produit : arrêter d'abord les nouvelles
sessions distantes, retirer le tunnel par `remove --label cartae-core` sur le Mac,
rétablir l'alias et les skills depuis la sauvegarde, puis réactiver explicitement
les anciennes unités daemon, UI et timers sur Cartae. Ne pas démarrer l'ancien
daemon avec le nouvel alias. Cette opération restaure un annuaire Cartae séparé,
pas une fédération. Ne pas recopier une sauvegarde SQLite sur une base ouverte.

L'ancienne session `rc1` n'a pas été arrêtée. Elle reste attachée à son ancien
runtime ; seule une relance humaine via le nouveau binaire peut la faire basculer.
