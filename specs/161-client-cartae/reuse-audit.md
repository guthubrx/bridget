# Audit de reutilisation de l'existant — SPEC-161 Client Cartae

## Decision

Statut: PASS
Date: 2026-10-13
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/161-client-cartae

Conclusion courte: les douze items du plan réel réemploient les outils, contrats ou procédures du projet. Aucun nouveau déployeur, nettoyeur, service, crate, endpoint, table ou dépendance n'est nécessaire. Deux précisions de réemploi sont tranchées ci-dessous, sans arbitrage utilisateur requis. Ce PASS porte sur le réemploi du plan, pas sur l'implémentation, la contre-revue ni la livraison.

Sources lues : spec.md, plan.md, research.md, data-model.md, quickstart.md et contracts/client-delivery.md dans le dossier ci-dessus. Le plan relu contient le réemploi SOURCE-MANIFEST.sha256 à la ligne34 et build.py à la ligne65. Préflight standard indisponible : absence vérifiée de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/scripts/bash/check-prerequisites.sh ; FEATURE_DIR fourni explicitement par ROOT. Fallback de lecture appliqué, aucun toolkit créé.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 12 |
| Items audites | 12 |
| Reutilisations deja prevues | 12 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 4 |

Les deux existants supplémentaires précisent les douze items ; ils ne sont pas deux nouveaux composants.

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| I01 Entrée client-only, « REUTILISER deploy-remote.sh », plan:33 | Déployeur client existant | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh:45 | REUTILISER. Étendre ce fichier, pas un second déployeur. |
| I02 Gardes SSH et chemins, plan:33 | Fonctions de fédération | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/federate-ssh.sh:7 ; :56 ; :66 ; :96 | REUTILISER. Hôte épinglé, absence de configuration SSH héritée, préfixe privé neuf, ancêtres sans liens et pipefail. |
| I03 Manifest SHA avant build, plan:34 | Format SOURCE-MANIFEST.sha256 et oracle de corruption | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/package-089-core.sh:67 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/package_089_test.sh:48 | REUTILISER le format et les outils SHA ; étendre deploy pour vérifier les octets avant Cargo. Le paquetage n'est pas directement substituable au transfert Git filtré. |
| I04 Source160 isolée du runner161, plan:34 | Identité Git du déployeur et capture de références distinctes | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh:45 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/measure-089-core.py:45 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/160-reprise-quota/validation/livraison160.md:5 | REUTILISER les primitives Git. Checkout160 propre requis par la livraison161 ; aucun Build ID160 forcé sur le chantier161. |
| I05 Cargo explicite, offline et charge bornée, plan:19 et :35 | Préflight Cargo et option jobs existante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh:60 ; :83 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/measure-089-core.py:65 | REUTILISER la commande Cargo, lui ajouter jobs2 et nice10. Aucun gestionnaire de compilation ni dépendance. |
| I06 Préfixe neuf et ancien client conservé, plan:36 et :38 | Réservation privée existante et disposition096 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh:58 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/096-federate-cli/implementation.md:39 | REUTILISER. Préfixe occupé refusé. Conservation096 pour rollback ; aucune migration DB. |
| I07 Activation/rollback du lien, plan:37–38 | Opération de bascule déjà utilisée en096 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/096-federate-cli/implementation.md:41 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/lib/pilotage-release.sh:47 | REUTILISER la primitive de remplacement exact, pas l'installateur pilotage entier. Vérification de cible avant bascule et rollback ; limite de concurrence déjà explicite dans le contrat161. |
| I08 Harnais de script, plan:46 | Doubles SSH/Git/rsync/Cargo du harness089 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/federation_089_test.sh:24 ; :258 ; :299 | REUTILISER et ajouter les refus de corruption/noms non représentables et les bornes jobs/nice. Les doubles exécutent le shell distant réel. |
| I09 Statut et échange sans modèle, plan:39 | Recette réseau095 existante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/federation_095_roundtrip.py:15 ; :44 ; :71 | REUTILISER. Le roundtrip parle directement aux sockets via Python ; il atteste tunnel/maître. Status/who exécutés par le candidat attestent séparément le client. |
| I10 Nettoyage borné, plan:65 | Nettoyeur projet build.py | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/build.py:94 ; :132 ; :146 ; :219 | REUTILISER ses gardes. Ne pas prétendre qu'il retire tout le préfixe ou qu'une taille logique est un gain physique. |
| I11 Procédure de fédération, plan:47 | Documentation existante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/docs/federation-services.md:1 | REUTILISER le guide pour expliquer la mise à jour cliente ; aucun nouveau guide concurrent. |
| I12 Reçus/progression/cleanup, plan:48–49 et :60–65 | Artefacts SpecKit et format de livraison160 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/160-reprise-quota/validation/livraison160.md:36 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/AGENTS.md:1 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/feature.json:1 | REUTILISER le format de preuve, avec nouvelles preuves161. Les sources, runner, documents et runtime restent distincts. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| I03 Hachage portable | federation_hash choisit shasum ou sha256sum | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/federate-ssh.sh:104 | Tranché : REUTILISER si un hash unitaire est nécessaire ; le contrôle de manifest réemploie le format standard existant. Aucun helper SHA concurrent. |
| I07 Remplacement exact du lien | pilotage_replace_entry utilise os.replace | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/lib/pilotage-release.sh:47 ; :204 | Tranché : REUTILISER la primitive/pattern dans l'opération de livraison. Ne pas importer tout pilotage_install_release, qui impose d'autres racines et une politique origin/main. Aucun helper permanent prévu. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun nouveau composant identique proposé | Plan161:33–55 relu | Aucune. Si l'implémentation ajoute un helper, le gate anti-doublon reste applicable. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/.speckit/constitution.md, I/III/X/XVI | Français, pipeline et worktree dédié | Artefacts161 en français ; pas d'édition du checkout main pour coder. |
| /Users/moi/.speckit/constitution.md, XVIII | Complexité explicite | Hash linéaire dans les octets ; liste réutilisée, pas scans inutiles répétés. |
| /Users/moi/.speckit/constitution.md, XIX/XX | Réemploi et charge future | Un déployeur et un harness ; pas de framework de packaging ou nettoyage. |
| /Users/moi/.speckit/constitution.md, XXI/XXII | Diagnostic réel et vérifications proportionnées | Après deux échecs de même hypothèse, observer ; pas de FULL Rust répété pour le script seul. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/AGENTS.md et consignes utilisateur de session | Préserver T3/maître/pont ; livraison et nettoyage requis | Aucun restart, aucune DB supprimée ; préserver travaux étrangers ; reçus distincts. CLAUDE.md absent, constaté par lecture. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/constitution.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/standards.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/.specify/memory/user-layer.json | Constitution globale et couche utilisateur canonique | Pas de modification directe du toolkit. Références globales consultées selon ce lot script. |
| /Users/moi/.speckit/ref/code-quality-details.md ; /Users/moi/.speckit/ref/standards-tests.md | Contrats aux frontières, erreurs propagées, mocks techniques | Préflight, refus sans activation, pipefail ; réemploi du harnais technique existant, recette réelle séparée. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/089-communication-core | Paquetage SHA et déploiement client-only ; harness089 | Manifest standard et tests de refus réemployés ; pas de deuxième paquetage. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/095-federation-services/implementation.md:32 | Tunnel maître unique et recette réelle sans modèle | Ne pas recréer le tunnel ; contrôler la communication sans mission. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/096-federate-cli/implementation.md:39 | Client Linux versionné, Cargo1.92 offline, lien atomique | Préserver096 comme rollback ; ne pas transformer client en daemon distant. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/160-reprise-quota/validation/livraison160.md:5 | Source approuvée distincte des métadonnées, preuves et cleanup | Source e6812e3e7b2d051d630192857cc3f440180d51fa ; résultats historiques non réétiquetés comme tests161. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg -n 'manifest|sha256|SHA256|jobs|nice|cleanup|remote-prefix|deploy-remote' | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts et specs095/096/160 | Déployeur existant ; manifest package089 ; jobs2 measure089 ; nettoyeur build.py. Sortie large tronquée, recherches ciblées suivantes. |
| rg -n 'SOURCE-MANIFEST|ls-files.*(sha|hash)|git.*(manifest|hash)|sha256sum.*-c|shasum.*-c|source_manifest' | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/docs | Aucun manifest de transfert Git filtré déjà branché avant Cargo ; format SHA déjà présent. |
| rg -n 'atomic|replace|symlink|rename' | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/lib/pilotage-release.sh et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/federate-ssh.sh | Primitive os.replace existante et création atomique de configs distincte. |
| Lectures intégrales deploy-remote, package089, harness089 ; lectures ciblées build.py, measure089, roundtrip095 et pilotage-release | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts | Réemploi confirmé et limites documentées ; aucun test lancé. |
| rg -n '^\[|sha|tempfile' | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/Cargo.toml et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/crates/bridget-daemon/Cargo.toml | SHA Rust déjà dépendance ; aucun ajout Rust nécessaire. Plan sans nouveau schéma, route, endpoint ou service, donc aucun audit de leur architecture hors lot. |
| Lectures spec/plan/research/data-model/quickstart/contrat161 et mémoires | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/specs/161-client-cartae et sources ci-dessus | Douze items extraits et chacun couvert. Préflight standard absent ; chemin de feature fourni explicitement. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Manifest pré-build | réutiliser | Format SOURCE-MANIFEST.sha256 ; même liste Git admissible pour hash/transfert. Exclure .git, composants target, *.db, *.db-*, *.sock, liens et entrées non régulières ; contrôler les ancêtres. Refus explicite des noms non représentables. Vérifier hash du manifest puis entrées avant build. | 2026-10-13 |
| Appeler package089 tel quel | réutiliser le format, pas la commande | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/package-089-core.sh:63 copie des arborescences avec cp -R et peut inclure des non-suivis ; le paquet n'a pas Git, exigé par deploy:45. Étendre deploy est plus petit. | 2026-10-13 |
| Nouveau vérificateur générique | ne pas créer | Le besoin est une frontière de sécurité de deploy. Étendre l'entrée existante, sans module générique anticipé. | 2026-10-13 |
| Bascule du lien | réutiliser | os.replace existant vise l'entrée exacte. Opération de livraison tracée et fixture ; aucun installateur global supplémentaire. | 2026-10-13 |
| Nettoyage source/target distant | réutiliser | build.py installed:221 applique seulement budget/âge aux targets de dépôt ; purge immédiate SHA réservée aux stagings enregistrés. clean avec budgets/âge adaptés garde ses verrous/lsof et ne retire que caches éligibles. Conserver preuves et rollback ; aucun nouveau nettoyeur. | 2026-10-13 |
| Disponibilité outils | constater pendant livraison | ROOT a vérifié Python3.12.3, lsof et Cargo1.92 sur Cartae. Cet auditeur n'a fait aucun SSH. Cache offline complet et succès de build non prouvés à ce stade. | 2026-10-13 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Aucun plan ni code modifié par cet auditeur. Seul ce rapport est écrit. Contre-revue du plan et Analyze restent des gates distincts avant implémentation ; aucune preuve de tests, build, push ou activation n'est déduite du présent PASS.

