# Recherche technique - SPEC-071

## Faits mesurés dans le dépôt

1. `AgentInfo` contient déjà `agent_type`, `transport`, `mode`, `model` et
   `effort`. La projection `UiAgentRowV1` ne transmet actuellement que
   `agent_type` parmi ces cinq faits.
2. `PresenceMode` possède exactement trois valeurs filaires : `acp`, `tmux`
   et `cli`. Le commentaire du protocole interdit de reconstruire une valeur
   absente.
3. Le tooltip actuel est un enfant de la ligne et ne contient que le dernier
   extrait. `.agent-pane` applique `overflow-x: hidden`, ce qui impose une
   fiche rendue hors de ce sous-arbre pour apparaître proprement à côté.
4. `ui.rs` sert déjà les assets embarqués avec ETag et `Cache-Control:
   no-cache`. Le même mécanisme suffit pour les logos.
5. La charte testée interdit les bordures visibles, ombres et gradients. La
   fiche utilisera donc uniquement surface, rayon, espacement et typographie.

## Clarification sémantique

Le besoin initial emploie « fournisseur » pour Codex, Claude, Cursor et
Gemini. Ces noms désignent des produits agentiques, pas nécessairement le
fournisseur du modèle exécuté. Cursor peut par exemple router vers plusieurs
fournisseurs de modèles.

La fiche affiche donc :

- le runtime explicite et son éditeur, via un catalogue local audité ;
- le modèle observé sous forme de libellé brut ;
- le mode et le transport observés.

Elle n'affiche pas un fournisseur de modèle non attesté. Cette séparation
évite une information fausse sans nécessiter une évolution du registre.

## Sources officielles des marques

| Produit | Source officielle | Asset retenu |
|---|---|---|
| Codex | https://openai.com/brand/ et https://cdn.openai.com/brand/OpenAI-Logos-2025.zip | symbole OpenAI sombre ou clair fourni dans le kit, sans modification |
| Claude Code | https://www.anthropic.com/news et https://www.anthropic.com/press-kit | Claude Spark SVG du kit presse Anthropic |
| Cursor | https://cursor.com/brand et https://ptht05hbb1ssoooe.public.blob.vercel-storage.com/assets/brand/cursor-brand-assets.zip | cube 2D, variante adaptée au fond sombre |
| Gemini CLI | https://gemini.google.com/ et https://www.gstatic.com/lamda/images/gemini_sparkle_v002_d4735304ff6292a690345.svg | sparkle Gemini vectoriel servi par le CDN officiel Google |

Les guides OpenAI et Cursor demandent de ne pas modifier les marques. Cursor
demande également le nom exact « Cursor ». Les fichiers retenus sont donc
copiés octet pour octet, sans recoloration ni effet.

La variante Gemini Aurora courante embarque une miniature JPEG en base64. Elle
a été écartée au profit de la variante vectorielle officielle `v002`, plus
petite et sans contenu raster actif ou embarqué.

## Accessibilité de la fiche

Le pattern Tooltip de WAI-ARIA prévoit une ouverture au survol ou au focus,
une fermeture avec Échap, le maintien du focus sur le déclencheur et une
relation `aria-describedby`.

La règle WCAG 1.4.13 exige qu'un contenu apparu au survol soit fermable,
survolable et persistant tant que l'utilisateur le consulte.

- Pattern : https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/
- Compréhension WCAG : https://www.w3.org/WAI/WCAG21/Understanding/content-on-hover-or-focus
- Technique de tooltip scripté : https://www.w3.org/WAI/WCAG22/Techniques/client-side-script/SCR39

## Réutilisation confirmée

| Besoin | Existant | Décision |
|---|---|---|
| Métadonnées runtime | `AgentInfo` dans `bridget-transport` | Réutiliser, sans modifier le protocole de présence. |
| Projection navigateur | `UiAgentRowV1`, `compose_agent_rows` | Étendre avec quatre champs existants. |
| Normalisation | `normalizeAgentRow` | Étendre avec des valeurs absentes explicites. |
| Rendu de ligne | `renderAgentButton` | Conserver sélection, avatar et contenu permanent. |
| Overlay | `.agent-row__tooltip` | Remplacer par une fiche globale, car le parent coupe horizontalement. |
| Assets statiques | `include_bytes!`, `write_asset` | Réutiliser pour quatre routes SVG. |
| Tests UI | tests Node embarqués dans `app.js` | Étendre la suite existante. |
| Tests relais | module Rust `ui.rs` et `ui_relay_test.rs` | Étendre les témoins de projection et d'assets. |

## Risques et garde-fous

- Confusion éditeur/modèle : libellés distincts et aucun fournisseur de modèle
  reconstruit.
- Marque illisible sur fond sombre : utiliser la variante officielle prévue,
  jamais une recoloration CSS.
- SVG actif : refuser script, référence externe, événement ou contenu raster
  embarqué avant intégration.
- Fiche instable : délai de fermeture court et position calculée après mesure.
- Régression de sélection : les événements d'ouverture ne déclenchent pas le
  clic de la ligne.

## Recherche DevKMS

La commande `mem` n'est pas installée sur le serveur. Aucune connaissance
existante n'a donc pu être consultée ni enrichie. Les résultats sont conservés
dans cet artefact.
