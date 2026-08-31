# Audit de réutilisation - SPEC-071

**Statut** : PASS

## Éléments prévus et recherche menée

| Élément envisagé | Recherche | Existant trouvé | Décision |
|---|---|---|---|
| Métadonnées d'identité | `AgentInfo`, `UiAgentRowV1`, `compose_agent_rows` | les cinq faits existent dans le protocole, quatre sont perdus par la projection UI | RÉUTILISER et étendre la projection, sans nouveau protocole. |
| Catalogue runtime | recherche `runtimeIdentity`, logos et mappings fournisseur | aucun catalogue UI concurrent | CRÉER une fonction pure locale, fermée et testée. |
| Mode TMUX/FLUX | `PresenceMode`, transports gérés et noms d'agents | `tmux`, `acp`, `cli` sont attestés ; les suffixes ne le sont pas | RÉUTILISER les faits explicites et créer une table fermée de présentation. |
| Overlay | `.agent-row__tooltip`, recherche tooltip/popover | un seul tooltip existe ; il ne porte que l'extrait et est coupé par le panneau | REMPLACER ce composant, pas en ajouter un second. |
| Positionnement | `getBoundingClientRect`, listeners de survol/focus | aucun utilitaire concurrent | CRÉER deux helpers purs de placement et un contrôleur local borné. |
| Assets de marque | `assets/ui`, `include_bytes!`, `write_asset` | pipeline d'asset embarqué, ETag et 304 déjà complet | RÉUTILISER exactement ce chemin pour les SVG. |
| Icônes fournisseur | recherche locale complète | aucun asset de marque existant | IMPORTER les assets officiels avec NOTICE et contrôle de sécurité. |
| Tests navigateur | suite Node intégrée à `app.js` | harness et tests CSS/HTML déjà présents | RÉUTILISER, sans framework ou dépendance nouvelle. |
| Tests serveur | tests unitaires `ui.rs`, `ui_relay_test.rs` | témoins de snapshot et assets déjà présents | RÉUTILISER et étendre. |

## Absence de doublon

La recherche ne révèle ni registre de marques, ni projection parallèle de
runtime, ni composant de popover réutilisable. La seule création logique est
donc le catalogue pur et le contrôleur de fiche. Les responsabilités réseau,
présence, cache, avatar, sélection et rendu de liste restent dans leurs
composants existants.

## Arbitrages

| Sujet | Preuve | Décision et raison |
|---|---|---|
| Fournisseur réel du modèle | aucun champ attesté dans `AgentInfo` | Exclu. Le déduire depuis `model` ou `agent_type` produirait des erreurs, notamment avec Cursor. |
| Logo dans chaque ligne | l'avatar est déjà l'identité permanente | Exclu. Le logo reste dans la fiche pour ne pas surcharger la liste. |
| Tooltip imbriqué | `.agent-pane` coupe horizontalement ses enfants | Exclu. Une fiche globale unique évite le clipping sans changer le layout. |
| Bibliothèque de popover | aucun package UI et besoin local borné | Exclue. Une dépendance serait disproportionnée. |
| Requête d'asset distante | relais d'asset local existant | Exclue. Les marques restent disponibles hors réseau et la surface de suivi tiers reste nulle. |
| Nouveau champ protocolaire | toutes les données opérationnelles utiles existent | Exclu. La projection UI suffit. |

## Gate avant revue adverse

- [x] Les composants, services, endpoints et dépendances proposés ont été recherchés.
- [x] Les composants existants seront étendus avant toute création.
- [x] Aucun doublon évident ne requiert un arbitrage utilisateur.
- [x] Aucun nouveau crate, package, endpoint métier ou stockage n'est justifié.
- [x] La confusion runtime, éditeur et fournisseur du modèle est résolue.
- [x] Les risques d'accessibilité, de clipping, de marque et de SVG sont documentés.