## Compléments après contre-revue du plan

Statut PASS maintenu, douze items et mêmes composants. I04 est précisé par --expect-commit optionnelle dans deploy-remote.sh, obligatoire en recette161 : FR-001/002 exigent la source exacte ; préserver le mode historique sans option évite une rupture inutile. I03 impose une seule liste Git filtrée pour SHA/transfert, SHA du manifest attendu par SSH, ensemble distant exact et absence de liens. C'est une extension du manifest standard existant, pas un nouveau format ou système de paquetage. I07 fige la primitive os.replace de scripts/lib/pilotage-release.sh:47 dans le contrat et la teste via le harness existant ; aucun helper permanent. I10 utilise build.py:35–39 (fallback repo sans Git), :94 (profils), via SSH dans source ; les ressources restantes sont conservées avec justification ou traitées par les gardes existantes. Aucun doublon nouveau non arbitré.

## Complément de réemploi après contrôle du paquet

ROOT a retenu un défaut constaté par l'audit : le harness089 livré dans le paquet référence le contrat161, absent du paquet. Réutiliser scripts/package-089-core.sh et scripts/tests/package_089_test.sh ; deux lignes ajoutées aux listes existantes, aucune nouvelle entrée CLI, dépendance, abstraction ou helper. I03 s'étend à l'embarquement du contrat nécessaire ; I08 à sa présence vérifiée dans le paquet et à l'exécution de la fixture atomique depuis ce paquet. PASS de réemploi maintenu. Test de paquet RC0 et fixture atomique du paquet1PASS rapportés parROOT ; ce sont des preuves de paquetage, pas une livraisonCartae.
