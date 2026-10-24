# Validation — 24 octobre 2026

## Contrôles ciblés

- `python3 -m unittest scripts.tests.test_install -v` : 16 tests PASS.
  Vraies archives, HOME avec espaces, téléchargement simulé sans réseau :
  corruption, erreur HTTP, exécutable invalide, lien dans archive, traversée de
  chemin, remplacement d'un lien vers un ancien binaire sans modifier sa cible,
  profils et comptes conservés (sans masquer le profil de connexion Bash),
  répétition sans perdre le rollback distinct,
  verrou, version figée (refus multi-lignes), SHA en majuscules, plateforme sans
  paquet, compilation par le wrapper et échec de build conservant le binaire.
- Cas cache : le vrai module build.py et deux vrais dépôts Git confirment qu'un
  clone temporaire différent perd son identité de cache ; le chemin stable
  conserve cette identité. Deux installations source sont aussi testées dans
  le contrat du script, avant les essais réels répétés.
- `sh -n scripts/install.sh` et `git diff --check` : PASS.
- `python3 scripts/check-docs.py` : PASS, 27 pages, 425 liens locaux.

## Essais réels isolés

- macOS arm64 : téléchargement et installation de v0.4.1 depuis GitHub, code 0.
- Linux x86-64 sur Cartae : même release, code 0.
- Linux, `--source` : clone réel de main au commit
  `1d49d9cf81abc48e045645e21b88419e8e4df164`, compilation optimisée réussie
  en 1 min 28 s. Wrapper Cargo utilisé avant/après ; commande `installed`
  exécutée après copie vérifiée, intermédiaires retirés par le wrapper.
- Les installations utilisent des HOME distincts des comptes actifs.
  Aucune installation sur SYS-LE-2 ou KS-STOR ; aucun daemon relancé.
- La compilation émet 18 avertissements préexistants ; aucune suite Rust
  complète n'est revendiquée. Ce changement ne modifie aucun code Rust.
- Après revue, checkout stable : deux installations `--source` réelles
  consécutives dans le même HOME, codes 0 et 0, compilations 1 min 34 s puis
  1 min 26 s. Le second passage conserve bien l'identité du cache.
- Version finale des tests exécutée sur Mac et Linux : 16 PASS de chaque côté.

Les sorties détaillées et les binaires d'essai restent hors Git.
Les tests sans réseau sont ajoutés au workflow CI existant.
Le chemin public et le push sont vérifiés à la livraison.

## Périmètre et limites

L'installateur pose la CLI. Il ne choisit pas entre maître et client fédéré,
ne démarre aucun service et ne copie aucun compte fournisseur. Il indique ces
limites à la fin. Pas de DMG ni de modification des releases historiques.
La somme SHA-256 provient du même éditeur que l'archive, sans signature tierce.
La construction source exécute le code du dépôt explicitement choisi par l'utilisateur.

Analyse de cohérence : spec, plan, tâches, code et documents comparés ; les deux
canaux et la version figée utilisent le même chemin de validation/remplacement.
Complexité linéaire dans la taille de l'archive/binaire, nombre de profils borné.
Minimalisme : aucun framework ni dépendance Python pour la release ; T3 ne
fournit pas d'installateur Rust réutilisable. Charge future limitée à un script
et ses tests. Le script est public et la redirection du site vise cette source
unique. Potentiel minimalisme retenu : 0 ligne supprimable sans perdre un
contrôle d'installation ou de conservation demandé.
