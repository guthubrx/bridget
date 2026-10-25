# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/research.md](../../../../../../specs/143-echanges-discrets/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Question | Source à lire | Décision après preuve |
|---|---|---|
| Quels noms MCP et payloads atteignent la timeline ? | Discovery du principal, `PlainWorkEntryRow:4915` | Envoi Codex et Claude seulement, formes ci-dessous |
| Où se trouvent destinataire et résultat brut ? | Paramètres `to`/`body` et résultat de la même work entry | Nom attesté/ID bref ; pas d'annuaire réseau ajouté |
| Quels styles sont hérités ? | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:2125` et `:2130` | Retirer bordure et `backgroundColor` seulement sur la branche Bridget |
| Quels tests couvrent déjà copie/toggle ? | Deux suites `MessagesTimeline` existantes | Étendre les tests, pas recréer un harness |

<a id="extrait-2"></a>

## Extrait 2

La revue indépendante signale que le helper natif de détection d'échec ne lit pas toolData : ne pas compter sur lui pour détecter les drapeaux MCP ou refus. Lecture confirmée de `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:4601` : `buildToolCallExpandedBody` applique `trim`, puis sérialise toolData par `JSON.stringify`. Les valeurs JSON restent accessibles au dépliage ; leur ordre/espacement sérialisé n'est pas un contrat d'octets. Corps entrants et copie restent exacts.
