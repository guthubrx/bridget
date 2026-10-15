# Plan SPEC169 — Priorité Bridget depuis T3

Date :2026-10-15. Implémentation14/14, P2 CONVERGED avec0 ajout. Pipeline :Livré avec conservations justifiées. Installation, activation unique, reprise, consigne ON, HTTP et recette native sont prouvés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md. Les checkpoints antérieurs ci-dessous restent historiques.

Spécification : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/spec.md

## Résultat attendu et périmètre

Ajouter `preferBridgetDelegation`, un booléen à `false` par défaut, dans les réglages T3 existants. Avant chaque tour, les consignes prennent l'un des états `off`, `prefer` ou `blocked`. Elles privilégient la délégation automatique Bridget si le réglage est actif et si Bridget est effectivement monté. Elles annoncent le blocage dans les autres cas actifs. OFF conserve le parcours T3 habituel.

Le moteur, le routage168, les droits et Lineage restent existants. Aucun Rust, RPC ou protocole Bridget/public, service, stockage distinct, interface visuelle ou algorithme de flotte n'est ajouté. Une correction causale de la réponse HTTP existante pour le sujet authentifié Bridget permet la reprise du gros fil ; elle est décrite ci-dessous. La livraison comprend un paquet T3 signé, son installation et son activation contrôlée, déjà autorisées par l'utilisateur. Elle ne nécessite pas une nouvelle confirmation.

## Contexte et lieux de travail

- Artefacts Bridget isolés : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget ; branche `session-169-priorite-bridget`, base `9077b81e`.
- Source T3 de référence relue par les explorations : /Users/moi/11.Repositories/t3code-local ; base `76780a20`, branche `local/main-20261009`.
- Source T3 de développement sur Cartae : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget ; base `76780a20`, provenance contrôlée par ROOT.
- Pile existante : TypeScript, Effect et schémas de contrats T3. Aucune nouvelle dépendance ni migration de base.
- Livraison T3 : remote `fork`, dépôt https://github.com/guthubrx/t3code.git, branche `local/main-20261009`. Le remote `origin` désigne l'amont et ne doit pas recevoir de push. Aucune PR n'est demandée.
- Livraison des seuls documents Bridget : branche principale du remote `github`. Pas de reconstruction ni de relance des services Bridget pour cette préférence.

## Architecture retenue

### 1. Un réglage et un fait de lecture fiable

Étendre seulement `ServerSettings` dans /Users/moi/11.Repositories/t3code-local/packages/contracts/src/settings.ts. Ne pas ajouter le champ à `ServerSettingsPatch` : l'utilisateur a demandé un réglage par fichier, sans mutation publique nouvelle. L'absence du champ conserve `false`. Un patch d'un autre réglage préserve la valeur lue. Le fichier absent dans une installation sans réglages conserve aussi les valeurs normales de compatibilité.

Réutiliser `ServerSettingsService` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:256. Son chargeur possède déjà `settingsFileTrusted` à la ligne772. Conserver en cache le tuple cohérent réglages/confiance et exposer ce fait à la lecture interne des consignes. Ne pas ajouter de clé JSON sentinelle. Une erreur ne devient jamais un OFF valide, même après un ancien OFF. Les autres consommateurs gardent leur contrat existant.

Dans /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:1250, `updateAndPersistSettings` peut repartir des defaults après une lecture non fiable. Refuser alors toute mise à jour par un `ServerSettingsError` nommé, avant traitement de secret ou écriture. Le fichier invalide reste byte-identique. La confiance revient après réparation valide relue, ou après une écriture valide depuis un état déjà fiable ; elle ne se rétablit pas en écrasant le fichier invalide.

La lecture est actualisée avant chaque `runtime.startTurn` par le gestionnaire existant : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823. La préparation MCP seule ne suffit pas pour un fil déjà attaché. Cette lecture n'ajoute ni sonde réseau de santé ni décision de flotte.

### 2. Une préférence qui ne porte aucun droit

Le mode est une métadonnée de consigne, calculée avec le fait de montage Bridget réel. Un endpoint MCP T3 ou le nom d'un fournisseur ne prouve pas ce montage.

Réutiliser `ProviderAdapterV2TurnInput` dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396. Ajouter son champ optionnel interne `bridgetDelegationPreference?: "off" | "prefer" | "blocked"`. Le gestionnaire décore l'input à chaque `runtime.startTurn`, dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823, avec la préférence et la confiance de lecture actuelles. Les compositeurs y associent le montage réel. Vérifier la propagation dans la compatibilité vers les drivers legacy.

