# Recherche native149 — permissions héritées et projection de descendance

Date : 2026-10-10. Statut : investigation et contrat proposé. Aucun code modifié.
Responsable : owner natif. Git, plan commun et livraison restent au principal.
Tous les tests et toutes les relectures sont confiés à GLM 5.3 Flash.
Aucun modèle lancé, redémarrage, fichier de production ou base de production modifié.

## Résultat recherché

Un appel Bridget crée un véritable enfant. Sans posture explicite, l'enfant hérite
de la politique effective du parent. La lecture seule reste un choix explicite.
Un parent autorisé à écrire n'a pas besoin d'un nouveau grant humain Bridget.
Les enfants Claude/GLM peuvent écrire. Bridget ne réduit pas silencieusement un
parent full-access au profil Codex148 sans réseau.
Le moteur reste natif et fonctionne sans T3. T3 conserve une projection de lecture.

## Faits observés dans le code

Racine de tous les chemins ci-dessous : aucune abréviation de chemin n'est requise.

| Fait | Source précise |
|---|---|
| Le parent externe n'obtient que `SameProject`, dont le maximum vaut `Discovery`. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:8`, `:57`, `:101` |
| L'héritage managed écrit seulement si plusieurs arguments Codex exacts prouvent workspace-write, sans réseau, sans racines supplémentaires et sans tmp. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:77` |
| Un parent externe qui demande development reçoit `development_grant_required`. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:208` |
| Le registre réserve development à `codex_app_server`. Discovery Claude impose `--restricted --permission-mode plan`. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/registry.rs:192`, `:216`, `:1288` |
| `permissions=allow` du wrapper ajoute un bypass global. Cette ancienne valeur ne doit pas être la nouvelle politique d'héritage. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/wrapper.rs:4787` |
| `bridget_session`148 atteste l'identité, pas les droits. Le parseur ferme les champs inconnus. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code_mcp.rs:12`, `:179` |
| L'endpoint T3 est limité au runtime local connu. Redirections interdites, taille64KiB, timeout3s, jeton par session de transport et fermeture après attestation. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code_mcp.rs:64`, `:85` |
| La demande MCP impose aujourd'hui posture. L'identité du parent n'est jamais un argument de mission. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_mcp.rs:9`, `:55` ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/protocol.rs:2001` |
| La tâche et la définition fournisseur sont durables avant lancement. La clé est unique par owner stable et request_id. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation.rs:12`, `:48`, `:85`, `:101` |
| Le lancement réutilise FleetSpawnOrder, SpawnOwnership, submit_spawn et la définition figée. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:442` |
| La lecture status reste pure. La réponse est capturée avant ACK; les descendants actifs retardent sa publication. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:414`, `:630`, `:875` |
| Une garde147 commune vérifie le binding T3 vivant, l'identité et le projet. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon.rs:13642`, `:13671` |
| Des événements de liens et des journaux natifs existent déjà. Ils ne décrivent pas toute la saga ni ses états terminaux après fermeture du lien. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/protocol.rs:2032`, `:3038`, `:3118` |
| Le pilote Claude natif sait traiter l'accusé d'interruption. Il ne traite pas les demandes `can_use_tool`. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/claude_stream_json.rs:1049` |
| Le pilote Codex lit déjà `config/read` avant reprise pour éviter que les anciens droits du fil remplacent ceux de la définition. Cette même source peut attester les droits standalone. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/codex_app_server.rs:489` |

La mémoire projet n'a pas été trouvée sous `/Users/moi/.Codex/projects`.
Les constitutions globale et locale, la référence d'orchestration et les contrats148
ont été lus. Aucun mécanisme nouveau de sandbox n'est justifié par ce besoin.

## Preuve locale du fournisseur Claude

Lecture seule de `/Users/moi/.local/bin/claude --version` : **2.1.296**.
Lecture seule de `/Users/moi/.local/bin/claude --help` :

- `--permission-mode` admet acceptEdits, auto, bypassPermissions, manual, dontAsk et plan.
- `--permission-prompts none`, avec print, refuse automatiquement les actions qui
  auraient demandé une permission. Le mode de permissions continue de décider du reste.
- `--restricted` retire les outils d'exécution et WebFetch, confine les outils
  fichiers aux répertoires de travail et refuse bypassPermissions. Ce n'est pas
  la preuve d'un confinement OS général.

Cette lecture n'est ni un test, ni une recette réelle. Elle ne prouve pas que la
configuration d'un fournisseur alternatif possède toutes les mêmes capacités.
L'admission doit connaître la capacité réellement supportée par sa cible.

## Source active du registre — correction G-P-05

Lecture seule le2026-10-10 : launchctl print gui/501/com.bridget.daemon retourne
state=running, pid=58394 et program=
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
Les seuls champs d'environnement relevés sont BRIDGET_HOME=
`/Users/moi/.cache/bridget-core`, BRIDGET_SOCKET=
`/Users/moi/.cache/bridget-core/bridget.sock` et HOME=`/Users/moi`.
La lecture ps de ce PID confirme le programme et PPID1. Aucun environnement
complet, credential ou clé n'a été affiché. Aucun daemon n'a été relancé.

Le chemin de registre vient de Namespace::from_environment, puis root/agents.json :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/environment.rs:24`
et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/registry.rs:913`.
Le chargement initial du daemon se trouve dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon.rs:3067`.
La racine runtime attestée sélectionne donc
`/Users/moi/.cache/bridget-core/agents.json`. Ce n'est pas une déduction depuis
le seul plist de lancement. Le contenu courant du fichier est distingué du
snapshot éventuellement déjà chargé par un processus ancien.

