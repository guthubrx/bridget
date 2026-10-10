# Contrat149 — provenance et héritage des permissions

Statut : proposé pour gate GLM. Date :2026-10-10.
Les permissions appartiennent au moteur natif. T3 fournit une attestation de
son fournisseur. Les wrappers standalone fournissent leur observation propriétaire.

## Demande et invariants

`bridget_delegate` conserve request_id, agent_type, model, effort facultatif,
task et cwd. Posture devient facultative. Son omission signifie inherit.
Les valeurs explicites restent discovery et development. Aucun champ nouveau
parent, permission, preuve, endpoint, credential, sandbox ou callback n'est accepté.
La posture demandée ne prouve jamais un droit.

Sans posture, Bridget conserve la politique attestée. Discovery réduit à une
politique de lecture seule réellement supportée. Development exige les droits
d'écriture prouvés; il n'ajoute aucun grant humain Bridget. Une absence de preuve
rend permission_attestation_unavailable. Un parent lecteur qui demande l'écriture
reçoit permission_not_inherited. Il ne reçoit pas une demande de grant supplémentaire.

La mission garde son cwd canonique dans le projet attesté et ses worktrees
autorisés par la politique. La portée logique d'une mission est distincte d'un
sandbox du système d'exploitation. Le catalogue annonce cette différence.
Il ne prétend jamais que Claude est confiné par un sandbox Codex.

Le snapshot est figé à admission, avant effet. Il conserve la politique d'origine,
le mappage appliqué et la définition enfant. Lancement, retry et reprise utilisent
ce même snapshot. Un changement ultérieur UI/registre n'élargit aucune mission.
Une preuve d'identité fraîche autorise l'accès; elle ne remplace pas le snapshot.
Le moteur admis continue si T3 devient indisponible. L'annulation humaine reste native.

Les révocations stables148 et l'opt-out MCP existant restent prioritaires pour
les nouveaux accès. La lecture humaine/cancel réutilise les gardes147.
Aucun opt-out ou feature toggle149 nouveau n'est ajouté.

## Introspection T3 version2

Outil `bridget_session`, arguments objet vide et fermés. Enveloppe exacte :

Le succès est une union : l'enveloppe148 version1, identité seule, reste possible
tant qu'aucun fait de permissions n'a jamais existé pour ce credential. Dès
qu'un fait valide existe, le succès utilise version2 stricte ci-dessous. Un fait
invalide, périmé, contradictoire, retiré (fin, échec ou fermeture du tour),
révoqué ou tourné produit un refus; il ne retombe jamais en version1.
Version1 préserve identité, status/cancel et lecture historiques148 seulement
pour un credential sans fait jamais existé. Une nouvelle admission149
inherit/development exige version2; version1 donne permission_attestation_unavailable sans grant ni fallback
discovery.

```json
{
  "version": 2,
  "environmentId": "environnement propriétaire",
  "threadId": "fil propriétaire",
  "providerSessionId": "session fournisseur courante",
  "providerInstanceId": "instance fournisseur courante",
  "permissions": {
    "version": 1,
    "source": "provider_turn",
    "run_id": "tour actif propriétaire",
    "provider_session_id": "session fournisseur courante",
    "provider_instance_id": "instance fournisseur courante",
    "revision": 1,
    "driver": "codex_app_server",
    "cwd": "/répertoire/effectif",
    "runtime_mode": "full-access",
    "interaction_mode": "default",
    "provider_policy": {
      "kind": "codex",
      "approval_policy": "never",
      "approvals_reviewer": "user",
      "sandbox_policy": { "type": "dangerFullAccess" }
    }
  }
}
```

L'identité148 reste en camelCase. `permissions` et les clés de politique149 sont
en snake_case, sauf l'objet sandbox_policy qui conserve le wire Codex camelCase.
Tous les objets sont fermés. Driver et kind forment une union corrélée.
Les identifiants non vides sont bornés à256octets, threadId à2048. Aucun contrôle
interdit. Cwd est absolu, non vide et au plus4096octets. Revision est un entier
1..9007199254740991. La réponse totale respecte64KiB et les bornes HTTP148.

