# Spécification 312 : travaux lourds hors du poste de travail

- **Statut** : spécification, plan à écrire
- **Demande** : François, 22 novembre 2026. Il ne veut plus que son Mac « s'effondre de fatigue » pendant les compilations. Bridget doit pouvoir déporter les travaux lourds sur les autres machines, en tenant compte de leurs capacités réelles. Celles qu'on ne peut pas remplacer comptent autant que celles qu'on peut installer.
- **S'appuie sur** :
  - `specs/311-console-locale-parametres-machines-cli/spec.md` : onglets Paramètres et Machines, inventaire des machines, catalogue fermé d'installation ;
  - placement des missions `workload: "compilation"` (`crates/bridget-daemon/src/daemon/native_delegation.rs`, terme `compilation` du score de machine) ;
  - réglage `compilation_capacity` par machine (`crates/bridget-daemon/src/registry.rs`).

## 1. Constats du 22 novembre 2026

1. Les livraisons 328 à 330 ont été compilées à la main :
   - le binaire Mac sur le poste de François (328 et 329), puis sur `macbook-m1` (330, release en 131 s) ;
   - le binaire Linux sur `sysle2` (Debian 12, glibc 2.36), en 3 min 42 s à 3 min 49 s.
2. `cartae` et `ksstore` ont glibc 2.39. Un binaire compilé chez eux ne démarre pas sur `sysle2`. Seul `sysle2` produit aujourd'hui le binaire Linux commun.
3. `macbook-m1` n'avait pas Rust. Il a fallu l'installer (Rust 1.92.0, version du dépôt). Il lui manque encore Python 3.11 : l'entretien des caches de `scripts/build.py` y est différé.
4. `scripts/build.py` compile toujours sur la machine qui le lance. Le placement « compilation » ne sert qu'aux missions déléguées.
5. Avec un seul Mac et des serveurs Linux, aucune autre machine ne pourrait produire le binaire Mac. La politique doit dire quoi faire dans ce cas.

## 2. Vocabulaire

- **Poste protégé** : machine où l'utilisateur travaille lui-même. Bridget évite d'y placer des travaux lourds.
- **Travail lourd** : compilation release ou tests complets de Bridget, build de T3 Code, calcul long délégué. Le Lot 1 ne traite que la compilation de Bridget.
- **Cible** : le système et le processeur visés par un binaire. Exemples : `macos-arm64` ; `linux-x86_64`, glibc 2.36 au plus.
- **Capacité fixe** : propriété de la machine qu'aucune installation ne change. Exemples : système, processeur, version maximale de glibc, certificat de signature, matériel.
- **Capacité à installer** : outil absent mais installable par une recette connue. Exemples : version Rust du dépôt, Python 3.11, outils de compilation Apple.
- **État du moment** : charge, mémoire, disque libre, joignabilité, activité de l'utilisateur sur la machine.
- **Compilation croisée** : produire un binaire pour une autre cible, par exemple Linux glibc 2.36 depuis une machine glibc 2.39. Elle peut lever une capacité fixe, seulement après qualification.
- **Mode discret** : exécution en priorité basse sur un poste protégé (`taskpolicy -b` sur macOS, `nice`/`ionice` sur Linux), avec moins de tâches parallèles.

## 3. Ce qui relève des paramètres, du fonctionnement normal et des actions autorisées

### 3.1 Paramètres (choix de l'utilisateur, rares et durables)

Ils sont réglés dans l'onglet Paramètres de la console 311 et enregistrés dans le registre existant.

| Paramètre | Portée | Valeurs | Défaut |
|---|---|---|---|
| Poste protégé | par machine | oui / non | oui pour la machine où tourne la console, non ailleurs |
| Seul un poste protégé convient | global | mode discret / attendre une machine libre / refuser | mode discret |
| Installation d'outils | par machine | demander à chaque fois / autoriser les outils connus / jamais | demander à chaque fois |
| Exclue des travaux lourds | par machine | oui / non | non |

Le réglage existant `compilation_capacity` reste la borne du nombre de compilations simultanées par machine.

### 3.2 Fonctionnement normal (sans réglage)

1. **Inventaire des capacités.** Bridget relève et rafraîchit, pour chaque machine joignable :
   - système, processeur et version de glibc ;
   - versions des outils du catalogue ;
   - charge, mémoire et disque libre.

   Il réutilise l'inventaire 311 et les faits de ressources du routage.
2. **Exigences du travail.** Chaque travail déclare sa cible et ses outils. Le binaire Mac de Bridget exige `macos-arm64` et la version Rust de `rust-toolchain.toml`. Le binaire Linux exige glibc 2.36 au plus et la même version Rust.
3. **Choix de la machine**, dans cet ordre :
   1. écarter les machines sans les capacités fixes, les machines exclues et les machines injoignables ;
   2. préférer les machines déjà prêtes ;
   3. à égalité, préférer la machine la moins chargée, sous la borne `compilation_capacity` ;
   4. n'utiliser un poste protégé qu'en dernier recours, selon le paramètre « seul un poste protégé convient ».
