# Recherche SPEC-163 — préparation ciblée après 162

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

**Préparation initiale — historique :** Draft. Les lectures initiales ci-dessous portaient sur 611bfa7557b97587614687f6c6379d5961101fa6. Le socle canonique est désormais la source livrée 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437, checkout dcfd9bc29140457b1fa016c276f5a48933415a5c. Les numéros de ligne initiaux restent historiques et seront réattestés par l’audit ciblé. Aucun test, build, modèle ou service exécuté pour ce document.

## Preuves du socle

| Source lue | Constat utile |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs:89 | agents et machines existent. MachineProvision est strict ; absence d’un service de décision dans ce schéma. enabled contrôle l’admission, credential la possession. Ne pas fusionner santé et provision. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs:418 | for_delegation vérifie modèle/effort du catalogue. Les registres fournisseur épinglés vident machines. La politique de choix doit rester autorité du maître. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation.rs:230 et :878 | Task porte request/definition/remote_placement ; by_agent_request compare le canon et retrouve Task/refus/tombstone. Il n’existe pas encore de reçu de choix avant Task résolue. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:1031 | Delegate distant refuse fallback non vide ; la décision163 doit précéder le puits, pas réactiver un remplacement après admission. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:2129 et :3070 | Résolveur local commun, mais catalogue canonicalise le cwd sur le maître et utilise son registre. Il n’atteste pas les tuples d’un worker distant. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/remote_worker.rs:1653 et :1670 | prepare peut créer un worktree avant de vérifier la définition. Son appel pour chaque candidat serait un effet non demandé. Gardes de lecture à extraire sans ce chemin. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:2498 et :2698 | WorkerPrepare et opérations fermées existantes. Aucun probe de candidats ; négociation nécessaire avant nouvelle opération. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/lineage_protocol.rs:44 | LineagePlacementV2 est fermé ; un champ supplémentaire ne peut être introduit silencieusement. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/Cargo.toml:26 | minreq2 sans fonctionnalités par défaut ; TLS n’est pas acquis. Aucun besoin prouvé d’un autre client HTTP. |

## Arbitrages proposés

| Décision | Raison | Alternative écartée | Impact maintenance |
|---|---|---|---|
| Profils explicites ≤8 | Configuration humaine contrôlable, candidats bornés | Énumération de toutes les combinaisons modèle/machine/effort | Petit catalogue à maintenir, pas classement universel. |
| Probe groupé sans effet | Prepare crée potentiellement des worktrees | Appeler Prepare pour tous ou faire confiance aux fichiers Mac | Une frontière de lecture supplémentaire, gardes partagées à auditer. |
| Reçu dans DelegationStore | Canon auto et choix précèdent Task résolue | Mémoire seule, stockage côté serveur décision, nouvelle base | Petite migration justifiée, modèle de crash explicite. |
| decision_started avant HTTP | Pas de double décision après crash | Réessayer une requête dont l’émission est ambiguë | Un crash peut imposer le repli, mais pas un second coût externe. |
| Adaptateur SystemOne étroit | Seul protocole inspecté | API prétendument compatible Jev sans preuve | Extension future possible sans prétendre déjà supportée. |
| Permis tenu jusqu’à fin réseau | Timeout demande ne tue pas une inférence | Relâcher à deux secondes puis accumuler les appels | Dégradation vers repli, aucun thread illimité. |
| Version Lineage négociée | Schémas anciens fermés | Ajouter reason dans v2 ou créer un écran séparé | Un niveau de compatibilité supplémentaire à tester, pas nouveau produit UI. |

## Preuves historiques réutilisables, pas observations actuelles

/Users/moi/.cache/bridget-plan-cartae-flotte-laya-20261013/research.md et /Users/moi/.cache/bridget-plan-cartae-flotte-laya-20261013/contracts/laya-admission.md décrivent le code Laya local inspecté, POST /v1/systemone et l’ancienne santé loopback8790. Le modèle de décision et le modèle de l’agent y sont distincts. Ces documents signalent que usage.input_tokens ne détecte pas la troncature du head. Le rédacteur n’a effectué aucun appel réseau. Le préflight délégué décrit plus bas actualise les observations de santé ; il ne remplace pas une recette de décision, une validation de protocole ou une preuve des poids chargés.