Runtime_mode : approval-required, auto-accept-edits, auto ou full-access.
Interaction_mode : default ou plan. Ces labels ne remplacent pas provider_policy.
Un mode non supporté, une donnée absente ou une contradiction ferme l'attestation.

### Politique Codex exacte

Objet fermé avec kind=codex et trois clés :

- approval_policy : untrusted, on-request, never ou l'objet granular ci-dessous;
- approvals_reviewer : user, auto_review ou guardian_subagent;
- sandbox_policy : une des quatre formes wire ci-dessous.

```text
approval_policy = {
  granular: {
    mcp_elicitations: bool,
    request_permissions?: bool,
    rules: bool,
    sandbox_approval: bool,
    skill_approval?: bool
  }
}

sandbox_policy =
  { type: "dangerFullAccess" }
  | { type: "readOnly", networkAccess?: bool }
  | { type: "workspaceWrite", writableRoots?: string[], networkAccess?: bool,
      excludeSlashTmp?: bool, excludeTmpdirEnvVar?: bool }
  | { type: "externalSandbox", networkAccess?: "restricted" | "enabled" }
```

Les champs facultatifs représentent une absence réelle. Bridget ne les remplit
pas depuis un défaut inventé. Une construction enfant qui exige ces valeurs
les résout auprès de la source propriétaire; sinon le mappage est refusé.
WritableRoots contient au plus128chemins absolus, chacun au plus4096octets.
ExternalSandbox atteste une délégation externe de confinement, pas son mécanisme.
Sans preuve transportable de ce mécanisme, le lancement inter-fournisseur est refusé.

Source T3 : paramètres finaux validés de turn/start après defaults et overrides.
Les flags launchArgs de sécurité non représentés rendent l'attestation indisponible.

### Politique Claude/GLM exacte

Objet fermé :

```text
{
  kind: "claude",
  permission_mode: "default" | "acceptEdits" | "bypassPermissions"
                   | "plan" | "dontAsk" | "auto",
  tools: string[] | { type: "preset", preset: "claude_code" },
  allowed_tools?: string[],
  disallowed_tools?: string[],
  additional_directories?: string[],
  allow_dangerously_skip_permissions?: bool,
  permission_callback: {
    kind: "t3_runtime",
    tool_approval: "prompt" | "allow",
    plan_exit: "deny"
  },
  settings_sources: "provider_default",
  launch_context?: {
    cli_path: string absolu,
    cli_revision: "sha256:<64 hex minuscules>",
    resolved_cli_path: string absolu canonique,
    resolved_cli_revision: "sha256:<64 hex minuscules>",
    config_dir: string absolu,
    settings_sources: "provider_default" | ("user" | "project" | "local")[],
    settings_overrides?: {
      permissions?: {
        allow?: string[], ask?: string[], deny?: string[],
        defaultMode?: "default" | "acceptEdits" | "bypassPermissions"
                      | "plan" | "dontAsk" | "auto",
        additionalDirectories?: string[],
        disableBypassPermissionsMode?: "disable"
      }
    },
    permission_sources: {
      kind: "user" | "project" | "local" | "managed" | "cli",
      path: string absolu,
      revision: "absent" | "sha256:<64 hex minuscules>"
    }[]
  }
}
```

Chaque liste est bornée à128entrées. Une règle outil est non vide, sans contrôle
interdit et au plus512octets. Additional_directories contient des chemins absolus.
Les valeurs facultatives ne deviennent pas des listes vides inventées.
`manual` observé en CLI est normalisé en default, alias documenté par le fournisseur.
T3 ne passe actuellement aucun sandbox_settings dans queryOptions. Il n'en publie
donc aucun. Un futur champ exige une version ou extension contractuelle validée.

