# Modèle de données - SPEC-075

## DesiredEquipier schema 4

| Champ | Type | Rôle |
|---|---|---|
| `type` | string | Type du registre au lancement initial |
| `cwd` | chemin absolu | Répertoire de travail attesté |
| `command_id` | string | Dernière génération connectée connue |
| `generation` | entier positif | Génération monotone |
| `created` | string | Date de création de la génération |
| `resolved_definition` | objet optionnel | Définition runtime figée |
| `domain` | string optionnel | Domaine Bridget |
| `project` | objet optionnel | Référence projet attestée |
| `agent_link` | objet optionnel | Lien parent et mandat |
| `persistent` | bool | Reprise automatique si l'état est `running` |
| `lifecycle_state` | `running|stopped|decommissioned` | État désiré du processus |

Compatibilité:

- Schémas 1 à 3: `persistent=true`, `lifecycle_state=running` par défaut.
- Le schéma écrit devient 4.
- Les champs inconnus restent refusés.
- Une entrée sans définition figée reste lisible mais ne peut pas être relancée.

## Machine d'états

```text
          spawn connecté
 absent ----------------------> running
                                  |
                                  | stop confirmé ou mort observée
                                  v
                               stopped
                                  |
                                  | relaunch connecté
                                  +----------------------> running

 running -- decommission + arrêt confirmé --> decommissioned (caché)
 stopped -- decommission -------------------> decommissioned (caché)
```

Transitions de sûreté:

- `running -> stopped` est écrit avant l'attente du superviseur afin qu'un
  crash ne provoque pas de reprise automatique.
- Une relance conserve l'ancienne entrée `stopped` jusqu'à la connexion de la
  nouvelle génération.
- Une compensation ne retire que la génération qu'elle possède.
- Un décommissionnement actif passe par `stopped`; l'entrée n'est retirée de la
  flotte visible qu'après arrêt confirmé.
- Une entrée `decommissioned` ne peut être ni reprise, ni relancée, ni remplacée
  par un nouveau spawn portant le même nom.

## Projection annuaire

Une entrée durable `stopped` absente du routeur est projetée dans `AgentInfo`
avec:

- `state=stopped`;
- `persistent=Some(valeur)` pour attester la gestion Bridget;
- type, provider, protocole, mode, modèle et effort dérivés exclusivement de la
  définition figée;
- aucune connexion active et aucun faux fait de présence.

Une présence live gagne sur la projection synthétique portant le même nom.
Une entrée `decommissioned` n'est jamais projetée dans l'annuaire normal.

## Résultats fermés

### RelaunchOutcome

- `started`: la nouvelle génération est connectée.
- `already_running`: une génération est active.
- `not_managed`: aucune preuve de gestion.
- `not_found`: aucune identité courante.
- `not_relaunchable`: définition durable incomplète.
- `rejected`: garde de spawn ou préparation refusée.
- `timeout`: pas de verdict terminal dans le délai.

### DecommissionOutcome

- `decommissioned`: arrêt éventuel confirmé et tombstone cachée persistée.
- `decommissioned_forced`: arrêt forcé confirmé puis tombstone cachée persistée.
- `not_managed`: aucune preuve de gestion.
- `not_found`: aucune identité courante.
- `already_decommissioned`: tombstone déjà présente.
- `timeout`: arrêt non confirmé, définition conservée.

## Migration héritée

L'adoption prend comme entrée un nom observé `stopped` avant redémarrage et
vérifie la dernière génération gérée dans SQLite. Elle copie uniquement les
métadonnées non secrètes déjà persistées: type, `cwd`, persistance, génération,
définition résolue, projet et lien si disponibles. Sans preuve complète, elle
refuse et ne crée aucune entrée.
