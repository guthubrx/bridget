# Plan d'implémentation - SPEC-081 Conversation structurée et rendu technique sûr

## Résumé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-1).

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-3).

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

### Lot 2b - Reprise de validation visuelle US1

1. Marquer la structure d'un tour au rendu avec ses faits déjà projetés :
   demande humaine, activité, travail et réponse agent. Ne pas ajouter de
   nouveau modèle de données ni de regroupement concurrent.
2. Donner à la réponse agent une signature documentaire discrète : repère de
   réponse, rail fin, largeur de lecture stable et méta lisible, sans la
   transformer en bulle humaine.
3. Rendre le contour du tour visible par un séparateur fin et un rythme
   vertical continu. Les activités et le travail restent compacts, mais
   portent une nature explicite.
4. Vérifier par assertions de projection, rendu DOM et revue visuelle que le
   texte simple bénéficie aussi de la composition, pas uniquement Markdown.

### Lot 3 - Markdown technique et réemploi adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-4).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-5).

## Contrats

- `contracts/local-content-preferences-v1.md`
- `contracts/file-preview-v1.md`

## Stratégie de tests

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/plan.md#extrait-6).

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
