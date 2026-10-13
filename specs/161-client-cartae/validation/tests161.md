# Vérifications du développement161

Date : 2026-10-13. Exécution locale en fixtures privées seulement. Aucun SSH réel, Cargo réel, modèle, service ou écriture Git. Tests de transport réels et déploiement restent des gates ROOT distincts.

## Commandes et résultats

Toutes les commandes ci-dessous partent de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae.

- `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` : RC127, script absent ; fallback implement appliqué en lisant les artefacts, checklist16/16, plan/tasks/contrat/research/data-model/quickstart. Aucun toolkit ajouté.
- `bash scripts/tests/federation_089_test.sh Federation089.test_strict_commit_accepte_source_propre Federation089.test_strict_refuse_avant_ssh` avant code : RC1, 1FAIL/1PASS ; option inconnue prouvée.
- Après T003 : mêmes deux cas + dry-run + client-only : RC0,4PASS,1.216s.
- T004 avant code : trois tests manifest, RC1,8assertionsFAIL ; les corruptions/manquants/extra/liens étaient acceptés et les filtres n'étaient pas attestés.
- Après T005 : mêmes trois tests manifest RC0,3PASS,3.531s.
- T006 avant code : test_build_ressources_bornees_et_target_explicite RC1,1FAIL (jobs absent).
- Après T007 : ressources + legacy dirty + préfixe occupé + toolchain absente + pipefail RC0,5PASS,2.533s.
- T009 : test_bascule_contractuelle_exacte_aller_retour_et_refus RC0,1PASS,0.106s. Extrait contractuel exact, sans copie parallèle de sa logique.
- Relecture de sécurité : heredoc du manifest explicitement quoted. Replay deux tests manifest RC0,2PASS,2.839s ; nom `docs/$(printf expanded).md` préservé littéralement.
- `bash -n scripts/deploy-remote.sh` et `bash -n scripts/tests/federation_089_test.sh` : RC0 chacun.
- **Harness complet unique** `bash scripts/tests/federation_089_test.sh` : RC1, **30PASS/1FAIL**,31tests,11.625s. Échec client-only : HOME/Library créé par Python Apple, pas par une opération Bridget.
- Diagnostic empirique isolé : `/usr/bin/python3 -c 'import hashlib, pathlib, stat, subprocess, sys'` avec HOME temporaire : RC0 ; fichiers réellement observés sous Library/Caches/com.apple.python/.../*.pyc. Correction : Python inline local/distant avec `-B`. L'oracle HOME intact reste inchangé.
- Relecture ROOT : les erreurs Git status/ls-files strict doivent refuser ; affectations Bash avec RC propagé et check_output Python. Sous-cas de refus ajoutés au test strict existant.
- **Replay final affecté** : RC0, **8PASS**,10.765s, liste exacte ci-dessous. Aucun deuxième harness complet et aucune FULL Rust. Les23autres cas du harness précédent n'ont pas été relancés ; leur résultat reste celui de cette exécution précédente.
- `git diff --check -- scripts/deploy-remote.sh scripts/tests/federation_089_test.sh docs/federation-services.md` : RC0.

```sh
bash scripts/tests/federation_089_test.sh \
  Federation089.test_deploiement_client_only_prive_sans_non_suivis \
  Federation089.test_strict_commit_accepte_source_propre \
  Federation089.test_strict_refuse_avant_ssh \
  Federation089.test_legacy_dirty_conserve_identite \
  Federation089.test_manifest_espaces_modes_et_exclusions \
  Federation089.test_manifest_refuse_transfert_altere_avant_build \
  Federation089.test_manifest_refuse_nom_ambigu_et_ancetre_lie_avant_ssh \
  Federation089.test_build_ressources_bornees_et_target_explicite
```

Sortie finale : `Ran 8 tests in 10.765s`, `OK`, exit0. Les sous-cas stricts incluent HEAD incorrect, valeur invalide, dirty, assume-unchanged, skip-worktree, erreur Git status et erreur Git ls-files, tous avant SSH. Les refus de transfert incluent manifest altéré, octets différents, absence, ajout et lien, sans appel Cargo build.

## Tâches et limites

T002–T007 vérifiées aux frontières modifiées. T008 bloc contractuel relu sans modification, lien quickstart confirmé ; T009 aller/retour, référence étrangère, temporaire occupé et cible non exécutable vérifiés. T011 procédure publiée sans prétendre livraison. T013 syntaxe et harness plus rejeux documentés. ROOT conserve l'autorité de cochage et de livraison.

Nice est observé via os.getpriority dans le vrai processus de la doublure Cargo : >=10. Jobs2, target absolu privé, RUSTUP_AUTO_INSTALL0, offline et build unique sont assertés. Ce n'est pas une mesure de ressources d'un vrai build Linux. Un seul build à la fois relève de l'opération de livraison, pas d'un nouveau verrou global.

L'extrait de bascule n'est pas un CAS inter-processus ; sa limite TOCTOU est conservée. Le manifest source et la source doivent rester sous contrôle d'un seul livreur pendant copie/build. Aucune assertion globale de succès Rust ou d'activation Cartae.

## Self-review Article XIX/XX

Le contrôle source et les bornes de ressources sont nécessaires pour ne pas étiqueter une autre source160 et pour limiter la charge. Réemploi du déployeur, de federation_hash, du format SHA, du harness et de os.replace ; aucun module, service, table, dépendance ou helper permanent créé. Deux blocs Python inline servent exclusivement cette frontière de livraison. Complexité linéaire dans les octets transférés. Aucun fichier Rust modifié. Le répertoire temporaire de manifest est privé, supprimé par chemins exacts au trap ; les fixtures retirent seulement leur propre racine. La relecture a corrigé le quoting du heredoc et la propagation des RC Git. Non vérifié ici : caches Cargo Linux, build réel, communication distante, activation, push et cleanup de production.

## Empreintes des fichiers vérifiés

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh : `5f705cabb98ad76d19c633178a318639d2d79fc49809f58e4ee57684d3950651`
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/federation_089_test.sh : `cd573c10556117e939c8336e282b55dc94251087f78047777c9fb10df2efef8b`
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/docs/federation-services.md : `d03535f204a9e0afd0e2532681e5b6af412208d96328c971a6dcb808b2d12914`
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/161-client-cartae/contracts/client-delivery.md : `e07ac087c9a58e5678be38be7004cf9fb045d786ddd386223062b26aaf8674d9`

## Correctif après audit — autonomie du paquet

Constat vérifié : le harness089 extrait le bloc contractuel161, mais le paquet089 ne le copiait pas. Correction limitée à une entrée dans la liste existante du paquet et à sa présence requise dans le test existant. Aucun nouveau helper.

Commandes : `bash -n scripts/package-089-core.sh` et `bash -n scripts/tests/package_089_test.sh` : RC0. `bash scripts/tests/package_089_test.sh` exécuté une fois : RC0, PASS paquet autonome, cible occupée, parents symboliques destination/source, mutation SHA-256.

Paquet produit : /private/tmp/b89-package-test.ljfkhJ/source. Depuis ce paquet uniquement :

```sh
bash /private/tmp/b89-package-test.ljfkhJ/source/scripts/tests/federation_089_test.sh Federation089.test_bascule_contractuelle_exacte_aller_retour_et_refus
```

Résultat : RC0,1PASS,0.111s. Aucune autre suite089 ni Rust rejouée.

Fixture conservée pour nettoyage ROOT : /private/tmp/b89-package-test.ljfkhJ. Inventaire :744 fichiers réguliers non symboliques,24668206 octets logiques, sans prétention de gain physique. Le test a volontairement ajouté une nouvelle ligne au README du paquet pour son oracle de corruption ; le manifest n'est donc plus globalement conforme après ce test. Le contrat et le harness utilisés ensuite sont inchangés.

Empreintes :
- /private/tmp/b89-package-test.ljfkhJ/source/SOURCE-MANIFEST.sha256 : `2a578cbb6823b46f2a7e90a0e1601bc45384c2297ac69012d4a444b7629eb5b5`.
- /private/tmp/b89-package-test.ljfkhJ/source/specs/161-client-cartae/contracts/client-delivery.md : `e07ac087c9a58e5678be38be7004cf9fb045d786ddd386223062b26aaf8674d9`.
- /private/tmp/b89-package-test.ljfkhJ/source/scripts/tests/federation_089_test.sh : `cd573c10556117e939c8336e282b55dc94251087f78047777c9fb10df2efef8b`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/package-089-core.sh : `cc33101a0b0fac3997c8b575c5714c22049d0ce01721e48450412dcc3e04cad4`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/package_089_test.sh : `8eb58b216b000ef5f9c9a5e9ebce38cb9dbdc981f79b5de8173657629bbb7e98`.