| Source lue | Sélection et contenu courant observé |
|---|---|
| `/Users/moi/.cache/bridget-core/agents.json` | Sélectionnée par la racine du daemon actif. Cibles explicites glm et codex-pro. Glm : claude_stream_json, commande `/Users/moi/.local/bin/gclaude`, profil `/Users/moi/.claude-glm`, modèle déclaré glm-5.3. Codex-pro : codex_app_server. |
| `/Users/moi/.config/bridget/agents.json` | Non sélectionnée par ce daemon. Cibles explicites cursor (cursor-agent/acp) et claude (`/Users/moi/.local/bin/claude`/claude_stream_json). Cette source explique le constat différent du reviewer. |

Empreintes des fichiers lus, sans contenu brut : registre actif
c8642f4c08a5b311dde2fa5881c83c42eeb5bcfabc47eb5325a7345e463df539;
autre registre6fee943e850817ee01138743f9c9af058ba9adf680b00316f596d676fcd33d5d.
Ces constats n'impliquent aucune modification de production. La recette GLM5.3Flash
emploie un registre privé isolé avec modèle exact. Elle conserve les références
launcher/profil effectivement attestées, sans substitution du modèle de production.

Le launcher `/Users/moi/.local/bin/gclaude` a pour SHA256
dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e.
Le chemin `/Users/moi/.local/bin/claude` résout actuellement vers
`/Users/moi/.local/share/claude/versions/2.1.296`, de SHA256
c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937.
Cette lecture de fichiers ne prouve pas à elle seule le PATH effectif d'un tour
T3 ou PTY. L'owner du lancement doit résoudre sous son environnement final,
sans cache, et attester le couple résolu. Le contrat distingue cli_path/revision
du launcher et resolved_cli_path/revision du binaire. Les deux couples et les
sources settings sont figés puis recontrôlés avant effet. Aucun secret ni PATH
complet n'est publié. Une chaîne de lancement non observable produit seulement
permission_source_unavailable pour ce contexte; elle ne nie pas la cible glm
du registre réellement sélectionné.

## Contrat de provenance convenu avec l'owner T3

T3 publie `bridget_session` version2 sous le credential MCP propre au fil.
La réponse conserve les identifiants148 et ajoute `permissions`, version1 :

```json
{
  "source": "provider_turn",
  "run_id": "identifiant du tour attesté",
  "provider_session_id": "identifiant effectif",
  "provider_instance_id": "instance effective",
  "driver": "codex_app_server ou claude_stream_json",
  "cwd": "répertoire effectif",
  "interaction_mode": "valeur effective",
  "runtime_mode": "valeur effective",
  "provider_policy": { "kind": "codex ou claude" }
}
```

Pour Codex, `provider_policy` porte `approval_policy`, `approvals_reviewer` et
`sandbox_policy` réellement envoyés au fournisseur. Le sandbox distingue read-only,
workspace-write avec racines/réseau effectifs, danger-full-access et éventuel
external-sandbox. Une valeur inconnue n'est jamais assimilée à full-access.

Pour Claude, il porte `permission_mode`, `tools`, `allowed_tools`, `disallowed_tools`,
`permission_callback` et `sandbox_settings` seulement lorsqu'ils existent réellement.
Le mode mutable du fournisseur, dont EnterPlanMode, prime sur le seul choix UI.
L'owner T3 a identifié les paramètres effectifs de démarrage de tour Codex et les
queryOptions Claude. Le contrat final doit couvrir la portée et la fraîcheur de ces faits.

