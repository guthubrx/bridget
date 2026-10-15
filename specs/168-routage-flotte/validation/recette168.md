# T045 — recettes réelles SPEC168, reçu partiel B

**État : In Progress, T045 ouverte, 51/56 tâches selon ROOT.** La voie Laya est établie sur deux candidats et le score déterministe sur 21 candidats. Le premier cas Haiku Mac a rendu un résultat sur 14 candidats par repli déterministe. La nouvelle recette Cartae établit lightweight_model sur 21 candidats, avec rejeu après redémarrage et mêmes canons/résultat. T045 est prête pour décision de ROOT après rapprochement ; ce document ne coche rien. Aucune release, clôture de recette ou réussite future n’est déduite.

## Périmètre et provenance

Source B de cette recette : 431b78f72bdd73eac4772af17bf36879d7a629d2. Les lectures ci-dessous proviennent des reçus privés exécutés par ROOT, sans nouveau test, modèle, build, service ou commande Git pour cette rédaction. Le parcours est natif Bridget/MCP dans un namespace privé, sans dépendance T3 selon la passation ROOT. L’état des services de production relève du reçu d’activation ; cette recette ne le remplace pas.

L’ancienne recette source A 269de64e172f71f8856373fe8b09cb2d8642da52 avait utilisé deterministic avec caller_deadline, sous le défaut de 2 s : elle reste historique et ne valide pas Laya. Le correctif du délai configurable et son oracle exact PASS à trois sous-cas sont publiés dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md, complément T045B. Ce test contrôlé reste distinct de la présente recette réelle.

## Voie Laya B — choix réel et résultat distant

- Clé : recette168-B-laya-0c2c55.
- Task : 5b08fb9c-1900-40ca-9757-4e268d82d0bb ; reçu automatique admitted.
- Ensemble durable : exactement 2 candidats. Choix c002, source decision_service, sans note de repli.
- Cible : machine cartae, worker worker-cartae, projet bridget, connexion codex-cartae-normal.
- Moteur codex, modèle gpt-6.1-sol, effort low.
- Répertoire exécutant : /home/moi/10.Scripts/64.bridget.
- Base demandée et constatée : 431b78f72bdd73eac4772af17bf36879d7a629d2.
- Résultat : marqueur RECETTE168_LAYA_B_OK, puis Linux, cartae, le répertoire ci-dessus et ce SHA exact.
- État final : result_available, error=null, result_sent=true, cleanup_done=true.
- Compteurs privés : un reçu automatique, une Task native, une exécution.
- Workspace Shared sans garde acquise : guarded=false ; shared_guard=none_acquired. guard_released=false ne signifie donc pas une garde encore détenue.

Les faits choisis sont figés : CPU 7 %, mémoire disponible 75 %, charge normalisée 6 %, âge 0 s/fresh=true au classement ; quota et crédits unknown. Ces valeurs historiques ne décrivent pas la charge actuelle.

## Rejeu après redémarrage privé — portée exacte

Le reçu compare le daemon privé avant 84508 et après 96580. La même enveloppe et la même clé sont rejouées via MCP, puis le statut de la même Task est demandé. Le reçu après redémarrage observe le marqueur terminal parent et confirme :

| Élément | Avant/après |
|---|---|
| Task ID, choix, projection métier et résultat | Identiques |
| Canon automatique | 9078a29490c7b9fdce230ea69f3c6a991caf4b13a0cdabe2324b9f723007dd4a |
| BLOB metadata | 101d1257e798ee9ac9ea5e40a2280b34812801cfa044f46fa3fc77f1c9fadb20 |
| Canon delegate | a8f9eec29c5fb4e2cd272a11b03d99e740040d6330f9a781576c8d3bdcf8a99c |
| Comptage durable | Un reçu, une Task, une exécution dans les deux lectures |
| Payload binaire Task | Différent ; aucune identité byte-à-byte revendiquée |

Le payload Task passe de 6afffdcd07d8ee68d7ea396bfb344be46803581ba420d8e077dc07b9f6f39623 à b0dd4a03b58be147cab97b508a2a44539995f83d295a06b246834bed5eed0a68. Cette différence est conservée avec la stabilité de la projection métier.

**Limite : le compteur HTTP réel et le compteur de nouveaux processus ne sont pas instrumentés.** Il est interdit de conclure « zéro HTTP » ou « zéro nouveau processus » depuis ces seuls reçus. L’absence de redécision est prouvée dans les oracles contrôlés déjà publiés ; la recette réelle montre ici la stabilité durable et les compteurs observés.