Le premier reçu /Users/moi/.cache/bridget-delivery162-20261013/delivery162-runtime-checkpoint.json est historique. Le reçu final /Users/moi/.cache/bridget-delivery162-20261013/git-final162.json atteste maintenant la livraison et le nettoyage 162, 46/46, G8/G9 validés. Les projets Bridget/Roadie/Opus2D sont autorisés, Cartae est exclu. Le catalogue Codex réel annoncé par ROOT contient gpt-5.6-terra et effort omis ; ne pas en déduire quota ou qualité comparative.

## Inconnues du brouillon initial — état historique avant vérifications ciblées

- Avant gel du plan : audit de réemploi des reçus, partage du plafond et rétention/tombstones ; possibilité de grouper le probe sans lire deux fois les mêmes sources ; négociation maître/worker sans casser162 ; contrat de projection v3 et fallback strict.
- Avant implémentation réseau : lire l’implémentation minreq utilisée pour deadline/DNS et lecture bornée réelle ; décider la primitive d’attente réemployée. Aucun succès de timeout déduit de l’option nommée timeout.
- Avant activation : endpoint exact par machine, protocole, accès maître, secret éventuel, checkpoint et santé ; choix de fixed_profile/fallback_profile, profils modèles autorisés et préférences. Aucun profil de production inventé.
- Avant activation du profil Laya : recette synthétique réelle sur le checkpoint, entrée maximale et réponse bornée ; aucune garantie de troncature fondée seulement sur les octets.
- Dépendance avant canonisation acquise : correctif d’annulation T045/T046, G8 réel et G9 162 attestés dans le reçu final.

Les vérifications et décisions plus récentes à la fin du document soldent les points de code/HTTP ; la relecture finale ROOT des corrections R1 est acquise ; les preuves d’implémentation et d’activation restent attendues. Le PASS de réemploi163 est indépendant et consigné dans reuse-audit.md ; il ne prouve pas la compatibilité réelle du service. Aucune question de scope nouvelle n’est nécessaire : les décisions utilisateur sont déjà fixées.

## Réemploi163 — corrections fraîches de CODE161 et ROOT

Sources relues par CODE161 sous /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/ : delegation_lineage.rs:28/30/32/347–408 établit4 096 tâches, purge d’arbre soldé7j hors pression mais possible plus tôt sous pression, tombstone de 30 jours après purge. delegation.rs:945–951 établit refus de 30 jours après refused_at sans plafond quantitatif. Les mêmes constantes et méthodes ont été relues dans la copie intégrée par le rédacteur. Le brouillon «30j après issue/4096 refus/16Kio total» était inexact et est corrigé.

by_agent_request à delegation.rs:878–907 consulte Task, tombstone puis refus ; canon différent=envelope_mismatch. record_refusal:913–940 et insert_projected dans delegation_lineage.rs:445–468 ne protègent pas encore une table auto. Proposition retenue ROOT : table technique dans le même DelegationStore/SQLite ; transaction Immediate de collision native/auto, Task+liaison atomiques ; quota d’admission par UNION des clés, pas addition reçu+Task. Le choix et le canon final restent durables avant Delegate ; aucune décision recommencée après crash ou changement de politique. Ce comportement163 n’est pas déclaré existant.

Bornes retenues ROOT : métadonnées≤ 16 Kio distinctes de la mission≤ 64 Kio ; enveloppe canonique bornée explicitement, proposée512Kio pour les échappements et champs annexes, à vérifier contre le transport. Aucun raccourcissement du body65536 ni troncature. La collecte auto proposée reprend la vie de l’arbre lié et les tombstones, avec refus auto bornés et comptés ; les refus natifs historiques restent un fait séparé.

CandidateProbe doit extraire les gardes de lecture de WorkerRuntime::prepare, sans son prepare_workspace/store/création de worktree. CODE161 relève workspace_git::preflight réemployable seul, create exclu. Le probe demeure groupé par worker, pas une préparation par candidat. Aucun backend, base ou scheduler neuf.

## Préflight163 délégué — santé observée, aucune inférence

**Provenance :** observations en lecture seule de l’agent reuse161 transmises dans le fil de travail. Aucun reçu fichier n’a été créé pour ce préflight : ne pas citer un JSON inexistant ni lui inventer une empreinte. Le rédacteur n’a effectué ni SSH ni requête HTTP. Santé Mac8790/Cartae8790 observée le2026-10-13 à13:53:32Z ; aucun POST /v1/systemone, aucun modèle sollicité, aucun fichier modifié.