L'utilisateur de l'outil ne fournit ni permissions, ni preuve, ni parent.
L'adaptateur privé transporte seulement l'endpoint runtime et un credential opaque.
Le daemon réatteste auprès de T3 **hors verrou**, avec les gardes HTTP148.
Il recontrôle ensuite le binding vivant agent/instance/fil/projet et la session
fournisseur avant admission. Un changement ou une preuve périmée refuse l'opération.
Le credential n'entre jamais dans la tâche, la définition, le prompt, les arguments,
l'environnement enfant ou le journal. Son affichage Debug doit rester masqué.

Le raccord T3 atteste; il n'exécute pas. Un descendant natif utilise le snapshot
de permissions de sa propre tâche et sa présence managed prouvée. Il n'a pas besoin de T3.

## Héritage et admission natifs proposés

1. Résoudre l'identité actuelle comme148. Refuser toujours les révocations explicites.
2. Déduire la politique depuis une attestation T3 fraîche, une définition native
   figée déjà admise ou le transport propriétaire standalone. Ne pas lire un
   fichier arbitraire d'agent comme source d'autorité.
3. Sans posture explicite, hériter. Avec discovery, réduire à lecture seule.
   Avec development, conserver les droits d'écriture attestés. Refuser une
   élévation depuis un parent effectivement lecteur avec un code de droits,
   sans demander un grant humain Bridget supplémentaire.
4. Vérifier le cwd canonique et l'appartenance au projet du parent. Cette frontière
   logique de mission ne vaut pas une limitation OS. Un sandbox Codex et les
   permissions d'outils Claude doivent être annoncés selon ce qu'ils appliquent vraiment.
5. Figer la politique normalisée, sa provenance, le modèle exact et la définition
   effective avant le premier effet. Le retry retourne la tâche initiale.
6. Le wrapper applique la politique figée, sans transformer `permissions=allow`
   en `--yolo` ni construire un autre sandbox global. Full-access explicite peut
   se traduire en paramètres sandbox/permission fournisseur full-access. Le même
   réglage n'est jamais appliqué aux autres agents du registre.

Les grants148 peuvent rester pour compatibilité et contrôle humain. Ils ne sont
plus une étape du chemin d'héritage normal. Les révocations durables restent prioritaires.
La reprise d'owner après changement d'instance peut utiliser une nouvelle attestation
exacte de la même identité, après départ de la présence primaire précédente.

### Claude/GLM sans demande humaine supplémentaire

Hériter du mode exact et des règles effectives. Autoriser les outils Bridget déjà
nécessaires au contrat. Ne pas imposer plan ou restricted à un parent qui peut écrire.
En mode full-access attesté, appliquer ce mode explicitement dans la seule définition enfant.
En modes acceptEdits/auto/manuels, conserver la politique et régler les prompts
non interactifs sur none si la cible le supporte. Une action hors politique est
refusée, sans nouveau dialogue Bridget. Une demande `can_use_tool` qui arrive malgré
cette politique doit être refusée automatiquement sous sa corrélation, jamais ignorée.
Le résultat `permission_denials` ou une permission non couverte doit produire une
issue explicite. Une fin de tour ne doit pas masquer le refus sous un faux succès.

Le mappage inter-fournisseurs doit être documenté. Une politique inconnue ou un
confinement exigé sans équivalent disponible produit un refus nommé. Il ne faut
pas prétendre reproduire un sandbox OS Codex avec des règles d'outils Claude.

## Projection durable native vers Lineage

La tâche ajoute `parent_task_id` stable, dérivé de l'enfant managed prouvé. Les
champs owner/instance restent la source d'autorité. Les descendants sont reliés
par tâche, même après nettoyage des liens Fleet. Une reprise ne recrée aucun enfant.

Contrat proposé avec T3 : `version`, `generation`, `seq`, root owner dérivé du
binding, snapshot atomique et pages bornées à100. Chaque tâche porte :

- task_id, parent_task_id, parent_agent_id, child_agent_id, child_instance_id;
- request_id ou titre court, agent_type, execution_protocol, model, effort, cwd;
- posture effective, status, created_at, updated_at, started_at et completed_at;
- result/error seulement selon le contrat de détail autorisé;
- référence de journal enfant, jamais un chemin RPC libre.

La vue de liste évite les corps de mission/résultat. Une lecture de détail bornée
porte le contenu demandé. Les journaux réutilisent la lecture native et l'attach
relay existants. L'autorité est vérifiée pour la tâche et sa descendance avant
d'ouvrir un journal. La forme publique exacte reste à verrouiller avec le plan.

