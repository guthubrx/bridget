# Modèle de données — SPEC-165

## ThreadSummary

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/data-model.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/data-model.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/data-model.md#extrait-3).