Ce porteur existe même quand `prepareMcpSession` retourne `undefined`. Les sorties anticipées à /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:471 et le fil déjà attaché à la ligne1311 ne doivent pas faire perdre ON. Un raccord sans MCP reçoit donc `blocked`, sans faux credential. Aucune map ni mutation de configuration MCP n'est nécessaire.

Conserver la même référence de configuration de session, lorsqu'elle existe, dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts. Remplacer cet objet efface les faits de permissions à la ligne153. Une modification de préférence ne change ni credential, ni objet de session, ni rotation ou remontage MCP.

Les attestations et vérifications existantes restent seules autorités. Une vraie rotation d'identité conserve son cycle166. Un changement de préférence ne simule pas cette rotation.

### 3. Un seul endroit pour composer les consignes

Étendre les compositeurs existants dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:3, :63, :72, :105 et :115. Extraire seulement les paragraphes de délégation qui doivent varier. Ne pas ajouter une consigne ON après un texte qui prescrit encore une délégation native T3. Éviter les remplacements fragiles de sous-chaînes et la copie intégrale du prompt par fournisseur.

| Situation | État effectif | Consigne de délégation |
|---|---|---|
| Lecture fiable, clé absente ou `false` | `off` | Consigne T3 habituelle. |
| Lecture fiable, `true`, Bridget réellement monté | `prefer` | Utiliser la délégation automatique Bridget ; conserver les contraintes explicites. |
| Lecture non fiable | `blocked` | Signaler le réglage indisponible ; ne pas prétendre avoir lu OFF. |
| Lecture fiable, `true`, montage Bridget absent ou désactivé | `blocked` | Signaler Bridget indisponible ; ne pas inventer l'outil ni prescrire un repli silencieux. |

`prefer` et `blocked` conservent l'exception d'un fil T3 expressément demandé. Les outils T3 hors délégation gardent leurs consignes. Inventorier et corriger les six injonctions de délégation T3 contradictoires, y compris les rondes de revue, la délégation depuis le scheduler et le repli terminal ACP. Elles restent dans OFF lorsqu'elles sont pertinentes. L'hôte n'ajoute aucun dispatch après une erreur Bridget.

Le montage ou un catalogue antérieur ne prouve pas l'authentification future du daemon. La consigne ON demande `bridget_auto_delegate`. Si le catalogue est paresseux, une seule recherche d'outil bornée par le mécanisme existant peut le rendre accessible avant de conclure à son absence. Sans outil ou découverte disponible, annoncer le blocage. Aucune nouvelle sonde HTTP de santé n'est ajoutée par tour.

La consigne ne promet pas l'obéissance absolue du modèle. Les refus et la non-duplication restent contrôlés par Bridget. L'absence de réponse après admission demande un suivi ou un rejeu avec les références existantes, jamais une autre délégation d'hôte.

### 4. Appliquer les changements aux tours et aux raccords existants

Codex et Claude montent déjà Bridget par /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/CodexMcp.ts:29 et /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/ClaudeMcp.ts:186. Réutiliser ces montages sans activer un outil désactivé.

- **Codex** : conserver le chemin `additionalContext` et les instructions développeur dans /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/CodexDeveloperInstructions.ts:197 et /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:772.
- **Claude** : mettre à jour l'append SDK et la clé de contexte dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:1443 et :2150. Le SDK peut conserver l'ancien append jusqu'à compaction. Utiliser son option réelle `snapshot: false` pour l'append T3 dynamique ; la clé seule ne suffit pas. Vérifier l'option et les deux sens ON/OFF après reprise sans compaction. Respecter le refus existant de remplacer une query avec travail de fond actif : ne pas le tuer, ne pas accepter un tour au mode périmé, signaler l'attente puis appliquer le mode au prochain tour après sa fin. Conserver preset, modèle, droits et textes hors délégation. L'invalidation possible du cache/contexte de réflexion (`thinking`) est un compromis nécessaire et non chiffré ; des options transmises ne prouvent pas encore un résultat réel du modèle.
- **ACP, OpenCode, Cursor et Pi** : sans montage Bridget existant, ON donne `blocked`. Étendre leurs chemins actuels d'injection. La comparaison d'état ACP inclut la préférence ; les deux variantes OpenCode sont couvertes. Cursor ne peut pas conserver un premier contexte OFF après passage ON. Les commandes natives commençant par `/` gardent leur sémantique ; un contexte non injecté ne doit pas être marqué comme injecté.

