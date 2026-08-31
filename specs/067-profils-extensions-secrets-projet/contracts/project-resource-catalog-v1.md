# Contrat de configuration v1: catalogue de ressources projet

## Autorité et chargement

Bridget possède ce catalogue hôte fermé. Le daemon le charge une fois depuis
un chemin absolu explicitement fourni au démarrage. Le fichier appartient à
l'UID du daemon et n'est modifiable ni par le groupe ni par les autres comptes.
Une proposition Maicie ne transporte que `source_ref`, jamais le chemin réel
ni la révision autoritative. Bridget résout `source_revision`; l'approbation
locale épingle ensuite la révision résolue.

L'absence ou l'invalidité du catalogue ferme uniquement les profils contenant
des ExtensionRef ou SecretRef. Les AgentProfiles historiques et les profils
sans ressource conservent leur comportement compatible.

## Forme fermée

```json
{
  "contract_version": 1,
  "extension_roots": ["/home/moi/.local/share/bridget/extensions"],
  "secret_roots": ["/home/moi/.local/share/bridget/secrets"],
  "sources": [
    {
      "source_ref": "catalog:speckit",
      "kind": "extension_directory",
      "canonical_path": "/home/moi/.local/share/bridget/extensions/speckit",
      "source_revision": 12,
      "expected_uid": 1002,
      "expected_gid": 1002,
      "allowed_project_ids": ["opaque-project-id"]
    },
    {
      "source_ref": "secret:codex-session",
      "kind": "secret_directory",
      "canonical_path": "/home/moi/.local/share/bridget/secrets/codex-session",
      "source_revision": 3,
      "expected_uid": 1002,
      "expected_gid": 1002,
      "allowed_project_ids": ["opaque-project-id"]
    }
  ]
}
```

Les champs inconnus, doublons, chemins relatifs, sources hors racines,
`allowed_project_ids` vide, wildcard, symlink et fichier spécial sont refusés.
Le chemin complet reste dans la configuration et l'audit privé Bridget.

## SecretSourceStamp

Bridget calcule sans lire le contenu:

- fichier: kind, device, inode, taille, mtime/ctime nanoseconde, mode, UID/GID;
- répertoire: même tuple pour la racine et manifeste récursif trié des chemins
  relatifs et métadonnées de chaque entrée régulière.

Les symlinks et fichiers spéciaux sont refusés. Le digest canonique de ce stamp
et `source_revision` sont épinglés à l'approbation, puis revérifiés avant create
et spawn. Aucun octet ni digest de contenu secret n'est calculé ou persisté.
Une divergence produit `secret_generation_stale` avant lecture ou montage.

Ce mécanisme détecte les modifications du modèle local coopératif. Un
administrateur root capable de falsifier toutes les métadonnées est hors du
modèle de menace v1.
