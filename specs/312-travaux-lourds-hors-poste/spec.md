# Spécification 312 : travaux lourds hors du poste de travail

- **Statut** : spécification v3 ; plan à écrire après revue
- **Demande** : François, 22 novembre 2026. Son Mac ne doit plus être chargé par les compilations. Bridget déporte les travaux lourds en tenant compte des capacités réelles de chaque machine :
  - celles qu'aucune installation ne remplace (un binaire Mac exige un Mac) ;
  - celles qu'une installation peut apporter.

  La spécification sépare nettement trois choses : les paramètres, le fonctionnement normal, et les actions seulement permises par un paramètre.
- **Revues Neon** (gemini-4-neon-high) :
  - v1 : 4,5/10 (tâche `3b000932`) ;
  - v2 : 6/10 (tâche `077d5b8b`).

  Ses corrections v2, de 1 à 7, sont reprises ici. Le §10 trace chaque défaut.

## 1. Constats vérifiés (22 novembre 2026)

1. **Livraisons 328 à 330 compilées à la main.**
   - Mac : sur le poste de François, puis sur `macbook-m1` (release en 131 s).
   - Linux : sur `sysle2` (Debian 12, glibc 2.36), en 3 min 42 s à 3 min 49 s.
   - `cartae` et `ksstore` ont glibc 2.39 : leur binaire natif ne démarre pas sur `sysle2`.
2. **`macbook-m1` a reçu Rust 1.92.0 à la main.** Il n'a que Python 3.9. L'entretien des caches de `scripts/build.py` exige Python 3.11 et y échoue sans le dire : ses caches ne sont jamais purgés. Python 3.9 est aussi celui de l'outil de mise à jour du Mac.
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

**Défaut sûr.** Si le fichier est absent, à la migration ou après un retour arrière, toute machine non déclarée est `protégée`. Le comportement reste alors celui d'aujourd'hui : compilation locale, ici en mode discret. L'initialisation explicite déclare le poste de travail `protégée` et les machines de compilation `ouverte`.

| Paramètre | Portée | Valeurs | Défaut |
|---|---|---|---|
| Rôle | par machine | `ouverte` / `protégée` / `exclue` | `protégée` si non déclarée |
| Cibles déclarées (Lot 1a) | par machine | liste de cibles | aucune |
| Baux simultanés | par machine | 1 à 4 | 1 |
| A. Machine capable occupée ou injoignable | global, par origine | `attendre` puis repli, ou `repli` immédiat | `attendre` |
| B. Aucune machine ouverte ne produit la cible | global, par origine | `discret` / `refuser` | `discret` pour l'utilisateur, `refuser` pour une mission |
| Repli après attente | global, par origine | `discret` / `refuser` | `discret` pour l'utilisateur, `refuser` pour une mission |
| Attente maximale | global | 60 à 3 600 s | 600 s |
| Installation d'outils | par machine | `demander` / `outils_du_catalogue` / `jamais` | `demander` |

- L'**origine**, « utilisateur » ou « mission d'agent », vient de l'identité Bridget du processus (`resolve_current_identity`, variables posées par le lanceur), jamais d'une option.
- `compilation_capacity` reste un poids entre machines ouvertes. La console est alignée sur 0 à 10.

### 3.2 Fonctionnement normal (aucun réglage)

1. **Capacités.**
   - Lot 1a : les cibles d'une machine sont celles déclarées dans la politique. Une pré-vérification SSH de quelques secondes contrôle avant chaque travail le système, le processeur, glibc et la version exacte de Rust.
   - Lot 1b : elles viennent d'une sonde unique, au format JSON versionné. Le worker l'exécute sur les machines fédérées, en étendant la sonde de candidat existante. Le maître ou la console la lance telle quelle par SSH sur les machines sans worker. L'inventaire 311 et `qualify_workload_tools` lisent cette sonde : aucun second inventaire.
2. **Exigences du travail.** Chaque travail déclare sa cible, sa toolchain et le dépôt.
3. **Choix de la machine.**
   1. Écarter les machines `exclue`, injoignables ou sans la cible.
   2. **Filtre de classe avant tout score** : tant qu'une machine `ouverte` admissible existe, toute machine `protégée` est écartée. Ses termes processeur, mémoire, charge et continuité ne comptent donc pas.
   3. Parmi les machines restantes, préférer une place de bail libre, puis la charge la plus basse, puis le poids `compilation_capacity`.
   4. Si une machine ouverte capable existe mais n'est pas disponible : cas A du tableau. Si aucune n'existe : cas B.
