# Audit T038 — routage de flotte 168

Audit V15 final r2 du 15 octobre 2026. Décision calculée : `pass`, `release_blocked=false`, grade A, compteur de pénalités 98. Les 14 contrôles applicables sont vérifiés. Aucun constat critique, élevé ou moyen actif ; deux faibles et un informatif restent ouverts. `source_drift=[]`.

Le chiffre 98 est le compteur de pénalités du protocole, pas une note des axes qualité. La couverture de 100 % concerne les 14 contrôles applicables, pas les lignes du dépôt ni la couverture des tests. La session a été initialisée avant la mise à jour quality1 : les 12 axes et 42 critères restent non notés (`N/A`), sans attribution automatique de 100.

## Corrections et preuves

Cinq constats sont corrigés et relus :

- Préparation Worktree distante : mode conservé lors du premier PrepareV2, jeton inerte refusé au lancement, préparation réelle après admission durable.
- Git répété par candidat : mutualisation bornée par tuple dans un lot, sans mutualiser les droits ni les contrôles ultérieurs. Le scénario initial mesure 521 appels pour 128 candidats et deux tuples ; l’oracle corrigé atteste au plus 40 appels. Aucune valeur 17 ni accélération temporelle n’est déduite.
- Échéance du petit décideur : même instant absolu transmis à la préparation et aux entrées/sorties ; permis conservé jusqu’à l’arrêt physique. Le cas déjà expiré ne lance aucun enfant. Aucun scénario initial en échec n’a été exécuté pour cette échéance ; aucune borne universelle des préflights synchrones n’est affirmée.
- Observation figée : définition, mapping et époque comparés entre les deux sondes ; les droits peuvent varier selon l’effort. Les dérives définition/mapping sont exécutées ; la variante d’époque distincte reste vérifiée par lecture.
- Branche inaccessible supprimée dans `source_base_commit`, avec oracle Git existant conservé.

Treize derniers oracles distincts sont PASS : douze dans le run r1, puis le seul scénario distant repris exactement en r2. Le run r1 conserve son échec de réponse attendue. Seule la fixture a ensuite intégré le corps réel complet avec annotation de workspace ; les sources produit sont identiques entre r1 et r2. Le replay r2 termine RC0 en 57,276 s (construction : 17,313 s ; test : 39,256 s). Ces résultats ne constituent pas une suite complète verte.

Les commandes réelles, leurs arguments, codes retour et sorties sont archivés dans les runs `20261015T112308785798Z` et `20261015T112655414155Z`. Le runner les expose sous son API `custom`, avec les commandes SSH/Cargo réelles et les contrôles de stabilité des sources. Les scanners fichiers/historique ont leurs runs propres. Le premier lancement standard qui scannait zéro octet est conservé et ne vaut pas preuve de succès. Le scan historique complet termine RC0 en 243,537 s : 280 sites ont été qualifiés puis réfutés par contre-lecture indépendante, sans utiliser les clés contre un service. Les 54 alertes du scan fichiers ont également une réfutation indépendante.

## Constats ouverts

- LOW `RUSTSEC-2026-0098` : contraintes de noms URI dans rustls-webpki 0.101.7, atteignables par le client HTTPS. Une chaîne signée mais mal émise est nécessaire ; aucune exploitation observée.
- LOW `RUSTSEC-2026-0099` : contraintes DNS de certificats wildcard dans la même dépendance, avec les prérequis de l’avis mainteneur. Son héritage ne réfute pas le risque.
- INFO `RUSTSEC-2024-0449` : unsafe-libyaml 0.2.11 non maintenu, via serde_yaml et la lecture locale des pins. Aucune vulnérabilité précise n’est inventée.

## Résultats calculés et conservation

Input final SHA-256 : `2cf0c3e3202ee21e094dd2620d5cb4c627cec76f7c150c0deb25c3aa6767435d`.
Manifest du protocole SHA-256 : `4d8a0c215db8e256a56e1489a4b3801fef767f52fb55356578bf69f30df2d1b3`.
Export de sources SHA-256 : `12e812fd7fffa9a45eb7b2c5500a22f3146c2cd270099920347f8713563a1e03`.

Validation observée par ROOT, sans nouvelle exécution lors de l’archivage :

`/Users/moi/.local/share/audit-v15/venv/bin/python /Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v15/scripts/audit.py validate /private/tmp/bridget-audit168-source-20261015/audits/2026-10-15/session-2026-10-15-spec-168-v15-02`

RC0, sortie exacte : `{"valid": true, "decision": "pass", "grade": "A"}`.

Archive complète privée, révisions et anciennes preuves incluses :

/Users/moi/.cache/bridget-delivery168/audit-final/audit168-v15-r2-20261015T121259Z.tar.gz

/home/moi/.local/state/bridget-delivery168/audit-final/audit168-v15-r2-20261015T121259Z.tar.gz

Archive SHA-256 : `222a54213abd1c3f378b8815d8e7cd8352cff0ba68f2eaf02f35e0d7b785ad49`.
Manifest exhaustif : /home/moi/.local/state/bridget-delivery168/audit-final/audit168-v15-r2-20261015T121259Z.tar.manifest.json
Manifest SHA-256 : `3104b3471d013d0e91363d14562965f633f5d530e9b249c2da80cf2de531839e`.
Les 231 fichiers ont été vérifiés par SHA-256 après lecture de leurs octets extraits de l’archive, puis la source a été relue pour vérifier son invariance. La copie distante est vérifiée de la même façon. Aucune révision existante n’est écrasée.

