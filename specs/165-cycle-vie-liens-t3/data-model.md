# Modèle de données — SPEC-165

## ThreadSummary

Vue interne d’un fil T3.

| Champ | Rôle | Source |
|---|---|---|
| `id` | Identité stable du fil | Snapshot T3 |
| `archived_at`, `deleted_at`, `settled_override` | Existence et visibilité T3 | Snapshot T3 |
| `relationship_to_parent` | `subagent`, `fork` ou absence pour une racine/ancien contrat | Lignée V2 |
| `latest_turn.state` | Dernier état de travail connu | Projection du run V2 |
| `session.active_turn_id` | Activité actuellement en cours | `activeRunId` V2 |

## État associé au lien

Ces valeurs ne sont pas des champs de `ThreadSummary`. Elles viennent de l'état Bridget ou de la boucle de réconciliation.

| Valeur | Rôle | Source |
|---|---|---|
| `has_obligation` | Corrélation ou réponse persistée qui doit finir | `ThreadState` durable |
| `drained_key` | Dernière empreinte entièrement traitée par le worker | État partagé du `Link` |
| `attempted_obligation` | Tentative de reprise déjà consommée dans ce processus | Mémoire volatile du pont |
| `drain_since` | Début non repoussable du drain courant | `Link` vivant |

## Décision d’exposition

La décision est calculée. Elle n’est pas persistée.

| Fil | État | Lien Bridget |
|---|---|---|
| Archivé, supprimé ou rangé | Tous | Fermé |
| Filiation absente | Vivant | Conservé pour compatibilité |
| Racine | Vivant, même terminal | Conservé |
| Fork | Vivant, même terminal | Conservé |
| Sous-agent | Activité courante | Conservé |
| Sous-agent | Préparation, file, démarrage, exécution ou attente | Conservé |
| Sous-agent | Terminal ou inactif avec obligation | Conservé pendant le drain |
| Sous-agent | Terminal ou inactif sans obligation | Fermé |

## Transitions

```text
sous-agent inactif ── activité non terminale ──> lien ouvert
lien ouvert ── état terminal ou inactif ──> dernier Tick puis drain
drain ── clé confirmée ou borne dure ──> fermeture complète
état persistant ── obligation non soldée ──> lien de drain rouvert
lien fermé ── nouvelle activité non terminale ──> lien rouvert
n’importe quel lien ── archive/suppression/rangement ──> fermeture complète
```

## Invariants

- Aucun changement d’état ne supprime l’historique.
- Une seule entrée de la collection possède un lien vivant par identifiant T3.
- La fermeture joint le worker, puis le lecteur réseau, avant la destruction du lien.
- Une clé n'est drainée que si le journal, la file, les corrélations et les réponses acquittables sont vides.
- Une obligation arrivée à la borne ne rouvre pas en boucle dans le même processus.
- Une remise ou une nouvelle attente invalide une clé drainée déjà publiée.
- Une mort imprévue relit l'état durable avant de décider une reprise bornée.
- Au plus 32 obligations possèdent un lien de drain au même instant.
- Une filiation inconnue ne provoque pas de régression de compatibilité.