## Arrêt du runtime privé Laya

Le reçu cleanup-runtime atteste l’arrêt propre du parent, puis SIGTERM au daemon privé 96580 et au tunnel SSH privé 84590, tous deux identifiés par PID, naissance et exécutable ; leur absence est vérifiée. Il ne représente pas un inventaire universel de tous les processus fournisseur et ne vaut pas nettoyage final T048. Les reçus privés sont conservés.

## Refus réel initial Haiku et isolation explicite de télémétrie

La première tentative Haiku a refusé resource_saturated avant tout reçu automatique ou Task : la charge locale Mac était au moins 100 %. ROOT transmet ce fait ; il est conservé, sans l’assimiler à une panne de Haiku ni à une admission réussie.

Pour isoler la décision sur 12+, ROOT a ensuite désactivé uniquement routing_policy.machine_policies.local.telemetry.enabled dans chacun des deux namespaces privés Haiku et score, **avant admission**. Les deux reçus attestent zéro reçu et zéro Task avant changement. Aucun chiffre de mesure n’est fabriqué. La production conserve la télémétrie ON et le registre SHA-256 59802f1a8e42bdca8fc01935b0638f448f493a8dcf30973bf46d0f846ef7bf47, selon ROOT et la référence des reçus.

Cette isolation écarte la garde locale de ressources pour ces deux seules recettes. Elle permet de tester le choix du décideur ; elle ne prouve pas qu’une machine réellement saturée est admissible avec la politique de production.

## Score B — décision réelle sur 21 candidats

Le catalogue de cette configuration compte 27 possibilités déclarées. Le reçu durable après filtres en contient 21 : ces nombres décrivent deux étapes différentes et aucun top 8 n’est appliqué.

- Clé recette168-B-score-824a08 ; Task 3147c551-4415-4bd7-8359-b6161edc26ad, admitted.
- Choix c001, source deterministic, sans note de repli ; connexion codex-cartae-normal, modèle gpt-5.6-luna, effort medium.
- Résultat RECETTE168_SCORE_B_OK puis Linux, cartae, /home/moi/10.Scripts/64.bridget et SHA 431b78f72bdd73eac4772af17bf36879d7a629d2.
- Résultat disponible, ACK observé, result_sent=true et cleanup_done=true ; Shared sans garde acquise.
- Un reçu automatique, une Task native et une exécution.

Après redémarrage privé 16923→32922, la même Task, le choix, la projection métier et le résultat restent identiques. Les trois empreintes durables sont conservées :

| Élément | SHA-256 avant/après |
|---|---|
| Canon automatique | 74eb197a953b613e56d3a56eea531248e894637e63cf0d344049320e64921bb1 |
| BLOB metadata | 98d5be1a06afa82c53b64c4e4910ff0408aa43fe74d7dc56c676901d7d55ef7e |
| Canon delegate | 95447c1f6a3773cbaea4b7f7d309dac69a7978f97eb0c1e2cb40f1432dad8a76 |

Le payload binaire Task change de 43365ad9ff67846c58f7bb96155188225f22a90795d4639d822c9073d8bc1b6c à ed0d4dcc2393721f6f2d127d3f74855d05a7f2bad8b548e5fffbb56ea563c1c4. La limite du cas Laya vaut aussi ici : HTTP et nouveaux processus réels ne sont pas comptés. La stabilité durable ne prouve pas à elle seule leur absence.

## Cas Haiku B — résultat réel, petit décideur non qualifié

Le catalogue local déclaré compte 17 possibilités ; le reçu durable contient 14 candidats admissibles. La Task 7dceb2b7-b42c-4cbf-854e-3d15c96cf50e est admise, puis rend RECETTE168_HAIKU_B_OK, Darwin, monordinateur, /Users/moi/Nextcloud/10.Scripts/64.bridget et le SHA 431b78f72bdd73eac4772af17bf36879d7a629d2.

**La décision est deterministic, avec lightweight_decision_unavailable.** Le marqueur de résultat ne prouve donc pas une décision Haiku. L’exécutant choisi est codex-mac-normal/gpt-5.6-luna, effort medium ; il est distinct du rôle de petit décideur recherché.