Le callback T3 existe réellement. Tool_approval reflète sa décision de politique,
pas le seul mode UI. Plan_exit=deny préserve la politique actuelle. Bridget ne
transporte pas la fonction callback T3 dans un enfant standalone.

Source T3 : queryOptions finales et mode mutable observé après init/status ou
setPermissionMode réussi. `settings_sources=provider_default` décrit honnêtement
les réglages chargés implicitement par le fournisseur. Il ne prétend pas exposer
les règles fusionnées de ces fichiers. Un mappage qui exige leur connaissance
doit obtenir un snapshot de permission complet de la source propriétaire ou
utiliser la voie de mêmes inputs décrite ci-dessous. Un mappage inter-driver
sans preuve reste permission_mapping_unavailable. Aucun JSON settings/env brut
n'est publié.

Settings_overrides comprend aussi les mutations sanitaires réellement appliquées
par prepareClaudeMcp/applyFlagSettings après succès du montage148. La capture
entoure openQuery et intègre ces overrides finaux avant publication. Les seules
queryOptions initiales ne suffisent pas. Sources et résolutions sont recontrôlées
avant attestation, lancement et reprise.

Launch_context est facultatif dans l'introspection, mais requis pour le chemin
de mêmes inputs Claude SDK/PTY→CLI sans fusion observée des règles. Son absence
ne refuse pas toutes les cibles; elle refuse seulement un mappage qui en dépend.
Il est choisi par l'adaptateur propriétaire, jamais depuis les arguments MCP.
Au plus32sources, chemins4096octets et liste de settings_sources sans doublon.
Les règles de settings_overrides respectent les mêmes bornes que les règles outils.
Les clés supplémentaires de sécurité non représentées dans des overrides inline
rendent ce chemin indisponible. Une source fichier reste héritée par son input
complet vérifié; Bridget n'en invente pas une fusion parallèle.

Cli_revision est SHA256 des octets du launcher choisi. Resolved_cli_path est
le chemin canonique du CLI effectivement exécuté par ce launcher; son digest
resolved_cli_revision porte sur les octets de ce binaire. Sans launcher, les
deux couples désignent le même exécutable. Les deux couples sont obligatoires
dans launch_context. Revision de source est
SHA256 des octets complets du fichier ou absent. Seuls chemin/digest sont publiés.
Ni fichier brut, ni secret, ni env, ni hook, ni configuration MCP ne sont copiés.
Le hash complet détecte aussi une règle de sécurité hors permissions, sans nouveau
schéma de fusion. Une rotation d'auth peut donc produire un refus conservateur de
revision; elle ne donne jamais des droits supplémentaires. Une source absente est
recontrôlée comme absente avant effet. Un nouveau fichier n'élargit pas un retry.
Les sources incluent celles réellement choisies, dont remote-settings.json et
managed-settings.d si le fournisseur les charge. Pour un répertoire de fragments,
le digest porte sur la liste triée des noms et les digests de fichiers, au plus
256entrées. Une origine MDM/policyHelper non observable ferme seulement ce contexte.

La résolution appartient au launcher natif ou à l'adaptateur T3 propriétaire.
Elle utilise l'environnement final de lancement, dont son PATH effectif, sans
publier cet environnement. Elle est fraîche, sans cache de résolution. Le cas
gclaude connu termine par exec claude "$@", sans changer PATH/cwd ni charger
un autre script. L'owner résout donc claude sous ce PATH précis, puis realpath
et SHA256. Un nom de wrapper ne constitue jamais cette preuve. Une chaîne non
observable, ou BASH_ENV/ENV/fonction shell exportée non attestée qui peut changer
la résolution, rend permission_source_unavailable pour cette voie seulement.

