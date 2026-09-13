# Réalisation 095

2026-09-07. Session validée, worktree distinct. Aucun changement du code Rust ni de la version binaire094.

## Préparation attestée

- Régression historique identifiée commit9066f10a ; serviceMac historique désactivé, pas réutilisé en place.
- Maître courant : /Users/moi/.cache/bridget-core/bridget.sock ; 5 agents au préflight. Aucun redémarrage Mac autorisé par la bascule Cartae ni effectué.
- Ancienne autorité Cartae : bridget-daemon.service PID1583850 et bridget-ui.service PID1583845, KillMode=control-group ; arrêt des agents Bridget/Jim explicitement autorisé.
- Linger utilisateur moi sur Cartae : yes.
- Namespace client /home/moi/.cache/bridget-core absent au préflight. Aucun conflit avec l'ancien /home/moi/.cache/bridget.
- BinaireLinux094 déjà vérifié : /home/moi/.local/bridget-communication-candidate-094/bin/bridget, SHA256 8b41fa7ad83cd33ba2158dcc347dc43aaca971021305ba62604dfd371475e976.
- BinaireMac installé : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget. Daemon déjà chargé plus ancien ; cette session ne revendique pas son rechargement094.

## État final

Service neuf actif, bascule Cartae et recette bidirectionnelle terminées. Aucun redémarrage du daemon ou des sessions Mac.

## Sauvegarde et baseline

- Baseline avant modifications scripts : `bash scripts/tests/federation_089_test.sh`, 23/23 réussis, 2.903s.
- Sauvegarde distante privée : /home/moi/.local/state/bridget-migration-095-20260907 ; unités/drop-ins/timers, cible de l'alias et binaire ancien conservés.
- SQLite `.backup` en ligne, deux `quick_check=ok` : databases/bridget.db SHA256 f2dfc960546b8a8263a3a46a22460d979aec629a15bb0b09c4aa124503a86b38 ; databases/maicie.sqlite3 SHA256 d9fa11a973865ea8a498b99a56d4a447e4b710544b586e0cf72227c6cb560028.
- Manifeste initial vérifié de20 entrées SHA256 1fae7fa7862024f3e78f2915514b96079f0333cf84382fb80d996fce29b3a30b ; compléments skills pourront produire un nouveau manifeste.
- Sauvegarde Mac des deux plist historiques : /Users/moi/.local/state/bridget-migration-095-20260907. Aucune activation/arrêt à ce jalon.
- Complément sauvegarde des skills Bridget et agent-bridge des trois profils Linux : manifeste91 entrées, SHA256 5c2d665c82c39e848cf83446ae77cd85c8b3e444cc29ece4f9533aa56147d638.
- Stage autonome Linux créé sans changer l'alias : /home/moi/.local/lib/bridget-communication/094/bridget (même SHA2568b41...e976 que candidat vérifié). Skills canoniques094 copiées sous /home/moi/.local/lib/bridget-communication/skills/bridget et alias agent-bridge Linux préparé à côté.
- Rouge pilote095 : 10 tests, 3 réussis et7 échecs attendus sur commandes admin absentes, 0.283s. Code transport encore inchangé à ce jalon.

## Validation et activation du tunnel neuf

