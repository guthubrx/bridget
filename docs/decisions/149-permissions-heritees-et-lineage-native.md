# Décision149 — droits hérités et projection native dans Lineage

Date :2026-10-10. Statut : proposé, en attente du gate GLM du plan commun.

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/149-permissions-heritees-et-lineage-native.md#extrait-1).

## Décision

Bridget reste seul moteur. Le parent de connexion fournit une provenance de
politique effective. Un appel sans posture hérite. Discovery explicite réduit
les droits. L'identité et les permissions ne sont pas des arguments d'agent.
Les révocations et l'opt-out MCP existants restent prioritaires. Aucun nouveau
feature toggle ou grant par mission n'est introduit.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/149-permissions-heritees-et-lineage-native.md#extrait-2).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/149-permissions-heritees-et-lineage-native.md#extrait-3).

Chaque tâche fige la politique effective et la définition enfant à admission.
Le retry et la reprise gardent le même enfant, modèle, mission et politique.
Un changement ultérieur de mode UI n'élargit pas les tâches déjà admises.
Le moteur natif continue par le parcours natif Bridget après admission. L'annulation explicite agit
toujours sur la tâche native et sa descendance.

Le mappage inter-fournisseurs distingue la frontière logique de projet et le
confinement OS réel. Full-access attesté peut autoriser le mode bypass Claude
dans la seule mission enfant. Un sandbox OS workspace-write Codex n'est pas
reproduit par de simples permissions d'outils Claude. Sans capacité équivalente
prouvée, ce mappage est refusé. Aucun framework de sandbox nouveau n'est ajouté.
Les permissions non couvertes restent refusées sans dialogue humain Bridget.

La saga native ajoute parent_task_id et root owner stables. Un snapshot indexé,
borné et cohérent expose les tâches. Une séquence transactionnelle sert uniquement
à l'invalidation. Les journaux existants restent source unique. La garde humaine147
contrôle liste, détail, journal et annulation. Aucun polling des corps n'est nécessaire.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/149-permissions-heritees-et-lineage-native.md#extrait-4).

## Alternatives écartées

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/149-permissions-heritees-et-lineage-native.md#extrait-5).

## Conséquences et validation requise

La provenance est plus précise. Les policies sont figées et les descendants
peuvent fonctionner dans un terminal. La présentation ne contrôle pas les modèles.
L'attestation standalone Claude PTY doit prouver le mode courant et les inputs
réels revisionnés. La recette positive de premier parent GLM doit fermer ce gate.

GLM détient tests et relectures. Le gate doit couvrir héritage lecture/écriture,
full-access Codex→GLM, permissions refusées sans dialogue, politiques opaques,
standalone, rejeu/reprise, résultat unique, projection atomique, journal après
nettoyage, annulation et absence de boucle de pont. Aucun restart ou état de
production ne fait partie de cette validation.

Contrats canoniques :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md`.
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md`.
