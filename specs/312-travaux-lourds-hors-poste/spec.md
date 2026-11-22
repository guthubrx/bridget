# Spécification 312 : travaux lourds hors du poste de travail

- **Statut** : spécification v4.2, prête pour le plan du Lot 1a (revue v4.1 : 8/10, aucun bloquant)
- **Demande** : François, 22 novembre 2026. Son Mac ne doit plus être chargé par les compilations. Bridget déporte les travaux lourds en tenant compte des capacités réelles de chaque machine :
  - celles qu'aucune installation ne remplace (un binaire Mac exige un Mac) ;
  - celles qu'une installation peut apporter.

  La spécification sépare nettement trois choses : les paramètres, le fonctionnement normal, et les actions seulement permises par un paramètre.
- **Revues Neon** (gemini-4-neon-high) :
  - v1 : 4,5/10 (tâche `3b000932`) ;
  - v2 : 6/10 (tâche `077d5b8b`) ;
  - v3 : 7/10 (tâche `b3c3f0d5`) ;
  - v4 : 7,5/10, aucun bloquant (tâche `07563ff2`).

  Ses corrections sont reprises ici. Le §10 trace chaque défaut.

## 1. Constats vérifiés (22 novembre 2026)

1. **Livraisons 328 à 330 compilées à la main.**
   - Mac : sur le poste de François, puis sur `macbook-m1` (release en 131 s).
   - Linux : sur `sysle2` (Debian 12, glibc 2.36), en 3 min 42 s à 3 min 49 s.
   - `cartae` et `ksstore` ont glibc 2.39 : leur binaire natif ne démarre pas sur `sysle2`.
2. **`macbook-m1` a reçu Rust 1.92.0 à la main.** Il n'a que Python 3.9.
   - L'entretien des caches de `scripts/build.py` exige Python 3.11 : garde de version, `shutil.rmtree(dir_fd=…)` (`build.py:297`) et `hashlib.file_digest` (`build.py:316`).
   - Sur Python 3.9, l'erreur est écrite sur la sortie d'erreur, mais la commande rend 0 (`build.py:1084`, `1165`) : les caches ne sont jamais purgés.
   - Python 3.9 est aussi celui de l'outil de mise à jour du Mac.
3. **`scripts/build.py` compile toujours en local.** Sans dossier `.git`, `owner_id` (`build.py:57-62`) rend le chemin du dossier. Le marqueur `.bridget-build-owner.json` est figé au premier build (`build.py:178-182`). Une source extraite dans un nouveau dossier temporaire sort donc du budget de cache (`build.py:185-198`).
4. **`bridget --version` n'affiche que la version du paquet** (`cli.rs:519-521`). `BUILD_ID` n'est visible que par un daemon vivant. La console 311 lit la première ligne de `--version` (`console_admin.rs:1097`).
5. **Routage des missions de compilation.**
   - `compilation_capacity` (`registry.rs:298-301`, 0 à 10) n'est qu'un poids de score (`native_delegation.rs:2250-2256`). La console accepte 1 à 16 (`console_admin.rs:691`).
   - Les termes processeur, mémoire et charge pèsent jusqu'à 60 points, et la continuité `Local` 20 (`native_delegation.rs:2223-2407`).
6. **Deux inventaires incompatibles.**
   - L'inventaire 311 est un script bash lancé par SSH depuis la console (`console_admin.rs:1089-1174`).
   - `qualify_workload_tools` tourne dans le worker et ne rend qu'un état par outil (`remote_worker_read.rs:7-92`).
7. **Les politiques du registre refusent les champs inconnus** (`registry.rs:294`). Le protocole worker ne distribue aucune politique.
8. **Les sous-agents délégués** n'ont droit qu'aux sous-commandes CLI listées (`cli.rs:145-183`).
9. **Questions de mission.**
   - Les questions de quota passent par `user_input`, créé par le daemon pour le propriétaire racine (`native_quota_questions.rs:91-101`).
   - `human_inbox` a une liste fermée de genres (`human_inbox.rs:58`).
