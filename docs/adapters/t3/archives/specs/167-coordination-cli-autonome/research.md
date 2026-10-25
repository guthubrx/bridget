# Archives techniques de l’adaptateur T3

Document d’origine : [specs/167-coordination-cli-autonome/research.md](../../../../../../specs/167-coordination-cli-autonome/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Comment rendre Bridget utile comme coordinateur interagent depuis une ligne de commande, sans reproduire T3 et sans créer un second moteur de délégation ?

<a id="extrait-2"></a>

## Extrait 2

| Alternative | Motif du rejet |
|---|---|
| Reproduire le coordinateur T3 dans Bridget | Hors métier. Cela ajouterait une UI et une seconde politique. |
| Créer un principal humain synthétique | Il n’aurait ni présence d’agent ni autorité de remise correcte. |
| Combiner `spawn` puis `send` | Deux effets sans transaction durable. Aucun reçu de tâche natif commun. |
| Ajouter un daemon de coordination | Le daemon Bridget existe déjà. Un second service augmenterait les pannes. |
| Exposer seulement des alias shell | Les sorties, erreurs et clés de rejeu resteraient incohérentes. |

<a id="extrait-3"></a>

## Extrait 3

La bonne unité de conception est une façade CLI vers les outils natifs existants. T3 reste un client possible. Il n’est ni la source d’identité Bridget, ni le moteur de tâche.
