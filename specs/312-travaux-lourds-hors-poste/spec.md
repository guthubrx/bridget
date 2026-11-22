# Spécification 312 : travaux lourds hors du poste de travail

- **Statut** : spécification v2, après la revue Neon v1 (4,5/10) ; plan à écrire
- **Demande** : François, 22 novembre 2026. Son Mac ne doit plus être chargé par les compilations. Bridget doit déporter les travaux lourds en tenant compte des capacités réelles de chaque machine :
  - celles qu'aucune installation ne remplace (un binaire Mac exige un Mac) ;
  - celles qu'une installation peut apporter.

  Il veut aussi une séparation nette entre trois choses : ce qui est un paramètre, ce qui relève du fonctionnement normal, et ce qui n'est qu'une action permise par un paramètre.
- **Revues** : v1 notée 4,5/10 par Neon (gemini-4-neon-high), tâche `3b000932`. Ses corrections A à E sont reprises ici. Les défauts tranchés autrement sont signalés en §9.

## 1. Constats vérifiés (22 novembre 2026)

1. Livraisons 328 à 330 compilées à la main :
   - binaire Mac sur le poste de François (328, 329), puis sur `macbook-m1` (330, release en 131 s) ;
   - binaire Linux sur `sysle2` (Debian 12, glibc 2.36), en 3 min 42 s à 3 min 49 s.
2. `cartae` et `ksstore` ont glibc 2.39 : leur binaire natif ne démarre pas sur `sysle2`.
3. `macbook-m1` n'avait pas Rust. Rust 1.92.0 (version de `rust-toolchain.toml`) y a été installé à la main. Il n'a que Python 3.9 : l'entretien des caches de `scripts/build.py` (`prune`, `digest`, Python 3.11 minimum) y échoue sans le dire. Les caches n'y sont donc jamais purgés.
4. `scripts/build.py` compile toujours sur la machine qui le lance.
5. Le routage des missions sait déjà marquer une mission « compilation ». Mais :
   - `compilation_capacity` (`registry.rs:298-301`, `0..=10`) n'est qu'un **poids de score**, pas une limite de compilations simultanées (`native_delegation.rs:2250-2256`) ;
   - la console l'accepte jusqu'à 16 (`console_admin.rs:691-693`) : incohérence à corriger ;
   - le terme de continuité `Local` vaut +20 (`registry.rs:226-238`), plus que le bonus de compilation (+10 au plus). Une mission de compilation préfère donc le poste local tant qu'il n'est pas saturé.
6. Une sonde d'outils existe déjà pour les missions de compilation (`qualify_workload_tools`, `native_delegation.rs:1345-1433` ; `remote_worker_read.rs:7-92`, `RUSTUP_AUTO_INSTALL=0`). Elle ne vérifie ni la version exacte de Rust épinglée, ni Python 3.11, ni glibc.
7. L'inventaire 311 (`console_admin.rs:1089-1134`) relève le système, le processeur et les 5 clients CLI. Il ne relève ni glibc, ni Rust, ni Python, ni le disque libre, ni la charge.
8. Les faits de ressources du routage (`host_resources.rs`, `lineage_protocol.rs:138-146`) ne mesurent ni le disque libre ni l'activité de l'utilisateur.
9. Les politiques du registre refusent les champs inconnus (`#[serde(deny_unknown_fields)]`, `registry.rs:294` et suivants). Un nouveau champ dans `RoutingMachinePolicy` casserait la lecture par un binaire plus ancien.

## 2. Vocabulaire

- **Poste de travail** : machine où François travaille. Elle est désignée explicitement, jamais déduite de la machine où la console est ouverte.
- **Travail lourd** : compilation release ou tests longs de Bridget, build de T3 Code, calcul long délégué.
- **Cible** : ce que le travail doit produire. Exemples :
  - `macos-arm64`, avec une version minimale de macOS fixée par `MACOSX_DEPLOYMENT_TARGET` ;
  - `linux-x86_64-glibc2.36` : binaire qui démarre partout où glibc vaut au moins 2.36.
- **Propriété hôte** : ce que la machine est, sans installation possible. Système, processeur, glibc native, matériel Apple.
- **Cible produisible** : couple (cible, mode) attesté pour une machine.
  - Mode **natif** : les propriétés hôtes suffisent.
  - Mode **croisé qualifié** : une chaîne de compilation croisée du catalogue est installée et a passé le contrôle ABI et un démarrage réel sur la cible.
