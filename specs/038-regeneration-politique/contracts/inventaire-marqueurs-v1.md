# Contrat d'inventaire de marqueurs v1

## Forme

```json
{
  "version": 1,
  "source": {
    "host": "hote-mesure",
    "marker_directory": "/chemin/absolu/canonique/agent-pids"
  },
  "observed_at": 1787820000,
  "complete": true,
  "live": [
    {
      "principal": "agent-approuve",
      "instance_id": "instance-nouvelle",
      "pid": 1234,
      "birth": 5678
    }
  ],
  "stale": [
    {
      "marker": "1200",
      "pid": 1200,
      "instance_id": "instance-ancienne",
      "reason": "process_not_live"
    }
  ]
}
```

## Garanties du producteur

1. `host` vient du système exécutant le scanner, pas d'une option déclarative.
2. `marker_directory` est absolu, canonique et n'est pas un lien symbolique.
3. chaque entrée du répertoire a été lue ; une entrée illisible ou mal formée
   fait échouer la commande entière ; `complete=false` n'est jamais présenté
   comme un succès.
4. une entrée `live` a été comparée au processus de ce même hôte par PID et
   naissance, puis son nom a été relu depuis `name_file`.
5. zéro entrée `live` produit une erreur, pas cet objet.

## Garanties du consommateur

1. la version, la fraîcheur, la complétude et l'unicité de la source sont
   vérifiées avant lecture métier ;
2. l'ensemble des sources est exactement celui attendu par la politique ;
3. un principal observé sur la mauvaise source ou deux fois est une erreur ;
4. le JSON n'accorde aucun droit : seuls les principaux déjà présents dans la
   politique peuvent être renouvelés.

## Transport distant

Le binaire est exécuté sur l'hôte distant via SSH. La réussite SSH et le code de
sortie du scanner conditionnent la création du fichier local. Une connexion
impossible ne doit jamais être remplacée par `{}` ni par un inventaire vide.
