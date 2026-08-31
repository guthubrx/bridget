# Plan d'implémentation - SPEC-081 Conversation structurée et rendu technique sûr

## Résumé

Faire évoluer le fil Bridget en une suite de tours explicites inspirée de T3
Code : demande humaine compacte à droite, réponse d'agent éditorialisée à
gauche, activités et résultats repliables dans le même tour. Le Markdown GFM
reste assaini, puis gagne une présentation de code, tableaux et copie. Les
liens, références de fichiers et images sont des capacités de consultation
locales, distinctes et désactivées par défaut.

Le réemploi T3 est sélectif : structure de liste, détection de métadonnées de
blocs et ergonomie de copie. L'UI statique Bridget garde ses fonds, ses gris,
son overlay et son modèle de déploiement. Toute copie directe conserve la
notice MIT T3 Tools Inc.

## Contexte technique

| Élément | Valeur confirmée |
|---|---|
| Daemon et relais | Rust 2024, HTTP loopback authentifié, journal existant |
| Panneau conversation | HTML, CSS, JavaScript sans framework dans `crates/bridget-daemon/assets/ui/` |
| Markdown existant | `marked` 15.0.12 et DOMPurify 3.2.6 embarqués, sans CDN |
| Client macOS | Tauri v2, WebView externe par serveur relié, profil et préférences atomiques |
| Source de tours | `projectTimeline` conserve prompt, actes, texte, erreur et `turn_end` |
| Autorité chemins serveur | `ProjectRootPolicy`, racines canoniques et bornées |
| Source de référence | T3 Code, MIT, commit local inspecté `b1670ac7d` |

**Langages** : Rust 2024, JavaScript navigateur moderne, CSS.

**Dépendances prévues** : composants existants, un colorateur JavaScript local
et sous licence compatible si son audit de paquetage passe. Aucun framework,
CDN, terminal ni service distant supplémentaire.

**Stockage** : journal SQLite déjà présent ; `preferences.json` Tauri local
0600 ; fallback `localStorage` versionné pour un panneau hors Tauri.

**Tests** : tests Rust de relais et lecture bornée, tests Tauri du store local,
programme Node déjà intégré à `app.js`, contrôle de licence et vérification
visuelle de `quickstart.md`.

**Plateformes** : macOS Bridget Desktop et panneau relayé dans navigateur.

**Objectif de performance** : rendu initial sans opération réseau secondaire ;
une mise à jour de tour restaure précisément l'ancre de lecture si le DOM est
réaffiché ; aperçu fichier plafonné et rejeté avant lecture complète si
dépassement connu.

## Constitution check initial

| Gate | Verdict | Justification |
|---|---|---|
| Branche de feature isolée | PASS | `081-conversation-renderer`, créée par le hook SpecKit avant les artefacts. |
| Réutilisation avant création | PASS | Journal, activité, assainissement, préférences Tauri et politique de racines sont étendus ; T3 MIT est inspecté et tracé. |
| Sécurité et secrets | PASS sous garde | Préférences locales, aucun nouveau secret, aucun shell, aucun lancement automatique, lecture de fichier sous racines canoniques. |
| Accessibilité | PASS | Boutons nommés, détails natifs ou équivalents, focus et états de copie observables. |
| Dette et complexité | PASS | Projection locale pure, pas de protocole agent nouveau ni portage React. |
| Décision structurante | PASS | ADR-022 créée pour la frontière WebView relayé / préférences natives. |
| DevKMS | WARN dégradé | La commande `mem` n'est pas installée dans cet environnement. Les décisions sont consignées dans ADR, contrats et artefacts SpecKit. |

## Architecture cible

```text
Journal de conversation
        |
        v
projectTimeline (faits déjà attestés)
        |
        v
deriveConversationTurns (projection pure et stable)
        |
        +--> message humain à droite
        +--> document agent à gauche
        +--> activité/résultat repliable
        +--> ronde/échange pair distinct

Markdown brut
        |
        v
marked GFM -> DOMPurify strict -> classificateur de références
        |                               |
        |                               +--> lien HTTPS explicite
        |                               +--> aperçu fichier relayé borné
        |                               +--> image HTTPS différée
        v
code/table/copie/coloration avec repli texte brut

Préférences locales
        |
        +--> navigateur : localStorage V2
        +--> Desktop : preferences.json Tauri
                          |
                          +--> injection lecture seule au panneau
                          +--> événement de rafraîchissement après save
```

## Plan par lots

### Lot 1 - Fondations de préférence et sécurité

1. Étendre les préférences du centre de contrôle avec
   `ContentSecurityPreferencesV1`, normalisation stricte et migration.
2. Étendre le store Tauri atomique, le dialogue de préférences et ses
   commandes sans y introduire de profil, jeton ou donnée de conversation.
3. Injecter un snapshot immuable dans un WebView relayé et transmettre les
   mises à jour depuis la coque, sans capability d'écriture du panneau.
4. Ajouter les tests de défaut sûr, corruption, migration de l'opérateur
   actuel et absence de requête relay lors d'une préférence.

### Lot 2 - Projection et fil de conversation

1. Extraire `deriveConversationTurns` de l'état rendu, couvrir les textes
   entre actes, erreurs, tours ouverts, rondes et dédoublonnage.
2. Remplacer la paire uniforme `message + bubble` par les présentations
   humaines et agent, en conservant les statuts de remise et l'activité.
3. Rendre le résumé d'activité repliable par tour, sans masquer les échecs ni
   déplacer la lecture de l'opérateur.
