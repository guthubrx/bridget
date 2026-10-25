# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/data-model.md](../../../../../../specs/165-cycle-vie-liens-t3/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Vue interne d’un fil T3.

<a id="extrait-2"></a>

## Extrait 2

| Champ | Rôle | Source |
|---|---|---|
| `id` | Identité stable du fil | Snapshot T3 |
| `archived_at`, `deleted_at`, `settled_override` | Existence et visibilité T3 | Snapshot T3 |
| `relationship_to_parent` | `subagent`, `fork` ou absence pour une racine/ancien contrat | Lignée V2 |
| `latest_turn.state` | Dernier état de travail connu | Projection du run V2 |
| `session.active_turn_id` | Activité actuellement en cours | `activeRunId` V2 |

<a id="extrait-3"></a>

## Extrait 3

- Aucun changement d’état ne supprime l’historique.
- Une seule entrée de la collection possède un lien vivant par identifiant T3.
- La fermeture joint le worker, puis le lecteur réseau, avant la destruction du lien.
- Une clé n'est drainée que si le journal, la file, les corrélations et les réponses acquittables sont vides.
- Une obligation arrivée à la borne ne rouvre pas en boucle dans le même processus.
- Une remise ou une nouvelle attente invalide une clé drainée déjà publiée.
- Une mort imprévue relit l'état durable avant de décider une reprise bornée.
- Au plus 32 obligations possèdent un lien de drain au même instant.
- Une filiation inconnue ne provoque pas de régression de compatibilité.
