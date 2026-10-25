# Archives techniques de l’adaptateur T3

Document d’origine : [specs/167-coordination-cli-autonome/validation/release-notes-0.3.0.md](../../../../../../../specs/167-coordination-cli-autonome/validation/release-notes-0.3.0.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `bridget delegate <MISSION>` permet à un agent Bridget authentifié de créer un enfant et de lui confier une mission en une commande.
- `bridget task` permet de lister, lire, attendre, continuer, répondre, annuler et accuser lecture depuis un terminal.
- Le parcours réutilise le moteur natif de Bridget. T3 reste compatible, mais il n’est pas requis.
- Les choix avancés restent facultatifs. Un choix explicite indisponible est refusé sans substitution silencieuse.
- La sortie `--json` fournit un objet stable pour les scripts.
