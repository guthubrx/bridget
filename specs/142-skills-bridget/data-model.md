# Modèle documentaire — SPEC142

Aucune base, migration, stockage de mission ou nouvelle donnée runtime.

## Entrées de compétence

| Identité canonique | Identité compatible | Source d'instructions | Scripts |
|---|---|---|---|
| `bridget` | `agent-bridge` | Dépôt Bridget | Scripts et CLI existants, aucun transport legacy relancé |
| `bridget-loop` | `agent-loop` | Source moderne Codex explicitement sélectionnée | Ancien dossier `agent-loop/scripts` |
| `bridget-handoff` | `agent-handoff-ledger` | Source Codex existante explicitement sélectionnée | Ancien dossier `agent-handoff-ledger/scripts` |

Chaque entrée a un `name` unique, une description française, un corps d'instructions et éventuellement `agents/openai.yaml`. Un alias a son ancien `name`, une description de compatibilité et un pointeur absolu vers le canon. Il ne contient pas une seconde implémentation.

## Métadonnées d'affichage

`display_name` est français et indique la compatibilité pour un alias. `short_description` fait 25 à 64 caractères. Le `default_prompt` des canons contient leur nom exact avec `$`. Les clés d'affichage ne changent pas les permissions ni l'invocation. Préserver les policies/dependencies existantes non liées à la présentation.

## Publication et sauvegarde

La correspondance source/cible est explicite pour les six noms. La date des fichiers n'est pas une autorité. La sauvegarde enregistre chemin original, cible résolue, type de fichier/lien et empreintes. Elle est stockée hors racines de découverte. Un test de seconde publication vérifie la stabilité des artefacts, liens et scripts.

États documentaires : `Planned` → `In Progress` → `Implemented` → publication vérifiée sur preuves. Un catalogue découvert ne prouve pas une exécution de modèle. Un commit n'est consigné qu'une fois réalisé et vérifié.
