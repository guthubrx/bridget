# Modèle de données : renommage d’agent

## Identité d’agent

| Champ | Règle |
|---|---|
| `name` | Chaîne non vide, unique parmi les agents connectés. |
| `agent_type` | Type déclaré à l’inscription ; inchangé par le renommage. |
| `connection_id` | Identifiant de connexion ; inchangé par le renommage. |

## Demande de renommage

| Champ | Règle |
|---|---|
| `new_name` | Cible demandée, validée selon les règles de nom. |
| origine | Déduite de la connexion active, jamais fournie par un tiers. |

## État du wrapper

| Champ | Règle |
|---|---|
| `nom courant` | Mis à jour uniquement après la confirmation du démon. |
| `fichier d’état local` | Chemin privé transmis à l’agent enfant pour consulter et mettre à jour le nom courant. |

## Transitions d’état

| État initial | Condition | État final |
|---|---|---|
| `nom_actuel` | nouveau nom valide et libre | `nouveau_nom` ; l’ancien nom disparaît de l’annuaire |
| `nom_actuel` | nouveau nom invalide ou déjà occupé | `nom_actuel` conservé |
