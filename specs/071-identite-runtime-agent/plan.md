# Plan - SPEC-071

## Décision de conception

Étendre la projection UI existante avec les faits déjà présents dans
`AgentInfo`, puis remplacer le tooltip de message par une fiche d'identité
unique rendue au niveau du document. Le catalogue d'identité reste une table
pure dans `app.js` : il associe un `agent_type` explicite à un nom de produit,
un éditeur et un asset local. Il ne déduit jamais le fournisseur réel du
modèle.

Aucun service, endpoint métier, stockage ou dépendance n'est créé.

## Architecture cible

```text
AgentInfo existant
  agent_type, transport, mode, model, effort
                    |
                    v
UiAgentRowV1 étendu dans ui.rs
                    |
                    v
normalizeAgentRow dans app.js
                    |
        +-----------+-----------+
        |                       |
        v                       v
catalogue runtime pur     mode explicite
nom + éditeur + logo      TMUX / FLUX / inconnu
        |                       |
        +-----------+-----------+
                    |
                    v
fiche globale role=tooltip, positionnée dans la fenêtre
```

## Lots réversibles

1. Ajouter des tests Rust qui prouvent que `transport`, `mode`, `model` et
   `effort` traversent la projection UI, puis étendre `UiAgentRowV1`.
2. Ajouter les fonctions pures de normalisation du runtime et du mode dans
   `app.js`, avec une matrice Codex, Claude Code, Cursor, Gemini CLI et inconnu.
3. Importer les quatre SVG officiels, les servir par le mécanisme d'assets
   existant et documenter leur provenance et leur intégrité.
4. Remplacer `.agent-row__tooltip` par une fiche globale compacte, sans
   bordure ni ombre, ouverte au survol et au focus, survolable, persistante et
   fermable avec Échap.
5. Vérifier la matrice fonctionnelle, l'accessibilité clavier, le
   repositionnement aux bords et les non-régressions UI/Rust.

## Sémantique des données

- `agent_type` identifie le runtime. Seules les valeurs normalisées exactes
  du catalogue sont reconnues. Le nom de l'agent n'est jamais consulté.
- `mode=tmux` produit la puce `TMUX`.
- `mode=acp` produit la puce `FLUX`, car ACP est un chemin géré.
- `mode=cli` produit `FLUX` seulement si le transport explicite est un
  transport géré connu, actuellement `codex_app_server` ou
  `claude_stream_json`.
- Toute autre combinaison produit `MODE INCONNU`.
- `model` et `effort` sont des libellés attestés. Le fournisseur du modèle
  n'est pas reconstruit depuis leur texte.

## Rendu et accessibilité

- Une seule fiche est ajoutée sous `body` afin d'échapper à
  `overflow-x: hidden` de `.agent-pane`.
- La fiche est positionnée en `fixed` près de la ligne et se retourne à gauche
  ou vers le haut si l'espace manque.
- Le bouton reçoit `aria-describedby` pendant l'ouverture ; la fiche porte
  `role=tooltip`.
- Un court délai de fermeture permet de déplacer le pointeur de la ligne vers
  la fiche. Le focus reste sur la ligne.
- Échap ferme la fiche sans déclencher `selectAgent`.
- Les logos sont des `<img>` décoratifs avec libellé visible adjacent.

## Fichiers touchés

| Fichier | Rôle |
|---|---|
| `crates/bridget-daemon/src/ui.rs` | projection des métadonnées déjà attestées et routes SVG |
| `crates/bridget-daemon/assets/ui/app.js` | catalogue, normalisation, fiche globale, positionnement et tests Node |
| `crates/bridget-daemon/assets/ui/theme.css` | hiérarchie compacte, puce de mode et responsive sans bordure |
| `crates/bridget-daemon/assets/ui/providers/*.svg` | marques officielles servies localement |
| `crates/bridget-daemon/assets/ui/providers/NOTICE.md` | provenance, date, source, intégrité et règles de marque |
| `crates/bridget-daemon/tests/ui_relay_test.rs` | contrat de projection si le test d'intégration existant est le meilleur point d'appui |

## Garde-fous

- Aucun suffixe `-flux`, avatar ou contenu de message n'identifie le runtime
  ou le mode.
- Cursor reste Cursor, même si son modèle contient un nom Anthropic, OpenAI ou
  Google.
- Les SVG ne contiennent ni script, ni référence externe, ni gestionnaire
  d'événement.
- Aucun chargement tiers n'a lieu lors du rendu.
- Le catalogue n'est pas présenté comme une preuve du fournisseur du modèle.
- Le clic sur une ligne conserve exactement le comportement de sélection
  existant.

## Validation

- Node : matrice de catalogue et de modes, données absentes, aucune inférence
  par nom, libellés accessibles et fermeture Échap.
- Node/CSS : une fiche globale, largeur maximale de 360 pixels, dernier
  message borné à deux lignes, aucune bordure visible, ombre ou gradient.
- Rust : les quatre faits traversent `compose_agent_rows` et les routes SVG
  utilisent `write_asset` avec `image/svg+xml`, ETag et `no-cache`.
- Sécurité : contrôle statique de chaque SVG importé.
- Manuel : 1280x720, zoom 200 %, première et dernière ligne, pointeur et
  clavier.

## Déploiement

La modification est compilée dans le binaire `bridget` par `include_bytes!`.
Elle ne devient visible qu'après compilation et redémarrage explicite du
relais UI. La SPEC ne réalise aucun déploiement automatique.
