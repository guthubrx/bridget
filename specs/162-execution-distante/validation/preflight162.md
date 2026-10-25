# T001 — Préflight162 et périmètre protégé

Document préparé le 2026-10-13T10:26:18.228841+02:00. T001 documentaire seulement ; case tasks.md laissée au principal après Analyze. Aucune compilation, SSH supplémentaire, recette métier, activation, relance ou suppression exécutée pour cette rédaction.

## Bases Git et préparation isolée

Le reçu161 du 13 octobre à09:16:46+02:00 établit main Bridget propre et égal à github/main : 84cd24305837b70f866ae09bff86235b9743071e. Cette égalité distante est une preuve161 réutilisée, pas un nouveau ls-remote162. Le relevé initial162 de09:26:25+02:00 atteste :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-1).

Nouvelle lecture Git locale seulement, effectuée pour ce document :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-2).

Ces lectures ne figent pas les edits concurrents ultérieurs du lot. Aucun index, branche ou source produit modifié par cet owner documentaire.

## Ressources162 et travaux étrangers

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-4).

Les vieux chemins149 apparaissent encore dans Git : un ancien reçu de nettoyage ne permet pas de les retirer maintenant. Toute archive, cache, DB, application et branche d’un autre lot reste protégée.

## Processus : historique161, pas preuve fraîche162

Le fichier /Users/moi/.cache/bridget-delivery161-20261013/runtime161-final.json attestait les identités suivantes après161 :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-5).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-6).

## Projet : activation toujours suspendue au choix utilisateur

Observation Cartae du13octobre07:22:44UTC : /home/moi/20.cartae est un dépôt Git, origine https://github.com/guthubrx/cartae.git, HEAD75e0ad6c4430cd2ded577cf01d4c5b490680b4f4, une entrée dirty. Son nom et son contenu n’ont pas été lus. Cette modification reste intégralement protégée ; ni reset, ni checkout forcé, ni stash automatique.

Le chemin /home/moi/10.Scripts/64.bridget est absent. SurMac, /Users/moi/20.cartae et /Users/moi/Nextcloud/10.Scripts/20.cartae sont absents ; /Users/moi/11.Repositories/cartae existe mais n’est pas un dépôt Git. Aucun mapping interhôtes d’un même projet n’est prouvé.

Le choix utilisateur des projets/racines autorisés reste ouvert. Le candidatCartae ne vaut pas approbation. Développement et tests isolés peuvent avancer ; activation d’une route et recette sur projet réel attendent ce choix. Routes désactivées par défaut.

## Préconditions worker et fournisseurs déjà observées

Préflight du13octobre07:49:45UTC : systemd --user joignable, Linger=yes. bridget-worker-cartae.service était not-found/inactive, donc aucune collision d’unité à cet instant. Manager degraded expliqué par deux services de portail desktop failed, hors périmètre. Aucun service n’a été réparé, arrêté ou relancé.

- /home/moi/.config/bridget : répertoire réel, propriétaire1002, mode0700.
- /home/moi/.local/share/bridget-worker : absent au préflight ; future racine privée worker à provisionner après gates.
- /home/moi/.config/systemd/user : propriétaire1002, mode0755 ; /home/moi/.local/share : propriétaire1002, mode0775.
- Claude installé /home/moi/.local/bin/claude version2.1.251. Codex /home/moi/.local/bin/codex version0.150.1 ; PATH SSH résout à la place /usr/local/bin/codex version0.98.0. Commande absolue à fixer.
- Registre /home/moi/.config/bridget/agents.json mode0600 : fixture k1-test-fixture et codex/codex_app_server, modèle déclaré gpt-5.6-terra. Registre core distant absent. Présence binaire et modèle déclaré ne prouvent pas authentification, quota ou disponibilité d’exécution.

Aucun secret ni config brute publié. Le futur credential /home/moi/.config/bridget/worker-cartae.credential, l’état /home/moi/.local/share/bridget-worker/cartae et ExecStart doivent être provisionnés avec les contrats162 finaux, pas déduits de ces observations.

## Caches et budget temporaire décidés par ROOT

Contrôle ROOT conservé dans /Users/moi/.cache/bridget-delivery162-20261013/cache-preflight.txt : BRIDGET_BUILD_CACHE_GIB=20 BRIDGET_BUILD_CACHE_DAYS=7 python3 scripts/build.py clean --dry-run, RC0 ; 32,80Gio logiques dans /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug. Aucune suppression exécutée.

Le nettoyeur agit par profil entier. Le budget20 aurait proposé la suppression de tout le cache, et non seulement12,80Gio. ROOT conserve le cache utile162/163 et fixe temporairement BRIDGET_BUILD_CACHE_GIB=40 pour les validations162/163. Il s’agit d’un budget de rétention, pas d’un plafond empêchant une compilation. Cette décision remplace la proposition20Gio de la note antérieure validation-commands.md. Ne modifier aucun outil, ne contourner ni lsof ni les verrous. Recontrôle et nettoyage gardé après la dernière réutilisation162/163.

Charge bornée : une seule compilation active, Cargo jobs2, nice10, deux threads de test. Les replays restent limités aux tests affectés. Aucun nouveau FULL exécuté dans ce préflight.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-7).

Linux : cacheCargo161, source/préfixe161 actif et rollback096 conservés. Le nettoyage target161 avait été refusé par garde lsof incomplète ; cette conservation ne donne aucune permission de forcer la purge en162. Aucun téléchargement, setup distant ou changement de toolchain dans ce préflight.

## Couverture de T001 et limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-8).

T001 est prête pour relecture ROOT après Analyze. Pas de case tasks.md modifiée par cet agent. Les gates d’activation (choix projet, fournisseurs réels, credentials, sauvegardes, CLI/unit finale et recette réelle) restent ouverts.

## Registre des preuves relues

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/preflight162.md#extrait-9).

## Relecture ROOT — T001 validée

Analyze deux passes relu, aucun constat actif. Préflight relu intégralement : provenance, propriété et limites explicites. Cette préparation évite de refaire les contrôles distants et protège le travail étranger. Hypothèse : les observations historiques devront être rafraîchies avant activation. Aucun test, runtime actuel ni route de production validé par cette case. Aucun code, helper ou dépendance ajouté. T001 cochée après cette relecture.
