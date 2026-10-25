# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/plan.md](../../../../../../specs/169-priorite-bridget/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Ajouter `preferBridgetDelegation`, un booléen à `false` par défaut, dans les réglages T3 existants. Avant chaque tour, les consignes prennent l'un des états `off`, `prefer` ou `blocked`. Elles privilégient la délégation automatique Bridget si le réglage est actif et si Bridget est effectivement monté. Elles annoncent le blocage dans les autres cas actifs. OFF conserve le parcours T3 habituel.

<a id="extrait-2"></a>

## Extrait 2

Le moteur, le routage168, les droits et Lineage restent existants. Aucun Rust, RPC ou protocole Bridget/public, service, stockage distinct, interface visuelle ou algorithme de flotte n'est ajouté. Une correction causale de la réponse HTTP existante pour le sujet authentifié Bridget permet la reprise du gros fil ; elle est décrite ci-dessous. La livraison comprend un paquet T3 signé, son installation et son activation contrôlée, déjà autorisées par l'utilisateur. Elle ne nécessite pas une nouvelle confirmation.

<a id="extrait-3"></a>

## Extrait 3

Étendre seulement `ServerSettings` dans /Users/moi/11.Repositories/t3code-local/packages/contracts/src/settings.ts. Ne pas ajouter le champ à `ServerSettingsPatch` : l'utilisateur a demandé un réglage par fichier, sans mutation publique nouvelle. L'absence du champ conserve `false`. Un patch d'un autre réglage préserve la valeur lue. Le fichier absent dans une installation sans réglages conserve aussi les valeurs normales de compatibilité.

<a id="extrait-4"></a>

## Extrait 4

Réutiliser `ServerSettingsService` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:256. Son chargeur possède déjà `settingsFileTrusted` à la ligne772. Conserver en cache le tuple cohérent réglages/confiance et exposer ce fait à la lecture interne des consignes. Ne pas ajouter de clé JSON sentinelle. Une erreur ne devient jamais un OFF valide, même après un ancien OFF. Les autres consommateurs gardent leur contrat existant.

<a id="extrait-5"></a>

## Extrait 5

Dans /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:1250, `updateAndPersistSettings` peut repartir des defaults après une lecture non fiable. Refuser alors toute mise à jour par un `ServerSettingsError` nommé, avant traitement de secret ou écriture. Le fichier invalide reste byte-identique. La confiance revient après réparation valide relue, ou après une écriture valide depuis un état déjà fiable ; elle ne se rétablit pas en écrasant le fichier invalide.

<a id="extrait-6"></a>

## Extrait 6

La lecture est actualisée avant chaque `runtime.startTurn` par le gestionnaire existant : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823. La préparation MCP seule ne suffit pas pour un fil déjà attaché. Cette lecture n'ajoute ni sonde réseau de santé ni décision de flotte.

<a id="extrait-7"></a>

## Extrait 7

Le mode est une métadonnée de consigne, calculée avec le fait de montage Bridget réel. Un endpoint MCP T3 ou le nom d'un fournisseur ne prouve pas ce montage.

<a id="extrait-8"></a>

## Extrait 8

Réutiliser `ProviderAdapterV2TurnInput` dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396. Ajouter son champ optionnel interne `bridgetDelegationPreference?: "off" | "prefer" | "blocked"`. Le gestionnaire décore l'input à chaque `runtime.startTurn`, dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823, avec la préférence et la confiance de lecture actuelles. Les compositeurs y associent le montage réel. Vérifier la propagation dans la compatibilité vers les drivers legacy.

<a id="extrait-9"></a>

## Extrait 9

Ce porteur existe même quand `prepareMcpSession` retourne `undefined`. Les sorties anticipées à /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:471 et le fil déjà attaché à la ligne1311 ne doivent pas faire perdre ON. Un raccord sans MCP reçoit donc `blocked`, sans faux credential. Aucune map ni mutation de configuration MCP n'est nécessaire.

<a id="extrait-10"></a>

## Extrait 10

