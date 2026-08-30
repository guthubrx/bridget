# Modèle de données - SPEC-077 Menu contextuel des agents

## AgentSidebarPreferencesV1

Préférence locale au navigateur, sans autorité sur le daemon.

| Champ | Type | Règle |
|---|---|---|
| version | entier | valeur exacte 1 |
| pinned | liste de chaînes | noms non vides, dédupliqués et bornés |
| hidden | liste de chaînes | noms non vides, dédupliqués et bornés |
| readThrough | objet nom vers nombre | horodatages positifs et finis seulement |

Invariant: aucune valeur de message, aucun jeton et aucun secret.

## AgentContextMenuItem

Projection pure d'une commande visible.

| Champ | Type | Règle |
|---|---|---|
| key | chaîne fermée | open, pin, read, hide, stop, relaunch ou decommission |
| label | chaîne | libellé dépendant de l'état local |
| group | chaîne fermée | navigation, organisation ou lifecycle |
| enabled | booléen | calculé depuis l'état et l'éligibilité |
| reason | chaîne | explication quand indisponible |
| danger | booléen | vrai uniquement pour decommission |

## AgentSidebarProjection

| Champ | Contenu |
|---|---|
| active | agents visibles qui ne sont ni arrêtés ni injoignables |
| stopped | agents visibles arrêtés ou injoignables |
| hidden | tous les agents masqués, quel que soit leur état |
| activeTotal | nombre total d'agents actifs avant masquage |

## Transitions locales

- non épinglé vers épinglé: l'agent passe avant les non épinglés de son groupe.
- épinglé vers non épinglé: l'ordre normal redevient applicable.
- visible vers masqué: la ligne rejoint Agents masqués.
- masqué vers visible: la ligne rejoint actif ou arrêté selon son état courant.
- non lu vers lu: readThrough prend le dernier last_message_at observé.

Ces transitions ne modifient jamais state, persistent, lifecycle_state ou l'historique du daemon.