Avant publication du fait, admission, premier lancement et reprise, l'owner
refait cette résolution depuis les inputs de lancement réellement utilisés.
Il compare chemin canonique et digest du CLI, ainsi que le chemin et digest du
launcher, aux couples figés. Changer l'ordre PATH, retargeter un lien ou remplacer
l'un des deux exécutables rend settings_revision_changed. Il ne lance aucun
processus enfant et n'adopte pas la nouvelle résolution. Le PATH n'est jamais
un argument MCP ni une variable parent copiée comme credential dans une tâche.

### Même famille Claude : héritage des inputs réels

Ce chemin positif réutilise le même launcher attesté, même profil, même cwd,
mêmes sources sélectionnées et overrides fournisseur finaux. Le fournisseur
Claude/GLM résout lui-même ces mêmes inputs. La preuve porte sur les inputs
inchangés, pas sur une fusion de settings prétendument observée.

Le daemon rapproche cli_path et config_dir de la définition native autorisée.
Il vérifie aussi le CLI effectivement résolu et les deux empreintes figées.
Il applique les gardes existantes du registre et du profil : source propriétaire,
chemin canonique et droits privés. Cwd vient de la mission et doit être égal au
contexte attesté pour cette voie. Tous les digests sont vérifiés hors verrou,
puis binding/instance/projet sont revérifiés avant admission. Les références et
digests sont figés dans la tâche. Ils sont recontrôlés avant premier lancement
et toute reprise de processus. Une divergence rend settings_revision_changed.
Le retry n'adopte jamais le nouveau fichier, profil, wrapper ou mode UI.

Cas réel identifié : l'instance T3 claude_glm et la cible native glm utilisent
`/Users/moi/.local/bin/gclaude` et `/Users/moi/.claude-glm`. Le launcher transmet
les arguments à Claude et n'ajoute aucun override permission/settings/tools.
La recette isolée GLM doit utiliser ce même couple launcher/profil attesté.
La cible conserve son branchement d'auth propre. Aucun credential parent n'est
transmis. Le modèle exact et la politique finale de mission restent ceux figés
par Bridget, avec aucune substitution automatique.

Des profils/cwds différents ne constituent pas automatiquement la même voie.
Une traduction de permissions sanitaires prouvée peut couvrir ces cas. À défaut,
refus permission_mapping_unavailable. Des sources managed opaques sans voie
vérifiable donnent permission_source_unavailable, limité à ces cas. Ce refus
n'interdit pas le chemin positif des inputs connus. Aucune nouvelle permission
humaine Bridget ou découverte forcée ne remplace la preuve.

## Fraîcheur et credential

Le registre privé T3 lie le fait à l'objet credential courant, environnement,
fil, session/instance fournisseur et run_id. L'adaptateur publie le fait juste
avant l'envoi réel au fournisseur. Un échec/end/close retire seulement le fait
du run propriétaire; un cleanup ancien ne retire pas le nouveau tour.
Rotation ou retrait de session MCP efface le fait. Revision augmente lorsque
le fait courant change. Elle ne prouve ni durée de validité ni TTL arbitraire.

L'introspection vérifie le tour réellement actif et l'égalité de tous ces liens.
Une politique du tour précédent ou d'une autre instance est refusée. Un mode
Claude qui change produit une nouvelle revision pour les futures admissions.

L'adaptateur Bridget privé transporte seulement BRIDGET_T3_MCP_ENDPOINT et la
preuve opaque d'autorisation148. Le daemon réatteste lui-même hors verrou. Il
utilise le runtime local connu, HTTP sans redirection, timeout3s,64KiB maximum,
session transport unique puis DELETE. Sous verrou, il recontrôle la route
primaire, identité/instance, binding T3, projet et absence de révocation.
Une course entre preuve et binding ferme l'admission.

Le token est une valeur privée dont Debug masque le contenu. Aucun token,
endpoint secret, fichier credential, clé provider, prompt ou environnement complet
ne rejoint tâche, définition, CLI visible, erreur libre, log ou enfant.
Une preuve invalide ne retombe jamais sur un PID ou une identité globale.

