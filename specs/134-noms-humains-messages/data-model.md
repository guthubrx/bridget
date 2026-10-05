# Modèle de données 134

## Identité routable

- `agent_id` : UUID stable et clé primaire.
- Invariant : toutes les décisions de routage et d’autorisation utilisent cet
  identifiant, jamais le nom affiché.

## Profil de présentation

- `agent_id` : référence vers l’identité.
- `display_name` : nom lisible et modifiable.
- `display_name_normalized` : valeur unique utilisée pour les conflits.
- Apparence, instructions et révisions : champs existants inchangés.

Invariant renforcé : après `ensure_agent_ids(id)`, l’identité, son profil et son
état d’application existent. Un second appel ne crée aucune ligne supplémentaire.

## Message remis

- `from` : UUID routable.
- `from_display_name` : copie optionnelle du nom au moment de la remise.
- `delegated_origin` : provenance optionnelle d’un sous-agent.

Projection visible :

- nom exploitable : `nom (UUID)` ;
- nom absent, vide ou égal à l’UUID : `UUID` ;
- provenance déléguée : ajouter le suffixe existant après ce libellé.

Aucun état nouveau n’est stocké.
