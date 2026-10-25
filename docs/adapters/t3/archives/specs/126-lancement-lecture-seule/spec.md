# Archives techniques de l’adaptateur T3

Document d’origine : [specs/126-lancement-lecture-seule/spec.md](../../../../../../specs/126-lancement-lecture-seule/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Vérifications du 2026-10-02 :
- `bridget spawn claude` sans `--posture`, depuis une session sans terminal : accepté, type
  `project-discovery-claude` (lecture seule), arrêté proprement par `bridget stop`.
- `bridget spawn glm` : refusé « capacité manquante posture_decouverte » : aucun type `glm` dans le
  registre du daemon (`~/.cache/bridget-core/agents.json` absent ; types par défaut codex, claude,
  cursor, gemini), et la clé `ZAI_API_KEY` du lanceur `gclaude` n'est connue que de T3.
