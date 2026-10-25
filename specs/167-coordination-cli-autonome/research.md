# Recherche — SPEC-167

## Question

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/167-coordination-cli-autonome/research.md#extrait-1).

## Sources locales

- `/Users/moi/.speckit/research/01-ai-agents-agentic-ai.md`
- `/Users/moi/.speckit/research/03-cognitive-load-productivity.md`
- `/Users/moi/.speckit/research/04-architectures-patterns.md`
- `/Users/moi/.speckit/ref/agent-orchestration.md`
- `/Users/moi/.speckit/ref/standards-tests.md`

## Sources officielles externes

- Anthropic, *Building effective agents* : https://www.anthropic.com/engineering/building-effective-agents
- Anthropic, *Trustworthy agents in practice* : https://www.anthropic.com/research/trustworthy-agents
- OpenAI, *Agents SDK* : https://developers.openai.com/api/docs/guides/agents/sdk
- OpenAI, *Orchestration and handoffs* : https://developers.openai.com/api/docs/guides/agents/orchestration

## Décisions

### Une façade mince est préférable à un orchestrateur neuf

Les sources recommandent des composants simples et composables. Elles recommandent aussi un coordinateur qui conserve le contrôle du travail spécialisé. Bridget possède déjà cette architecture avec sa délégation native. La CLI doit donc traduire une intention en requête native. Elle ne doit pas piloter elle-même les agents.

### Le parent doit rester un agent authentifié

La coordination interagent exige une propriété claire des tâches. Un terminal humain n’est pas un agent. Lui fabriquer une identité brouillerait les droits, la filiation et la remise des résultats. La solution garde l’identité vivante actuelle et refuse l’absence d’identité avant tout effet.

### Le parcours nominal doit demander une seule donnée métier

La charge cognitive baisse quand le chemin principal expose la mission et masque les paramètres facultatifs. Les paramètres avancés restent disponibles. Ils ne deviennent pas des valeurs implicites quand ils sont absents.

### Les résultats doivent être durables et pilotables

Une coordination fiable ne se limite pas au lancement. Elle exige des identifiants stables, une attente sans polling, une continuation, une réponse aux demandes et une annulation contrôlée. Ces primitives existent déjà dans Bridget.

## Alternatives rejetées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/167-coordination-cli-autonome/research.md#extrait-2).

## Conclusion

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/167-coordination-cli-autonome/research.md#extrait-3).