**Exception Pi bornée, arbitrée ROOT** : réutiliser l'extension existante dans /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts:364. Son état initial est bloqué/inconnu. Envoyer `/t3-bridget-policy` avec l'enum `off/prefer/blocked` et un identifiant court neuf validé, par la requête `prompt` Pi existante avant le premier vrai prompt, même OFF, puis lors des changements. Vérifier sa présence dans `get_commands` de ce processus ; sinon elle pourrait partir au modèle. Le handler synchrone valide avant mutation, puis émet APRÈS affectation un reçu privé fermé par le canal existant `ctx.ui.setStatus`. L'adaptateur intercepte et masque ce reçu. Avancer `lastAcknowledgedDelegationPolicy` seulement après reçu du mode corrélé à cette tentative ET réponse de requête corrélée `success: true`. Le SDK peut avaler les erreurs du handler et renvoyer success : l'ACK seul ne prouve donc pas l'application. Ignorer les reçus anciens/étrangers ; sur erreur, reçu manquant ou ACK invalide, aucun vrai prompt ni avancement. Filtrer la commande du catalogue utilisateur, sans modifier ses slash commands. Ce contrat privé évite un redémarrage Pi par changement ; aucun RPC Bridget/public, service ou fichier n'est ajouté.

Le nom `/t3-bridget-policy` est réservé au chemin interne de mise à jour. L'adaptateur refuse son invocation depuis un prompt ou une slash command utilisateur avant tout envoi à Pi. Le filtre du catalogue seul ne suffit pas : un appel direct pourrait changer l'extension derrière le cache `lastAcknowledgedDelegationPolicy`. Les autres slash commands natives restent inchangées. Le principal ajoute l'oracle négatif dans le banc existant.

Les références détaillées figurent dans l'audit de réutilisation : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/reuse-audit.md. Les variantes qui partagent ces compositeurs sont couvertes par leurs chemins effectifs, sans annonce de montage supplémentaire.

### 5. Garder le moteur et son suivi existants

Ne pas modifier le daemon Bridget, les contrats168 ou la projection149. Réutiliser /Users/moi/11.Repositories/t3code-local/apps/server/src/bridget/BridgetLineage.ts:101 et sa régression149. Un changement de mode n'agit que sur les nouveaux tours et délégations. Une tâche admise garde son moteur, ses identifiants, son suivi et son résultat.

### 6. Corriger le prérequis causal de reprise du gros fil

ROOT a constaté sur le fil long l'erreur `champ response limit` : la réponse complète fait33 868 206 octets, au-delà du plafond16Mio. `visibleTurnItems` en représente environ27,5Mo redondants et n'est pas consommé par Bridget. La simulation ROOT qui conserve tous les champs utiles et rend cette seule liste vide donne6 372 704 octets. Cette simulation ne prouve pas encore le correctif installé.

Réutiliser le handler `environment.orchestration.threadSnapshot` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166. Après `requireEnvironmentScope(AuthOrchestrationReadScope)`, utiliser le sujet du principal authentifié retourné. Pour le seul sujet `bridget`, rendre la projection existante avec `visibleTurnItems: []`, tout en conservant intégralement `thread`, `runs`, `nodes`, `messages`, `turnItems` et les autres champs. Les autres sujets restent inchangés. Ne pas prendre cette décision depuis un paramètre client ou un nom de fil. Aucun helper produit, Rust, endpoint, RPC ou contrat public supplémentaire n'est nécessaire.

T012 couvre cette correction avant T011. Réutiliser le banc HTTP réel et son helper `withHttp` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235, confirmés par l'explorer. Les cas exigent sujet Bridget, autre sujet, champs consommateurs intacts et reprise réelle au jalonJ5. Le diagnostic vérifié ROOT est /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md, SHA-256 `25df7f44115ddcb43cb38e89a1ba49214bf1eaafc3292b2c5d3309506334b434`. Cette correction ferme un prérequis de FR017/SC009 ; elle n'ajoute aucun besoin utilisateur.

T012 peut être codée en parallèle par un second worker propriétaire des seuls fichiers HTTP et de leur banc. T001–T010 restent séquentielles chez le principal. Le second ne lance aucun contrôle ; le principal garde l'unique permis de tests/types/lint lourds. T011 attend toujours T010 et T012. Couverture et dépendances inchangées.

## Ordre de réalisation et groupes de preuves

