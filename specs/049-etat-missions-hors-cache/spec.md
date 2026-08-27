# Spécification 049 — État des missions hors cache

**Statut** : Prêt pour revue

**Base gelée du lot** : `06d54831761107f5ccf3d501047eb10561e4c569`

**Objectif** : `756b05d6-da06-407e-b1a5-1924cff24c8a`

## Problème mesuré

Le 27 août 2026, la configuration vivante
`/home/moi/.config/maicie/config.json` désigne la base SQLite
`/home/moi/.cache/bridget/maicie-state/maicie.sqlite3`. Cette base contient le
greffe durable des missions ; elle mesure 6 389 760 octets, son mode est 0600
et son répertoire parent est privé en 0700.

Le risque n'est pas une purge automatique actuellement configurée : aucune
règle tmpfiles mesurée ne nettoie ce cache. Le risque est qu'un opérateur
libère légitimement un répertoire présenté comme destructible et supprime le
greffe.

L'inventaire montre que le défaut ne vient pas de Maicie : `database_path` est
un champ obligatoire de sa configuration et ne possède aucune valeur implicite
Rust. Le générateur fautif est `scripts/install-k1.sh`, qui écrit une
configuration absente sous `${HOME}/.cache/bridget/maicie-state/`. Son banc
d'installation verrouille encore cette destination.

La [spécification XDG Base Directory
0.8](https://specifications.freedesktop.org/basedir/0.8/) réserve
`XDG_STATE_HOME` aux états qui doivent survivre aux redémarrages et
`XDG_CACHE_HOME` aux données non essentielles. Sans surcharge, le premier vaut
`${HOME}/.local/state` et le second `${HOME}/.cache`.

## Propriété

Une installation sans configuration Maicie existante place sa base de missions
dans un répertoire d'état durable, jamais dans le cache par défaut.

Une installation existante conserve sa configuration octet pour octet. Si son
`database_path` se trouve déjà sous le cache utilisateur, l'installateur rend
le risque visible et nomme la cible durable recommandée ; il ne déplace pas la
base et ne prétend pas l'avoir migrée.

## Scénarios

### US1 — Installation neuve

En l'absence de configuration, l'installateur crée une configuration dont la
base vaut `${XDG_STATE_HOME}/maicie/maicie.sqlite3` lorsque cette variable est
absolue, ou `${HOME}/.local/state/maicie/maicie.sqlite3` sinon. Le répertoire
privé est créé en 0700.

### US2 — Installation existante sous cache

Une configuration explicite pointant sous `${HOME}/.cache` reste strictement
inchangée, y compris avec `--force`. L'installation réussit après son préflight
mais affiche un avertissement qui nomme le chemin à risque, la cible durable et
la nécessité d'un arrêt coordonné pour migrer.

### US3 — Installation existante durable

Une configuration explicite hors cache reste inchangée et ne produit aucun
avertissement de migration.

### US4 — Variable XDG relative

Une valeur relative de `XDG_STATE_HOME` est invalide selon XDG. L'installateur
l'ignore, le signale et retombe sur `${HOME}/.local/state`, sans produire de
chemin relatif ni de chemin de cache.

## Arbitrage des installations existantes

### Déplacement automatique — rejeté dans ce lot

Une base SQLite vivante peut posséder des fichiers WAL et SHM. Une migration
sûre exige l'arrêt du service, une copie ou un déplacement cohérent de tous les
artefacts, une vérification, la mise à jour atomique de la configuration, un
redémarrage et une voie de retour. Ce lot n'a pas l'autorité de couper les
agents connectés.

### Refus — rejeté

Refuser une configuration de cache rendrait immédiatement indisponible le
greffe existant avant que sa migration humaine soit planifiée. Cela
transformerait un risque de conservation en panne certaine.

### Avertissement avec conservation — retenu

L'avertissement rend le risque visible sans mentir sur une migration ni couper
le service. Il conserve la paire binaire/configuration déjà préflightée et
laisse la migration vivante à une opération distincte autorisée par l'humain.

## Exigences fonctionnelles

- **FR-4901** : le générateur de configuration utilise un répertoire d'état
  durable et non `CACHE_DIR` pour `database_path`.
- **FR-4902** : une valeur `XDG_STATE_HOME` absolue est honorée ; une valeur
  absente, vide ou relative retombe sur `${HOME}/.local/state`.
- **FR-4903** : le répertoire parent de la base par défaut est créé en 0700.
- **FR-4904** : une configuration existante est conservée octet pour octet,
  avec ou sans `--force`.
- **FR-4905** : une configuration existante sous le cache utilisateur produit
  un avertissement nommé après validation, sans déplacement ni refus.
- **FR-4906** : une configuration existante hors cache ne produit pas cet
  avertissement.
- **FR-4907** : le chemin du catalogue, du socket Bridget et des journaux de
  service reste inchangé ; ils ne contiennent pas la base des missions.

## Critères de succès

- **SC-4901** : configuration absente, le JSON publié porte exactement le
  chemin d'état attendu et aucun composant de ce chemin n'est le cache par
  défaut.
- **SC-4902** : la cible d'état existe en 0700 après installation.
- **SC-4903** : configuration cache existante, les octets avant/après sont
  identiques et la sortie contient l'avertissement avec source et cible.
- **SC-4904** : configuration durable existante, les octets restent identiques
  et l'avertissement est absent.
- **SC-4905** : remettre le générateur sous `CACHE_DIR` fait mourir le témoin
  SC-4901 sans faire mourir les contrôles de conservation.

## Hors périmètre

- Arrêter les services ou déplacer la base vivante.
- Modifier la configuration vivante de cette machine.
- Migrer, copier, ouvrir en écriture ou supprimer un fichier SQLite existant.
- Déplacer le catalogue de constats, le socket ou les journaux actuellement
  placés dans le cache.
- Refuser au runtime toute configuration explicite contenant un composant de
  cache.