Le reçu observe result_available, error=null, ACK, result_sent=true, cleanup_done=true et un reçu/une Task/une exécution. Les ressources locales restent inconnues conformément à telemetry.enabled=false dans ce namespace ; aucune mesure libre artificielle n’est injectée. Le refus resource_saturated antérieur reste conservé dans la preuve MCP.

Le diagnostic en lecture seule transmis par ROOT constate loggedIn=false pour le profil Claude courant. Ce constat concerne ce profil Mac et ce cas historique ; il n’est pas transposé à Cartae. Aucune authentification automatique, attribution à un quota, ni correction réussie sur le Mac ne sont inventées. Ce diagnostic ne remplace pas une recette native avec source lightweight_model.

## Nouveau parcours Haiku Cartae — choix réel et rejeu prouvés

Le namespace privé /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18 utilise un maître Bridget sur Cartae, sans serveur ou pont T3. Son runtime-topology conserve le catalogue 27 et la source B 431b78f72bdd73eac4772af17bf36879d7a629d2. Dans cette topologie, machine_id=local désigne Cartae ; il ne désigne pas le Mac.

### Première étape à sept candidats, conservée séparément

Le reçu first-seven-receipt.json porte la clé recette-haiku-cartae-759bee18 et la Task 891e2347-afbc-47c5-addf-d402ff778e68. Il constate lightweight_model, sept candidats, résultat RECETTE168_HAIKU_CARTAE_OK sur Linux/cartae au SHA B, ACK, cleanup_done=true, un reçu/une Task/une exécution. Le modèle de petit décideur déclaré est claude-haiku-5-5 ; le préflight de ce reçu constate loggedIn=true et authMethod=claude.ai sur Cartae. Ce n’est ni le profil Mac précédemment déconnecté ni une preuve 12+ : minimum12_qualified=false est conservé.

### Matrice de 21 candidats — reçus terminaux rapprochés

- Request_id : recette-haiku-cartae-759bee18-matrix ; owner 8f2f787f-9eb7-4ce9-8cd0-070f50c577a5.
- Task 67f97d3e-be24-4f0d-a31f-258a56c744e5, admitted, exactement 21 candidats durables.
- Source lightweight_model, choix c001, connexion codex-cartae-normal, moteur codex, exécutant gpt-5.6-luna, effort low.
- Résultat RECETTE168_HAIKU_MATRIX_OK, Linux, cartae, /home/moi/10.Scripts/64.bridget, SHA 431b78f72bdd73eac4772af17bf36879d7a629d2.
- ACK Unix 1792075876, result_sent=true, cleanup_done=true, error=null ; Shared sans garde acquise.

L’exécutant Codex est distinct du petit décideur Haiku déclaré dans ce namespace. Les métadonnées terminales source lightweight_model, le reçu initial et la preuve MCP réelle qualifient ce rôle sur l’ensemble 21 ; le reçu à sept candidats reste séparé.

Le daemon privé passe de 3804689 à 3834811. La preuve native après redémarrage contient exactement deux appels MCP : même bridget_auto_delegate, puis bridget_task_status. ROOT observe le marqueur RECETTE168_PARENT_RESTART_REPLAY_OK ; les sorties MCP et les reçus avant/après conservent le même ID, la même décision et la même projection métier/résultat.

| Élément | SHA-256 avant/après |
|---|---|
| Canon automatique | 350dddd9483f72bc2e877bd15a15c37e8ba39723d23db63e8291487448274041 |
| BLOB metadata | 9d9e44b6d5a1401c880ece29b31f59fc41b95544f0067165a927dba894f68a06 |
| Canon delegate | 8e0fb5bba0d918a3fa1b228300861ed2bcd497d8faea715aaedf04f7dbabbca2 |

Le payload binaire Task passe de 55769cdc6df6f4a5965793e63c9893b7e6ca92d90197ec7ef306610a0f8b1552 à 51bbc3d9a63b271f467d9c592d48acb7b0d3bdd0e01abdadbf92d0e209eb3726 ; il n’est pas revendiqué byte-identique.

Les compteurs globaux 2 reçus/2 Tasks/2 exécutions incluent la première recette à sept candidats. Le comptage de la seule clé matricielle est un reçu/une Task. Le compteur d’exécutions est global ; aucune valeur par clé supplémentaire n’est inventée. HTTP et nouveaux processus réels restent non instrumentés ; zéro redécision côté fournisseurs n’est pas déduit de ces seuls compteurs.

