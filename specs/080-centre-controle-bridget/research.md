# Recherche - SPEC-080 Centre de contrôle Bridget

## Décision 1 - Le panneau de serveur est la surface des réglages serveur

**Décision:** placer l'engrenage dans la barre gauche des assets UI du daemon, car ce panneau est ouvert par Bridget Desktop à travers un tunnel SSH propre à un seul profil.

**Rationale:** le client macOS gère déjà l'identité SSH, le tunnel et les profils locaux. Le relais distant possède l'état du daemon et les routes HTTP versionnées. Déplacer les réglages serveur dans le client dupliquerait un proxy, des autorisations et un état réseau qui existent déjà.

**Alternatives considérées:** une page globale dans le client macOS a été écartée pour les réglages serveur car elle mélangerait plusieurs serveurs et demanderait de nouvelles requêtes de transport. Un terminal SSH embarqué a été écarté pour des raisons de sécurité et de périmètre.

**Impact mainteneur:** un seul panneau représente un seul serveur. La correspondance profil SSH - tunnel - relais reste visible et testable.

## Décision 2 - Catalogue fermé et confirmation humaine locale

**Décision:** n'exposer que des clés de réglage connues et typées. Toute écriture passe par une prévisualisation, une confirmation dans le WebView local, une génération attendue et un reçu.

**Rationale:** la spécification MCP recommande de conserver un humain capable de refuser les invocations d'outils et de limiter les surfaces d'autorisation. Le relais Bridget a déjà une frontière d'identité et un jeton transporté uniquement dans l'URL locale du panneau.

**Alternatives considérées:** éditeur de JSON, API générique clé-valeur et approbation par agent rejetés. Ils permettraient de transformer la page en administration de l'hôte ou de confondre l'intention d'un agent avec celle de l'opérateur.

**Impact mainteneur:** ajouter un réglage impose d'ajouter explicitement une clé, son schéma, sa validation, son aperçu, son test et son rendu. Il n'existe aucune échappatoire générique.

Sources live: [MCP Tools](https://modelcontextprotocol.io/specification/draft/server/tools), [MCP Authorization](https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization). Consultées le 2026-08-31.

## Décision 3 - Usage observé avant coût estimé

**Décision:** réutiliser `usage_samples` horodatés comme source primaire. Afficher les coûts seulement lorsqu'un tarif daté applicable et des dimensions attestées existent.

**Rationale:** l'API Usage d'OpenAI distingue activité, coûts et facture, et indique que les comptages de flux peuvent manquer si le flux est interrompu. Les jetons cachés et cache ne se déduisent pas de la longueur visible d'une réponse.

**Alternatives considérées:** calcul à partir des messages, prix par défaut, assimilation d'un abonnement à une facture et conservation du dernier modèle connu ont été rejetés. Ils inventeraient des données.

**Impact mainteneur:** une nouvelle source d'usage peut être ajoutée de façon additive. L'absence de modèle ou de tarif reste une valeur explicite de produit, pas une anomalie à masquer.

Sources live: [OpenAI - Reviewing API usage and costs](https://help.openai.com/en/articles/10478918-api-usage-dashboard), [OpenAI - Usage API](https://platform.openai.com/docs/api-reference/usage/audio_transcriptions_object). Consultées le 2026-08-31.

## Baseline locale appliquée

- `04-architectures-patterns.md`: éviter de créer un nouveau framework d'agents ou de protocole alors que le relais Bridget porte déjà le transport et l'observabilité.
- `01-ai-agents-agentic-ai.md`: éviter les promesses d'autonomie complète et exposer les limites de coût et de contrôle humain.
- `06-security-compliance.md` et `10-data-privacy.md`: secret, prompt et contenu d'agent restent hors projections, diagnostic et métriques.

## Questions résolues

| Question initiale | Réponse |
|---|---|
| Où se trouve la barre demandée? | Dans le panneau web du daemon, pas dans la première fenêtre de profils Tauri. |
| Existe-t-il une écriture atomique réutilisable? | Oui, `ProjectRootPolicy::replace_atomically` avec génération attendue. |
| L'usage est-il déjà historisé? | Oui, `usage_samples` horodatés; il faut enrichir les dimensions, pas créer une seconde source de vérité. |
| Peut-on afficher un coût fiable aujourd'hui? | Pas sans tarif versionné et modèle attesté. L'écran doit l'indiquer. |
| Peut-on modifier des paramètres de serveur? | Oui, si et seulement si le serveur expose une clé fermée dotée d'une validation et d'une application atomique. |