## Sources standalone

La politique standalone est un fait de la connexion primaire propriétaire.
Un auxiliaire d'agent ne peut pas fournir ce fait comme JSON autodéclaré.
Les identifiants provider/session et le projet doivent correspondre à la présence
vivante. Déconnexion ou contradiction retire le fait; la reconnexion le réatteste.

| Chemin existant | Source normative de droits | Application149 |
|---|---|---|
| Enfant native managed déjà admis | permission_snapshot de tâche + définition digérée, lease et lien Fleet prouvés. | Hérite sans T3 et sans grant. La tâche parent est déduite de child_agent_id. |
| Managed historique sans snapshot149 | Définition réellement lancée, arguments finaux du wrapper et politique observée par transport. | Compatibilité148 conservée; droits non observés refusés, aucune extension automatique. |
| Codex interactif natif partagé | Transport app-server propriétaire, config/read effectif et paramètres/retour du thread ou du tour en cours. | Les profils et overrides sont résolus par Codex. Pas de parsing TOML parallèle ni repli permissif. |
| Claude/GLM interactif PTY | Wrapper propriétaire, arguments finaux, launcher/profil/cwd/sources revisionnés et mode courant du fournisseur. | Même famille via mêmes inputs; observation ciblée PreToolUse sur délégation, liée session/cwd/prompt_id. Le hook ne prend aucune décision d'autorisation. |
| MCP externe sans wrapper/source attestée | Identité148 uniquement. | Lecture historique possible. Nouvelle écriture sans preuve149 refusée, sans demande de grant de remplacement. |

Le hook Claude officiel fournit permission_mode courant, session_id et cwd.
Il ne fournit pas les règles fusionnées. Le wrapper fige les références et
revisions des inputs réellement sélectionnés, ou les seuls overrides sanitaires
connus. Le même fournisseur résout ces inputs. Aucun chemin à lire ne vient de
la requête MCP. Ne pas copier auth, env, hooks ou settings bruts dans une attestation.
Une règle/session dynamique ou source managed opaque non observable doit rester
un refus nommé. Le mode seul ne rend pas ces règles prouvées.

Le chemin PTY ne peut être déclaré validé avant une recette GLM qui prouve la
source sélectionnée, son mode courant, la fraîcheur et les inputs revisionnés
de permission. Cette exigence ne peut être remplacée par discovery ou un grant humain.
L'observation ciblée n'ajoute ni serveur d'autorité, ni framework sandbox.

### Observer propriétaire temporaire du CLI

Le wrapper possède un overlay PreToolUse pour ses seuls outils de délégation
et de catalogue Bridget. Cet overlay est un mécanisme d'observation séparé de
settings_overrides. Il ne remplace pas les sources settings sélectionnées et
ne fusionne pas leurs règles. Il n'est jamais recopié dans l'enfant.

Chaque lancement crée un fichier privé0600 et une socket privée0600 dans son
namespace. L'overlay ajoute seulement un hook command connu. Son contenu et
son empreinte restent dans la provenance privée du wrapper. Le CLI reçoit le
chemin temporaire via --settings; les chemins, revisions et overrides de
permission attestés restent ceux du lancement avant cet ajout d'observation.
La recette doit confirmer la composition réelle des sources par ce CLI.
Un CLI qui remplace ces sources au lieu de composer l'overlay est refusé.

Le hook transmet uniquement mode courant, session_id, cwd, prompt_id et
identifiant de la demande Bridget. Il utilise une preuve opaque propre à ce
lancement. Le wrapper vérifie cette preuve, la session et le cwd. Il recontrôle
les entrées revisionnées. Puis il publie le fait par sa connexion primaire.
Un auxiliaire ne peut publier ce fait. Le daemon accuse la publication avant
que le hook laisse continuer l'appel. La demande de délégation doit correspondre
à l'identifiant observé; un fait d'une autre demande ne donne aucun droit.

