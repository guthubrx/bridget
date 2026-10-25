# Archives techniques de l’adaptateur T3

Document d’origine : [specs/081-conversation-renderer/tasks.md](../../../../../../specs/081-conversation-renderer/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [X] T001 Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les fixtures Node de tours multi-segments, Markdown hostile, tableau, fence, lien, chemin et image, avec identités stables et résultats attendus.
- [X] T002 [P] Vérifier puis documenter dans `crates/bridget-daemon/assets/ui/vendor/NOTICE.md`, `crates/bridget-daemon/assets/ui/vendor/SHA256SUMS` et `specs/081-conversation-renderer/implementation.md` la version, licence, hash et procédure de repli du colorateur choisi.
- [X] T003 [P] Ajouter dans `crates/bridget-daemon/assets/ui/vendor/NOTICE.md` l'avis MIT complet de T3 Tools Inc. et dans `specs/081-conversation-renderer/implementation.md` la table des helpers T3 réellement adaptés, sans attribuer de code non repris.
- [X] T004 Vérifier le point de départ avec `node crates/bridget-daemon/assets/ui/app.js`, `cargo test -p bridget-daemon ui`, les tests Desktop applicables et `git diff --check`, puis consigner seulement les résultats réels dans `specs/081-conversation-renderer/implementation.md`.

<a id="extrait-2"></a>

## Extrait 2

- [X] T014 [P] [US2] Adapter avec provenance MIT dans `crates/bridget-daemon/assets/ui/app.js` l'extraction du langage et du titre de fence issue de `apps/web/src/components/ChatMarkdown.tsx` de T3, avec tests pour métadonnées, fichier et langage inconnu.
- [X] T015 [P] [US2] Ajouter le colorateur local validé à `crates/bridget-daemon/assets/ui/vendor/`, le référencer depuis `crates/bridget-daemon/assets/ui/index.html` et prouver son chargement conditionnel, son hash et le repli texte brut dans `crates/bridget-daemon/assets/ui/app.js`.
- [X] T016 [US2] Remplacer le rendu brut des fences dans `crates/bridget-daemon/assets/ui/app.js` par un composant de bloc code avec titre ou langage, coloration thémée, bouton Copier, état de réussite ou échec accessible et copie exacte du texte source.
- [X] T017 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` le retour à la ligne local par bloc de code, indépendant de la préférence globale, et couvrir sélection, texte long et grammaire indisponible.
- [X] T018 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` la sérialisation de tableau Markdown et CSV, les boutons de copie et leurs tests, sans utiliser `innerHTML` non assaini ni perdre les cellules.
- [X] T019 [US2] Modifier `crates/bridget-daemon/assets/ui/theme.css` pour les blocs, contrôles, tableaux défilants, citations, alertes textuelles et états de copie dans les thèmes clair et sombre, en réutilisant les variables de police Bridget.
- [X] T020 [US2] Tester dans `crates/bridget-daemon/assets/ui/app.js` que contenu actif, événements HTML et attributs de lien restent refusés pendant le rendu enrichi, puis exécuter le programme Node.

<a id="extrait-3"></a>

## Extrait 3

- [X] T035 Retirer ou simplifier dans `crates/bridget-daemon/assets/ui/app.js` et `crates/bridget-daemon/assets/ui/theme.css` les chemins de rendu devenus inatteignables, puis prouver par les tests qu'aucune bulle doublon ni surface active ne subsiste.
- [X] T036 [P] Relire `crates/bridget-daemon/assets/ui/vendor/NOTICE.md`, `LICENSE.*` et `SHA256SUMS` pour vérifier les avis T3 et colorateur, puis documenter le résultat dans `specs/081-conversation-renderer/implementation.md`.
- [X] T037 [P] Exécuter `cargo fmt --check`, le programme Node, les tests Rust et Tauri ciblés, ainsi que `git diff --check`, puis inscrire chaque commande, succès ou échec réel dans `specs/081-conversation-renderer/evidence/validation.md`.
- [ ] T038 Jouer entièrement `specs/081-conversation-renderer/quickstart.md` sur Bridget Desktop et un serveur approuvé, avec thèmes clair et sombre, puis mettre à jour `spec.md`, `tasks.md`, `implementation.md` et `audit.md` sans cocher une preuve non observée.
- [X] T039 Tenter la contre-revue externe du diff via Bridget si le daemon est joignable ; sinon consigner objectivement son indisponibilité dans `specs/081-conversation-renderer/reviews/adversarial-code.md`. En cas de verdict, corriger les constats prouvés avant toute livraison.
