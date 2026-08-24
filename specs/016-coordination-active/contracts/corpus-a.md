# Corpus Bridget A épinglé — G-1601

**Gate** : G-1601 ne peut ouvrir les consommateurs 016 qu'après vérification
du commit `cb7a6d8c6f122da38dccc4c2ebe643f4bf0cac74` et des octets ci-dessous.
Ce commit est l'autorité du producteur Bridget après T1603 et T1604 ; il est
un descendant du gel amont 015 vérifié par `verify-amont-015.sh`.

## Corpus filaire relu par le couloir C

| Surface | Fixture 016 | Lecteur explicite | SHA-256 |
|---|---|---|---|
| Sept trames 015 inchangées (dépôt, opérations, cycle) | `fixtures/service-frames-v1.jsonl` | harnais C `686bb25`, `coordination_client.rs` | `42e0ea32690e267221804a8af9aee538162f7c141a9cffacf6d6a44a9a8cda1f` |
| Négociation et événement attesté v1 | `fixtures/coordination-events-v1.jsonl` | T1613, compatibilité service 015/v1 | `257215547c2858ee83fb64c61aeacdecbbdaee96df5661ced4e7403aaf40b590` |
| Relève cursée v2 | `fixtures/coordination-stream-v2.jsonl` | T1613 : curseur, `SnapshotCaughtUp`, `Gap`, `Unavailable` | `a384b492b53644a9357c5254df2d46b80b36b7d5761987d88d21c336dacdf00c` |

Le harnais C de `686bb25` relit déjà la première ligne de cette table par
`include_bytes!`, puis vérifie son empreinte : elle reste donc strictement
identique au corpus 015 gelé. Les deux autres fixtures sont les seules
extensions publiques du producteur A que C peut consommer après G-1601. Elles
incluent les états `Gap` et `Unavailable` comme trames 016 distinctes : elles
ne sont ni attribuées rétrospectivement à 015, ni réduites à une fraîcheur.

## Frontière publiée

Sont gelés : les trois fichiers JSONL ci-dessus, leur ordre de lignes, chaque
octet JSON, le saut de ligne final, leurs versions et les significations
publiques décrites dans `evenements-coordination.md`. Le vérificateur compare
le worktree à l'objet Git épinglé, pas à une représentation JSON équivalente.

Restent ouverts : le réducteur et les décisions du couloir B, ainsi que les
projections et l'intégration du couloir C. Ils peuvent tenir pour acquis que
la relève cursée restitue les bytes et `event_id` persistés, que seule
`SnapshotCaughtUp` rend une vue fraîche, et que `Gap` et `Unavailable` sont
deux refus d'observation non interchangeables. Ils ne peuvent ni ajouter une
trame au corpus, ni reconstruire un événement depuis un DTO local.

## Vérification mécanique

Depuis la racine du dépôt :

```bash
specs/016-coordination-active/contracts/verify-corpus-a.sh
specs/016-coordination-active/contracts/verify-corpus-a.sh --self-test
```

La seconde commande modifie un octet d'une copie temporaire du flux v2 ; elle
ne réussit que si cette mutation est refusée par la même comparaison
d'empreinte et de bytes utilisée pour le corpus réel.
