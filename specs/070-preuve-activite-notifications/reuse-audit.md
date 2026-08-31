# Audit de réutilisation - SPEC-070

**Statut** : PASS

## Eléments prévus et recherche menée

| Elément envisagé | Recherche | Existant trouvé | Décision |
|---|---|---|---|
| Indicateur d'activité | `projectTimeline`, `JOURNAL_ACT_KINDS`, `renderWork` | les actes sont accumulés dans le tour, mais rendus uniquement si `turn.terminal` | REUTILISER et étendre `app.js`. |
| Avatar actif | `agent-avatar`, apparence, `renderAgents` | système de bouille déjà personnalisé | REUTILISER, pas d'asset ni composant nouveau. |
| Suivi d'envoi | `deliveryStateLabel`, `pendingUiMessages`, `prompt_dispatched` | état local de remise issu de SPEC-069 | REUTILISER, supprimer la promesse « traitement démarré » non prouvée. |
| Ciblage de message | `messageId`, `deliveryId`, `renderThread`, `new-messages` | identité stable des messages et politique de défilement | REUTILISER, ajouter une cible de rendu. |
| Notification | `Notification`, `requestPermission` | aucun producteur existant | CREER une adaptation locale dans `app.js`, car aucun équivalent ne couvre l'API navigateur. |
| Echéance humaine | `handle_idempotent_send`, `stamp_turn_deadline_for_delivery` | défaut `reply_timeout=60` recopié dans `deadline_at` | REUTILISER et corriger la séparation des responsabilités. |
| Arrêt à échéance | `wait_for_turn`, `turn/interrupt` | interruption bornée déjà utilisée pour le pilotage humain non attesté | REUTILISER pour l'échéance fournisseur normale. |

## Absence de doublon

La recherche ne révèle ni module de notification, ni endpoint d'état d'activité,
ni mécanisme de ciblage concurrent. Une nouvelle couche serveur ou une nouvelle
dépendance serait donc une duplication ou une abstraction prématurée.

## Arbitrages

| Sujet | Preuve | Décision et raison |
|---|---|---|
| Notification persistante | aucune Push API ou service worker existant | Exclue. La page ouverte suffit au besoin demandé, sans serveur ni abonnement. |
| Etat d'activité serveur | journal EventSource déjà source de vérité | Exclu. L'état client est dérivé et reconstructible. |
| Avatar dédié au travail | bouille sélectionnée existante | Exclu. Réutiliser l'identité visuelle existante réduit la charge cognitive. |

## Gate avant tasks

- [x] Les composants, services, endpoints et dépendances proposés ont été recherchés.
- [x] Les composants existants seront étendus avant toute création.
- [x] Aucune duplication évidente ne requiert un arbitrage utilisateur.
- [x] Aucun nouveau crate, package ou backend n'est justifié.
- [x] Les risques de corrélation, confidentialité et défilement sont documentés.
