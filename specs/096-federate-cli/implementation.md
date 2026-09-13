# Réalisation096

## Préparation

2026-09-07. Session096 explicitement validée. Worktree /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/096-federate-cli créé depuis HEAD ea52cd04 puis peuplé par copie de la source cumulative094+095 du worktree091, sans target, .git ni .wip-mesures. Les WIP antérieurs restent inchangés. Plus de cinq worktrees existent déjà ; aucun n'est supprimé sans vérifier leur état.

Synchronisation SpecKit projet seule : déjà à jour. Scripts officiels de setup/prérequis et mémoire projet absents ; modèles canoniques lus directement, commande mem absente. Checklist096 :6/6. Artefacts et Gherkin avant code. Pas de commit des changements hérités.

## Responsabilités

Zeno : script/tests shell096 et documentation ; Aquinas : façade Rust et tests binaires, puis compilation Mac/Linux ; Hypatia : revue indépendante ; root : specs, packaging, validation/installation. Ownership disjoint, un seul Cargo actif sur le target partagé.

## Validation

En cours. Réutilisation cible de compilation094 pour économiser l'espace et les rebuilds. Aucun service de production redémarré. Retrait limité aux fixtures.

Rouges observés avant implémentation : 4/4 tests binaires096 et 7/7 tests shell096 échouent sur la commande absente. Test paquet autonome vert (inclut refus de cible occupée, parents symboliques et détection de mutation), preuves : /private/tmp/b89-package-test.2nWBlR.

État de production avant recette096 : tunnel SSH PID95403, runner SHA256 be486961d7a74b8ed95d2ebbdf732df839899223b54545537605dae4a28cb794 ; reçu c00d587291c6ff70d2dd4d2e389fa0ad9b4090925e3edea4257f93bb5b8d23b9 ; configuration 14be682d1b4255e89b33f83b8cfb51612b55a042e8451e3a16f76c0dcc7be0d8 ; plist ea8f353540ca4c52ec6b6188f3c363dc96e122bf862bfaeca3bf91ec187b7512. Ces objets ne doivent pas changer pendant la recette de réutilisation.

## Gel et revue

Revue indépendante APPROVE sur le script SHA256 996b1e437c08727b72700788c636f856d0a536baac0ad492b1f5c8b437a0c186 et la façade Rust. Corrections de revue intégrées : port zéro/contrôles bruts refusés ; sélection DNS par label ; options explicites contradictoires refusées ; un label neuf ne permet pas de cloner une destination connue ; retrait label seul fermé aux autres options.

Tests shell Mac :096 12/12 (39,769 s),095 22/22 (41,059 s),089 23/23 (3,404 s). Bash syntaxe et diffcheck verts. Paquet final autonome vérifié à /private/tmp/b89-package-test.NZVDUD. La recette du script final reconnaît cartae.app:2222 comme cartae-core, sans réinstallation ; who distant voit toujours les cinq agents du Mac. Compilation finale et installation encore en cours à ce jalon.

## Publication Mac

Build-id explicite commun :096-9571d3eb2b32 (empreinte du manifeste des quatre fichiers de production096). Rust autonome096 :5/5 ; inventaire MCP094 :1/1 ; fmt et clippy workspace sans avertissement. Release construite avec le target partagé pour ne pas dupliquer les caches.

Binaire installé atomiquement : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; Mach-O arm64, mode0700, SHA256 07dc720a985d6cc10ca24fe4799e057adfde2e0e103f9c4d3916cabd6915f395. L'alias /Users/moi/.local/bin/bridget reste inchangé. Sauvegarde du binaire précédent et des sources remplacées : /Users/moi/.local/state/bridget-migration-096-20260907.

Le binaire candidat puis le binaire installé exécutent `federate ssh://cartae.app -p 2222` et `federate status` avec succès. Les quatre empreintes du service ci-dessus restent identiques ; PID tunnel95403 et daemon73764 inchangés. L'état natif du service ne revendique pas une connectivité ; celle-ci est vérifiée séparément par `who` depuis Cartae. Aucun retrait de production, redémarrage ou commit réalisé.

Le répertoire de distribution contenait encore des sources antérieures aux binaires094 (notamment attach, MCP et transport). Pour ne pas publier un arbre mixte, crates et Cargo.lock ont été alignés sur la source cumulative effectivement compilée ; comparaison des contenus sans différence restante. Sauvegarde préalable : /Users/moi/.local/state/bridget-migration-096-20260907/cumulative-source-before-sync.tar. Aucun fichier tiers absent de la source n'a été supprimé ; les worktrees antérieurs sont conservés.

## Publication Linux

Cache vendor/target094 réutilisé, Cargo1.92.0 offline/locked. Tests shell Linux :096 12/12,095 22/22,089 23/23. Oracle Rust portable corrigé uniquement dans la fixture (systemctl doublé pour atteindre le refus SSH37), revu APPROVE ; cinq tests096 verts. Un ETXTBSY transitoire a nécessité une relance des cinq tests, ensuite tous verts. Aucun comportement de production changé pour ce défaut de fixture.

Binaire /home/moi/.local/lib/bridget-communication/096/bridget, ELF x86_64 mode0700, SHA256 12ed07dd1dfa47ab79bf40033989cab51eaaabf9e3c21aa64237350811d24a0f ; build-id096-9571d3eb2b32 identique au Mac. Alias /home/moi/.local/bin/bridget remplacé atomiquement ; version094 conservée. Recette root du binaire installé : aide, statut local et annuaire partagé verts, cinq agents visibles. `federate status` répond correctement aucune liaison locale sur Cartae : le service du tunnel appartient au Mac.

Le daemon reste chargé au build9e55e5150d0c. L'avertissement de version ancienne est attendu ; aucun reload nécessaire à la nouvelle commande CLI, aucune session interrompue. La commande status décrit le service natif, pas une preuve de livraison ; la preuve réseau095 n'a pas été réinventée ni remplacée par cet état.

Documentation Linux publiée sous /home/moi/.local/lib/bridget-communication : guide docs/federation-services.md SHA256 d93adc9e126025b480ee79f4dbda5cd240f320306922feed1678a306fae894ad ; script SHA996b1e437c08727b72700788c636f856d0a536baac0ad492b1f5c8b437a0c186 ; skill canonique SHA9355ebdadbb262f89c68bbcec94bdb153f1be4df29d24a6958291c18ac50c488 ; référence commandes SHA5d23fa5c81047ac54c97346f553e187fc1757b26c35805f809dd3df491f130bb. Les symlinks Codex/Claude/Agents restent liés à ce canon. T001–T007 terminées ; aucun commit des WIP hérités.