| Chemin observé | Fait attesté par le préflight | Limite |
|---|---|---|
| Mac http://127.0.0.1:8790 | HTTP200, status ok, modèles déclarés english/multilingual, device auto. PID59290 laya-serve ; installation editable0.3.20 dans /Volumes/8TB2/50-repos-archives/11.Repositories/laya/.venv. | La santé n’expose aucune révision ; pas de preuve de checkpoint chargé. |
| Code Laya local | /Users/moi/11.Repositories/laya résout au dépôt ci-dessus. HEAD23a17522aa4942da6cce53a995a275760320b691, laya/serve.py modifié et laya/serve.py.orig-079 non suivi. SHA du serve.py sur disque1e751a557f50ab98330a7d883432e4e2b96d0a6dc19eea6bfe3f7fc51dc4e081. | Le HEAD seul ne décrit pas les octets installés ; la source sur disque ne prouve pas quels octets Python sont déjà chargés dans le processus. |
| Cache HF Mac | /Users/moi/.cache/huggingface/hub/models--convaiinnovations--laya/refs/main contient cf7c54c0586eede67d827dfaab8cd2d2007273e9. | Référence cache, pas attestation des poids actifs. |
| Cartae http://127.0.0.1:8790 | HTTP200, trois modèles déclarés, trois révisions annoncées7b928d828b7b0e022f929d9bd2e44165aa270148. CaddyPID10896 écoute en loopback. | Ce port est un proxy, pas la preuve d’un modèle exécuté sur Cartae. |
| Chaîne du proxy | /etc/caddy/Caddyfile:96–106 route vers https://88.190.8.167:50002, avec authentification dont la valeur reste masquée, CA privée /etc/caddy/laya-macbook.pem et tls_server_name88.190.8.167. | La machine d’hébergement déclarée du décideur doit être distinguée du relais Cartae ; le nom de fichier CA n’atteste pas à lui seul une identité physique. Aucun secret recopié. |
| Mac http://127.0.0.1:8792 | HTTP200, mêmes trois révisions ; tunnel SSH existant PID86687 vers cartae, transfert local127.0.0.1:8792 vers distant127.0.0.1:8790. | Chemin réseau actuellement observé, pas configuration de décision Bridget ni garantie de pérennité. |
| Cartae127.0.0.1:8792 | Connexion refusée. Ancien opus2d-laya.service inactive/dead PID0. | Le contenu de /srv/laya/depot n’atteste pas un serveur actif. |
| Contrat HTTP visible | Mac /openapi.json HTTP200, title laya-serve, version0.1.0, routes /health et /v1/systemone. Cartae /openapi.json HTTP404. | Version OpenAPI/app ne vaut pas négociation de protocole ; compatibilité SystemOne distante non testée. |
| Bridget | /Users/moi/.cache/bridget-core/agents.json et /home/moi/.config/bridget/agents.json contiennent agents+machines, sans DecisionService ni endpoint163. | Aucun endpoint activé dans Bridget ; les nouvelles options restent désactivées par défaut. |

Les fichiers de configuration Opus2D /Users/moi/Nextcloud/10.Scripts/69.opus2D/monde/laya.json:27–28 et /home/moi/10.Scripts/69.opus2D/monde/laya.json désignent local8790/macbook8792. Ils appartiennent à Opus2D et ne configurent pas Bridget. Ne pas recopier leur choix ou leurs secrets implicitement.

La borne huit candidats est une politique Bridget proposée, pas une limite Laya attestée. usage.input_tokens ne prouve pas l’absence de troncature du head. Le service Jev indépendant reste non attesté. Aucun préflight de santé ne lève le gate de recette du checkpoint/template réel.


## Vérifications sur le socle 1dc0c602 et décisions ROOT

Les références ci-dessous portent sur le worktree /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission, après sync projet réussi par ROOT. Constitution et standards projet ont été relus. Les gates d’implémentation restent distinctes de la contre-revue du plan.

