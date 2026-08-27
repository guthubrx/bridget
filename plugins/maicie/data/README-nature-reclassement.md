# Reclassement nature v1

Table déterministe produite par le rapport cursor2 (base 6528afa), **pas** par un modèle à l'exécution.

- 52 `regle` + 30 `resultat` = 82 lignes
- Les lignes `constat(blocker) → regle` portent `severity_de`/`severity_vers`
  (abaissement explicite `blocker → major` dans la **même** transition). La
  garde domaine « une règle ne peut pas porter blocker » reste ; la migration
  la franchit proprement, une fois, avec sa trace.
- Le script saute (`SKIP_ALREADY`) toute ligne dont la nature courante vaut
  déjà `nature_vers` — rejeu sans dupliquer les lignes déjà passées.

Appliquer uniquement par le référent :

```bash
export MAICIE_CONFIG=/chemin/absolu/maicie.json
export NATURE_RECLASS_REF=sha:<commit-du-lot-nature>
export MAICIE_BIN=/chemin/absolu/maicie
./scripts/registre-nature-reclasser.sh plugins/maicie/data/nature-reclassement-v1.jsonl
# puis avec --apply après relecture
```

Ne ferme rien. Ne touche pas `pending_qualification`.