- **Outil installable** : outil versionné absent, installable sans droits administrateur ni interface graphique, par une recette idempotente du catalogue fermé de la 312. Exemples :
  - la version de `rust-toolchain.toml` et ses composants `rustfmt` et `clippy` ;
  - Python 3.11 en espace utilisateur ;
  - plus tard `cargo-zigbuild`, une fois qualifié.
- **Action manuelle requise** : prérequis qui exige `sudo` ou une fenêtre (outils de compilation Apple, `xcode-select --install`). Bridget le signale ; il ne l'installe jamais.
- **État du moment** :
  - joignable ou en veille ;
  - charge processeur et mémoire libre ;
  - disque libre sur la racine de compilation ;
  - places de travaux lourds occupées ;
  - trousseau macOS déverrouillé, si une signature par certificat est demandée ;
  - activité de l'utilisateur sur le poste de travail.
- **Mode discret** : exécution sur le poste de travail en priorité basse et avec un parallélisme limité.
  - Priorité basse : `taskpolicy -b` sur macOS ; `nice -n 15` et `ionice -c 3` sur Linux.
  - Parallélisme : `CARGO_BUILD_JOBS = max(1, min(2, nombre de cœurs / 4))`.
- **Bail de travail lourd** : réservation d'une place sur une machine, avec une durée de vie limitée. Il est relâché à la fin ou à l'expiration, et c'est lui qui borne la concurrence réelle.

## 3. Paramètres, fonctionnement normal, actions permises

### 3.1 Paramètres (choix de François, rares et durables)

Ils sont enregistrés dans un fichier de politique dédié et versionné, `heavy-work-policy.json`, possédé par le registre maître. Ils ne sont pas dans `RoutingMachinePolicy`, pour ne pas casser les binaires plus anciens (§1.9). On les édite :
- dans la console 311, quand elle tourne sur le maître attesté (aperçu, application, relecture, comme les réglages de routage) ;
- ou par la commande `bridget heavy-work policy`.

Un worker lit la politique par le daemon et ne l'écrit jamais.

| Paramètre | Portée | Valeurs | Défaut |
|---|---|---|---|
| Rôle pour les travaux lourds | par machine | `ouverte` / `protégée` / `exclue` | `ouverte` ; le poste de travail est marqué `protégée` explicitement à l'installation |
| Travaux lourds simultanés | par machine | entier de 1 à 4 | 1 |
| Si seul un poste protégé convient | global, une valeur par origine | `discret` / `attendre` / `refuser` | `discret` si l'utilisateur lance lui-même ; `attendre` pour une mission d'agent |
| Attente maximale | global | 60 à 3 600 s | 600 s |
| Installation d'outils | par machine | `demander` / `outils_du_catalogue` / `jamais` | `demander` |

`compilation_capacity` reste un poids de préférence entre machines ouvertes. Sa borne est alignée sur 0 à 10 dans la console.

### 3.2 Fonctionnement normal (aucun réglage)

1. **Inventaire unifié.** L'inventaire 311 et la sonde `qualify_workload_tools` sont étendus en une seule source.
   - Relevés : propriétés hôtes, glibc, version exacte de Rust et de ses composants, Python, disque libre sur la racine de compilation, outils Apple présents ou non.
   - États du moment, rafraîchis : charge, mémoire, places occupées.
   - Aucun second inventaire n'est créé.
2. **Exigences du travail.** Chaque travail déclare sa cible et ses outils.
   - Binaire Mac de Bridget : `macos-arm64`, `MACOSX_DEPLOYMENT_TARGET=11.0`, toolchain de `rust-toolchain.toml`.
   - Binaire Linux : `linux-x86_64-glibc2.36`, même toolchain.
3. **Choix de la machine.**
   1. Écarter les machines exclues, injoignables, ou sans la cible produisible.
   2. Préférer les machines prêtes, sans outil manquant.
   3. Parmi elles, prendre celle qui a une place libre et la charge la plus basse ; à égalité, le meilleur poids `compilation_capacity`.
   4. Sur une machine `protégée`, ne jamais appliquer le bonus de continuité `Local` à un travail lourd tant qu'une machine ouverte et prête est disponible.
   5. N'utiliser un poste protégé qu'en dernier recours, selon le paramètre « si seul un poste protégé convient ».
