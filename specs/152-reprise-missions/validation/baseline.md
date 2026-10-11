# Référence de tests — session 152

Date : 2026-10-11. Exécution par le principal, avant tout code 152.

Base : cd2bb015755fde7af26cf87fdb8f4e658172e237.
Isolation : /tmp/b.gjyiuull ; umask 077, HOME conservé, BRIDGET_HOME privé.

Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-results.json.

| Filtre | PASS | FAIL | Ignorés | RC | Durée (s) |
|---|---:|---:|---:|---:|---:|
| native148 | 27 | 0 | 1 | 0 | 56.05 |
| native149 | 78 | 0 | 0 | 0 | 31.13 |
| native150 | 51 | 0 | 0 | 0 | 15.62 |
| delegation149 | 10 | 0 | 0 | 0 | 60.05 |
| spec_087 | 11 | 0 | 0 | 0 | 9.22 |
| native151 | 62 | 0 | 0 | 0 | 29.06 |
| **Total six filtres** | **239** | **0** | **1** | **0 pour chaque run** | — |

T001 est validée par ses cinq filtres prévus : 177 PASS, 0 FAIL et 1 ignoré.
Le filtre native151 existant ajoute 62 PASS, 0 FAIL et 0 ignoré. Aucun nouveau
test ni changement de code 152 pendant ces runs. Cette référence ne vaut pas
résultat de suite complète. Les tests propres à 152 restent à écrire.

## Commandes et journaux

Répertoire de travail : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions.
Le principal a utilisé le script de build existant, en mode locked/offline.
Les six commandes exactes et leurs résultats sont conservés dans le JSON source.
La première commande est :

```sh
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native148 -- --test-threads=4
```

Les autres gardent les mêmes arguments avec leur filtre, selon le JSON source.
Journaux bruts et SHA-256 :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-native148.log — a651f8131f6a62d4ebd7e06cb209bab392ffa0c4a052dfddca8d22d4ef66bf8b.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-native149.log — ee8b7355f8f08cf07544be992129050bd6c925cf10ce2bd9f7ceda749b963d42.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-native150.log — 80ffdce6c0b481fd4429069237e7ff0606ea4d819d9868540528314cf29f8b94.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-delegation149.log — 6016a3664d4784c8f79f581b0425c1cc5cb55fffd4b10a862d2e3b0673a40d2e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-spec_087.log — 66124489ccea3f9d35487e4bebbe31ed30337a9737108d66b9f8efd21763e78a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-native151.log — 91bb3968f49434fae2eb28b753ecc623eb28f6deb2d058ed917fca66b7ff8520.