Conserver la même référence de configuration de session, lorsqu'elle existe, dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts. Remplacer cet objet efface les faits de permissions à la ligne153. Une modification de préférence ne change ni credential, ni objet de session, ni rotation ou remontage MCP.

<a id="extrait-11"></a>

## Extrait 11

Étendre les compositeurs existants dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:3, :63, :72, :105 et :115. Extraire seulement les paragraphes de délégation qui doivent varier. Ne pas ajouter une consigne ON après un texte qui prescrit encore une délégation native T3. Éviter les remplacements fragiles de sous-chaînes et la copie intégrale du prompt par fournisseur.

<a id="extrait-12"></a>

## Extrait 12

| Situation | État effectif | Consigne de délégation |
|---|---|---|
| Lecture fiable, clé absente ou `false` | `off` | Consigne T3 habituelle. |
| Lecture fiable, `true`, Bridget réellement monté | `prefer` | Utiliser la délégation automatique Bridget ; conserver les contraintes explicites. |
| Lecture non fiable | `blocked` | Signaler le réglage indisponible ; ne pas prétendre avoir lu OFF. |
| Lecture fiable, `true`, montage Bridget absent ou désactivé | `blocked` | Signaler Bridget indisponible ; ne pas inventer l'outil ni prescrire un repli silencieux. |

<a id="extrait-13"></a>

## Extrait 13

`prefer` et `blocked` conservent l'exception d'un fil T3 expressément demandé. Les outils T3 hors délégation gardent leurs consignes. Inventorier et corriger les six injonctions de délégation T3 contradictoires, y compris les rondes de revue, la délégation depuis le scheduler et le repli terminal ACP. Elles restent dans OFF lorsqu'elles sont pertinentes. L'hôte n'ajoute aucun dispatch après une erreur Bridget.

<a id="extrait-14"></a>

## Extrait 14

Codex et Claude montent déjà Bridget par /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/CodexMcp.ts:29 et /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/ClaudeMcp.ts:186. Réutiliser ces montages sans activer un outil désactivé.

<a id="extrait-15"></a>

## Extrait 15

- **Codex** : conserver le chemin `additionalContext` et les instructions développeur dans /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/CodexDeveloperInstructions.ts:197 et /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:772.
- **Claude** : mettre à jour l'append SDK et la clé de contexte dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:1443 et :2150. Le SDK peut conserver l'ancien append jusqu'à compaction. Utiliser son option réelle `snapshot: false` pour l'append T3 dynamique ; la clé seule ne suffit pas. Vérifier l'option et les deux sens ON/OFF après reprise sans compaction. Respecter le refus existant de remplacer une query avec travail de fond actif : ne pas le tuer, ne pas accepter un tour au mode périmé, signaler l'attente puis appliquer le mode au prochain tour après sa fin. Conserver preset, modèle, droits et textes hors délégation. L'invalidation possible du cache/contexte de réflexion (`thinking`) est un compromis nécessaire et non chiffré ; des options transmises ne prouvent pas encore un résultat réel du modèle.
- **ACP, OpenCode, Cursor et Pi** : sans montage Bridget existant, ON donne `blocked`. Étendre leurs chemins actuels d'injection. La comparaison d'état ACP inclut la préférence ; les deux variantes OpenCode sont couvertes. Cursor ne peut pas conserver un premier contexte OFF après passage ON. Les commandes natives commençant par `/` gardent leur sémantique ; un contexte non injecté ne doit pas être marqué comme injecté.

<a id="extrait-16"></a>

## Extrait 16