Après contre-revue et audit de réutilisation PASS, générer les tâches d'implémentation. Ce plan ne les génère pas.

| Groupe | Changement et preuve utile | Critères |
|---|---|---|
| G1 — Réglages et continuité | Ajouter le booléen lu par fichier et le cache réglages/confiance ; tester absence, OFF, ON, JSON invalide et I/O. Refuser les mises à jour non fiables avant secrets/écriture, fichier byte-identique. Actualiser le mode avant chaque tour et prouver la stabilité d'identité. | SC001, SC002, SC004, SC006 |
| G2 — Composition et raccords | Exercer les six raccords, changements de mode, contextes et exceptions. Vérifier le champ par tour vers legacy et sans MCP. Prouver Claude repris sans compaction, avec attente sûre du travail de fond. Pour Pi, exiger reçu privé corrélé + ACK avant tout premier vrai prompt ; tester erreur avalée, ancien reçu et reçu absent. Retirer les six injonctions T3 contradictoires en gardant les outils non concernés. | SC001, SC005, SC007 |
| G3 — Parcours natif et suivi | Réutiliser les bancs MCP et Lineage149. Vérifier contraintes/refus, rejeu sans doublon, suivi des moteurs initiaux et absence de dispatch de secours. T012 vérifie aussi la réponse complète du gros fil : branche authentifiée Bridget seule, champs consommateurs intacts, autres sujets inchangés. | SC003, SC004, SC005, SC006, SC009 |
| G4 — Livraison activée | Après validations, intégrer sur le fork, construire, signer, sauvegarder, installer et activer la cible. Prouver le code chargé, ON, les nouveaux contextes, une reprise et une sous-tâche corrélée depuis un raccord existant vérifié. | SC003, SC008, SC009 |

Bancs ciblés existants : réglages serveur, instructions partagées, instructions développeur Codex, driver MCP Codex, adaptateur Claude, gestionnaire de sessions et Lineage149. Ajouter seulement les cas manquants dans ces groupes fonctionnels. Aucun test de simple câblage ni fichier de test par fichier de code n'est demandé.

Le binaire Pi est hors PATH dans l'environnement constaté. Le contrat privé Pi est donc vérifié par tests/fixtures ; aucun parcours Pi réel n'est attesté et aucun fournisseur supplémentaire ne sera installé pour ce lot.

Les vérifications T3 suivent explicitement /Users/moi/11.Repositories/t3code-local/AGENTS.md : `vp test run` ciblé, lint ciblé et types des paquets affectés. Les contrôles de tout le dépôt sont interdits sans demande explicite. Aucun FULL ni contrôle global répété n'est prévu. Conserver les résultats exacts et les limites héritées ; ne pas en déduire un succès global.

## Installation, activation et retour arrière

ROOT exécute la livraison. Avant remplacement, relever les PID, dates de naissance, chemins chargés, versions et empreintes réels. Vérifier l'état durable des conversations et tâches. Préparer la sauvegarde de l'application installée et du fichier de réglages réellement résolu. Ne pas utiliser un ancien checkpoint comme mesure courante. Ne pas restaurer ni effacer une base de conversations pour faire une recette.

Construire le paquet par le circuit T3 existant, version `0.0.45-local.169`, depuis la source intégrée identifiée. Vérifier sa signature et ses empreintes. Installer le paquet. Ajouter `preferBridgetDelegation: true` dans le fichier de réglages existant de la cible, en conservant les autres valeurs. Le reçu contient l'état et les empreintes utiles, sans secrets.

Activer T3 par la procédure normale et contrôlée de l'application. L'utilisateur a déjà donné son accord. Capturer les identités des processus avant et après. Ne pas utiliser `launchctl submit`, ni créer un service d'activation ou une relance persistante : l'incident166 a montré leur risque de boucle. Toute terminaison nécessaire vise un PID identifié et respecte les protections du projet.

Après activation, vérifier le code effectivement chargé, la lecture ON et les nouvelles consignes. Reprendre une conversation existante par son cycle normal. Vérifier ses références, son identité courante et le suivi des tâches des deux moteurs. La recette de sous-tâche ne lance qu'un raccord et une identité existants, déclarés et vérifiés. Une capacité manquante se rapporte comme telle.

Si l'activation échoue, appliquer le retour arrière préparé de l'application et du réglage, puis vérifier la reprise. Conserver les bases, journaux et travaux vivants. Une correction ne supprime aucune preuve antérieure. L'installation seule ne clôt pas le jalon d'activation.

