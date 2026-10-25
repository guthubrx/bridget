# Archives techniques de l’adaptateur T3

Document d’origine : [specs/081-conversation-renderer/plan.md](../../../../../../specs/081-conversation-renderer/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.

<a id="extrait-2"></a>

## Extrait 2

| Élément | Valeur confirmée |
|---|---|
| Daemon et relais | Rust 2024, HTTP loopback authentifié, journal existant |
| Panneau conversation | HTML, CSS, JavaScript sans framework dans `crates/bridget-daemon/assets/ui/` |
| Markdown existant | `marked` 15.0.12 et DOMPurify 3.2.6 embarqués, sans CDN |
| Client macOS | Tauri v2, WebView externe par serveur relié, profil et préférences atomiques |
| Source de tours | `projectTimeline` conserve prompt, actes, texte, erreur et `turn_end` |
| Autorité chemins serveur | `ProjectRootPolicy`, racines canoniques et bornées |
| Source de référence | T3 Code, MIT, commit local inspecté `b1670ac7d` |

<a id="extrait-3"></a>

## Extrait 3

| Gate | Verdict | Justification |
|---|---|---|
| Branche de feature isolée | PASS | `081-conversation-renderer`, créée par le hook SpecKit avant les artefacts. |
| Réutilisation avant création | PASS | Journal, activité, assainissement, préférences Tauri et politique de racines sont étendus ; T3 MIT est inspecté et tracé. |
| Sécurité et secrets | PASS sous garde | Préférences locales, aucun nouveau secret, aucun shell, aucun lancement automatique, lecture de fichier sous racines canoniques. |
| Accessibilité | PASS | Boutons nommés, détails natifs ou équivalents, focus et états de copie observables. |
| Dette et complexité | PASS | Projection locale pure, pas de protocole agent nouveau ni portage React. |
| Décision structurante | PASS | ADR-022 créée pour la frontière WebView relayé / préférences natives. |
| DevKMS | WARN dégradé | La commande `mem` n'est pas installée dans cet environnement. Les décisions sont consignées dans ADR, contrats et artefacts SpecKit. |

<a id="extrait-4"></a>

## Extrait 4

1. Adapter, avec provenance MIT, l'extraction du langage et du titre de fence
   de T3 Code.
2. Créer le rendu de bloc code : en-tête, titre/langage, copie, retour à la
   ligne local au bloc, coloration différée, repli texte brut.
3. Créer le conteneur de tableau défilant avec copie Markdown/CSV et retour
   de succès accessible.
4. Ajouter une petite couche de présentation GFM : alertes GitHub non actives,
   citations, détails repliables et listes normalisées.
5. Enregistrer l'origine des portions T3 adaptées et les licences du
   colorateur dans les notices embarquées.

<a id="extrait-5"></a>

## Extrait 5

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.

<a id="extrait-6"></a>

## Extrait 6

1. Tests purs en premier : classification d'URL, normalisation de préférence,
   projection de tour, extraction de fences, sérialisation de tableau.
2. Tests hostile Markdown : script, attributs d'événements, URI active,
   `data:`, SVG, image distante et lien fichier hors racine.
3. Tests Rust de route : token, chemin relatif, root absent, symlink sortant,
   fichier trop grand, type interdit, réponse texte/image admise.
4. Tests Desktop : V1 vers V2, JSON corrompu, permissions, injection de
   snapshot et impossibilité pour le panneau de modifier le store.
5. Tests de comportement : pas de double message, pas de retour forcé au bas,
   copie exacte, contrôle clavier, thèmes clair/sombre.
6. Contrôle de licence : hash du vendor, licence du colorateur et avis T3
   conservé avant de livrer le code adapté.