**Exception Pi bornée, arbitrée ROOT** : réutiliser l'extension existante dans /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts:364. Son état initial est bloqué/inconnu. Envoyer `/t3-bridget-policy` avec l'enum `off/prefer/blocked` et un identifiant court neuf validé, par la requête `prompt` Pi existante avant le premier vrai prompt, même OFF, puis lors des changements. Vérifier sa présence dans `get_commands` de ce processus ; sinon elle pourrait partir au modèle. Le handler synchrone valide avant mutation, puis émet APRÈS affectation un reçu privé fermé par le canal existant `ctx.ui.setStatus`. L'adaptateur intercepte et masque ce reçu. Avancer `lastAcknowledgedDelegationPolicy` seulement après reçu du mode corrélé à cette tentative ET réponse de requête corrélée `success: true`. Le SDK peut avaler les erreurs du handler et renvoyer success : l'ACK seul ne prouve donc pas l'application. Ignorer les reçus anciens/étrangers ; sur erreur, reçu manquant ou ACK invalide, aucun vrai prompt ni avancement. Filtrer la commande du catalogue utilisateur, sans modifier ses slash commands. Ce contrat privé évite un redémarrage Pi par changement ; aucun RPC Bridget/public, service ou fichier n'est ajouté.

<a id="extrait-17"></a>

## Extrait 17

Le nom `/t3-bridget-policy` est réservé au chemin interne de mise à jour. L'adaptateur refuse son invocation depuis un prompt ou une slash command utilisateur avant tout envoi à Pi. Le filtre du catalogue seul ne suffit pas : un appel direct pourrait changer l'extension derrière le cache `lastAcknowledgedDelegationPolicy`. Les autres slash commands natives restent inchangées. Le principal ajoute l'oracle négatif dans le banc existant.

<a id="extrait-18"></a>

## Extrait 18

Ne pas modifier le daemon Bridget, les contrats168 ou la projection149. Réutiliser /Users/moi/11.Repositories/t3code-local/apps/server/src/bridget/BridgetLineage.ts:101 et sa régression149. Un changement de mode n'agit que sur les nouveaux tours et délégations. Une tâche admise garde son moteur, ses identifiants, son suivi et son résultat.

<a id="extrait-19"></a>

## Extrait 19

Réutiliser le handler `environment.orchestration.threadSnapshot` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166. Après `requireEnvironmentScope(AuthOrchestrationReadScope)`, utiliser le sujet du principal authentifié retourné. Pour le seul sujet `bridget`, rendre la projection existante avec `visibleTurnItems: []`, tout en conservant intégralement `thread`, `runs`, `nodes`, `messages`, `turnItems` et les autres champs. Les autres sujets restent inchangés. Ne pas prendre cette décision depuis un paramètre client ou un nom de fil. Aucun helper produit, Rust, endpoint, RPC ou contrat public supplémentaire n'est nécessaire.

<a id="extrait-20"></a>

## Extrait 20

T012 couvre cette correction avant T011. Réutiliser le banc HTTP réel et son helper `withHttp` dans /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235, confirmés par l'explorer. Les cas exigent sujet Bridget, autre sujet, champs consommateurs intacts et reprise réelle au jalonJ5. Le diagnostic vérifié ROOT est /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md, SHA-256 `25df7f44115ddcb43cb38e89a1ba49214bf1eaafc3292b2c5d3309506334b434`. Cette correction ferme un prérequis de FR017/SC009 ; elle n'ajoute aucun besoin utilisateur.

<a id="extrait-21"></a>

## Extrait 21

| Groupe | Changement et preuve utile | Critères |
|---|---|---|
| G1 — Réglages et continuité | Ajouter le booléen lu par fichier et le cache réglages/confiance ; tester absence, OFF, ON, JSON invalide et I/O. Refuser les mises à jour non fiables avant secrets/écriture, fichier byte-identique. Actualiser le mode avant chaque tour et prouver la stabilité d'identité. | SC001, SC002, SC004, SC006 |
| G2 — Composition et raccords | Exercer les six raccords, changements de mode, contextes et exceptions. Vérifier le champ par tour vers legacy et sans MCP. Prouver Claude repris sans compaction, avec attente sûre du travail de fond. Pour Pi, exiger reçu privé corrélé + ACK avant tout premier vrai prompt ; tester erreur avalée, ancien reçu et reçu absent. Retirer les six injonctions T3 contradictoires en gardant les outils non concernés. | SC001, SC005, SC007 |
| G3 — Parcours natif et suivi | Réutiliser les bancs MCP et Lineage149. Vérifier contraintes/refus, rejeu sans doublon, suivi des moteurs initiaux et absence de dispatch de secours. T012 vérifie aussi la réponse complète du gros fil : branche authentifiée Bridget seule, champs consommateurs intacts, autres sujets inchangés. | SC003, SC004, SC005, SC006, SC009 |
| G4 — Livraison activée | Après validations, intégrer sur le fork, construire, signer, sauvegarder, installer et activer la cible. Prouver le code chargé, ON, les nouveaux contextes, une reprise et une sous-tâche corrélée depuis un raccord existant vérifié. | SC003, SC008, SC009 |

