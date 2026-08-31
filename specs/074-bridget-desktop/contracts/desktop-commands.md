# Contrat local - SPEC-074 Bridget Desktop

## Découverte d'endpoint sur le serveur

La seule commande distante appelée par Bridget Desktop avant l'ouverture du tunnel est :

```text
bridget ui endpoint --json
```

Elle lit l'état déjà créé par le relais UI et écrit exclusivement sur stdout :

```json
{"version":1,"port":17888,"token":"ephemere"}
```

Contraintes : le port est entier non nul, le jeton est non vide, aucune autre clé n'est acceptée par le client et les erreurs ne contiennent jamais le jeton. La commande ne démarre ni ne redémarre le relais. Le JSON reste dans le processus backend Desktop et en mémoire seulement.

## Commandes de la coque locale

La webview locale peut uniquement invoquer des commandes typées. Aucun appel ne reçoit une ligne de shell, un jeton ou une clé privée.

| Commande | Entrée | Sortie | Règle de sûreté |
|---|---|---|---|
| `profiles_list` | aucune | profils non secrets | jamais de jeton ni de chemin privé implicite |
| `profile_save` | profil validé | profil non secret | aucun mot de passe ni contenu de clé accepté |
| `profile_delete` | `profile_id` | confirmation | ferme d'abord toute session associée |
| `host_identity_check` | `profile_id` | état et empreinte SHA-256 | une clé hôte inconnue crée un ticket mémoire, pas une confiance implicite |
| `host_identity_approve` | `profile_id`, ticket | état | persiste uniquement la clé publique approuvée dans le `known_hosts` applicatif |
| `connection_open` | `profile_id` | état de session | construit SSH côté Rust, attend le relais avant `connected` |
| `connection_close` | `profile_id` | état fermé | termine seulement l'enfant SSH possédé par la session |
| `panel_open` | `profile_id`, emplacement | label de panneau | crée une webview `panel-*` sans capability |
| `panel_close` | label | confirmation | détruit la webview et libère la session correspondante |
| `diagnostics_redacted` | `profile_id` | état catégorisé | masque jetons, clés, commandes et URLs signées |

## Événements localisés

`connection-state` est émis uniquement vers la coque locale avec `{ profile_id, state, category }`. Une webview `panel-*` ne reçoit pas cet événement et ne possède aucune capability. Les états sont ceux de `data-model.md`; `connected` exige une réponse HTTP du relais à travers le tunnel.