4. **Exécution distante vérifiée.**
   - **Source.** `git archive` d'un commit exact déjà poussé. Si l'arbre local est modifié ou le commit non poussé, refus avec cause et action : pousser, ou relancer avec `--local`. Archive vérifiée par SHA-256 avant extraction, dans un dossier privé `0700`, avec `--no-same-owner`, sans transfert d'agent SSH.
   - **Identité.** Injection de `BRIDGET_BUILD_ID=<sha>` et, sur macOS, de `MACOSX_DEPLOYMENT_TARGET`.
   - **Bail et coupure.**
     - Le lanceur prend un bail (`acquire`), le renouvelle toutes les 20 s (`renew`) et le relâche à la fin (`release`). Un bail non renouvelé expire au bout de 60 s.
     - Sur la machine distante, un verrou `flock` sur le dossier de cache sérialise aussi les appels `--host` faits sans daemon.
     - Un gardien distant lance la compilation dans son propre groupe de processus. Il reçoit un battement par l'entrée standard du canal SSH. À la fin de l'entrée ou sans battement pendant 30 s, il envoie `SIGTERM` au groupe, puis `SIGKILL` 10 s plus tard.
     - Sur macOS distant, `caffeinate -i` empêche la mise en veille par inactivité. La fermeture du capot reste possible : la machine devient alors injoignable et le travail échoue avec sa cause.
   - **Cache.**
     - `scripts/build.py` résout la racine par `target-dir` : racine externe de `~/.config/bridget/build.json` si elle existe, sinon `~/.cache`.
     - Le dossier s'appelle `bridget-build-<dépôt>-<cible>`.
     - Le lanceur passe un identifiant logique de dépôt (`BRIDGET_BUILD_OWNER`). `build.py` l'utilise quand il n'y a pas de `.git`, pour que le marqueur de propriétaire et la purge restent valides d'un build à l'autre.
     - Seul le dossier source temporaire est retiré.
   - **Résultat.** Le binaire rapatrié est posé là où `Makefile` l'attend : `$CARGO_TARGET_DIR/release/bridget` côté local.
   - **Contrôles.**
     - Nouvelle sous-commande `bridget build-info --json`, qui rend `BUILD_ID` sans daemon. Le format de `--version` est inchangé, car la console le lit.
     - Cible Linux : ABI (`scripts/check-linux-abi.py`), démarrage et `build-info` sur l'hôte Linux, puis SHA-256 avant le rapatriement.
     - Après le rapatriement : SHA-256 de nouveau.
     - Cible macOS : `codesign --verify --strict` (et autorité si un certificat est demandé), démarrage et `build-info`.
5. **Transparence.**
   - La console affiche un tableau machines × cibles. Chaque case vaut `prête`, `à installer` (recette, durée, espace), `action manuelle requise` (geste exact) ou `impossible` (cause exacte, par exemple « glibc 2.39 > 2.36 »).
   - Chaque travail garde sa machine, la raison du choix, sa durée par phase (envoi, compilation, contrôles, retour) et ses contrôles.
6. **Repérage d'une installation utile.** Il est toujours fait ; le paramètre décide de la suite.

### 3.3 Actions permises par un paramètre

1. **Installer un outil du catalogue.** Un seul à la fois par machine, sous verrou.
   - `demander`, quand l'utilisateur lance lui-même : question unique dans son terminal (machine, outil, version, durée, espace).
   - `demander`, quand une mission lance : au moment d'`acquire`, le daemon crée une question `user_input` pour le propriétaire racine de la mission, comme pour les questions de quota. `acquire` rend « question en attente ». `build.py` relit l'état jusqu'à l'attente maximale. Sans réponse, rien n'est installé et le repli s'applique.
   - Un accord vaut pour cette seule installation. L'outil devient « prêt » après vérification de sa version.
   - `outils_du_catalogue` : installation sans question, limitée au catalogue.
   - `jamais` : aucune installation. La machine exécute toujours ce pour quoi elle est prête.
2. **Utiliser le poste protégé** : selon les cas A et B et le repli. `discret` exécute en mode discret ; `refuser` rend la cause et l'action.
3. **Dépannage.** `scripts/build.py --local` ou `--host <machine>` passe outre le choix, jamais le rôle `exclue`. Sans daemon, `--host` reste sérialisé par `flock`.

## 4. Architecture

1. **Commandes du daemon maître.**
   - Lecture : `bridget heavy-work status` et `bridget heavy-work plan --dry-run`.
   - Bail : `bridget heavy-work acquire | renew | release`.
   - Politique : `bridget heavy-work policy`, réservé au maître.
2. **Exception bornée** dans la garde CLI des sous-agents délégués, pour les seules sous-commandes `heavy-work status | plan | acquire | renew | release`. Aucune autre commande n'est ouverte.
3. **Sans daemon joignable**, `build.py` compile en local en mode discret. Il le dit et propose `--host`.
4. **Exécution distante** par le chemin SSH des livraisons, plus le gardien. Le worker n'est pas modifié dans le Lot 1a.
5. **Entretien des caches compatible Python 3.9**, la version des Mac et de l'outil de mise à jour. Un échec d'entretien est affiché, jamais avalé.

## 5. Lots

1. **Lot 1a, le plus petit lot utile : compilation release déportée.**
   - Politique avec rôles, cibles déclarées, baux simultanés, cas A et B, repli et attente.
   - `heavy-work status | plan | acquire | renew | release` et l'exception de garde CLI.
   - Filtre de classe.
   - `build-info --json`.
   - `BRIDGET_BUILD_OWNER` et entretien Python 3.9 dans `build.py`.
   - Archive, gardien distant, `flock`, `caffeinate`, contrôles, `--local` et `--host`.
   - Pas d'installation automatique.
2. **Lot 1b : sonde unique, routage des missions et console.** Sonde JSON, missions de compilation soumises aux baux et au filtre de classe, tableau machines × cibles, borne console 0 à 10.
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
   - aucun `cargo` ni `rustc` distant au-delà de 60 s ;
   - le bail expire au plus 60 s après le dernier renouvellement.
9. **Défaut sûr.** Sans `heavy-work-policy.json`, aucun travail ne part hors du poste, et `build.py` compile en mode discret.

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
