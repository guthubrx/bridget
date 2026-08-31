# Recherche et décisions - SPEC-079

Date: 2026-08-31

## D1 - Exécution durable distincte du processus

Décision: conserver une identité logique de travail et créer une nouvelle
tentative d'exécution après perte du provider.

Raison: Temporal documente la reprise d'un workflow après panne comme propriété
du modèle durable, tandis que LangGraph sépare l'identifiant de thread et ses
checkpoints du processus qui l'exécute. Bridget possède déjà cette séparation
avec `WorkSubmission` et `Execution`.

Sources primaires:
- https://docs.temporal.io/
- https://docs.langchain.com/oss/python/langgraph/persistence

## D2 - Rejeu déterministe et effets idempotents

Décision: chaque reprise et chaque occurrence de ronde possède une identité
stable; les effets externes passent par la saga idempotente existante.

Raison: Google SRE exige, pour une tâche périodique confrontée aux pannes
partielles, un effet idempotent ou une vérification durable de l'état. LangGraph
impose également d'encapsuler les effets non déterministes pour permettre le
rejeu.

Sources primaires:
- https://sre.google/sre-book/distributed-periodic-scheduling/
- https://docs.langchain.com/oss/python/langgraph/functional-api

## D3 - Ne pas confier la reprise au shutdown gracieux

Décision: la reprise part du store au prochain Register et ne dépend pas d'un
hook d'arrêt.

Raison: Kubernetes précise que les hooks de cycle de vie sont livrés au moins
une fois et que certaines fins de processus ne permettent pas d'exécuter un
chemin gracieux. Le store doit donc rester l'autorité.

Sources primaires:
- https://kubernetes.io/docs/concepts/containers/container-lifecycle-hooks/
- https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/

## D4 - Une ronde par politique projet, pas un timer par projet

Décision: conserver un scheduler global qui énumère les politiques activées.

Raison: SPEC-065 donne à `ProjectBinding` l'autorité technique du projet,
SPEC-066 conserve Bridget et Maicie comme automates hôte uniques et SPEC-067
définit le projet comme frontière de confiance. Multiplier les timers ou les
daemons contredirait ces trois décisions.

## D5 - La ronde ne crée pas la vérité du travail

Décision: une ronde peut lancer un tour périodique explicite, mais elle ne
répare et ne recrée jamais une soumission perdue.

Raison: Anthropic recommande des composants simples et observables. Fusionner
surveillance périodique et recovery rendrait impossible de savoir si un travail
provient d'un humain ou d'une heuristique.

Source primaire:
- https://www.anthropic.com/engineering/building-effective-agents