| Source vérifiée | Décision et raison |
|---|---|
| crates/bridget-daemon/src/registry.rs:89 et :175–190 | Racine tolérante, lecteur privé existant ; nouvelles sections strictes et politique relue par nouvelle admission, puis figée. Aucun watcher existant revendiqué. |
| crates/bridget-daemon/src/delegation.rs:409–414, :878–950 | Deux unicités conservées ; owner durable est le demandeur attesté. Collision commune Task/refus/tombstone/auto sous Immediate. Cinq codes de refus natif seulement sont persistés ; aucun élargissement opportuniste. |
| crates/bridget-daemon/src/delegation_lineage.rs:445–465 | insert_projected ouvre sa transaction Deferred puis publie après commit. Extraire l’écriture dans la transaction fournie et la publication après commit pour rendre atomiques Task + lien reçu auto. Aucun appel imbriqué à la méthode actuelle. |
| crates/bridget-transport/src/jsonl.rs:12 ; protocol.rs:2393 et :2407 | Trame maître 16 Mio, worker 2 Mio et réserve de cadrage 64 octets. Canon auto 512 Kio compatible ; valider la trame complète, y compris enveloppe et LF. |
| crates/bridget-transport/src/protocol.rs:2441 ; remote_worker.rs:2825–2851 | Hello v1 fermé ; sonde historique DaemonIdentityRequest réemployée. Nouveaux Hello/AcceptedWithCapabilities seulement après feature annoncée ; capacités mémorisées par époque, aucun champ v1 supplémentaire. |
| crates/bridget-daemon/src/cli.rs:269 ; delegation_mcp.rs:315 | Nouvelle sous-commande auto-delegate et outil bridget_auto_delegate vers la même opération native. Le guichet delegate historique est une autre surface et reste intact. |
| /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:351–382 | with_timeout peut retourner en laissant son thread interne vivant. Ne pas l’utiliser : tâche réseau unique avec permis jusqu’au vrai retour ; deadline du demandeur indépendante. DNS, connexion, TLS, écriture ou lecture serveur bloqués peuvent immobiliser ce seul travail ; repli/refus des suivants, sans accumulation. La garde MINREQ_TIMEOUT ajoutée après R1 est décrite plus bas. |
| /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/request.rs:204, :239, :258, :308 | send_lazy, corps 8 Kio + 1 octet de détection, en-têtes 8 Kio, ligne de statut 1 Kio, redirects 0. HTTPS via feature https-rustls du même client, certificat validé. |

ROOT a retenu le probe sans processus fournisseur/bootstrap ; les sous-processus Git de lecture bornés restent permis. Le délai service de deux secondes n’englobe pas ces vérifications Git, dont la borne existante est de dix secondes.

Pour les crashs, coordinator_epoch + revision CAS distinguent une requête concurrente et une préparation orpheline. pending repris sur politique figée n’a jamais émis HTTP ; decision_started orphelin impose repli/refus sans nouvel appel ; chosen rejoue seulement son canon. La raison courte est bornée à 256 octets. Le snapshot de politique est inclus dans les 16 Kio de métadonnées, sans le canon mission ni credential ; dépassement refusé avant insertion.

secret_ref est retiré : aucun mécanisme de secrets existant ne justifie cette abstraction pour les endpoints étudiés. Une exigence d’authentification non supportée provoque un refus explicite. La lecture v3 → v2 ne se replie que sur unsupported_version structuré ; invalid_request en v3 reste une erreur pour ne pas cacher un défaut du schéma v3. Le comportement historique v2 → v1 sur unsupported_version ou invalid_request structurés est conservé.

## Contre-revue R1 — faits rapportés et arbitrages ROOT

Le 13 octobre 2026, Claude Opus 5.5 conclut APPROVE_WITH_CHANGES. ROOT a vérifié les six objections et transmis les arbitrages suivants au rédacteur ; cette section ne prétend pas à une nouvelle vérification de code par le rédacteur.

