# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/contracts/presentation.md](../../../../../../../specs/164-conversation-t3-distante/contracts/presentation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Ce contrat est interne au connecteur client T3. Il ne change aucun schéma public. Les événements source restent ceux de BridgetJournalEvent ; le statut métier reste celui de la tâche native. La projection ne fait aucun I/O et ne persiste aucun état.

<a id="extrait-2"></a>

## Extrait 2

Le contexte de machine distante est explicite. Un chemin `/home/moi/...` est copiable ; aucun accès au `/home/moi/...` local de l'environnement T3 n'est induit. Aucun iframe, HTML brut, script, bouton d'exécution ou chargement automatique de média d'un outil. Réutiliser les gardes Markdown T3 et une prop opt-in inerte web. Le renderer mobile entier lié aux threads/actions n’est pas exporté : reprendre ses primitives Markdown natives, règles et styles dans un contexte inerte. Aucune dérogation au rendu Markdown mobile. Si son composant complet ne permet pas ce contexte, extraire/reprendre seulement le rendu sûr utile sans nouvelle bibliothèque ni couplage au runtime fournisseur.
