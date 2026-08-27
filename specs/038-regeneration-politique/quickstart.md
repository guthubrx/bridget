# Quickstart — Régénération explicite de la politique

> Toutes les commandes ci-dessous ciblent une copie privée de test. Ne jamais
> commencer par la politique en service.

## 1. Construire le binaire

```bash
cargo build -p bridget-daemon --bin bridget-greffe-policy-refresh
```

## 2. Annoter la copie de politique

Copier puis adapter l'exemple de la session 038 :

```bash
cp /chemin/absolu/vers/bridget/specs/038-regeneration-politique/contracts/greffe-authorization-refresh.example.json \
  /chemin/absolu/vers/greffe-authorization.fixture.json
chmod 600 /chemin/absolu/vers/greffe-authorization.fixture.json
```

Chaque principal déjà approuvé reçoit sa source mesurable :

```json
"marker_source": {
  "host": "cartae",
  "marker_directory": "/home/moi/.cache/bridget/agent-pids"
}
```

La garde accepte encore une politique historique sans ce champ, mais le
rafraîchisseur la refuse.

## 3. Scanner chaque hôte sur place

Localement :

```bash
/chemin/absolu/vers/bridget-greffe-policy-refresh scan \
  --markers /chemin/absolu/vers/agent-pids \
  --output /chemin/absolu/vers/inventory-local.json
```

À distance, exécuter le scanner sur l'hôte qui possède les PID et vérifier le
code de sortie SSH avant de conserver la sortie :

```bash
ssh hote-distant '/chemin/absolu/vers/bridget-greffe-policy-refresh scan --markers /chemin/absolu/vers/agent-pids' \
  > /chemin/absolu/vers/inventory-distant.json
chmod 600 /chemin/absolu/vers/inventory-distant.json
```

Si SSH ou le scanner échoue, supprimer la sortie incomplète et ne pas lancer la
régénération.

## 4. Prévisualiser puis appliquer

```bash
/chemin/absolu/vers/bridget-greffe-policy-refresh refresh \
  --policy /chemin/absolu/vers/greffe-authorization.fixture.json \
  --inventory /chemin/absolu/vers/inventory-local.json \
  --inventory /chemin/absolu/vers/inventory-distant.json
```

Lire `dead_principals` et `unapproved_principals`, puis appliquer explicitement :

```bash
/chemin/absolu/vers/bridget-greffe-policy-refresh refresh \
  --policy /chemin/absolu/vers/greffe-authorization.fixture.json \
  --inventory /chemin/absolu/vers/inventory-local.json \
  --inventory /chemin/absolu/vers/inventory-distant.json \
  --apply
```

La réussite annonce les générations avant/après. Le fichier final est régulier
en `0600`. Un principal mort reste autorisé par son ancienne instance et doit
faire l'objet d'une décision de révocation séparée s'il ne doit plus revenir.