| Objection | Preuve et décision retenue |
|---|---|
| O1 — permis réseau réel | ROOT a relu /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:72, :111, :197–207 et :351–380. Sans with_timeout ; MINREQ_TIMEOUT présent, même vide/invalide, rend le seul service non activable et sans réseau. Garde démarrage/préappel, raison visible. DNS, connexion, TLS, écriture ou lecture peuvent retenir le travail indéfiniment ; deadline demandeur distincte de deux secondes, permis jusqu’au retour réel, aucun thread de remplacement. |
| O2 — entretien des reçus | Entretien existant propriétaire, premier passage après réconciliation pour tous les reçus d’époque précédente, lots/CAS/E/S hors mutex. Aucun nouvel HTTP de reprise. pending → probes figés puis choix unique/repli autorisé ou decision_interrupted ; decision_started → repli/refus ; chosen → admission idempotente + lien atomique. Erreurs techniques/SQL restent visibles, non résolues et entretenues. Tout refus après chosen est terminal auto sans étendre les refus natifs. |
| O3 — pression de rétention | UNION 4 096 inclut les refus auto ; seuls les terminaux sans Task sont purgeables sous pression, du plus ancien au plus récent, tombstone trente jours atomique. Rejeu exact jusqu’à purge puis request_expired. Natif et auto tentent la purge sûre avant refus ; états pending/chosen/ambigus/vivants protégés. Les cinq refus natifs restent hors compteur. |
| O4 — lecture des versions | v3 → v2 seulement unsupported_version structuré ; v2 → v1 conserve unsupported_version ou invalid_request structurés. Pas de mélange de pages ni de repli journal/cancel. |
| O5 — collision des entrées | Natif externe consulte reçu auto avant by_agent_request sous Immediate. L’autorité interne chosen inscrit les refus uniquement dans le reçu auto, sans native_refusal sous la même clé. owner stable et unicités historiques conservés. |
| O6 — réalisation sur Cartae | Rust/compilation/tests Linux sur Cartae, namespace privé et fixtures isolées ; Mac pour T3/tests OS/signature/installation. Recette décideur depuis maître Mac, même commit source pour les builds. Une compilation active dans la flotte ; défaut de connecteur hors163 mais bloquant pour lancement distant. |

Le reçu de revue est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/adversarial-review-claude.md. Aucun test ni build exécuté pour ces corrections. Aucun APPROVE final présumé ; relecture ROOT ouverte.

## Préflight lecture seule actualisé — service distant et révision résidente

**Provenance :** observations du chercheur decision_preflight163, transmises par ROOT. Le rédacteur archive ces faits sans effectuer de SSH, requête réseau, tokenisation ou POST. **Observation :** 2026-10-13T18:49:55Z, depuis le maître Mac.

http://127.0.0.1:8792/health répond 200. Les modèles chargés annoncés sont typed-decisions, english et multilingual ; les trois révisions annoncées valent 7b928d828b7b0e022f929d9bd2e44165aa270148, device=auto. OpenAPI répond404 ; le chercheur l’explique par le retrait des routes de documentation dans le lanceur. Cette absence ne suffit pas à déclarer l’API de décision absente, et la santé ne démontre pas son fonctionnement sur une requête de décision.

La route SSH de lecture existante utilise BatchMode, StrictHostKeyChecking, UpdateHostKeys=no, ConnectTimeout=3 et ConnectionAttempts=1 vers moi@192.168.1.93 ; le hostname rapporté est macbook-pro-de-moi.home. Laya PID 91463, né le 10 octobre 2026 à 21:13:44 CEST, écoute sur 8443. Chaîne de lancement : /Users/moi/Library/LaunchAgents/com.moi.laya-serve.plist → /Users/moi/.config/laya-serve/run.py → /Users/moi/11.Repositories/laya/.venv/bin/python. Package editable 0.3.20 ; aucune preuve Git distante fournie par ce contrôle.

### Sources et artefacts sur disque rapportés

| Chemin absolu | SHA-256 et fait lu |
|---|---|
| /Users/moi/11.Repositories/laya/laya/agent.py | 1473ffd335a1becf55bf7aadde6d31f9d6be27862f5deaa5f4ebd3451fa154e3 ; lignes 307–309 : agent.revision vient du snapshot retourné. |
| /Users/moi/11.Repositories/laya/laya/router.py | 3e4a65b33f2d36076ac522945470d9ec10cfff97eb389b6ba1ad5f482ce8e079 ; ligne 412 : loaded_revisions lit les Agents résidents. |
| /Users/moi/11.Repositories/laya/laya/serve.py | ffe31e0a02e005fd2e6b78b79852fb5a223f6df1561eba07674dcb959c29176f ; /health exposé ligne 328. |
| /Users/moi/11.Repositories/laya/laya/common.py | 4ab41be6a4d9fdbad74eccf4f64c43e2c7404d227988d97ed8233ded8b182cb1. |
| /Users/moi/.cache/huggingface/hub/models--convaiinnovations--laya/snapshots/7b928d828b7b0e022f929d9bd2e44165aa270148/typed-decisions/model.safetensors | 4fa56de72383a9d3efa9cfa78955733c81b9fc8067a587ca4beb82c78107a24e. |
| /Users/moi/.cache/huggingface/hub/models--convaiinnovations--laya/snapshots/7b928d828b7b0e022f929d9bd2e44165aa270148/typed-decisions/tokenizer/tokenizer.json | 6c8aaa9a542084f2457eab775d4eeb51f92a70c0fd9de28d5edb0ddec3c08d30. |

