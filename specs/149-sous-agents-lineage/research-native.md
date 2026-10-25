# Recherche native149 — permissions héritées et projection de descendance

Date : 2026-10-10. Statut : investigation et contrat proposé. Aucun code modifié.
Responsable : owner natif. Git, plan commun et livraison restent au principal.
Tous les tests et toutes les relectures sont confiés à GLM 5.3 Flash.
Aucun modèle lancé, redémarrage, fichier de production ou base de production modifié.

## Résultat recherché

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-1).

## Faits observés dans le code

Racine de tous les chemins ci-dessous : aucune abréviation de chemin n'est requise.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-3).

## Contrat de provenance convenu avec l'owner adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-4).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-7).

## Héritage et admission natifs proposés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-8).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-9).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-10).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-12).

La requête native utilise un index parent/root et une pagination stable. Objectif
O(log N + P), P<=100, profondeur<=8 et N<=4096. Éviter une requête par descendant
ou une recherche linéaire répétée dans la liste Fleet.

## Périmètre proposé et ordre d'implémentation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-13).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-14).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-15).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-native.md#extrait-16).

Ce document ne constitue pas une preuve de fonctionnement. Aucun test ni revue
n'a été exécuté par cet owner. Le code attend le GO du principal sur le plan commun.
