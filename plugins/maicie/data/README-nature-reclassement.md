# Reclassement nature v1

Table déterministe produite par le rapport cursor2 (base 6528afa), **pas** par un modèle à l'exécution.

- 52 `regle` + 30 `resultat` = 82 lignes
- Appliquer uniquement par le référent :

```bash
export MAICIE_CONFIG=/chemin/absolu/maicie.json
export NATURE_RECLASS_REF=sha:<commit-du-lot-nature>
export MAICIE_BIN=/chemin/absolu/maicie
./scripts/registre-nature-reclasser.sh plugins/maicie/data/nature-reclassement-v1.jsonl
# puis avec --apply après relecture
```

Ne ferme rien. Ne touche pas `pending_qualification`.