### Horodatages et budget SC-007

Le reçu matriciel porte created_at=1792075807 et updated_at=1792075812. Ils décrivent l’écriture durable, pas un chronomètre monotone de toute la chaîne. Leur écart de 5 s n’est donc pas une mesure de conformité temporelle universelle. Les oracles contrôlés déjà PASS restent la preuve des deadlines absolues, du budget partagé et des bornes ; aucune nouvelle FULL ou mesure n’est ajoutée ici.

### Arrêt privé et protection de T3

cleanup-runtime-cartae.json atteste l’arrêt natif du parent, puis SIGTERM après vérification d’identité au daemon 3834811 et au relais 3813547 ; chacun est absent après 3 s. DB et preuves sont conservées, production_services_signaled=false. Le worker Mac privé 74195 est observé absent au redémarrage et n’a pas été tué, selon ROOT. Le reçu Mac final confirme SIGTERM au tunnel privé 74191 après vérification d’identité et son absence après 3 s ; le worker privé 74195 était déjà absent, sans signal. Les runtimes privés des recettes sont soldés ; DB et preuves sont conservées.

L’activation B active_verified est attestée par /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json, SHA-256 6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e : source 431b78f72bdd73eac4772af17bf36879d7a629d2, version 0.4.0, daemon 71927, pont 72468, worker 3767260, catalogue 27, routage ON, DB quick_check=ok/FK0. Ce checkpoint ne prouve pas la release GitHub.

Le serveur T3 ancien 65608 a été remplacé extérieurement par 97328, né à 16:51:02 CEST, pendant la restriction d’accès, selon le constat frais ROOT. Aucun signal T3 n’a été envoyé par notre activation ou ces recettes. L’application 65544 née 04:07 et le tunnel de production 60206 sont inchangés. Le reçu B conserve son état historique initial ; il ne justifie pas d’affirmer que 65608 est encore vivant.

Cette topologie conserve la télémétrie locale Cartae réelle ; private_mac_telemetry_isolation=true dans le reçu indique l’isolation de la cible Mac privée. Aucun chiffre de charge artificiel ni changement de production n’est déduit.

## État des trois voies

| Voie | Candidats durables | Issue métier | Qualification du décideur |
|---|---:|---|---|
| Laya B | 2 | Résultat distant Cartae, SHA B, rejeu après redémarrage | decision_service établi |
| Score B | 21 | Résultat distant Cartae, SHA B, rejeu après redémarrage | deterministic établi |
| Cas Haiku Mac B historique | 14 | Résultat local Darwin, SHA B, ACK/cleanup | Non qualifié : deterministic et lightweight_decision_unavailable |
| Haiku Cartae, matrice B | 21 | Résultat Linux/cartae, SHA B, ACK/cleanup ; rejeu après redémarrage | lightweight_model établi par les reçus et le MCP natif |

Les catalogues 17/27 et les ensembles admissibles 14/21 sont distingués. La télémétrie locale privée OFF des deux cas reste explicite ; aucune conclusion sur l’admissibilité d’une machine saturée en production n’en découle. T045 reste ouverte.

## Reçus privés canoniques

- /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/before-restart-receipt.json — SHA-256 e8349619a079789720b6ac7df84264396d885f48167746032e4d20d00d8440f9.
- /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/after-restart-receipt.json — SHA-256 8098115f4ec4a6fb2515a871ecd6819e146eeaa6ece4ed0b425bc24b926d270d.
- /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/replay-after-restart-send.json — SHA-256 a5d248a0d0c7e68932aec70282e4fcf366deb0b433085d51643c745c8858ceab.
- /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/cleanup-runtime.json — SHA-256 570144914ec0f5e89cb0e1be90f785e3125268c8d921b58469ee43faefa66b6f.
- /Users/moi/.cache/bridget-delivery168/r168-b-haiku-ee884a/private-telemetry-isolation.json — SHA-256 4d0d103297ee64caea51a67fc0ee65de6aefed3fe8365d3e5b18393df32d16aa.
- /Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/private-telemetry-isolation.json — SHA-256 09ff09c27fb6db6317388313515de4a7959bdc41dd6379220bf12ef6f3cff393.

Les deux fichiers private-telemetry-isolation portent le changement true→false du seul champ local de télémétrie et l’empreinte de production de référence. Ils ne sont pas des reçus de résultat métier.