Cette voie ne décide pas les permissions fournisseur. Une preuve absente,
contradictoire ou un observer indisponible bloque seulement l'accès Bridget
non attesté, avec son refus nommé. Aucun grant, acceptation générale de tool,
tour fournisseur ou rejeu de mission n'est créé. Le nonce, le fichier et la
socket privés ne rejoignent ni snapshot durable, ni journal, ni enfant.
La sortie du fournisseur retire le fait, ferme la socket et supprime l'overlay.
La reconnexion ne récupère aucun fait de permission d'une ancienne connexion.

## Mappage inter-fournisseurs

Chaque mappage fige la politique enfant et sa provenance. Les politiques propres
à la cible peuvent restreindre l'exécution; elles ne donnent jamais des droits
supérieurs au snapshot parent. Un refus fournisseur est une issue explicite.

| Politique parent attestée | Cible Codex | Cible Claude/GLM |
|---|---|---|
| Codex dangerFullAccess | Sandbox danger-full-access explicite; politique approbation conservée, jamais flag --yolo global. | BypassPermissions uniquement dans la définition de cette mission si le parent ne requiert pas de nouvelle médiation humaine. Réseau non réduit silencieusement. |
| Codex readOnly | read-only effectif et restrictions réseau conservées. | Discovery via restricted+plan et outils effectivement lecteurs. Network exigé doit être représentable; sinon permission_mapping_unavailable. |
| Codex workspaceWrite avec confinement OS | Workspace-write exact: roots/réseau/tmp et politique approbation conservés. | Seulement si un confinement fournisseur équivalent est réellement disponible et attesté. Sinon provider_confinement_unavailable. Les règles outils seules ne suffisent pas. |
| Codex externalSandbox | Confinement externe transportable attesté requis. Sinon provider_confinement_unavailable. | Même exigence; aucun sandbox nouveau inventé. |
| Claude bypassPermissions, règles complètes et aucune restriction non représentée | Full-access seulement si les éventuelles restrictions et médiations parent sont représentables. Sinon permission_mapping_unavailable. | Mode bypass local mission et règles figées. Aucun réglage global du registre n'est modifié. |
| Claude plan/discovery explicite, outils lecteurs complets | Read-only compatible, sans ajout d'outil ou réseau non autorisé. | Plan/restricted avec mêmes limites de lecture. |
| Claude acceptEdits/auto/default/dontAsk et règles complètes | Seulement si décisions approbation et restrictions sont représentables. Sinon permission_mapping_unavailable. | Même mode/règles; prompts non interactifs none. Les actions hors politique restent refusées. |
| Claude SDK/PTY avec inputs launcher/profil/cwd/settings inchangés et mode live attesté, sans merge observé | Cross-driver seulement si restrictions et médiation sont prouvées et représentables; sinon permission_mapping_unavailable. | Voie positive same-family des mêmes inputs revisionnés, sans prétendre règles fusionnées observées. |
| Source nécessaire opaque/non observable, dont MDM/policyHelper, chaîne launcher ou source implicite non attestée, sans voie vérifiable | permission_source_unavailable. | Même refus spécifique, limité au contexte concerné. |
| Policy inconnue ou flags de sécurité non représentés dans une attestation effective | permission_attestation_unavailable. | Même refus. Aucun fallback lecteur/bypass/grant. |
| Policy connue mais traduction vers la cible non représentable | permission_mapping_unavailable. | Même refus de mappage. |

Une politique de demande humaine n'est pas transformée en autorisation silencieuse.
Les outils safe Bridget existants restent disponibles selon leur contrat.
L'absence de posture n'impose pas découverte. Le catalogue présente les mappages
réellement supportés et leurs refus, sans substitution de modèle ou fournisseur.

## Permissions non interactives Claude