Les mutations effectives mettent à jour une séquence durable dans la même
transaction que la tâche. Le suivi ne contient que generation/seq et statut de
continuité. Il réveille une lecture de snapshot bornée; il ne poll pas les corps.
Un signal n'est émis qu'après commit. Replay/read/ACK/refus n'invalident pas la vue.
La perte de continuité exige un nouveau snapshot. Pas de journal métier parallèle.

Les lectures et annulations humaines passent par `--t3-thread` et projet. Le
daemon réutilise les gardes147. L'UI ne choisit jamais un agent root arbitraire.
L'annulation appartient toujours à la saga native; elle ne dispatch pas un tour T3.

T3 persiste des fils virtuels descendants, origin `bridget_native`, liés à son
root courant. Il ne crée aucune session fournisseur, aucun effet de lancement,
aucun tour à exécuter. Les identifiants déterministes rendent la projection
idempotente. Le journal natif reste la source des détails. Le résultat parent
conserve sa remise corrélée unique148.

Le connecteur Bridget inventorie aussi les fils T3. Les enfants virtuels doivent
porter un `bridgetTaskRef` facultatif, validé par le contrat T3 puis exclu du
montage des wrappers et des présences. Le seul préfixe d'identifiant ne suffit
pas comme attestation. Cette garde évite une boucle de projection et de faux agents.
Source du montage :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code.rs:1285`.

La requête native utilise un index parent/root et une pagination stable. Objectif
O(log N + P), P<=100, profondeur<=8 et N<=4096. Éviter une requête par descendant
ou une recherche linéaire répétée dans la liste Fleet.

## Périmètre proposé et ordre d'implémentation

1. Types de permission/provenance, attestation privée et admission native héritée.
2. Construction fournisseur locale par mission et application wrapper/transport.
3. Parent_task_id, données de projection, séquence commit et lectures bornées.
4. Vue/suivi/annulation humains, lecture du journal descendant sous la garde147.
5. Raccord Bridget/T3 de lecture et projection, coordonné avec l'owner T3.

Fichiers pressentis : protocol.rs, delegation.rs, delegation_mcp.rs,
daemon/native_delegation.rs, registry.rs, wrapper.rs, claude_stream_json.rs,
codex_app_server.rs, t3code_mcp.rs, daemon.rs, cli.rs et connecteur t3code.
Les modifications natives sont réservées à cet owner. Les tests, fixtures et
relectures appartiennent à GLM. Aucun code T3 ou document principal n'est modifié ici.

Estimation native après contrat commun stable : 3 à5 heures d'implémentation;
1 à2 heures de raccord avec T3; validation et relecture en plus, par GLM.
Le plus grand risque de délai est l'attestation standalone interactive et le
mappage inter-fournisseurs. Ne pas le masquer par un grant ni par un profil discovery.

## Sources officielles consultées par le principal

La baseline d'architecture a été consultée dans
`/Users/moi/.speckit/research/04-architectures-patterns.md`.
La baseline agentique a été consultée dans
`/Users/moi/.speckit/research/01-ai-agents-agentic-ai.md`, par l'owner natif.

- [Permissions du SDK Claude](https://code.claude.com/docs/en/agent-sdk/permissions)
  : modes et décisions d'outils; ne pas les confondre avec un confinement OS.
- [Sécurité MCP](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices)
  : préserver les frontières de credential et l'identité attestée.

Le principal a fourni ces références live. Le code local et l'aide locale du
fournisseur sont les preuves précises des comportements listés ci-dessus.

## Points à verrouiller avant GO de code

- Schéma final version2 de bridget_session et fraîcheur du tour attesté.
- Mappage Codex/Claude, surtout auto/workspace-write et exigences de réseau.
- Source attestée standalone interactive pour chaque fournisseur réellement supporté.
- Voie Claude SDK/PTY→CLI de même famille : mêmes inputs permissions/profil/cwd
  attestés, contrôlés par revision au lancement/reprise; aucun merge effectif
  inventé. Refus spécifiques seulement pour sources opaques ou non représentables.
- Transport exact snapshot/watch/détail/journal, bornes et persistance du snapshot.
- Propriétaire du connecteur partagé Bridget/T3 et changements de capacités CLI.

Ce document ne constitue pas une preuve de fonctionnement. Aucun test ni revue
n'a été exécuté par cet owner. Le code attend le GO du principal sur le plan commun.