## Jalons du pipeline et statuts

Les14 tâches d'implémentation portent le code et ses preuves, dont T013/T014 ajoutées après R1. T011 dépend de T010 et T012 ; T013/T014 partent du gel T011 et précèdent le passage2. Les identifiants restent stables. Elles ne contiennent pas une condition circulaire exigeant leur propre futur commit. Les jalons distincts suivent la revue, l'intégration, le paquet, l'installation, l'activation, les reçus et le nettoyage. `Implemented` exige100% des tâches réellement terminées. Le pipeline n'est `Livré` qu'après les preuves de livraison et d'activation, dont la reprise du gros fil.

L'estimation initiale de 75–145 minutes publiée à 19:15 CEST précédait l'autorisation d'activation. ROOT publiera une estimation révisée après le découpage des tâches. Aucun délai futur n'est annoncé comme réalisé.

## Constitution, minimalisme et responsabilité future

| Gate | Décision de conception |
|---|---|
| SpecKit et isolation | Branche/worktree169 ; spec, plan et contrats avant tâches. Scripts et templates absents : protocole lu, primitives manuelles, sans réparer les outils globaux hors périmètre. |
| Réutilisation XIX | Un champ dans les réglages, les services et compositeurs existants. Aucun doublon de politique ou d'orchestrateur. Audit item par item avant tasks. |
| Complexité XVIII | Résolution du mode en O(1). Aucun parcours de catalogue, lecture par événement, polling ou RPC ajouté. Les recompositions restent de la taille bornée des consignes existantes. |
| Vertus XX | Un contrat explicite sur la confiance de lecture et la stabilité de session. Le mainteneur peut modifier le texte partagé, observer le mode et vérifier les tests ciblés sans contexte caché. |
| Charge cognitive | Le booléen est public ; la confiance et le mode sont internes. Les cas bloqués ont une raison. Pas d'état supplémentaire à configurer ni de promesse d'obéissance absolue. |
| Tests et preuves | Groupes fonctionnels ciblés conformément au dépôt T3. Aucun changement de rendu ; Lineage conserve son banc existant. Les preuves d'activation restent distinctes des tests de consignes. |
| Git et exploitation | Auteur humain seul, remote fork T3, aucune PR non demandée. Activation autorisée et retour arrière prêt ; aucune boucle de relance. |

Potentiel minimalisme : 0 ligne de fonctionnalité nouvelle identifiée comme supprimable dans le plan. Aucun volume de code n'est encore mesuré. L'implémentation devra relire son diff pour retirer les branches, textes et états redondants.

La frontière durable est documentée une seule fois dans le contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/contracts/delegation-preference.md. L'ADR169 y renvoie. Le contexte par tour ferme le transport sans MCP. Aucune inconnue de conception bloquante ne reste identifiée ; l'exécution des gates reste à prouver.

## Arbitrages documentaires après contre-revue R2

ROOT ne confirme pas C1 dans le SDK0.3.276 installé. La preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md explique l'absence de sessionStore T3, le miroir inactif, le lecteur libéré et la course d'attente de sortie de2 s. close réutilise exactement cleanupPromise : le timeout+close proposé ne force rien et pourrait rouvrir trop tôt. Pas de correctif ni faux oracle return infini ; aucune preuve absolue d'absence d'un processus figé.

C2 est conservé : absence/OFF garde la politique habituelle avec enveloppe de reset sans MCP, même au premier tour inconnu. Codex reçoit t3_code_orchestration et t3_code_runtime ; ordinal1 n'exclut pas import/reprise. Seuls6 986 octets OFF avec MCP sont byte-identiques. Contexte identique dédupliqué, coût restant non mesuré ;87 transcriptions adaptées, pas87 bugs produit. Pas de branche/cache supplémentaire ni de passage3.

Cache Claude blocked jusqu'à réouverture/changement de clé ou processus, message UI et reprise existante positive non prouvés restent explicites. Pi absent et75 erreurs TS non GREEN sont conservés. P2 et J1 restent clos ; J2 est partiel et FR017/SC003 réel/SC009/J3–J6 restent ouverts.

## Checkpoint historique après clôture J1

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.

## Bilan opérationnel

Une compilation du paquet et une activation seulement. Les tests source ne sont pas relancés après les seules mises à jour documentaires. Retraits et conservations du lot169 sont consignés dans /Users/moi/.cache/bridget-delivery169/final169.json ; aucun nettoyage étranger ni suppression de base.