La cible doit prouver qu'elle supporte --permission-prompts none. Ce flag garde
son mode et refuse ce qui aurait demandé un prompt. Le parent full-access attesté
utilise son mode explicite; le flag ne lui invente aucune autorisation supplémentaire.
Une trame can_use_tool éventuelle reçoit un refus automatique corrélé quand
elle ne correspond à aucune autorisation héritée. Aucun allow global ni boîte
de dialogue Bridget n'est créé. Une demande ignorée ne doit pas bloquer la mission.

Les permission_denials et refus outils doivent être visibles dans l'état/résultat.
Une tâche dont le travail demandé est bloqué par une permission rend un échec
explicite permission_not_inherited ou provider_permission_denied. La seule fin
du tour n'est jamais une preuve de succès. Une lecture n'acquitte ni ne relance rien.

## Inventaire normatif des refus de permissions

Les codes ci-dessous constituent l'inventaire unique149. La cause précise reste
un détail sanitaire borné. Un refus n'annonce jamais un grant humain à obtenir.

| Code | Cause |
|---|---|
| permission_attestation_unavailable | Fait propriétaire absent/périmé, identité/run incohérent ou politique effective inconnue/non représentée. |
| permission_source_unavailable | Fait identifiable dont une source nécessaire reste opaque/non observable; inclut la chaîne launcher non prouvée. Ne signifie pas refus de tous les profils Claude. |
| permission_mapping_unavailable | Politique connue qui ne peut pas être conservée dans le fournisseur cible. |
| provider_confinement_unavailable | Confinement OS requis sans équivalent cible prouvé. |
| permission_not_inherited | Droit demandé absent de la politique parent attestée. |
| provider_permission_denied | Fournisseur refuse un outil ou rapporte permission_denials qui bloque le travail demandé. |
| settings_revision_changed | Input figé changé, supprimé, ajouté ou résolution launcher/CLI différente avant effet. |

Les refus d'identité, révocation, binding et opt-out148 restent ceux du contrat148.
Ils ont priorité; cet inventaire ne les remplace pas.

## Oracles de validation à implémenter par GLM

Ce sont des scénarios de conception. Leur présence ferme le gate du plan.
Les preuves d'exécution sont attendues après implémentation, avant livraison.

G-P-01 : un parent écrit avec une politique attestée qui exclut un outil précis.
Une fixture de transport émet can_use_tool pour cet outil avec request_id connu.
La seule control_response porte ce request_id et une décision deny. L'outil
interdit ne produit aucun effet. La réponse fournisseur corrélée contient
permission_denials, puis une fin de tour. La tâche devient failed avec
provider_permission_denied ou permission_not_inherited selon le point de refus.
Elle n'atteint jamais result_available et n'émet aucun résultat de succès.
Le rejeu/lecture n'émet ni réponse supplémentaire ni lancement supplémentaire;
le compteur launched reste égal à sa valeur avant la demande de permission.
Une variante réelle GLM atteste le refus non interactif lorsque le fournisseur
permet de le provoquer; elle ne remplace pas l'oracle déterministe request_id.

G-P-02 : figer une admission positive same-family et enregistrer le compteur
launched avant le premier spawn. Trois variantes indépendantes modifient une
source existante, la suppriment, ou créent une source précédemment attestée
absente. Chacune rend settings_revision_changed; launched reste inchangé.
La même matrice s'applique après réouverture du magasin, avant reprise d'un
processus. Le snapshot, le modèle et les droits initiaux restent identiques.
Ajouter les variantes launcher remplacé, binaire CLI remplacé, symlink retargeté
et résolution déplacée par PATH. Aucun retry ne réatteste ces inputs comme un
élargissement admis. Une recette positive utilise les deux digests réellement
résolus sous l'environnement propriétaire du premier parent GLM, sans grant.