4. Conserver les identifiants de tour, restaurer précisément l'ancre lors d'un
   ré-affichage, préserver le brouillon et ajouter un bouton de retour au
   direct quand l'opérateur lit l'historique.

### Lot 3 - Markdown technique et réemploi T3

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

### Lot 4 - Liens, fichiers et images

1. Classer les destinations avant tout rendu : HTTPS externe, chemin absolu de
   projet, image HTTPS ou refus. Interdire les schémas actifs et les données
   intégrées.
2. Étendre DOMPurify et le DOM post-assainissement pour afficher un état
   bloqué compréhensible lorsque la préférence est désactivée.
3. Ajouter `GET /v1/content/file-preview`, son contrat, ses plafonds, la
   canonicalisation et la comparaison à `ProjectRootPolicy`.
4. Rendre l'aperçu de fichier seulement après un clic explicite. Charger les
   images HTTPS de manière différée et sans référent seulement après opt-in.
5. Rendre les liens HTTPS seulement par geste explicite : dans un navigateur,
   une ancre sûre ouvre un nouvel onglet ; dans Bridget Desktop, une navigation
   locale `bridget-open:` est interceptée par la coque et ne transmet au
   navigateur système qu'une URL HTTPS validée. Le panneau relayé ne reçoit
   aucune capability Tauri et le rendu seul ne peut jamais naviguer.

### Lot 5 - Régression, preuve et documentation

1. Ajouter les fixtures de conversation longue et de contenu hostile.
2. Lancer formatage, tests Rust ciblés, tests Desktop, programme Node, contrôle
   de diff et inventaire de licences.
3. Jouer `quickstart.md` sur un serveur relié et consigner les résultats dans
   `implementation.md` et `evidence/validation.md`.
4. Mettre à jour cette SPEC, `tasks.md`, l'ADR et les notices avec les preuves
   réellement obtenues, sans déclarer une validation visuelle non faite.

## Structure du projet

```text
crates/
  bridget-daemon/
    assets/ui/
      app.js                 # projection, rendu, tests Node
      theme.css              # hiérarchie du fil et contenu technique
      index.html             # assets locaux et régions accessibles
      vendor/                # bibliothèques figées, hashes et licences
    src/
      ui.rs                  # route relay d'aperçu de fichier
      project_policy.rs      # autorité réutilisée de racines canoniques
apps/
  bridget-desktop/
    src-tauri/src/
      preferences_store.rs   # persistance locale atomique
      lib.rs                 # injection et rafraîchissement vers le panneau
    ui/
      index.html             # réglages locaux de contenu
      app.js
specs/081-conversation-renderer/
  plan.md
  research.md
  data-model.md
  contracts/
  quickstart.md
  tasks.md
docs/decisions/
  022-contenu-conversation-local-et-non-actif.md
```

## Fichiers principaux prévus

| Surface | Évolution |
|---|---|
| `crates/bridget-daemon/assets/ui/app.js` | Projection pure de tours, rendu stable, Markdown enrichi, préférences V2, tests Node. |
| `crates/bridget-daemon/assets/ui/theme.css` | Réponses documentaires, activité rattachée, code, tableaux, liens et états bloqués. |
| `crates/bridget-daemon/assets/ui/index.html` | Assets de coloration locaux et zones ARIA. |
| `crates/bridget-daemon/assets/ui/vendor/` | Colorateur, hash, licence et notice T3 si nécessaire. |
| `crates/bridget-daemon/src/ui.rs` | Route de preview tokenisée, bornée, lecture seule. |
| `apps/bridget-desktop/src-tauri/src/preferences_store.rs` | V2 sécurisé, migration et tests de défaut. |
| `apps/bridget-desktop/src-tauri/src/lib.rs` | Snapshot non modifiable et diffusion contrôlée au WebView. |
| `apps/bridget-desktop/ui/*` | Réglages de contenu côté application Mac. |
| `docs/decisions/022-contenu-conversation-local-et-non-actif.md` | Frontière de sécurité et réemploi T3. |

## Contrats

- `contracts/local-content-preferences-v1.md`
- `contracts/file-preview-v1.md`

## Stratégie de tests

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

## Risques et parades

| Risque | Parade |
|---|---|
| Le panneau relayé active lui-même les liens et images | Préférences Tauri en autorité, injection lecture seule, pas de capability d'écriture. |
| Une image déclenche une requête surprise | Aucun nœud image si préférence inactive ; HTTPS, lazy et `no-referrer` seulement après opt-in. |
| Un lien fichier devient un accès arbitraire | Route lecture seule, canonicalisation, racines autorisées, plafond et refus strict. |
| Une coloration casse un code | Texte brut et copie restent la source de vérité ; échec de coloration sans échec du fil. |
| Les nouveaux styles cassent la lisibilité Bridget | Variables et fonds Bridget conservés, revue visuelle claire/sombre prévue. |
| Un long fil régresse | Nœuds de tours stables, conservation d'ancre et fixture multi-tours. |
| Réemploi MIT insuffisamment attribué | Notice, commentaires de provenance, inventaire dans `implementation.md`. |

## Constitution check post-conception

PASS sous les vérifications suivantes : les trois nouveaux concepts ont des
responsabilités concrètes et distinctes - projection de tour, préférence de
contenu, aperçu relayé. La route de fichier réutilise une politique d'autorité
existante. La présence du WebView externe n'est pas contournée par une
permission Tauri générale. Aucun comportement n'est déclaré validé avant ses
tests et sa preuve manuelle.