10. **Les canaux SSH de fédération n'ont pas de terminal** (`federate-ssh.sh:81`). Une coupure ne tue pas un `cargo` distant.

## 2. Vocabulaire

- **Poste de travail** : machine où François travaille. Elle est désignée explicitement, jamais déduite.
- **Travail lourd** : compilation release ou tests longs de Bridget, build de T3 Code, calcul long délégué.
- **Cible** : ce que le travail produit.
  - `macos-arm64`, avec `MACOSX_DEPLOYMENT_TARGET=11.0`.
  - `linux-x86_64-glibc2.36` : démarre partout où glibc vaut au moins 2.36.
- **Propriété hôte** : système, processeur, glibc native, matériel Apple. Rien ne s'installe.
- **Cible produisible** : couple (cible, mode) attesté pour une machine.
  - **Natif** : les propriétés hôtes suffisent.
  - **Croisé qualifié** : la chaîne croisée du catalogue a passé le contrôle ABI et un démarrage réel sur la cible.
- **Outil installable** : outil versionné, installable sans `sudo` ni fenêtre, par une recette idempotente du catalogue 312. Exemples : toolchain de `rust-toolchain.toml` avec `rustfmt` et `clippy` ; plus tard, `cargo-zigbuild` une fois qualifié.
- **Action manuelle requise** : prérequis qui exige `sudo` ou une fenêtre, comme les outils Apple. Bridget le signale et ne l'installe jamais.
- **État du moment** :
  - joignable ou injoignable (une machine en veille est injoignable) ;
  - charge et mémoire ;
  - disque libre sur la racine de compilation ;
  - baux occupés ;
  - trousseau macOS déverrouillé, si une signature par certificat est demandée.
- **Mode discret** : exécution sur le poste de travail en priorité basse (`taskpolicy -b` sur macOS ; `nice -n 15` et `ionice -c 3` sur Linux). Le parallélisme est limité à `CARGO_BUILD_JOBS = max(1, min(2, cœurs / 4))`.
- **Bail** : réservation d'une place de travail lourd sur une machine, détenue par le daemon maître. Sa durée de vie est de 60 s, renouvelée toutes les 20 s par le lanceur et relâchée à la fin.

## 3. Paramètres, fonctionnement normal, actions permises

### 3.1 Paramètres (choix de François, rares et durables)

**Stockage.** Ils sont dans `heavy-work-policy.json`, un fichier versionné du namespace du daemon maître.
- Il n'est écrit que sur le maître attesté : par la console 311 (aperçu, application, relecture) ou par `bridget heavy-work policy`.
- Les workers ne le reçoivent pas et ne le lisent pas. Les clients (`scripts/build.py`, console) interrogent le daemon maître par la socket locale ou fédérée.

**Défaut sûr.**
- Si `heavy-work-policy.json` est absent (migration ou retour arrière), ou si le daemon maître est injoignable, le déport est inactif. Aucun travail ne part hors du poste. Toute origine, utilisateur comme mission, compile en local en mode discret.
- Une fois le fichier présent, toute machine non déclarée est `protégée`, et les cas A et B du tableau s'appliquent.
- L'initialisation explicite déclare le poste de travail `protégée` et les machines de compilation `ouverte`, avec leurs cibles et leur profil SSH.

**Écriture.** `bridget heavy-work policy` s'exécute sur l'hôte maître, en local. Il passe par la même résolution d'autorité que la console 311 (`console_ui.rs:259-338`) : aperçu, application atomique, sauvegarde, relecture. Il n'existe pas d'écriture à distance par la socket.