<a id="extrait-22"></a>

## Extrait 22

Les vérifications T3 suivent explicitement /Users/moi/11.Repositories/t3code-local/AGENTS.md : `vp test run` ciblé, lint ciblé et types des paquets affectés. Les contrôles de tout le dépôt sont interdits sans demande explicite. Aucun FULL ni contrôle global répété n'est prévu. Conserver les résultats exacts et les limites héritées ; ne pas en déduire un succès global.

<a id="extrait-23"></a>

## Extrait 23

Construire le paquet par le circuit T3 existant, version `0.0.45-local.169`, depuis la source intégrée identifiée. Vérifier sa signature et ses empreintes. Installer le paquet. Ajouter `preferBridgetDelegation: true` dans le fichier de réglages existant de la cible, en conservant les autres valeurs. Le reçu contient l'état et les empreintes utiles, sans secrets.

<a id="extrait-24"></a>

## Extrait 24

Activer T3 par la procédure normale et contrôlée de l'application. L'utilisateur a déjà donné son accord. Capturer les identités des processus avant et après. Ne pas utiliser `launchctl submit`, ni créer un service d'activation ou une relance persistante : l'incident166 a montré leur risque de boucle. Toute terminaison nécessaire vise un PID identifié et respecte les protections du projet.

<a id="extrait-25"></a>

## Extrait 25

| Gate | Décision de conception |
|---|---|
| SpecKit et isolation | Branche/worktree169 ; spec, plan et contrats avant tâches. Scripts et templates absents : protocole lu, primitives manuelles, sans réparer les outils globaux hors périmètre. |
| Réutilisation XIX | Un champ dans les réglages, les services et compositeurs existants. Aucun doublon de politique ou d'orchestrateur. Audit item par item avant tasks. |
| Complexité XVIII | Résolution du mode en O(1). Aucun parcours de catalogue, lecture par événement, polling ou RPC ajouté. Les recompositions restent de la taille bornée des consignes existantes. |
| Vertus XX | Un contrat explicite sur la confiance de lecture et la stabilité de session. Le mainteneur peut modifier le texte partagé, observer le mode et vérifier les tests ciblés sans contexte caché. |
| Charge cognitive | Le booléen est public ; la confiance et le mode sont internes. Les cas bloqués ont une raison. Pas d'état supplémentaire à configurer ni de promesse d'obéissance absolue. |
| Tests et preuves | Groupes fonctionnels ciblés conformément au dépôt T3. Aucun changement de rendu ; Lineage conserve son banc existant. Les preuves d'activation restent distinctes des tests de consignes. |
| Git et exploitation | Auteur humain seul, remote fork T3, aucune PR non demandée. Activation autorisée et retour arrière prêt ; aucune boucle de relance. |

<a id="extrait-26"></a>

## Extrait 26

ROOT ne confirme pas C1 dans le SDK0.3.276 installé. La preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md explique l'absence de sessionStore T3, le miroir inactif, le lecteur libéré et la course d'attente de sortie de2 s. close réutilise exactement cleanupPromise : le timeout+close proposé ne force rien et pourrait rouvrir trop tôt. Pas de correctif ni faux oracle return infini ; aucune preuve absolue d'absence d'un processus figé.

<a id="extrait-27"></a>

## Extrait 27

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
