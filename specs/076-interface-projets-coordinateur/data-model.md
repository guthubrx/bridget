# Modèle de données - SPEC-076

## Autorités conservées

| Entité | Autorité | Usage SPEC-076 |
|---|---|---|
| ProjectIdentity | Maicie, SPEC-065 étendue par SPEC-076 | Identité durable, état métier et réactivation typée. |
| ProjectBinding | Bridget, SPEC-065 étendue par SPEC-076 | Racine canonique, backend, génération et état technique. |
| ProjectAuditEvent | Bridget, SPEC-065 étendue par SPEC-076 | Historique register, rebind, disable et réactivation. |
| AgentDefinition résolue | Bridget, SPEC-072 | Outil agent, capabilities et digest. |
| ProjectRuntimePolicy | Bridget, SPEC-066 | Politique runtime du coordinateur. |
| ProjectProfile | Maicie et Bridget, SPEC-067 | Permissions et références approuvées, sans secret UI. |
| Cycle agent | Bridget, SPEC-075 | Stop, relaunch et decommission du coordinateur. |

Aucune de ces entités n'est copiée dans un store navigateur.
Pour SPEC-076, `ProjectIdentity.display_name` est dérivé une seule fois du
nom de dossier au moment de la création ou de l'import. L'interface ne le rend
pas éditable et ne crée pas de label concurrent.

## ProjectRootSettings

Politique versionnée de racines par instance Bridget.

| Champ | Rôle |
|---|---|
| policy_generation | Concurrence optimiste et preuve de reload. |
| allowed_roots | Chemins canoniques autorisés. |
| updated_at | Diagnostic et audit. |
| update_reason | Motif non sensible. |
| applied | Verdict que le daemon utilise cette génération. |

Invariants :
- aucune racine relative, absente, large, dupliquée ou de permissions invalides;
- aucun retrait excluant un projet actif;
- écriture atomique et permissions strictes;
- une génération non appliquée n'est jamais affichée active.

## CoordinatorConfiguration

Instantané durable choisi lors de l'onboarding.

| Champ | Rôle |
|---|---|
| launcher_id | Outil agent déclaré. |
| provider_id | Fournisseur ou upstream déclaré. |
| model_id | Modèle sélectionné ou résolu. |
| effort | Niveau retenu. |
| permission_profile_ref | Référence de profil, jamais valeur secrète. |
| resolved_definition_digest | Détection incompatibilité. |
| runtime_policy_version | Lien SPEC-066. |
| created_from_defaults | Provenance informative des valeurs. |

Invariants :
- aucun secret ni argument sensible;
- toute composante est attestée avant sélection;
- un changement de défaut ne réécrit pas l'instantané;
- une migration d'outil ou modèle reste hors SPEC-076.

## ProjectOnboarding

Projection durable de l'entrée dans le projet.

| Champ | Rôle |
|---|---|
| project_id | Référence SPEC-065. |
| folder_mode | create ou import, à valeur historique. |
| git_initialization | absent, refusé, demandé, effectué ou non applicable. |
| coordinator_name | Coordinateur courant unique. |
| coordinator_state | absent, starting, ready, unavailable, incompatible ou stopped. |
| discovery_state | not_started, running, awaiting_confirmation, reported ou stopped. |
| current_discovery_deadline | Borne du créneau actif. |
| last_reason | Cause non sensible d'état anormal. |

Invariants :
- un seul coordinateur courant;
- aucun état prêt avant verdict réel;
- les messages système ne remplacent pas les messages agent;
- le retrait conserve cette projection.

## DiscoveryRun

| Champ | Rôle |
|---|---|
| run_id | Identifiant du créneau. |
| project_id | Projet concerné. |
| coordinator_generation | Génération logique. |
| mode | read_only obligatoire. |
| started_at, deadline_at, finished_at | Borne et résultat. |
| requested_duration | 10, 30, 60 ou jusqu'à 120 minutes. |
| outcome | report_ready, awaiting_confirmation, stopped ou failed. |
| continuation_of | Créneau antérieur si autorisé. |

Invariants :
- pas de durée illimitée;
- pas de poursuite sans confirmation humaine;
- pas d'écriture projet ni création d'agent dans le contrat.

## Transitions

dossier inconnu -> prévisualisé -> dossier prêt -> liaison -> projet actif ->
coordinateur starting -> coordinateur ready -> découverte running -> rapport.

projet actif -> coordinateur unavailable ou incompatible -> retrait demandé ->
arrêt si nécessaire -> projet retiré -> réactivation explicite -> projet actif.

La réactivation utilise une transition métier typée de `disabled` vers
`active`, suivie d'une liaison active validée et d'un audit idempotent.
Les états identitaires, runtime, profil et cycle de vie restent les autorités
des spécifications dont ils proviennent.

## Réalisation v1 - 2026-08-31

La projection durable effectivement écrite par SPEC-076 est un document privé
local au relais, voisin de la politique de racines. Il contient, par projet,
le chemin canonique, le mode create ou import, le verdict Git, la configuration
complète attestée du coordinateur, son nom courant, son état, le dernier motif
et le DiscoveryRun éventuel. Ses permissions sont 0600 et son schéma est fermé.

Le DiscoveryRun persiste aussi baseline_fingerprint et completion_fingerprint.
Ces empreintes SHA-256 ne conservent ni contenu de fichier, ni secret, ni liste
lisible de fichiers. Elles couvrent les métadonnées de l'arbre sans suivre les
liens symboliques ainsi que le résultat Git status. Une différence finale
bascule la découverte en failed et interdit sa poursuite sans nouvelle décision
humaine.

Les valeurs par défaut restent dans un document privé distinct. Elles ne sont
lues qu'à la création d'un nouvel onboarding et ne réécrivent jamais une
configuration coordinateur déjà durable.

La vue UI publie enfin le dernier audit de liaison sous forme reduite :
operation, issue, generation et instant. Cette vue reste derivee de
ProjectAuditEvent et ne constitue pas un historique concurrent.