Les seuls résultats calculés utiles sont conservés dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/audits/2026-10-15/session-2026-10-15-spec-168-v15-02 ; les données brutes complètes restent dans l’archive privée canonique.

## Limites de cette clôture

Périmètre : code et contrats T002–T035/T050–T056 avant livraison. La suite complète, la qualification des registres, l’activation, les recettes réelles et la release restent des étapes futures à cette clôture. Aucun succès de production, appel modèle réel, ou résultat global vert ne découle de cet audit.


## Annexe T038 — delta de tests après la FULL T036 (15 octobre 2026)

L’audit V15 r2 précédent et son archive restent immuables. Sa décision `pass`, son grade A et son compteur de pénalités 98 concernent son périmètre et son manifeste d’alors. Les axes qualité restent non notés. Aucun nouveau score, contrôle de scanner ou audit global n’est déduit de cette annexe.

Les neuf fichiers modifiés après la FULL contiennent seulement des correctifs de fixtures et une instrumentation conditionnelle de test. Le code produit reste inchangé. Deux contre-lectures indépendantes ont approuvé les deltas dans leur périmètre :

- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-post-full168.md — SHA-256 `c8240ff0386b00d27214c776028c8237b300f79dc1173b9983336bd3369737bd`.
- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-delta-core-t3.md — SHA-256 `42ea238ebad7c0a9e89c431bdba77f12ba3ce70129a34e3bf334d373a34068e2`.

Les limitations de ces revues restent : la sentinelle Docker observe les appels par son PATH, sans prouver tous les chemins absolus ; le PATH transmis est celui du lanceur qualifié ; le diagnostic T3 n’a pas été atteint au replay PASS et ne ferme pas la cause initiale inconnue.

Le reçu final T036 est publié dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md. Chronologie : FULL unique RC101, **2 820 PASS / 32 FAIL / 71 ignorés** ; neuf migrations PASS au binaire original ; r1 **12 PASS / 3 FAIL / 8 différés** ; r2 **8 PASS**, compilation `test-support`, umask 0077, sources stables. Bilan des échecs initiaux : **29 PASS après reprise et trois annulations 159 non validables sur Linux**, faute d’observateur kqueue dans cette plateforme. Aucun `ignored` n’a été ajouté ; aucun GREEN global n’est revendiqué.

Reçu terminal r2 : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/result.json — SHA-256 `2b359ce75ecc546c76dba1e4a3122c467566854df1739c160924d42fea3efe65`. Les noms exacts, logs et hashes sont publiés dans le reçu de tests. Les trois FAIL intermédiaires r1 et la FULL initiale restent conservés. Aucun nouveau scan ou recalcul V15 n’a été lancé pour cette annexe. L’activation, les recettes réelles et la release restent hors de cette preuve.


### Copie publique de présentation

Au contrôle d’index avant commit, une espace finale dans scoring.md ligne671 a été retirée de la seule copie publique Markdown. Aucun résultat, finding, score ou objet JSON n’est modifié. L’archive privée canonique reste byte-identique ; sa copie originale de scoring.md a le SHA256 0114879cb61a55c24bca6e37abaa4b6cc4c15854492c48f58203c85e9b618d60. Copie publique sans espace finale : 496fed0ce6eb52717a89715e7fb266663d237afe070698b89bddc59c48809cd1. Ce changement de présentation n’est ni un recalcul ni un nouvel audit.


## Annexe T045B — délai service configurable après activation A

La recette privée sur source A a utilisé le repli deterministic avec caller_deadline ; elle ne prouve pas une décision Laya valide. Le correctif produit retire le plafond fixe de 2 s et respecte service_timeout_ms, de 1 à 30 s, dans le budget global restant ; défaut 2 s inchangé. Le diff produit d’une ligne est relu APPROVE par verify_git_handoff selon ROOT ; aucun point bloquant n’est signalé dans ce delta.

Le test exact existant daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete passe : RC 0, 1 PASS / 0 FAIL, avec les trois sous-cas service 4 500/global 5 000/POST 2 200 ms, défaut 2 000 ms et santé 600+POST 650 ms dans un global 1 000 ms. Source finale SHA-256 5668719253fb44f96ccf28a878c952da559c0a3349f73adf0ebf56462c90da7c, stable à l’exécution.

Preuve : /home/moi/.local/state/bridget-delivery168/tests/t045-budget-service-config-r1.json ; SHA-256 738ac4d040cd13d59d5273821b229f323aa5da2cb406a0de991b0308f6f43149. Détails et log : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md, complément T045B.

L’audit V15 pass A sur son manifeste source A est conservé. Ce delta ciblé ne constitue ni un nouveau scan V15, ni un score global recalculé, ni une nouvelle FULL ou passe P4. Les deux LOW TLS et l’INFO restent ouverts. T045 reste ouverte ; les nouveaux paquets, l’activation et les trois recettes devront viser le futur commit B.
