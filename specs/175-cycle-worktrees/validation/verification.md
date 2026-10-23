# Vérification — Session 175
## Code et tests
- Rust : 1 test natif175 via le vrai handler de statut, 10 états, PASS. Le premier RED échouait sur l'absence de retention. Une fixture intermédiaire réutilisait une garde libérée, monotone dans save_projected ; correction par tâche distincte, sans changer la production.
- Rapports154 : 5 PASS, dont budget UTF-8 et ligne unique ; isolation154 : 1 PASS.
- Python : 42 tests, 39 PASS, 3 SKIP explicites. Deux skips remplacent les deux échecs préexistants dus aux avertissements lsof ; le troisième exige aussi un lsof complet. Le test de processus réel tenu hors cwd passe avec l'inventaire contrôlé de la fixture ; la politique fail-closed réelle est testée séparément.
- Documentation : 27 pages, 422 liens locaux, 11 paires EN/FR, PASS.
- git diff --check : PASS.

## Périmètre et limites
Les tests ne qualifient pas toute la délégation distante. Aucun fournisseur métier ni mission du projet69 créé/rejoué.
La projection utilise le dernier rapport enregistré, pas une inspection fraîche. Le JSON d'inventaire expose le total du registre, principal inclus, et le compte sous le cwd choisi.
Un avertissement lsof protège les dossiers : il ne retire pas la valeur informative des observations Git.
Les droits, schémas persistés, protocoles et dossiers existants sont inchangés.
Correction technique, sans changement d'interface graphique : pas de diaporama requis.

## Relecture de responsabilité
Deux fonctions Rust existantes sont étendues. Les observations Git/processus et les gardes du nettoyeur sont partagées avec l'inventaire ; aucune seconde suppression.
Le registre enrichi garde locked/prunable. L'index des ancêtres des fichiers ouverts est construit une fois par passage, puis recherché en O(1) par worktree. Les contrôles d'imbrication sont O(W²), W borné opérationnellement ; aucune boucle réseau.
Les noms de fonctions d'observation isolent des règles existantes. Worktree remplace le tuple existant pour porter les flags Git. La table de remédiation sert les deux rendus humain/JSON.
Les suppressions réelles restent soumises aux gardes existantes et ne sont pas exécutées par l'inventaire.

## Observation réelle Cartae
Commande : python3 scripts/build.py worktrees --repo /home/moi/10.Scripts/64.bridget --json. Code0, 10,78s. 48 inscriptions dans le périmètre, 49 principal inclus, état normal. Processus inconnu (lsof incomplet), aucun candidat autorisé. Empreintes identiques avant/après : refs, registre et index. Données brutes privées : /home/moi/.cache/bridget-worktrees175/inventory-proof.json. Aucun retrait.

## Retouches de contre-revue finale
Task06d96a63, Claude : APPROVE_WITH_CHANGES, aucun retrait protégé trouvé. D1 (lsof sans secondaire), D3 (préfixe système lié) et D4 (nom candidat_a_examiner dans les guides) corrigés. Test final : 44 testsPython, 41PASS/3SKIP explicites ; RuffF/E9, docs et diffcheck PASS. Rust inchangé depuis les sept tests PASS.
D2 non retenu : la capacité compare les chemins inscrits sous le cwd canonique, comme preflight_on_host (workspace_git.rs, filtre path.starts_with(worktree_dir)). Résoudre chaque inscription changerait ce contrat. Les chemins symboliques sont protégés séparément par l’évaluateur, et aucune capacité d’admission exacte n’est promise.
La limitation de revue sur les processus est précisée : le test réel tenant un fichier horscwd passe avec lecture contrôlée ; la complétude lsof de production reste non prouvée.
