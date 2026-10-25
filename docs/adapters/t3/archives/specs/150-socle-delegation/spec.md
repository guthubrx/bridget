# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/spec.md](../../../../../../specs/150-socle-delegation/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le rejeu d'une demande dont l'arbre a été purgé rend un refus explicite
(`request_expired`). Il ne crée jamais une nouvelle tâche en silence. La vue
Lineage T3 ne montre pas de tâche fantôme après une purge.

<a id="extrait-2"></a>

## Extrait 2

Les tâches actives au moment de la mise à jour continuent avec leurs règles
d'origine, y compris leur délai de 3600 s. Les contrats MCP et CLI existants
restent compatibles. La vue Lineage T3 garde ses états et ses clés.

<a id="extrait-3"></a>

## Extrait 3

- **Tâche native** : demande de délégation d'un parent vers un enfant, avec
  état, délai, lignée, résultat et clé de rejeu.
- **Arbre de tâches** : une tâche sans parent et ses descendants.
- **Reçu d'annulation** : trace idempotente d'une annulation humaine.
- **Pierre tombale** : trace d'une clé de rejeu purgée, gardée 30 jours.
- **Projection de lignée** : vue lue par T3, avec génération et séquence.
- **État de contrôle** : pause ou activité du référent.

<a id="extrait-4"></a>

## Extrait 4

- La durée de conservation par défaut est de 7 jours, comme l'historique du
  ledger.
- La borne totale reste 4096 tâches, valeur déjà publiée aux lecteurs T3.
- Le délai de 86 400 s maximum couvre les tâches longues connues.
- Un résultat dont la remise reste indéterminée bloque la purge de son arbre.
  Si ces arbres saturent la borne, la délégation est refusée
  (`retention_saturated`) plutôt que de perdre un résultat.
- Les tests suivent la règle du projet : `umask 077`, environnement privé,
  racine temporaire courte.
- La livraison suit les sessions 147 à 149 : fichiers sur disque, sans relance
  automatique des services.
