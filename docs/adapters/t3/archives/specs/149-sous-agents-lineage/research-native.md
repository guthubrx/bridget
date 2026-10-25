# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/research-native.md](../../../../../../specs/149-sous-agents-lineage/research-native.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Un appel Bridget crée un véritable enfant. Sans posture explicite, l'enfant hérite
de la politique effective du parent. La lecture seule reste un choix explicite.
Un parent autorisé à écrire n'a pas besoin d'un nouveau grant humain Bridget.
Les enfants Claude/GLM peuvent écrire. Bridget ne réduit pas silencieusement un
parent full-access au profil Codex148 sans réseau.
Le moteur reste natif et fonctionne par le parcours natif Bridget. T3 conserve une projection de lecture.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

T3 publie `bridget_session` version2 sous le credential MCP propre au fil.
La réponse conserve les identifiants148 et ajoute `permissions`, version1 :

<a id="extrait-5"></a>

## Extrait 5

Pour Claude, il porte `permission_mode`, `tools`, `allowed_tools`, `disallowed_tools`,
`permission_callback` et `sandbox_settings` seulement lorsqu'ils existent réellement.
Le mode mutable du fournisseur, dont EnterPlanMode, prime sur le seul choix UI.
L'owner T3 a identifié les paramètres effectifs de démarrage de tour Codex et les
queryOptions Claude. Le contrat final doit couvrir la portée et la fraîcheur de ces faits.

<a id="extrait-6"></a>

## Extrait 6

L'utilisateur de l'outil ne fournit ni permissions, ni preuve, ni parent.
L'adaptateur privé transporte seulement l'endpoint runtime et un credential opaque.
Le daemon réatteste auprès de T3 **hors verrou**, avec les gardes HTTP148.
Il recontrôle ensuite le binding vivant agent/instance/fil/projet et la session
fournisseur avant admission. Un changement ou une preuve périmée refuse l'opération.
Le credential n'entre jamais dans la tâche, la définition, le prompt, les arguments,
l'environnement enfant ou le journal. Son affichage Debug doit rester masqué.

<a id="extrait-7"></a>

## Extrait 7

Le raccord T3 atteste; il n'exécute pas. Un descendant natif utilise le snapshot
de permissions de sa propre tâche et sa présence managed prouvée. Il n'a pas besoin de T3.

<a id="extrait-8"></a>

## Extrait 8

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

<a id="extrait-9"></a>

## Extrait 9

Contrat proposé avec T3 : `version`, `generation`, `seq`, root owner dérivé du
binding, snapshot atomique et pages bornées à100. Chaque tâche porte :

<a id="extrait-10"></a>

## Extrait 10

Les lectures et annulations humaines passent par `--t3-thread` et projet. Le
daemon réutilise les gardes147. L'UI ne choisit jamais un agent root arbitraire.
L'annulation appartient toujours à la saga native; elle ne dispatch pas un tour T3.

<a id="extrait-11"></a>

## Extrait 11

T3 persiste des fils virtuels descendants, origin `bridget_native`, liés à son
root courant. Il ne crée aucune session fournisseur, aucun effet de lancement,
aucun tour à exécuter. Les identifiants déterministes rendent la projection
idempotente. Le journal natif reste la source des détails. Le résultat parent
conserve sa remise corrélée unique148.

<a id="extrait-12"></a>

## Extrait 12

Le connecteur Bridget inventorie aussi les fils T3. Les enfants virtuels doivent
porter un `bridgetTaskRef` facultatif, validé par le contrat T3 puis exclu du
montage des wrappers et des présences. Le seul préfixe d'identifiant ne suffit
pas comme attestation. Cette garde évite une boucle de projection et de faux agents.
Source du montage :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code.rs:1285`.

<a id="extrait-13"></a>

## Extrait 13

1. Types de permission/provenance, attestation privée et admission native héritée.
2. Construction fournisseur locale par mission et application wrapper/transport.
3. Parent_task_id, données de projection, séquence commit et lectures bornées.
4. Vue/suivi/annulation humains, lecture du journal descendant sous la garde147.
5. Raccord Bridget/T3 de lecture et projection, coordonné avec l'owner T3.

<a id="extrait-14"></a>

## Extrait 14

Fichiers pressentis : protocol.rs, delegation.rs, delegation_mcp.rs,
daemon/native_delegation.rs, registry.rs, wrapper.rs, claude_stream_json.rs,
codex_app_server.rs, t3code_mcp.rs, daemon.rs, cli.rs et connecteur t3code.
Les modifications natives sont réservées à cet owner. Les tests, fixtures et
relectures appartiennent à GLM. Aucun code T3 ou document principal n'est modifié ici.

<a id="extrait-15"></a>

## Extrait 15

Estimation native après contrat commun stable : 3 à5 heures d'implémentation;
1 à2 heures de raccord avec T3; validation et relecture en plus, par GLM.
Le plus grand risque de délai est l'attestation standalone interactive et le
mappage inter-fournisseurs. Ne pas le masquer par un grant ni par un profil discovery.

<a id="extrait-16"></a>

## Extrait 16

- Schéma final version2 de bridget_session et fraîcheur du tour attesté.
- Mappage Codex/Claude, surtout auto/workspace-write et exigences de réseau.
- Source attestée standalone interactive pour chaque fournisseur réellement supporté.
- Voie Claude SDK/PTY→CLI de même famille : mêmes inputs permissions/profil/cwd
  attestés, contrôlés par revision au lancement/reprise; aucun merge effectif
  inventé. Refus spécifiques seulement pour sources opaques ou non représentables.
- Transport exact snapshot/watch/détail/journal, bornes et persistance du snapshot.
- Propriétaire du connecteur partagé Bridget/T3 et changements de capacités CLI.
