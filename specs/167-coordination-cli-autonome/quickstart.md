# Démarrage rapide — Coordination CLI autonome

## 1. Lancer un coordinateur Bridget

L’agent doit être lancé ou rattaché par Bridget. T3 n’est pas requis.

## 2. Confier une mission

```sh
bridget delegate "Analyse ce dépôt et propose les trois risques principaux"
```

Conserver le `task_id` rendu.

## 3. Attendre le résultat

```sh
bridget task wait <TASK_ID>
```

## 4. Lire puis accuser réception

```sh
bridget task status <TASK_ID>
bridget task ack <TASK_ID>
```

## 5. Automatiser

```sh
bridget delegate "Analyse les tests" --json
```

La sortie standard contient un seul objet JSON.