4. **Exécution à distance vérifiée.**
   - Envoi du commit exact (`git archive`), jamais du checkout avec ses changements non commités.
   - Compilation dans une racine de compilation autorisée par `scripts/build.py`.
   - Rapatriement du binaire, puis contrôle de la somme SHA-256, de la signature (`codesign --verify --strict`) et de la compatibilité glibc (`scripts/check-linux-abi.py`).
   - Retrait des fichiers de compilation sur la machine distante.
5. **Transparence.**
   - La console affiche un tableau machines × cibles : prête, à installer (avec la recette) ou impossible (avec la cause).
   - Chaque travail garde où il a tourné, pourquoi, sa durée et ses vérifications.
6. **Détection d'une installation utile.** Si une machine non protégée n'a que des capacités à installer manquantes, Bridget le signale toujours. Le paramètre « Installation d'outils » décide de la suite.

### 3.3 Actions permises par un paramètre

1. **Installer un outil** sur une machine :
   - **« demander à chaque fois »** : une question unique et précise, avec la machine, l'outil, la version, la durée et l'espace estimés. Un accord vaut pour cette installation seulement. Le résultat vérifié est mémorisé comme capacité de la machine.
   - **« autoriser les outils connus »** : installation sans question, limitée au catalogue fermé (version Rust du dépôt, Python 3.11, outils Apple, cibles de compilation croisée qualifiées).
   - **« jamais »** : aucune installation. La machine reste « à installer » dans le tableau.
2. **Utiliser un poste protégé**, seulement quand il est la seule machine qui convient : en mode discret, après attente bornée d'une machine libre, ou refus avec la cause, selon le paramètre.

## 4. Lots

1. **Lot 1 : compilation de Bridget.** `scripts/build.py` gagne une sortie distante pour `cargo build --release` et les tests longs. La recette de livraison utilise les mêmes choix. Les missions `workload: "compilation"` utilisent l'inventaire des capacités fixes.
2. **Lot 2 : build de T3 Code** (Electron, macOS uniquement), même mécanisme.
3. **Lot 3 : autres calculs longs délégués**, déjà placés par le routage : appliquer le poste protégé et les exclusions.

## 5. Hors périmètre

- Compiler un binaire macOS hors d'un Mac (SDK Apple réservé au matériel Apple).
- Installer un logiciel hors du catalogue fermé.
- Arrêter ou ralentir les missions métier d'autres agents pour libérer une machine.

## 6. Critères d'acceptation

1. Pendant une compilation release de Bridget lancée depuis le poste protégé, ce poste reste sous 10 % de processeur en moyenne. La mesure se fait sur la durée de la compilation, avec un relevé avant et après.
2. La durée totale, transferts compris, n'excède pas de plus de 20 % la même compilation locale. Mesure sur le même commit, cache froid des deux côtés.
3. Le binaire rapatrié passe la vérification de somme, de signature et de glibc. Il démarre et répond à `bridget --version` sur chaque cible de livraison.
4. Avec `macbook-m1` débranché, une compilation Mac suit le paramètre « seul un poste protégé convient » : elle tourne en mode discret, elle attend, ou elle est refusée avec la cause. Les trois cas sont testés.
5. Avec le réglage « demander à chaque fois », une machine sans Rust déclenche une seule question. Sans accord, rien n'est installé. Après accord, la machine devient « prête » après vérification.
6. Une machine marquée « exclue » ou « jamais » n'exécute aucun travail lourd et aucune installation. C'est vérifié par test et par le journal.
7. Le tableau de la console montre, pour le réseau actuel :
   - `macos-arm64` prête sur deux machines ;
   - Linux glibc 2.36 prête sur `sysle2` ;
   - `cartae` et `ksstore` impossibles pour cette cible, cause « glibc 2.39 », tant que la compilation croisée n'est pas qualifiée.

## 7. Hypothèses à éprouver

1. **Compilation croisée Linux** depuis `cartae` ou `ksstore` vers glibc 2.36, par exemple avec `cargo-zigbuild`. À qualifier par le contrôle ABI et un démarrage réel sur `sysle2` avant de l'ouvrir au choix.
2. **Coût du transfert** (archive d'environ 39 Mo, binaire de 38 à 47 Mo) sur chaque lien. À mesurer avant de fixer les seuils.
3. **Signal d'activité de l'utilisateur** sur un poste protégé (inactivité clavier et souris sous macOS). Utile pour « attendre une machine libre » ; à valider sans nouvelle permission système.
4. **Python 3.11 sur `macbook-m1`** : installation par le catalogue, ou entretien des caches compatible Python 3.9. À décider au plan.

## 8. Risques

- **Binaire différent selon la machine.** Parade : commit exact, toolchain épinglée, vérifications du §3.2.4, comparaison avec un build local lors de la qualification.
- **Machine distante chargée par d'autres missions.** Parade : borne `compilation_capacity` et faits de charge frais.
- **Installation qui échoue ou laisse un état partiel.** Parade : recette idempotente, vérification après installation ; capacité marquée « prête » seulement après vérification.
- **Fuite de secrets dans l'archive envoyée.** Parade : `git archive` du commit poussé uniquement, sans fichiers ignorés ni non suivis.
