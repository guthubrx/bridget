# Recherche — SPEC-167

## Question

Comment rendre Bridget utile comme coordinateur interagent depuis une ligne de commande, sans reproduire T3 et sans créer un second moteur de délégation ?

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

| Alternative | Motif du rejet |
|---|---|
| Reproduire le coordinateur T3 dans Bridget | Hors métier. Cela ajouterait une UI et une seconde politique. |
| Créer un principal humain synthétique | Il n’aurait ni présence d’agent ni autorité de remise correcte. |
| Combiner `spawn` puis `send` | Deux effets sans transaction durable. Aucun reçu de tâche natif commun. |
| Ajouter un daemon de coordination | Le daemon Bridget existe déjà. Un second service augmenterait les pannes. |
| Exposer seulement des alias shell | Les sorties, erreurs et clés de rejeu resteraient incohérentes. |

## Conclusion

La bonne unité de conception est une façade CLI vers les outils natifs existants. T3 reste un client possible. Il n’est ni la source d’identité Bridget, ni le moteur de tâche.