G-P-07 : union d'introspection v1/v2. Trois branches. (a) Un credential
sans aucun fait de permissions jamais existé reçoit l'enveloppe148 v1,
identité seule, quatre champs camelCase, parseur fermé. (b) Pour un credential
dont le fait de permissions a été retiré (fin, échec ou fermeture du tour),
révoqué ou tourné, la fonction `bridget_session` de T3 rend le refus nommé
`permission_attestation_unavailable`, jamais l'enveloppe v1. La façade MCP
privée de Bridget traite ce refus comme un échec de preuve de session. Tout
appel de ce credential est fermé avant effet avec `t3_session_unavailable` :
`bridget_delegate` (nouvelle requête et rejeu d'une requête admise),
`bridget_task_status`, `bridget_task_cancel` et `bridget_who`. Il n'y a ni
repli v1, ni repli discovery, ni repli par PID. Aucun lancement n'a lieu.
(c) Une admission 149 inherit/development
présentée avec la seule enveloppe v1 rend permission_attestation_unavailable,
sans grant demandé, sans fallback discovery, sans spawn ; status, cancel et
identité148 continuent de fonctionner.
« Lecture et rejeu inchangés » de la branche (b) s'entend ainsi : (i) la
lecture humaine Lineage native (gardes 147), l'annulation humaine native et la
saga native continuent, car elles ne dépendent pas du credential T3 ; (ii) un
credential neuf qui n'a jamais eu de fait reçoit l'enveloppe v1 : identité,
status, cancel et rejeu 148 d'une requête déjà admise restent possibles, sans
nouvelle exécution.
Clarification du 2026-10-10 (revue source Sonnet r1, section 5) : branche (b)
précisée, sans changement de contrat. Preuves réelles du cas : recovery149.md
R8.4 et interop149.md S5.1/S5.1b, sur binaire r5.

G-P-08 : observer propriétaire du chemin PTY. (a) Un lancement avec
l'overlay --settings publie un fait portant permission_mode, session_id,
cwd, prompt_id et l'identifiant de la demande Bridget ; le daemon accuse
réception avant que le hook laisse continuer l'appel observé ; la demande
délégataire correspond à l'identifiant observé. (b) Preuve absente,
non corrélée ou observer indisponible : l'appel Bridget est bloqué avec
son refus nommé ; aucun effet fournisseur, aucun grant, compteur launched
inchangé. (c) Un fait d'une autre demande ne donne aucun droit.
L'overlay, son nonce, la socket et la preuve n'apparaissent ni dans le
snapshot enfant, ni dans la définition ou l'environnement enfant, ni dans
le journal ; fichier et socket 0600 sont supprimés en sortie ; la
reconnexion ne récupère aucun fait d'une ancienne connexion.

## Compatibilité et gate

Les réponses identité148 version1 restent recevables pour les lectures historiques
avec un credential valide pour lequel aucun fait de permissions n'a jamais existé.
Elles ne rendent pas recevable un credential retiré, révoqué ou tourné.
Elles ne prouvent pas une permission d'écriture149. Les tâches148 gardent leur
définition et checkpoints sans migration d'élargissement. Aucun downgradev1
permissif n'est admis après une preuve149 invalide.

Le gate GLM doit prouver les sources réellement utilisées sur chaque chemin livré.
La voie positive same-family exige la recette d'un premier parent GLM externe
avec les inputs réels ci-dessus, sans T3 et sans grant supplémentaire. Les cas
managed opaques restent explicitement limités. La preuve source est prévue par
le contrat; elle ne constitue pas encore une recette validée.

Références officielles : [hooks Claude](https://code.claude.com/docs/en/hooks),
[réglages et priorités Claude](https://code.claude.com/docs/en/settings),
[permissions SDK](https://code.claude.com/docs/en/agent-sdk/permissions).
Le schéma T3 exact vient de l'owner de son attestation. Les faits natifs sont
référencés dans la recherche149. Ces contrats ne constituent pas une recette validée.