| Paramètre | Portée | Valeurs | Défaut |
|---|---|---|---|
| Rôle | par machine | `ouverte` / `protégée` / `exclue` | `protégée` si non déclarée |
| Cibles déclarées (Lot 1a) | par machine | liste de cibles | aucune |
| Profil SSH | par machine | hôte, utilisateur, port, clé, `known_hosts` | aucun |
| Baux simultanés | par machine | 1 à 4 | 1 |
| A. Machine capable occupée ou injoignable | global, par origine | `attendre` puis repli, ou `repli` immédiat | `attendre` |
| B. Aucune machine ouverte ne produit la cible | global, par origine | `discret` / `refuser` | `discret` pour l'utilisateur, `refuser` pour une mission |
| Repli après attente | global, par origine | `discret` / `refuser` | `discret` pour l'utilisateur, `refuser` pour une mission |
| Attente maximale | global | 60 à 3 600 s | 600 s |
| Installation d'outils | par machine | `demander` / `outils_du_catalogue` / `jamais` | `demander` |

- L'**origine** est résolue sans option. Elle vaut « mission d'agent » si l'un de ces éléments est présent :
  - une identité agent (`resolve_current_identity`) ;
  - une session T3 (`resolve_current_mcp_identity`, `mcp_identity.rs:288-315`, et `t3code_mcp::resolve_current`, `t3code_mcp.rs:40-60`) ;
  - un marqueur de sous-agent délégué (`delegated-pids`).

  L'authentification passe par le rattachement auxiliaire quand il est disponible. Seule l'absence de tout contexte agent, T3 ou délégué vaut « utilisateur ».
- Le **profil SSH** déclare l'hôte, l'utilisateur et le port. Les chemins de la clé et de `known_hosts` sont **relatifs à `$HOME`**, sans `..` ni lien symbolique.
  - Ils sont résolus et vérifiés (`0600`) sur la **machine appelante**, car la politique vit sur le maître Linux alors que l'appelant peut être un Mac.
  - SSH est lancé comme dans `scripts/federate-ssh.sh` : `-F /dev/null`, `BatchMode=yes`. Aucun alias de `~/.ssh/config` n'est utilisé.
- `compilation_capacity` reste un poids entre machines ouvertes. La console est alignée sur 0 à 10.

### 3.2 Fonctionnement normal (aucun réglage)

1. **Capacités.**
   - Lot 1a : les cibles d'une machine sont celles déclarées dans la politique. Une pré-vérification SSH de quelques secondes contrôle avant chaque travail le système, le processeur, glibc et la version exacte de Rust.
   - Lot 1b : elles viennent d'une sonde unique, au format JSON versionné. Le worker l'exécute sur les machines fédérées, en étendant la sonde de candidat existante. Le maître ou la console la lance telle quelle par SSH sur les machines sans worker. L'inventaire 311 et `qualify_workload_tools` lisent cette sonde : aucun second inventaire.
2. **Exigences du travail.** Chaque travail déclare sa cible, sa toolchain et le dépôt.
3. **Choix de la machine.**
   1. Écarter les machines `exclue`, injoignables ou sans la cible.
   2. **Filtre de classe avant tout score** : tant qu'une machine `ouverte` admissible existe, toute machine `protégée` est écartée. Ses termes processeur, mémoire, charge et continuité ne comptent donc pas.
   3. Parmi les machines restantes, préférer, dans cet ordre :
      - une place de bail libre ;
      - la charge la plus basse, seulement quand une mesure fraîche existe (à partir du Lot 1b) ;
      - le poids `compilation_capacity` ;
      - l'ordre alphabétique de `machine_id`.
   4. Si une machine ouverte capable existe mais n'est pas disponible : cas A du tableau. Si aucune n'existe : cas B.
   5. **Échec de la pré-vérification (Lot 1a).**
      - Le lanceur relâche le bail et rappelle `acquire`, en excluant cette machine pour l'essai en cours et en donnant le motif : injoignable, ou prérequis absent.
      - Une autre machine ouverte est alors essayée.
      - S'il n'en reste aucune :
        - cas A dès qu'au moins une machine ouverte qui déclare la cible est seulement injoignable ou occupée ;
        - cas B si toutes ont échoué sur un prérequis absent.

        La cause exacte est rendue dans les deux cas.
   6. **Appelant choisi.** Si la machine ouverte retenue est la machine appelante, la compilation s'exécute en local sous bail, sans SSH et sans mode discret.
