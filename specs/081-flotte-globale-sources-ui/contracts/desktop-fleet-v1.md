# Contrat local - desktop-fleet-v1

## Frontière

Ce contrat relie la page locale Bridget Desktop à son backend Tauri. Il n'est ni une API réseau, ni un contrat MCP, ni une permission de la WebView distante.

## fleet_snapshot

Entrée : aucune. Sortie : `DesktopFleetSnapshotV1`.

Garanties :

- « Cet ordinateur » est ajouté seulement après découverte locale réussie par une commande constante et endpoint versionné ; il n'est jamais un profil sauvegardé ;
- les sources non connectées sont présentes avec leur état local, sans appel réseau ;
- une erreur de lecture affecte uniquement la source concernée ;
- aucun jeton, URL, hôte ou utilisateur SSH, chemin de clé ou `canonical_path` n'est retourné.

## panel_open

Entrée :

```json
{ "source_id": "source-opaque", "agent_name": "nom-de-routage", "desktop_action": null }
```

Garanties :

- `source_id` désigne une session connectée ou la source locale découverte ;
- `agent_name` est encodé dans une URL construite par Rust ;
- l'URL reste de boucle locale et porte `native_attention=1`, `desktop_shell=1`, `agent` et, seulement pour Créer ou Importer, `desktop_action` ;
- un seul panneau enfant reste ouvert.

## Routes relais lues

| Route | Méthode | Usage |
|---|---|---|
| `/v1/snapshot?token=...` | GET | Agents de la source. |
| `/v1/projects?token=...` | GET | Noms et état des projets, avant suppression du chemin. |

Aucune écriture n'est ajoutée.
