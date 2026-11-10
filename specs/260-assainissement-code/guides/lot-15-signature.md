# Lot 15 - Proposition de signature des publications (T141-T142)

But : écrire une **proposition** pour signer les fichiers publiés (installateur, archives). Aujourd'hui l'empreinte SHA-256 est publiée au même endroit que l'archive : si l'endroit est compromis, les deux le sont. Mesures : `releves.md` section 11 (dernière ligne).
Branche : `session-260-lot15-signature`. Worktree : `.worktrees/260-lot15-signature`. **Aucun code, aucun script, aucune clé, aucune installation d'outil.** Porte D8 : la décision revient à François. Action par défaut de D8 : écrire la proposition, ne rien mettre en place.

## T141 - Rédiger la proposition

Créer `specs/260-assainissement-code/validation/lot15-proposition-signature.md`. Sections, dans cet ordre :

1. **Constat** (lu dans le dépôt, avec les commandes) :
   - Comment l'installateur vérifie l'archive : `rg -n 'sha256|checksum' scripts/install.sh` (téléchargement de `$archive.sha256` depuis la même release).
   - Quels fichiers sont publiés : `ls docs` et `rg -n 'releases/download' scripts/install.sh docs README.md | head`.
   - Qui publie et comment : lire la procédure de publication (`rg -n 'publier' AGENTS.md docs scripts | head` ; mémoire du dépôt : script `publier.py`, s'il existe). Écrire seulement ce qui est lu.
2. **Menace** : une phrase. Un attaquant qui remplace l'archive et son empreinte sur le serveur de publication n'est pas détecté.
3. **Options** (tableau : option / qui garde la clé / ce que vérifie l'utilisateur / effort / limites) :
   - A : ne rien changer.
   - B : signature détachée par une clé privée gardée hors ligne (outil à choisir parmi `minisign`, `gpg`, `ssh-keygen -Y sign`) ; l'installateur vérifie avec une clé publique intégrée.
   - C : attestations de provenance de la plateforme d'hébergement (signature liée à la source et au flux de publication, sans clé à garder).
   Chaque cellule se rapporte à une propriété connue de l'option. Une propriété qu'on ne peut pas vérifier ici s'écrit « à vérifier ».
4. **Effet sur l'installateur** : quelles lignes de `scripts/install.sh` changeraient (par nom de fonction), sans les écrire. Le contrat de l'installateur est testé par `scripts/tests/test_install.py` : l'écrire.
5. **Décision demandée à François** : choisir A, B ou C, et qui garde la clé pour B. Aucun délai.
6. **Ce qui n'est pas fait** : aucune clé générée, aucun outil installé, aucun changement de script.

Règles : aucune adresse, aucun nom de compte ni chemin privé ; aucune clé publique inventée ; ne pas écrire qu'une option est « sûre » sans dire contre quoi. Commit : `docs(specs): Proposer la signature detachee des publications`.

## T142 - Porte du lot

1. `python3 scripts/check-docs.py` et `bash scripts/check-secrets.sh` : code 0.
2. `git diff origin/main --stat -- . ':!specs'` : **vide** (rien hors `specs/`).
3. `decisions.md` : D8 reste `en attente`. Ne pas la modifier : François la remplit.
4. Bilan : lien vers la proposition, « décision D8 demandée ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- La tentation de « préparer » un script de signature : non (R6.7, R6.10). L'idée va dans la section « Idées ».
- La procédure de publication n'est pas lisible dans le dépôt : l'écrire (« procédure non lue ») et continuer.