4. **Exécution distante vérifiée.**
   - **Source.** `git archive` d'un commit exact et propre, déjà ancêtre de la branche distante suivie (ou de `origin/main`), sans récupération réseau implicite.
     - Fichier suivi modifié ou commit non poussé : refus avec cause et action (pousser, ou relancer avec `--local`).
     - Fichiers non suivis : non inclus, signalés par un avertissement.
     - L'archive est vérifiée par SHA-256 avant extraction, dans un dossier privé `0700`, avec `--no-same-owner`, sans transfert d'agent SSH.
   - **Périmètre Lot 1a.** Seul `cargo build --release` est déporté. Les autres commandes (`build` en debug, `check`, `clippy`, `test`) restent locales jusqu'au Lot 1c.
   - **Identité.** Injection de `BRIDGET_BUILD_ID=<sha12>` (12 caractères hexadécimaux, comme `build_identity.rs` et `deploy-remote.sh`) et, sur macOS, de `MACOSX_DEPLOYMENT_TARGET`.
   - **Bail et coupure.**
     - Le lanceur prend un bail (`acquire`), le renouvelle toutes les 20 s (`renew`) et le relâche à la fin (`release`). Un bail non renouvelé expire au bout de 60 s.
     - Sur la machine distante, un verrou `fcntl.flock` en Python (compatible macOS et Linux) porte sur `<cache>/.heavy-work.lock`. Ce fichier est hors de `debug/` et de `release/` et n'est jamais supprimé.
     - Ce verrou sérialise les compilations d'un même couple (dépôt, cible), y compris par `--host` sans daemon. Avec plusieurs baux, seules des compilations de dépôts ou de cibles différents tournent en parallèle.
     - **Gardien distant.**
       - Il est déposé comme fichier, puis lancé par son chemin, jamais par `bash -s` (l'entrée standard doit rester ouverte pour le battement).
       - Il lance `cargo` dans son propre groupe de processus, avec l'entrée standard sur `/dev/null`.
       - Il attend un battement toutes les 10 s sur l'entrée standard SSH. À la fin de l'entrée ou après 30 s sans battement, il envoie `SIGTERM` au groupe, puis `SIGKILL` 10 s plus tard.
     - Sur macOS distant, `caffeinate -i` empêche la mise en veille par inactivité. La fermeture du capot reste possible : la machine devient alors injoignable et le travail échoue avec sa cause.
   - **Cache.**
     - `scripts/build.py` gagne une résolution dédiée de la racine de cache distante.
       - Si `~/.config/bridget/build.json` est présent : sa racine externe. Si le fichier ou le volume est invalide, refus sans repli, comme `external_staging_parent` (`build.py:84-109`).
       - Si le fichier est absent : `~/.cache`.
       - `target-dir`, utilisé par `Makefile`, est inchangé.
     - Le dossier distant s'appelle `bridget-build-<dépôt>-<cible>`. C'est un enfant direct de cette racine, donc accepté par `configure_build_target` (`build.py:136-143`).
     - **Identifiant de propriétaire canonique**, calculé de la même façon en local et à distance : le SHA du commit racine du dépôt.
       - En local, `build.py` le calcule depuis `.git`.
       - À distance, sans `.git`, le lanceur le transmet dans `BRIDGET_BUILD_OWNER`.
       - `owned_staging` (`build.py:185-198`) reconnaît cet identifiant **et** l'ancien chemin `.git`, pour ne pas rendre orphelins les caches locaux existants.
       - La clé locale de `configure_build_target` (`build.py:132`) est conservée.
     - Seul le dossier source temporaire est retiré.
   - **Résultat.**
     - Cible native du poste appelant : le binaire est posé à `$CARGO_TARGET_DIR/release/bridget`, puis le staging local est enregistré (`register_staging`). `make install` et `build.py installed` fonctionnent donc comme aujourd'hui.
     - Autre cible, demandée par l'option `--build-target <cible>` de `build.py` (distincte de `--target` de Cargo) : le binaire est posé à `$CARGO_TARGET_DIR/<cible>/release/bridget`, sans écraser le binaire local.
   - **Contrôles.**
     - Nouvelle sous-commande `bridget build-info --json`, aux arguments stricts. Elle rend `{"version", "build_id", "os", "arch"}`, sans daemon et sans initialiser de namespace : exemptée dans `command_initializes_namespace` et `command_requires_principal` (`cli.rs:127-168`).
     - Le format de `--version` est inchangé, car la console le lit.
     - Cible Linux : ABI (`scripts/check-linux-abi.py`), démarrage et `build-info` sur l'hôte Linux, puis SHA-256 avant le rapatriement.
     - Après le rapatriement : SHA-256 de nouveau.
     - Cible macOS : `codesign --verify --strict` (et autorité si un certificat est demandé), démarrage et `build-info` **sur l'hôte macOS**, puis SHA-256 avant et après le rapatriement.
5. **Transparence.**
   - La console affiche un tableau machines × cibles. Chaque case vaut `prête`, `à installer` (recette, durée, espace), `action manuelle requise` (geste exact) ou `impossible` (cause exacte, par exemple « glibc 2.39 > 2.36 »).
   - Chaque travail garde sa machine, la raison du choix, sa durée par phase (envoi, compilation, contrôles, retour) et ses contrôles.
6. **Repérage d'une installation utile.** Il est toujours fait ; le paramètre décide de la suite.

### 3.3 Actions permises par un paramètre

1. **Installer un outil du catalogue.** Un seul à la fois par machine, sous verrou.
   - `demander`, quand l'utilisateur lance lui-même : question unique dans son terminal (machine, outil, version, durée, espace).
   - `demander`, quand une mission lance :
     - Au moment d'`acquire`, le daemon déduplique par (machine, outil, version). Il crée une seule question `user_input`, rattachée au propriétaire racine connecté de la première mission demandeuse, comme pour les questions de quota.
     - Propriétaire racine absent ou déconnecté : pas de question, repli immédiat selon le tableau.
     - Le daemon borne la question à l'attente maximale. À l'expiration ou à l'annulation, il la clôt lui-même (`expired`, `cancelled`), sans attendre le délai de 7 jours de `user_input`.
     - `build.py` ne lit jamais `user_input`, interdit aux sous-agents délégués (`daemon/user_input.rs:135-144`). Il relit l'état par `heavy-work acquire` et `heavy-work status`.
     - Sur accord, le daemon marque la question `applied` et délivre un permis d'installation à usage unique. Le détenteur du permis exécute la recette sous le verrou d'installation de la machine. Le plan du Lot 1c fixe qui le détient : l'appelant, ou le maître.
   - Un accord vaut pour cette seule installation. L'outil devient « prêt » après vérification de sa version.
   - `outils_du_catalogue` : installation sans question, limitée au catalogue.
   - `jamais` : aucune installation. La machine exécute toujours ce pour quoi elle est prête.
2. **Utiliser le poste protégé** : selon les cas A et B et le repli. `discret` exécute en mode discret ; `refuser` rend la cause et l'action.
   - **Garde de la compilation locale.** Le repli `discret`, le défaut sûr (sans politique ou sans daemon) et `--local` ne s'appliquent que si la machine appelante produit la cible demandée : même système et même processeur, glibc inférieure ou égale à celle de la cible. Sinon, refus immédiat avec cause et action (`--host`).
   - Si la machine appelante est `exclue` (daemon joignable), tout repli local et `--local` sont refusés avec la cause.
3. **Dépannage.**
   - `scripts/build.py --local` compile en local, en mode discret sur une machine `protégée`.
   - `--host <machine>` passe outre le choix du routage. Quand le daemon maître est joignable, il respecte toujours le rôle `exclue`.
   - Sans daemon joignable, `--host` exige des paramètres SSH explicites (hôte, utilisateur, port, clé, `known_hosts`). Il ne peut pas vérifier `exclue` et le dit. Il reste sérialisé par le verrou de la machine cible.

## 4. Architecture

1. **Commandes du daemon maître.**
   - Lecture : `bridget heavy-work status` et `bridget heavy-work plan --dry-run`.
   - Bail : `bridget heavy-work acquire | renew | release`.
   - Politique : `bridget heavy-work policy`, réservé au maître.
2. **Exception bornée** dans la garde CLI des sous-agents délégués. Elle est vérifiée sur le couple (commande, sous-commande), et non sur le seul premier mot (`cli.rs:145-183`). Seuls `heavy-work status | plan | acquire | renew | release` et `build-info --json` sont ouverts. `heavy-work policy` reste fermé.
3. **Sans daemon joignable**, toute origine se comporte comme sans politique : compilation locale en mode discret. `build.py` le dit et propose `--host`.
4. **Exécution distante** par SSH avec le profil de la machine (§3.1), plus le gardien. Le worker n'est pas modifié dans le Lot 1a.
5. **Entretien des caches compatible Python 3.9**, la version des Mac et de l'outil de mise à jour.
   - `shutil.rmtree(…, dir_fd=…)` est remplacé par un parcours qui ouvre chaque dossier avec `O_NOFOLLOW | O_DIRECTORY`, puis supprime avec `os.unlink` et `os.rmdir` relatifs à ce descripteur. Aucun lien symbolique n'est suivi.
   - `hashlib.file_digest` est remplacé par un hachage par blocs.
   - Un échec d'entretien est affiché et repris dans le bilan, sans transformer en échec une compilation réussie.

## 5. Lots

1. **Lot 1a, le plus petit lot utile : compilation release déportée.**
   - Politique avec rôles, cibles déclarées, profils SSH, baux simultanés, cas A et B, repli et attente.
   - `heavy-work policy` (local au maître), `heavy-work status | plan | acquire | renew | release`, et l'exception de garde CLI.
   - Origine résolue (agent, T3, délégué).
   - Seul `cargo build --release` est déporté.
   - Filtre de classe.
   - `build-info --json`.
   - `BRIDGET_BUILD_OWNER` et entretien Python 3.9 dans `build.py`.
   - Archive, gardien distant, `flock`, `caffeinate`, contrôles, `--local` et `--host`.
   - Pas d'installation automatique.
2. **Lot 1b : sonde unique, routage des missions et console.**
   - Sonde JSON et mesure de charge fraîche.
   - Missions de compilation soumises aux baux et au filtre de classe.
   - Tableau machines × cibles ; la console garde sa sonde SSH minimale pour une machine sans `bridget`.
   - Borne console 0 à 10.
3. **Lot 1c : installation et tests distants.** Catalogue, verrou, question terminal ou `user_input`, tests longs avec un bundle Git minimal, alignement de `scripts/install-k1.sh` sur la toolchain épinglée et la politique d'installation.
4. **Lot 2 : build de T3 Code**, macOS uniquement.
5. **Lot 3 : autres calculs longs délégués.**

## 6. Hors périmètre

- Compiler pour macOS hors d'un Mac.
- Installer hors du catalogue, ou un prérequis qui exige `sudo` ou une fenêtre.
- Arrêter ou ralentir les missions d'autres agents pour libérer une machine.

## 7. Critères d'acceptation

1. **Poste préservé.**
   - Mesure : compilation `cargo build --release -p bridget-daemon` lancée par `build.py` depuis le poste protégé.
   - Résultat attendu : aucun processus `cargo`, `rustc` ou éditeur de liens n'apparaît sur ce poste. Le temps processeur cumulé des processus locaux du lanceur reste sous 5 s.
   - Relevé : `ps -o time` du groupe de processus du lanceur, une fois par seconde, pendant toute la durée.
2. **Surcoût.**
   - Mesure : même machine distante, même commit, `target` froid, `CARGO_HOME` rempli (`--offline`).
   - Résultat attendu :
     - surcoût d'orchestration (archive, transferts, contrôles) sous 20 s en réseau local et sous 60 s à distance ;
     - durée de la phase `cargo` dans les 15 % d'une compilation lancée directement sur la machine.
3. **Contrôles.** SHA-256, ABI (Linux, sur l'hôte Linux) ou `codesign` (macOS). Démarrage réussi, et `build-info` rend le `BRIDGET_BUILD_ID` du commit.
4. **Cas A et B.**
   - Tests automatisés, avec l'injoignabilité et l'absence de cible simulées : `attendre` borné puis repli, `discret`, `refuser` avec cause et action, pour chaque origine.
   - Un essai réel, `macbook-m1` éteint.
5. **Consentement (Lot 1c).**
   - Une machine sans la toolchain exacte produit une seule question, par le bon canal selon l'origine.
   - Sans réponse dans l'attente maximale, rien n'est installé.
   - Deux demandes simultanées ne lancent qu'une installation.
6. **Rôles et concurrence.**
   - `exclue` : aucun travail lourd.
   - `jamais` : aucune installation, mais les travaux pour lesquels la machine est prête restent possibles.
   - Deux compilations vers une machine à 1 bail s'exécutent l'une après l'autre, y compris avec `--host` sans daemon, sans corrompre le cache.
7. **Tableau (Lot 1b).** Il montre les quatre états avec la cause exacte, qualifiés sur le réseau du moment. La preuve datée est conservée hors spécification.
8. **Coupure.** Coupure SSH pendant la compilation :
   - aucun `cargo` ni `rustc` distant au-delà de 60 s après la dernière réception de battement par le gardien ;
   - le bail expire au plus 60 s après son dernier renouvellement.

   Avec les réglages SSH actuels (`ServerAliveInterval=15`, `ServerAliveCountMax=3`), le lanceur peut ne voir une coupure silencieuse qu'après environ 45 s. Le test mesure donc depuis le dernier battement et le dernier renouvellement, pas depuis la coupure.
9. **Défaut sûr.** Sans `heavy-work-policy.json`, ou sans daemon joignable, aucun travail ne part hors du poste. `build.py` compile en mode discret, pour l'utilisateur comme pour une mission.
10. **Origine.** Un appel depuis un agent, un fil T3 ou un sous-agent délégué est classé « mission ». Un appel depuis un terminal sans aucun de ces contextes est classé « utilisateur ».
11. **Garde CLI.** Un sous-agent délégué peut appeler `build-info` et `heavy-work status | plan | acquire | renew | release`, mais pas `heavy-work policy`.
12. **Cible étrangère.**
    - Une compilation Linux lancée depuis le Mac n'écrase pas `$CARGO_TARGET_DIR/release/bridget`.
    - `--build-target linux-x86_64-glibc2.36 --local` depuis le Mac, ou la même cible sans daemon et sans machine Linux ouverte : refus immédiat, aucune compilation locale.

## 8. Risques et parades

- **Binaire différent.** Commit exact, toolchain épinglée, cible macOS minimale fixée, contrôles du §3.2.4. Pas d'exigence d'égalité bit à bit (elle demanderait `--remap-path-prefix`).
- **Cache distant saturé.** `BRIDGET_BUILD_OWNER`, nom stable, entretien Python 3.9 visible.
- **Installation partielle.** Recette idempotente, verrou, capacité marquée après vérification.
- **Secrets dans l'archive.** Seulement `git archive` d'un commit poussé.
- **Machine distante occupée.** Baux et charge fraîche.
- **Bail orphelin.** Expiration à 60 s sans renouvellement.

## 9. Hypothèses à éprouver

1. **Compilation croisée** `linux-x86_64-glibc2.36` depuis `cartae` ou `ksstore` (par exemple `cargo-zigbuild`). Qualification par ABI et démarrage sur `sysle2` avant toute ouverture automatique.
2. **Coût réel du transfert** par lien : archive d'environ 39 Mo, binaire de 38 à 47 Mo.

## 10. Traçabilité de la revue v2

| Défaut v2 | Traitement |
|---|---|
| B1 `--version` sans build-id ; démarrage Linux depuis macOS | `build-info --json`, contrôle sur l'hôte Linux (§3.2.4) |
| B2 cache, `owner_id`, racine externe | `BRIDGET_BUILD_OWNER`, racine par `target-dir` (§3.2.4) |
| B3 distribution et commande en lecture qui écrit | politique sur le maître seul, commandes séparées, exception de garde (§3.1, §4) |
| B4 bail et coupure | durée de vie, renouvellement, `flock`, gardien (§2, §3.2.4, §7.8) |
| B5 canal de question | `user_input` pour le propriétaire racine, délai borné (§3.3.1) |
| I1 tableau et replis | cas A et B, repli explicite, défaut `protégée`, origine attestée (§3.1) |
| I2 inventaire | sonde unique JSON, Lot 1b (§3.2.1) |
| I3 dernier recours | filtre de classe avant score (§3.2.3) |
| I4 arbre modifié, emplacement du binaire | refus et action, `$CARGO_TARGET_DIR/release/bridget` (§3.2.4) |
| Mineurs 1 à 5 | §7.2 séparé, §7.1 précisé, `install-k1.sh` au Lot 1c, borne 0 à 10, limite de `caffeinate` |

| Défaut v3 | Traitement |
|---|---|
| B1 défaut sûr contre `refuser` | sans politique ni daemon : déport inactif, mode discret pour toute origine (§3.1, §4.3, §7.9) |
| B2 origine, garde CLI, `build-info` | origine agent, T3 ou délégué ; garde sur (commande, sous-commande) ; `build-info` exempté (§3.1, §3.2.4, §4.2, §7.10, §7.11) |
| B3 `target-dir`, cible étrangère, Python 3.9 | résolution dédiée de la racine distante, chemin par cible, remplacements Python 3.9 (§3.2.4, §4.5, §7.12) |
| B4 `user_input` | déduplication, propriétaire déconnecté, clôture par le daemon, lecture par `heavy-work`, permis (§3.3.1) |
| I1 `flock`, verrou, baux | `fcntl.flock`, `<cache>/.heavy-work.lock`, série par (dépôt, cible) (§3.2.4) |
| I2 gardien et SSH | lancement par fichier, `/dev/null`, battement 10 s, test depuis le dernier battement (§3.2.4, §7.8) |
| I3 `policy`, charge, SSH | `policy` au Lot 1a, tri sans charge avant 1b, profil SSH (§3.1, §3.2.3, §5) |
| I4 build-id, périmètre Cargo, poussé, staging | `sha12`, `build --release` seul, ancêtre de la branche suivie, `register_staging` (§3.2.4) |
| I5 propriétaire mixte, mission sans daemon | `BRIDGET_BUILD_OWNER` partout, sans daemon = mode discret (§3.2.4, §4.3) |
| M1, M2 | constat corrigé (§1.2), `policy` local au maître, sonde SSH minimale gardée (§3.1, §5) |

| Défaut v4 | Traitement |
|---|---|
| I1 profil SSH multi-plateforme, `--host` sans daemon | chemins relatifs à `$HOME` résolus sur l'appelant ; `--host` sans daemon avec paramètres explicites (§3.1, §3.3.3) |
| I2 propriétaire canonique et migration | SHA du commit racine, double reconnaissance de l'ancien chemin `.git` (§3.2.4) |
| I3 échec de la pré-vérification | relâche, exclusion pour l'essai, machine suivante, puis cas A ou B (§3.2.3) |
| M1 `build.json` invalide | refus sans repli (§3.2.4) |
| M2 appelant choisi, `--local`, cible étrangère | local sous bail, `--local` discret sur poste protégé, `--build-target` (§3.2.3, §3.2.4, §3.3.3) |
| M3 citation T3, « comme aujourd'hui » | corrigés (§3.1) |

| Réserve v4.1 | Traitement |
|---|---|
| I1 repli local sur une cible impossible ou une machine `exclue` | garde de la compilation locale (§3.3.2, §7.12) |
| M1 contrôles macOS à distance | sur l'hôte macOS, SHA-256 avant et après (§3.2.4) |
| M2 motifs mixtes de pré-vérification | cas A s'il reste une machine seulement injoignable, sinon cas B (§3.2.3) |
