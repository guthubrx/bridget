# Lot 3 - Consignes allégées (T031-T035)

But : ramener `AGENTS.md` de environ 126 Ko à moins de 20 Ko. Le fichier est chargé au début de chaque session d'agent. Un fichier lourd consomme le contexte et pousse à la compaction. Mesures : `releves.md` section 14.
Branche : `session-260-lot03-consignes`. Worktree : `.worktrees/260-lot03-consignes`. Aucun code exécutable. Aucun redémarrage.
Principe : les reçus de session sont **déplacés sans modification**. Les consignes encore vivantes qu'ils contiennent sont **remontées** en règles permanentes avant le déplacement.

## T031 - Constat de départ

```sh
wc -c -l AGENTS.md                                         # attendu : environ 126000 octets, 1111 lignes
rg -n '^Session ?[0-9]+' AGENTS.md | head -1               # attendu : première ligne du bloc (environ 112)
rg -n '^Les reçus datés des sessions précédentes' AGENTS.md  # attendu : une ligne (environ 1087)
rg -n '^## ' AGENTS.md                                     # attendu : 7 titres de section
```

Écrire les sorties dans `validation/lot03-constat.txt`. Noter les deux numéros de ligne : D = première ligne du bloc, F = numéro de la ligne « Les reçus datés ... ». Le bloc des reçus va de la ligne D à la ligne F-1.
Arrêt : si l'une des deux lignes manque ou apparaît deux fois, arrêter le lot (le fichier a changé de forme).

## T032 - Copier le bloc dans un fichier dédié

Créer `specs/recus-de-session.md`. Ligne 1 : `# Reçus de session (déplacés depuis AGENTS.md)`. Ligne 2 : texte de la dernière ligne « Les reçus datés ... » d'AGENTS.md. Ligne 3 vide. Puis le bloc, **octet pour octet** :

```sh
D=<première ligne>; F=<ligne Les reçus datés>
{ printf '# Reçus de session (déplacés depuis AGENTS.md)\n'; sed -n "${F}p" AGENTS.md; printf '\n'; sed -n "${D},$((F-1))p" AGENTS.md; } > specs/recus-de-session.md
diff <(sed -n "${D},$((F-1))p" AGENTS.md) <(tail -n +4 specs/recus-de-session.md) && echo identique
```

Attendu : `identique`. Ne pas reformater, ne pas corriger les fautes ni les espaces manquants des reçus (FR09 : conservés à l'identique).
Commit : `docs(consignes): Copier les recus de session dans un fichier dedie`.

## T033 - Remonter les consignes vivantes

Ces huit consignes sont dans les reçus mais valent toujours. Les écrire dans une nouvelle section d'`AGENTS.md`, **avant** la ligne D, titre `## Règles permanentes issues des reçus`. Chaque ligne cite le reçu source (numéro de session) et commence par un mot qui n'est pas `Session` (sinon l'extraction de T034 se trompe).

| Consigne | Source à relire avant d'écrire la ligne |
|----------|------------------------------------------|
| Ne pas arrêter, redémarrer ni remplacer T3 sans assentiment explicite de François. | `rg -n 'NE PAS arrêter' specs/recus-de-session.md \| head -3` |
| Repère d'empreinte du registre Mac spécial : dernier connu session 256, confirmé 257. Ne pas restaurer un ancien registre à l'aveugle. | `rg -n 'registre ?Mac' specs/recus-de-session.md \| head -5` (copier l'empreinte lue, jamais de mémoire) |
| Registre dédié du worker Mac : préserver le fichier ; son dossier parent reste le namespace attesté. | reçu « Session221/worker Mac » |
| SYS02 : ne plus la reprendre. | `rg -n 'SYS02' specs/recus-de-session.md \| head -3` |
| Pas d'essai Claude ni de sentinelle 244 sans demande utile nouvelle. | reçu session 244 |
| Codex KS désactivé jusqu'à la connexion de l'utilisateur. | reçus 212 à 216 |
| L'outil de mise à jour Mac utilise le Python système 3.9.6. | reçu session 224 |
| Linux commun construit sur Debian 12 / glibc 2.36. | reçu session 224 |

Règles d'écriture : une phrase par consigne, courte, sans adresse IP ni jeton. Une consigne dont la source ne se retrouve pas dans `specs/recus-de-session.md` n'est pas écrite : la noter dans le journal.
Vérifier : `rg -c '^## Règles permanentes issues des reçus' AGENTS.md` donne 1.
Commit : `docs(consignes): Remonter les consignes vivantes des recus`.

## T034 - Retirer le bloc d'AGENTS.md

Le bloc va de la première ligne `Session...` jusqu'à la ligne qui précède « Les reçus datés ... ». Retrouver ces bornes **par motif** (T033 a décalé les numéros) :

```sh
cp AGENTS.md /tmp/AGENTS.avant.md
sed -nE '/^Session ?[0-9]/,/^Les reçus datés des sessions précédentes/p' AGENTS.md | wc -l        # attendu : lignes du bloc + 1
sed -E '/^Session ?[0-9]/,/^Les reçus datés des sessions précédentes/{/^Les reçus datés des sessions précédentes/!d;}' AGENTS.md > /tmp/AGENTS.apres.md
```

`sed -E` (macOS et Linux) accepte `?`. Contrôles avant de remplacer :

1. `wc -c /tmp/AGENTS.apres.md` : moins de 20480 octets (critère SC04). Si le résultat dépasse : arrêter, noter la taille, ne pas remplacer.
2. `diff /tmp/AGENTS.avant.md /tmp/AGENTS.apres.md | rg -c '^<'` : égal au nombre de lignes du bloc de T032.
3. `diff /tmp/AGENTS.avant.md /tmp/AGENTS.apres.md | rg '^>' | wc -l` : 0 (aucune ligne ajoutée).

Puis `cp /tmp/AGENTS.apres.md AGENTS.md`. Remplacer la ligne « Les reçus datés ... » par : `Les reçus de session sont dans specs/recus-de-session.md. Ils ne sont pas des instructions de conception ni une preuve du runtime courant.`
Commit : `docs(consignes): Retirer les recus de session d'AGENTS.md`.

## T035 - Porte du lot

1. `wc -c AGENTS.md` : moins de 20480.
2. `rg -n '^## ' AGENTS.md` : les 6 sections d'origine plus « Règles permanentes issues des reçus » plus « Communications attentives ».
3. `diff <(tail -n +4 specs/recus-de-session.md) <(git show origin/main:AGENTS.md | sed -n "${D},$((F-1))p")` : vide (le bloc conservé est identique à l'ancien, si `main` n'a pas changé AGENTS.md depuis T031).
4. `python3 scripts/check-docs.py` et `bash scripts/check-secrets.sh` : code 0.
5. Pas de `C-TESTS` : aucun code.
6. Bilan dans `implementation.md`. Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- `AGENTS.md` a été modifié sur `main` pendant le lot (nouveau reçu) : arrêter, refaire T031 à T034 depuis un worktree neuf. Ne pas fusionner à la main.
- Le fichier reste au-dessus de 20 Ko : arrêter, noter la taille et les sections les plus lourdes (`awk 'length>0{n+=length($0)+1} /^## /{print n, $0}' AGENTS.md`).
- Il est interdit de résumer ou de raccourcir un reçu : seuls le déplacement et la remontée de huit lignes sont autorisés.