4. **Exécution distante vérifiée.**
   - **Source.** Archive `git archive` d'un commit exact, déjà poussé. Sa somme SHA-256 est vérifiée avant l'extraction. L'extraction se fait dans un dossier privé `0700`, avec `--no-same-owner`. Pas de transfert d'agent SSH.
   - **Identité du binaire.** `BRIDGET_BUILD_ID` est injecté, sinon le binaire porterait `unknown`. Sur macOS, `MACOSX_DEPLOYMENT_TARGET` est injecté aussi.
   - **Bail.** Un bail est pris sur la machine. Sur macOS distant, `caffeinate -i` empêche la mise en veille. Le groupe de processus distant est arrêté si la connexion coupe ou si l'utilisateur interrompt. Un nettoyage au démarrage suivant reprend ce qui serait resté.
   - **Compilation.** Elle passe par `scripts/build.py`, dans `bridget-build-<dépôt>-<cible>`, un nom stable par dépôt et par cible. Le cache est donc conservé et entretenu dans le budget de 10 Gio et 7 jours. Seul le dossier source temporaire est retiré.
   - **Contrôles.**
     - Linux : `scripts/check-linux-abi.py` sur l'hôte Linux lui-même, où `readelf` existe, puis SHA-256 avant le rapatriement.
     - Après le rapatriement : SHA-256 de nouveau.
     - macOS : `codesign --verify --strict`, et l'autorité de signature si un certificat est demandé.
     - Chaque cible : démarrage et `bridget --version`, qui doit rendre le `BRIDGET_BUILD_ID` attendu.
5. **Transparence.**
   - La console affiche un tableau machines × cibles, avec une case par couple : `prête`, `à installer` (recette, durée et espace estimés), `action manuelle requise` (geste exact) ou `impossible` (cause exacte, par exemple « glibc 2.39 > 2.36 »).
   - Chaque travail garde où il a tourné, pourquoi, sa durée par phase (envoi, compilation, contrôles, retour) et ses vérifications.
6. **Repérage d'une installation utile.** Il est toujours fait ; le paramètre « Installation d'outils » décide de la suite.

### 3.3 Actions permises par un paramètre

1. **Installer un outil du catalogue.** Un verrou par machine garantit une seule installation à la fois.
   - `demander` : une question unique et précise (machine, outil, version, durée, espace).
     - L'utilisateur qui lance un build reçoit la question dans son terminal.
     - Une mission d'agent la reçoit dans la boîte humaine de Bridget, rattachée au fil parent.
     - Sans réponse, rien n'est installé et le travail suit le paramètre « si seul un poste protégé convient ».
     - Un accord vaut pour cette seule installation.
     - L'outil devient une capacité « prête » après vérification de sa version.
   - `outils_du_catalogue` : installation sans question, limitée au catalogue de la 312.
   - `jamais` : aucune installation. La machine peut toujours exécuter les travaux pour lesquels elle est déjà prête.
2. **Utiliser le poste protégé**, seulement s'il est la seule machine qui produit la cible :
   - `discret` : exécution en mode discret ;
   - `attendre` : attente qu'une machine ouverte devienne disponible, au plus « attente maximale » ;
   - `refuser` : refus avec la cause et l'action.
3. **Dépannage explicite.** `scripts/build.py --local` ou `--host <machine>` passe outre le choix automatique, sans passer outre les rôles `exclue`. Il sert hors ligne ou quand le daemon est arrêté.

## 4. Architecture

1. **Planification par le daemon.** `scripts/build.py` demande au daemon local un plan pour sa cible, par une commande en lecture (`bridget heavy-work plan`). Le plan rend la machine, le bail et les variables à injecter. Le daemon applique la politique, l'inventaire et les baux.
2. **Sans daemon**, `build.py` compile en local en mode discret. Il le dit et propose `--host`.
3. **Exécution distante** par le chemin SSH déjà utilisé pour les livraisons. Le worker distant n'est pas modifié dans le Lot 1a.
4. **Entretien des caches.** `scripts/build.py` doit fonctionner avec Python 3.9, ou Python 3.11 doit être installé par le catalogue. À trancher au plan, sur mesure. Un échec d'entretien doit être visible, jamais avalé.
5. **Alignement.** `scripts/install-k1.sh` installe aujourd'hui `rustup` en version `stable` sans question (`install-k1.sh:199-225`). Il est aligné sur la version épinglée et sur la politique d'installation.

## 5. Lots

1. **Lot 1a, le plus petit lot utile : compilation release déportée.**
   - Rôles par machine, un seul travail simultané, `discret` ou `refuser`, options `--local` et `--host`.
   - `git archive` avec `BRIDGET_BUILD_ID`, bail distant, `caffeinate`, cache distant stable, contrôles par cible.
   - Aucune installation automatique.
