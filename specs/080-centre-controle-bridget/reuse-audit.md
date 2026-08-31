# Audit de reutilisation de l'existant - SPEC-080 Centre de contrôle Bridget

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `/home/moi/bridget-referent/.worktrees/session-080-centre-controle-bridget/specs/080-centre-controle-bridget`

Conclusion courte: le plan étend les frontières déjà en place au lieu de créer un second client réseau, un second journal d'usage ou un éditeur de configuration généraliste. Les routes projets et la politique de racines restent l'autorité de la première écriture. Aucun doublon évident ne reste non arbitré: les nouveaux objets portent une responsabilité absente de l'existant.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 11 |
| Items audites | 11 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Panneau de réglages par serveur | Panneau WebView relié au profil SSH | `apps/bridget-desktop/src-tauri/src/lib.rs:366` | Le profil ouvre déjà un relais isolé; l'engrenage est ajouté dans ce panneau. |
| Routes de contrôle | Relais HTTP UI versionné | `crates/bridget-daemon/src/ui.rs:1007` | Les routes control suivent le même parseur, jeton et réponse que les routes projets. |
| Première clé modifiable | `ProjectRootPolicy` atomique | `crates/bridget-daemon/src/project_policy.rs:52` | Pas de second fichier de racines ni de second validateur. |
| Concurrence de configuration | Génération de politique | `crates/bridget-daemon/src/project_policy.rs:38` | La prévisualisation et application utilisent la génération déjà prouvée. |
| Affichage et état UI | UI sans framework et helpers de ressources | `crates/bridget-daemon/assets/ui/app.js:3805` | Les nouvelles requêtes réutilisent le jeton et l'URL locale existants. |
| Usage horodaté | Table `usage_samples` et agrégat | `crates/bridget-daemon/src/store.rs:614`, `crates/bridget-daemon/src/store.rs:1529` | Les dimensions et projections sont étendues sans second ledger de compteurs. |
| Persistance des préférences Mac | Écriture atomique du profil | `apps/bridget-desktop/src-tauri/src/profile_store.rs:128` | Un document distinct réutilise le format, la validation et le remplacement atomique du store existant. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Catalogue de contrôle fermé | Registre des définitions d'agent | `crates/bridget-daemon/src/registry.rs:86` | Ne pas le réutiliser: il décrit les agents et peut contenir des éléments de lancement exclus du centre de contrôle. Reprendre seulement son style de validation fermée. |
| Reçus de mutation | Événements d'audit de projet | `crates/bridget-daemon/src/store.rs:1500` | Ne pas fusionner: le reçu de contrôle est idempotent par commande et serveur, tandis que l'audit projet est propriétaire de ProjectIdentity. Conserver des liens de corrélation si pertinents. |
| Préférences de rendu | Préférences locales de barre agents | `crates/bridget-daemon/assets/ui/app.js:2896` | Ne pas persister ici: l'origine de la WebView change avec le port du tunnel. Le stockage reste côté Bridget Desktop. |

## Duplications evidentes

Aucune. Les routes historiques de projets ne sont pas dupliquées: elles restent compatibles et seront déléguées au validateur de contrôle pour la clé de racines.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Articles III, VII, XV, XVI, XVIII, XIX et XX | Spec, ADR, worktree isolé, tests comportementaux, requêtes indexées, minimalisme et justification de toute abstraction. |
| Instructions projet utilisateur | Code serveur dans le worktree, aucun secret, chemins absolus, pas de commit automatique | Le travail reste dans la branche SPEC-080; aucune modification de `main`. |
| `/home/moi/.speckit/ref/standards-tests.md` | Scénario métier et test traçable | Ajouter la feature 080 et les tests Rust/Node correspondant aux critères de succès. |
| `/home/moi/.speckit/ref/standards-observability.md` | Corrélation et compteurs sans contenu | Reçus et usage contiennent des dimensions techniques, pas des prompts ni sorties. |
| `/home/moi/.speckit/ref/standards-frontend.md` | État borné et composants par feature | Applicable par analogie: ne pas introduire de store global ni framework dans l'UI vanilla existante. |
| `/home/moi/.speckit/ref/standards-backend.md` | Secrets et moindre privilège | Applicable par analogie au relais Rust: réponses et logs non sensibles. |
| `/home/moi/.speckit/research/04-architectures-patterns.md` | Réutiliser le protocole et l'observabilité existants | Aucun nouveau service de coordination ni protocole extérieur. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-064 | Usage borné, observation absente distincte de zéro | Réutiliser `UsageTokens` et préserver l'inconnu. |
| SPEC-065 | Identité projet et audit propriétaire | Les réglages projet ne deviennent pas globaux au serveur. |
| SPEC-066 | Politique runtime par projet | Les limites Docker restent `project_controlled` dans cette vue. |
| SPEC-067 | Secrets et approbation locale | Secrets et approbations exclus des routes control. |
| SPEC-068 | Incident borné vers coordinateur | Diagnostics réduits, sans contenu d'agent. |
| SPEC-076 | Routes de réglages et prévisualisation de racines | Étendre le validateur de politique au lieu de créer une deuxième administration projet. |
| SPEC-077 | Préférences UI validées et tolérantes au stockage | Garder le même comportement de dégradation pour la vue de contrôle. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "settings|preview|usage|timezone"` | daemon, transport et Desktop | Localisation des routes projets, usage et panneaux. |
| `rg "ExecutionUsageSample|usage_samples"` | daemon et store | Les échantillons horodatés existent déjà; aucun nouveau ledger nécessaire. |
| `rg "ProjectRootPolicy|replace_atomically"` | daemon | Première mutation réutilisable identifiée. |
| `rg "ProfileStore|rename"` | Desktop | Pattern de persistance atomique locale identifié. |
| `rg "sidebar|localStorage|agentPane"` | assets UI | La barre à modifier est celle du daemon, non la fenêtre de profils. |
| lecture SPEC-064 à SPEC-068, SPEC-076 et SPEC-077 | specs | Autorités projet, secret, usage et UI identifiées. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Réglage initial modifiable | reutiliser `ProjectRootPolicy` | Validation atomique et génération déjà réelles. | 2026-08-31 |
| Registre des agents | ignorer comme persistance de réglages | Il porte des paramètres de lancement et risque d'exposer une configuration non administrable. | 2026-08-31 |
| Usage | reutiliser `usage_samples` | Faits horodatés existants, sans zéro inventé. | 2026-08-31 |
| Coût | creer catalogue de tarifs strictement typé | Aucune table tarifaire versionnée n'existe; un prix implicite serait trompeur. | 2026-08-31 |
| Préférences locales | creer document Desktop séparé | `localStorage` du panneau est lié au port de tunnel et n'est pas une persistance stable de l'application. | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
