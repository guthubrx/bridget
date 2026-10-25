# Archives techniques de l’adaptateur T3

Document d’origine : [specs/167-coordination-cli-autonome/plan.md](../../../../../../specs/167-coordination-cli-autonome/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Parser nominal : mission seule, dossier courant, clé générée.
- Options : type, modèle, effort, posture, délai, garde, demande enfant, urgence et livrable.
- Refus : option inconnue, doublon, mission vide, borne dépassée et identité absente.
- Cycle : chaque sous-commande produit le nom d’outil et la charge attendus.
- Rejeu : une clé fournie est conservée à l’octet près.
- Sortie : JSON unique sur stdout ; vue humaine avec identifiants utiles.
- Compatibilité : tests existants de `auto-delegate`, délégation MCP et garde CLI.
- Recette : agent Bridget actif, T3 arrêté, délégation puis status/wait/ack.