2. **Lot 1b : inventaire unifié, routage et console.**
   - Sonde étendue.
   - Neutralisation du bonus `Local` sur un poste protégé.
   - Borne réelle par bail pour les missions de compilation.
   - Tableau machines × cibles.
3. **Lot 1c : installation et attente.**
   - Catalogue d'outils, verrou, politique de consentement et boîte humaine.
   - Mode `attendre`.
   - Tests longs à distance, avec un dépôt Git minimal (bundle) pour les tests qui lisent `.git`.
4. **Lot 2 : build de T3 Code**, macOS uniquement, même mécanisme.
5. **Lot 3 : autres calculs longs délégués.**

## 6. Hors périmètre

- Compiler pour macOS hors d'un Mac.
- Installer un logiciel hors du catalogue, ou un prérequis qui exige `sudo` ou une fenêtre.
- Arrêter ou ralentir les missions d'autres agents pour libérer une machine.

## 7. Critères d'acceptation

1. **Charge du poste.** Pendant une compilation release déportée depuis le poste protégé, aucun processus `cargo`, `rustc` ou éditeur de liens ne tourne sur ce poste. Le temps processeur cumulé du groupe local (`build.py`, `git archive`, `ssh`, `scp`) reste sous 5 s. Les processus sont relevés pendant toute la durée.
2. **Surcoût.** Sur une même machine distante et un même commit, avec un `target` froid et un `CARGO_HOME` déjà rempli (`--offline`), le surcoût d'orchestration reste sous 20 s sur le réseau local et sous 60 s à distance. Le surcoût comprend l'archive, les transferts et les contrôles. L'écart avec une compilation lancée directement sur cette machine reste sous 15 %.
3. **Contrôles.** Le binaire rapatrié passe SHA-256, le contrôle ABI (Linux) ou `codesign` (macOS). Il démarre sur sa cible et rend le `BRIDGET_BUILD_ID` du commit.
4. **Poste protégé seul possible.**
   - Test automatisé, où l'injoignabilité est simulée : les trois réponses (`discret`, `attendre` borné, `refuser` avec cause et action) sont vérifiées.
   - Un essai réel, `macbook-m1` éteint, confirme `discret`.
5. **Consentement.**
   - Avec `demander`, une machine sans la version exacte de Rust produit une seule question.
   - Une mission sans réponse n'installe rien.
   - Après accord, la vérification de version marque la machine « prête ».
   - Deux demandes simultanées ne lancent qu'une installation.
6. **Rôles et concurrence.**
   - Une machine `exclue` n'exécute aucun travail lourd.
   - `jamais` empêche toute installation, mais pas les travaux pour lesquels la machine est prête.
   - Deux compilations simultanées vers une machine à 1 place s'exécutent l'une après l'autre, sans corrompre le cache.
7. **Tableau.** Le tableau machines × cibles reflète l'inventaire avec les quatre états et la cause exacte. Il est qualifié sur le réseau du moment, avec une preuve datée conservée hors spécification.
8. **Interruption.** Une coupure SSH pendant la compilation ne laisse aucun `cargo` ni `rustc` sur la machine distante au-delà de 60 s. Elle ne laisse aucun bail occupé au-delà de son expiration.

## 8. Risques et parades

- **Binaire différent selon la machine.** Commit exact, toolchain épinglée, cible minimale macOS fixée, contrôles du §3.2.4.
  - Une égalité bit à bit exigerait aussi `--remap-path-prefix`. Elle n'est pas un critère.
- **Cache distant saturé.** Nom de cache stable et entretien visible (§4.4).
- **Installation partielle.** Recette idempotente, verrou, capacité marquée seulement après vérification.
- **Secrets dans l'archive.** Seulement `git archive` d'un commit poussé, sans fichiers ignorés ni non suivis.
- **Machine distante occupée par des missions.** Baux, charge fraîche et places réservées.
- **Mise en veille du Mac distant.** `caffeinate -i` pendant le bail ; une machine en veille est vue injoignable, donc écartée.

## 9. Hypothèses à éprouver

1. **Compilation croisée** `linux-x86_64-glibc2.36` depuis `cartae` ou `ksstore`, par exemple avec `cargo-zigbuild`. Qualification : contrôle ABI et démarrage réel sur `sysle2`, avant toute ouverture au choix automatique.
2. **Coût réel du transfert** par lien : archive d'environ 39 Mo, binaire de 38 à 47 Mo.
3. **Signal d'activité de l'utilisateur** sur macOS sans nouvelle permission système. Il sert à `attendre`.
4. **Python 3.11 sur `macbook-m1`**, ou entretien compatible Python 3.9.