- Contre-validation Mac : 22/22 tests095 (41.127s), 23/23 tests089 (3.392s), paquet autonome et mutation SHA256 réussis. Source cumulative094+095 également testée par le test de paquet ; aucun rebuild Rust depuis le worktree095.
- Revue indépendante finale APPROVE, aucun blocker scripts.
- Script distribué et runner installé de même SHA256 : be486961d7a74b8ed95d2ebbdf732df839899223b54545537605dae4a28cb794.
- LaunchAgent réel : /Users/moi/Library/LaunchAgents/com.bridget.federation.cartae-core.plist. Runner : /Users/moi/.local/share/bridget-federation/cartae-core/runner.sh. `state=running`, PID initial81539 ; aucune référence à l'ancien dépôt dans la commande.
- Nouveau namespace Cartae /home/moi/.cache/bridget-core : répertoire0700, socket0600, federation.env0600 avec channel=ssh-unix. `who` exécuté avec le binaireLinux094 voit les mêmes cinq agents que le Mac ; `ledger --limit 1` lit le ledger maître.
- Premier harnais d'échange : inscription et annuaire identique validés, puis erreur du client de test ; journal /private/tmp/federation-095-roundtrip.jsonl. Ce passage ne vaut pas validation aller-retour.
- Anciennes unités protégées avant arrêt par drop-ins runtime /run/user/1002/systemd/user/<unité>.d/095-no-sigkill.conf : SendSIGKILL=no relu sur daemon, UI, maicie-releve et ronde. Aucun service arrêté à ce jalon.
- Session ancienne rc1 hors cgroup conservée : elle ne peut pas redémarrer le daemon, mais doit être relancée humainement pour rejoindre le nouveau namespace.
- Suite095 exécutée aussi sur Linux : 21/22 verts, oracle de remplacement d'inode en diagnostic avant clôture.

## Clôture et limites attestées

- Diagnostic harnais : daemon Mac chargé9e55e5150d0c envoie `Deliver` historique, pas `DeliverIdempotent`. Le harnais accepte les deux formes et ne fabrique aucun ACK de livraison legacy. Recette réelle verte dans /private/tmp/federation-095-roundtrip-3.jsonl : même annuaire puis messages4c7da861-10c1-44c5-b677-4771902ade65 (Mac→Cartae) et d751f226-e951-4cf5-9f10-d1f6b4aa1cd0 (Cartae→Mac).
- Coupure contrôlée : PID81539 vérifié SSH exact de ce tunnel, SIGTERM uniquement ; disparu après3s. launchd reprend automatiquement, runs=2, nouveau PID95403. Nouvelle recette bidirectionnelle verte /private/tmp/federation-095-roundtrip-after-reconnect.jsonl.
- Fixtures de recette volontairement arrêtées ; l'annuaire durable peut garder leurs entrées stopped/unreachable. Aucun modèle n'a été lancé pour ces échanges. Aucun test d'idempotence du nouveau daemon094 n'est revendiqué sans son rechargement humain.
- Anciennes unités bridget-daemon.service, bridget-ui.service, bridget-maicie-releve.timer, bridget-ronde.timer : inactive+disabled relus. Daemon/UI et deux wrappers gérés disparus ; rc1 PID2723501 conservé. Aucun SIGKILL ni suppression de données.
- Alias /home/moi/.local/bin/bridget pointe maintenant vers /home/moi/.local/lib/bridget-communication/094/bridget, empreinte8b41...e976 revérifiée. Les six chemins skills bridget/agent-bridge de .codex/.claude/.agents résolvent les canons du nouveau préfixe. .agents/skills est déjà un alias de .codex/skills : les quatre répertoires physiques seulement ont été déplacés dans retired-skills. Le garde de la première commande a refusé de retraiter ce doublon, puis les six liens ont été vérifiés explicitement avec succès.
- `bridget who` et `status` ordinaires sur Cartae rejoignent le maître monordinateur via le namespacecore. Aucune configuration MCP statique ancienne trouvée ; seul le runtime rc1 conserve son ancien exécutable supprimé.
- Oracle Linux corrigé sans code de production : conserver l'inode initial par renommage dans la fixture, car Linux réutilisait immédiatement l'inode après unlink+bind (12/12 mesures). Suite095 finale sur Cartae22/22 réussis en26.880s ; Mac22/22 et08923/23 réussis.
- Systemd utilisateur est implémenté et testé avec doubles sur les deux OS. Pas de recette tunnel maître Linux réelle : aucune clé SSH self déjà autorisée sur Cartae, aucune nouvelle clé ou autorisation ajoutée. Le service actif de cette installation est launchd, conformément au rôle Mac maître.
- Pas de rebuild Rust, commit ou migration DB. Script autonome, tests, docs et spec095 publiés vers la distributioncore et la source cumulative094 ; runner autonome et scripts distribués sur Cartae. Skills094 conservées, référence administration095 ajoutée sans exposer de mutation MCP.