Sources, poids et tokenizer ont un ctime du 26 septembre, antérieur à la naissance du PID. Ce rapprochement et le code de loaded_revisions renforcent la provenance de la révision résidente annoncée. Ils ne sont ni un dump mémoire des poids actifs ni une vérification d’un digest au chargement. Aucune configuration de révision/digests désignée « LAYA_REVISION/REVISIONS/SHA256_DIGESTS » n’est observée dans le plist ; aucun contrôle de digest à l’ouverture des poids n’est attesté. routing.model=typed-decisions contrôle le nom du modèle, pas l’empreinte de son contenu.

### Précision de la conclusion historique

Les anciennes observations proxy/santé et leurs limites sont conservées avec leur provenance. Ce préflight plus récent précise maintenant la santé depuis le maître, la révision annoncée par les Agents résidents et les artefacts sur disque du serveur. Il ne transforme pas la santé historique en recette de décision, ni le nom routing.model en preuve cryptographique des poids en mémoire. Laya est le profil étudié ; aucun support Jev n’est déduit.

### Recette ultérieure de template — non exécutée

Réemployer le tokenizer existant en mode offline, sans créer Agent/Router ni charger les poids. Utiliser build_sequence(tok, state, q, max_len=1024, head_max_len=512). Les clés internes q sont t/ins/crit ; les champs HTTP correspondants sont type/instructions/criteria. Comparer la séquence complète des options, instructions et state aux tokens effectivement construits. Cas maximal prévu : huit options de 32 octets, plafond de 48 tokens par option et budget de head 512. usage.input_tokens ne prouve pas l’absence de coupe ; le contrôle doit comparer le contenu complet au résultat de construction.

**Limites :** aucun service créé, aucun redémarrage, aucun POST, aucun tokenizer chargé dans ce préflight. Pas de nouvelle tâche ou campagne autonome. Aucun cochage, gate d’activation ou G8 déduit ; la recette de tokenisation puis de décision reste à effectuer selon les tâches existantes.

## Modèle de DecisionService et normalisation Laya — précision courante

ROOT rapporte dans les sources Cartae courantes /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs que DecisionService porte pub model: String obligatoire, figé dans le snapshot du service. Pour laya_systemone_v1, valeur attendue normalisée typed-decisions. Ce nom n’est ni modèle de l’agent futur ni preuve du digest des poids résidents.

La requête Laya corrigée a questions objet : candidate:{type:"choice", instructions, criteria:{c01:description,…,c08:description}}, racine model="typed-decisions", state borné, max_len=1024/head=512. La réponse porte un model racine informatif laya-rl-agent ; seul routing.model doit être égal au modèle attendu du snapshot. routing.reason explique le routage, pas le candidat. Aucune raison de candidat n’est inventée : reason absente est normalisée None/null. Une raison réellement fournie reste du texte affichable 1–256 octets UTF-8.

Le journal développeur réel rapporte un test ciblé de normalisation PASS à 20:25:22Z le 13 octobre 2026. Ce fait est transmis par ROOT, sans replay ni nouvelle lecture de sources compilées par le rédacteur. Aucun groupe final ni recette HTTP encore prouvé. Les anciens artefacts questions liste conservent leur empreinte et leur statut historique incompatible ; ils ne sont pas remplacés rétroactivement par le correctif. Bornes instructions 128/state 1536/JSON 4096/description 32 inchangées. Serveur générique, seul adaptateur Laya étudié, aucune compatibilité Jev inventée. Progression 4/45 inchangée, T040 et G8/G9 non prouvés.