### Complément de reçus score, cas Haiku et preuves MCP

- /Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/before-restart-receipt.json — SHA-256 cb47a03ae272d9e9dee9525c8d7492da268e8400a3d3b30486d0cba5dbbc5212.
- /Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/after-restart-receipt.json — SHA-256 49020adfed54f87f411ac8e17ef92064b19642ec5f3bf1121a2781bbd0779971.
- /Users/moi/.cache/bridget-delivery168/r168-b-haiku-ee884a/before-restart-receipt.json — SHA-256 b21e3d821a4fcb17d3ace4934a5408490beb1994a965d90bcec2c2721818e211.
- /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/native-mcp-proof.json — SHA-256 1ce78f44ced3065a7f29d376152d5f9e427d5dec1aad432f5b5cef3eb0d6cc4c.
- /Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/native-mcp-proof.json — SHA-256 4b296e364a724a7473deb97577b2bafa9687e0b2a73cb0c49e04b2e601a169bb.
- /Users/moi/.cache/bridget-delivery168/r168-b-haiku-ee884a/native-mcp-proof.json — SHA-256 7e54a283d4c17707df918b500db048be1524b77bbffa9937e2aa9742dc0bb297.

Les preuves MCP conservent les appels natifs réels, leurs résultats et les ACK. Celle du cas Haiku contient aussi le refus initial resource_saturated ; aucun nouveau tour n’est exécuté pour lire ces traces.

### Preuves existantes du nouveau parcours Cartae

- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/first-seven-receipt.json — SHA-256 255448402eaeaada2dc75b5b2c1dc362b95cb96202b1c5cf371ad4b773f969dd.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/matrix-mcp-send-receipt.json — SHA-256 d48c5010afff7a7a160a860b1b6b3bfa189abb99b07175f32cd82c52c7988e8c.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/runtime-topology.json — SHA-256 a6293a4764f875e83a55846b719e3cec265cfcd807637b6f602668d36a8c55d5.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/prepared-topology.json — SHA-256 3e0e2ee738140f46472cd6850ff63d715ea6566a7ea50631ce7d8ce9c2de7645.

### Reçus terminaux Haiku matrice et arrêt Cartae

- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/before-restart-matrix-receipt.json — SHA-256 4839f52ca8c680a573f0fe01d8bef79cef120d96c49a47ac01e40d496299d11e.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/after-restart-matrix-receipt.json — SHA-256 f6d3d046bfe8cd1c3665b3a583931a3dbc170aa1b9603be20ef239a52c740581.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/native-mcp-matrix-proof.json — SHA-256 287d297af71ee7b1028d1ebb676e99c41ca3413830593bbd9c211532619bdd24.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/native-mcp-matrix-restart-proof.json — SHA-256 aea8a42db1b2eae8265030cb669669cdd31fe9b761cc30f173607873fdc9a003.
- /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/cleanup-runtime-cartae.json — SHA-256 fbf22a7028aad700ddc05dd4345f9b974db7b4504c710ec2cc920f231552e2e9.

## Limites de clôture

Les trois voies et les replays réels sont maintenant rapprochés ; T045 est prête pour décision de ROOT, sans cochage par cette rédaction. Les runtimes privés des recettes sont soldés ; release, récupération et nettoyage source final relèvent des étapes suivantes. Aucun droit universel aux modèles n’est déduit du catalogue. Le rejeu de cette recette ne remplace pas tous les scénarios contrôlés de concurrence/crash/permissions.

Les limites historiques restent : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 échecs initiaux passent après reprise, trois annulations 159 non validables sur Linux ; aucun GREEN global. Audit A pass conservé sur son manifeste, deux LOW TLS et un INFO ouverts ; aucun nouveau scan ou score d’audit ici. Release et nettoyage restent des gates distinctes.

## Complément final — runtimes privés soldés

/Users/moi/.cache/bridget-delivery168/r168-mh-759bee18/cleanup-runtime-mac.json — SHA-256 829c9e658dae179945a8c48025745d11d39940867f86f4b591c9bd98a1641a8a. Tunnel privé 74191 : identité vérifiée, SIGTERM, absent après 3 s. Worker privé 74195 : déjà absent, aucun signal. Production non signalée ; DB et preuves conservées. Avec le reçu Cartae, tous les runtimes de recettes sont soldés. Cela ne vaut pas nettoyage des branches/worktrees/stages T048 ni release.
