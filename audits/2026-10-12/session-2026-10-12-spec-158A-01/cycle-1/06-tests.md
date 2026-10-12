# Module06 — tests et fiabilité, cycle1 SPEC158A

Mode : lecture seule. Capture : scope5081979 / diff590c8bbb. Une anomalie MEDIUM, TEST-001, ouverte ; aucun CRITICAL/HIGH. Correctif indépendant préparé par protocole154, non appliqué et non revalidé sur ce snapshot.

Les47 nouveaux corps d’oracles prescrits ont été relus. Ils possèdent215 sites statiques de macros d’assertion ; aucun corps sans assertion. Les tableaux et boucles exercent plusieurs valeurs : ce chiffre n’est pas un nombre d’assertions indépendantes exécutées ni un taux de couverture instrumenté.

Preuves lues : T001253P/0F/2I ; ciblé daemon49P/0F (46nouveaux158+3régressions156) ; transport1P/0F. La FULL unique est encore en cours et son JSON est absent au moment de cette clôture. Aucun GREEN global. L’auditeur n’a lancé aucun test.

## Finding TEST-001

Le scénario historique guidance156_mcp_queue_schema accepte maintenant un credential fictif et un CallerScope avant de vérifier zéro connexion suivante. Il ne prouve plus le refus ChildProof avant toute connexion exigé par156 T024/T026. La production ouvre la connexion de plafond avant la garde privée execute_queue. Le refus final reste fermé ; aucune lecture de file ni escalade n’est démontrée.

Correction proposée : réutiliser le refus au début de execute_tool_at_with_identity, puis restaurer le même oracle avec listener non bloquant dès l’ouverture, sans credential fictif ni serveur CallerScope. Rejouer uniquement ce scénario après la FULL. ROOT et protocole154 portent déjà ce correctif cache ; résolution pending, pas fixed.

## Points vérifiés

- Rejeu durable A/B/C : réponse historique, état courant inchangé, C actif ; le payload illisible et les callbacks panic prouvent l’absence de nouveau parcours. Reprise Store sur disque.
- Arrêt/pause : quatre combinaisons dans la garde pure, Delegate et Continue ; refus sans création, clé libre après levée, rejeu avant garde, vues dynamiques suspended distinctes des Task durables. Sans cancel_active, tâches/epoch/pause inchangés.
- RPC de plafond : credential+instance, booléen et version explicites, deux clés seulement, réponses malformées fermées ; tâches réelles à posture omise, terminal avant nettoyage, ronde suivante et nettoyage après StopControl confirmé.
- Lecture/ACK : vraie SQLite, six entrées sur trois pages, receipts et séquences uniques. Les actions interdites ne font aucun échange métier suivant.
- Contrôle humain : ordre négociation/capacité/acteur, refus d’un client agent, cascade et racines uniques. La CLI exige son terminal pour mutation et ses cas d’usage invalide sortent2 avant socket. La borne ADR011 du même compte reste hors modèle ; aucun confinement CLI nouveau promis.

## Limites

Le parent-all-tools prouve l’accès aux validateurs30outils, pas30opérations. Le p95 de plafond est SQLite/décodage, pasRPC. La réponse automatique native reçoit une trame wrapper ; le générateur privé n’est pas exécuté par ce scénario mais son oracle spec105 et son code ont été relus. Pas de nouveau crash ou de panne SQL/RPC provoqué. Pas de scanner, service, DB runtime, build ou fournisseur. Les11autres documents du scope ne sont pas relus sémantiquement par ce module. Les gros fichiers sont échantillonnés ; seule la table scopes/lib est lue entièrement.

Couverture de lecture : 13/24 fichiers du scope, dont10Rust ; 3893/90476 lignes du scope. Aucun crédit repris des préparations antérieures. Cinq consommateurs/preuves hors scope sont distingués dans le read_manifest. Les24SHA du snapshot ont été revérifiés sans mutation.

## Plages fraîches

| Fichier | SHA256 | Plages |
|---|---|---|
| specs/158-reveils-controle/tasks.md | 8db90f488b1a8f9edc23858383786221858dcc5d5abf76af8dc180f505cd412a | 1–89 |
| specs/158-reveils-controle/validation/tests-158.md | fc5bd7e8916d69b8c749bec953f473f554832edd311851b63488425cffa10f56 | 1–160 |
| specs/158-reveils-controle/contracts/control158.md | f9cd5db490a35a41b22ea005ec466901c0eb99720bbbcd1cb6176e47cc0bf4e0 | 1–136 |
| crates/bridget-daemon/src/mcp_scopes.rs | eddc29d884ddfa43684f675aecd26f57070698b243d551ee46aa388ae3494a85 | 1–134 |
| crates/bridget-daemon/src/delegation.rs | 456f4066e4d95696d505d86b474a34f54da5a673124ac1c5e9ad79aafac240ea | 330–555, 1790–2125 |
| crates/bridget-transport/src/protocol.rs | 7f1792dbf9d95470a87b2bad35c7c648a2e4077cc603b943b9445d07f860a07c | 1260–1282, 3634–3663, 4496–4526, 8474–8535 |
| crates/bridget-daemon/src/mcp.rs | 5c8619f11dd5dbf1bfe7febeffd19b206d38f13b6452e5c4285a78d12bc6e68f | 618–674, 1130–1150, 2670–2728, 5435–5881 |
| crates/bridget-daemon/src/daemon.rs | a4ec4a19c6904e616fd40b8ee3cb46650eceb4cbc9e77816dd98bfe836e5aa7b | 10685–10938, 16035–16185, 16330–16390, 16730–16780, 22279–22792 |
| crates/bridget-daemon/src/daemon/native_delegation.rs | 4ea51409dad43bf864bb8048d0d60b807265bb10d8f31ad2368a1f8b8a43aeed | 1055–1105, 1447–1474, 1788–1826, 10120–10555 |
| crates/bridget-daemon/src/referent_control.rs | 91f40dfc1f66bdb74bb0be90723ef9cdb142855579e4e319c2b100c4b994e0f0 | 26–37, 145–160, 889–916, 940–959 |
| crates/bridget-daemon/src/cli.rs | a632f1ae095ce4bbd1dc8f589ffd45fee908ae873f485341ce1b9dc5247e847e | 6093–6335, 10343–10388 |
| specs/158-reveils-controle/validation/plafond-recherche.md | e57e60a341c606bf806068ef80872f96d62427be225cc166bfeb3fe112456258 | 1–110 |
| crates/bridget-daemon/src/wrapper.rs | 34efdf46f6c6b1fa517abc0440251ed1cc264615cbf27284f045be740ced9b81 | 6615–6640, 7665–7755 |
| crates/bridget-daemon/src/communication/client.rs | 4aefa1961876c4521d235e778b9e1fedb7044134578ac2a0509b5a22dcda807c | 1–145, 973–1110, 1138–1209 |
| specs/158-reveils-controle/data-model.md | 9519b5000fadbc8cf4b1dce832e4965bfbb0a06b324f7b5de901d9e629e561a7 | 1–80 |
| crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs | b5153a39df2b5cca49e81d4cfe2c335cf8a5638cbb893a84b44ab96eb1797969 | 1–115, 660–688 |
| crates/bridget-daemon/src/lib.rs | 1b4961f224a15666fe0fb3351095c3d47ba1b5e47fc60f9e6828e1f2b2f3e522 | 1–68 |
| specs/156-guidage-file/tasks.md | 7c9bd9a8e469a9f6aac9176ba8d645c0e3261749adf7bf8521fff4da349cb842 | 45–72 |
