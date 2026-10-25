# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/research.md](../../../../../../specs/147-panneau-bridget-vivant/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le principal a conduit les explorations Bridget, runtime T3 et UI. Leurs preuves sont réutilisées ici. Un contrôle documentaire ciblé a confirmé la garde humaine, la distinction Done/Replayed, la résolution serveur, la subscription runtime et les reconstructions du store. Aucun deuxième balayage complet n'a été réalisé.

<a id="extrait-2"></a>

## Extrait 2

- [W3C WAI — On Focus](https://www.w3.org/WAI/WCAG21/Understanding/on-focus.html) : une prise de focus ne doit pas provoquer de changement de contexte. Décision locale : ne pas déplacer le focus lors d'une revalidation et conserver les contrôles inchangés. Cette page ne prouve pas une conformité complète de T3.
- [web.dev — Back/forward cache](https://web.dev/articles/bfcache?hl=en) : une page peut être suspendue puis restaurée. Décision locale : libérer le suivi quand la page n'est plus active, puis revalider à la reprise ; ne pas supposer que la connexion a livré tous les événements.
- [Testing Library — Guiding Principles](https://testing-library.com/docs/guiding-principles/) : les tests utiles ressemblent à l'usage du logiciel. Décision locale : vérifier sélection, contenu, copie et focus avec les contrôles visibles, pas seulement un compteur de mocks.
- [The Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html) : répartir tests rapides, intégration et parcours complets. Décision locale : tester le transport et la garde aux frontières, puis un parcours réel isolé ; éviter une recette exclusivement simulée.

<a id="extrait-3"></a>

## Extrait 3

Décision : Rust et Vitest/Testing Library/Effect, avec scénario intégré isolé. Les standards génériques Next.js, Axios et Pytest ne remplacent pas les outils réels de T3/Bridget.

<a id="extrait-4"></a>

## Extrait 4

Pourquoi : ces prescriptions visent d'autres projets. Ajouter leurs runtimes ou restructurer T3 ne testerait pas le transport effectivement livré.

<a id="extrait-5"></a>

## Extrait 5

Cause vérifiée en lecture seule par le principal/RPC : agent regional-wrkr-1, thread1288a673-d16e-473a-80dc-def0b2f776c8, adapter ClaudeAgent, provider claude_glm, modèle glm-5.3-flash. Home effectif /Users/moi/.claude-glm, exécutable gclaude, launchArgs sans flags. Les noms MCP home/projet sont vides. La commande `CLAUDE_CONFIG_DIR=/Users/moi/.claude-glm /Users/moi/.local/bin/claude mcp get bridget`, cwd /Users/moi/Nextcloud/10.Scripts/69.opus2D, sort RC1 et confirme l'absence de Bridget. Sortie assainie64 octets, raw non conservé. T3 injecte seulement t3-code ; ce constat ne signifie pas que GLM est interdit.
