# Archives techniques de l’adaptateur T3

Document d’origine : [specs/081-conversation-renderer/implementation.md](../../../../../../specs/081-conversation-renderer/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Licence MIT complète ajoutée sous `vendor/LICENSE.t3code.MIT.txt`.
- Référence inspectée : `/Users/moi/11.Repositories/t3code`, commit
  `b1670ac7d`, `apps/web/src/components/ChatMarkdown.tsx`.
- Adaptations directes prévues : uniquement l'analyse de métadonnées de fence
  `langage + titre`, avec commentaire de provenance à côté du helper final.
- Les principes de tours, documents agent et activités repliables inspirent
  l'interface mais ne reprennent pas de code React T3.

<a id="extrait-2"></a>

## Extrait 2

- Pourquoi cette solution est nécessaire : les fixtures, l'intégrité des
  dépendances et la baseline empêchent de confondre une régression SPEC-081
  avec l'existant.
- Pourquoi elle reste simple : une fixture immuable, un seul moteur local et
  les fichiers de notices déjà présents remplacent un nouveau harnais ou une
  chaîne de construction JavaScript.
- Hypothèses prises : `highlight.js` reste disponible avant `app.js` dans le
  panneau ; son absence conserve du texte brut. La socket Unix du worktree
  temporaire explique les échecs Rust de baseline observés.
- Vérifications réalisées : 101 tests Node, cinq empreintes SHA-256 valides,
  cinq tests Tauri ciblés réussis et lecture de la licence T3 locale.
- Non vérifié : le rendu graphique final et la suite Rust complète depuis un
  chemin court seront rejoués dans les preuves finales.
- Code supprimé ou évité : aucune pile React, aucun CDN, aucun chargeur de
  dépendance à l'exécution.
- Complexité ajoutée et justification : environ 132 Ko de ressources locales
  pour une coloration hors ligne et déterministe, avec licences et hashes.

<a id="extrait-3"></a>

## Extrait 3

- `extractFenceLanguage` et `extractFenceTitle` adaptent uniquement
  l'extraction de métadonnées de fence observée dans T3 Code
  `ChatMarkdown.tsx`, avec provenance MIT à côté du code.
- `highlight.js` 11.12.0 et ses deux thèmes GitHub sont embarqués localement.
  Un langage absent ou le moteur indisponible conserve le code en texte brut,
  sans affecter la copie exacte.
- Les blocs de code ont un titre ou langage, une copie accessible et un retour
  à la ligne indépendant. Les tableaux gardent leur défilement horizontal et
  peuvent être copiés en Markdown ou CSV.
- `node crates/bridget-daemon/assets/ui/app.js` : succès, 110 tests dont
  assainissement hostile, copie, thème et contenu actif refusé.
