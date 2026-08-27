# Implémentation 049 — État des missions hors cache

## Objet mesuré

- Plateforme : Linux x86_64.
- Base gelée : `06d54831761107f5ccf3d501047eb10561e4c569`.
- Branche : `session-049-etat-missions-hors-cache`, publiée vide avant toute
  écriture.
- Base vivante observée en lecture seule :
  `/home/moi/.cache/bridget/maicie-state/maicie.sqlite3`, 6 389 760 octets,
  mode 0600, parent 0700.
- `XDG_STATE_HOME` était absent pendant la mesure ; son repli normal vaut
  `/home/moi/.local/state`.

## Inventaire avant modification

`MaicieConfig::load` exige un champ `database_path` absolu ; ni le type, ni le
parseur, ni le store n'inventent de chemin. Les commandes Maicie et les outils
de ronde ouvrent la valeur explicitement configurée.

Le seul générateur par défaut est `scripts/install-k1.sh` : il dérive
`MAICIE_STATE_DIR` de `CACHE_DIR`, écrit ce chemin dans le JSON puis crée le
répertoire privé. `scripts/test-install-k1-preflight.sh` reproduit cette même
destination dans ses fixtures. Le script préserve déjà toute configuration
existante, même avec `--force`.

Le dépôt place déjà d'autres états durables sous `${HOME}/.local/state`, dont
les ordres de spawn et les reçus idempotents. Le constat durable demande
explicitement `${HOME}/.local/state/maicie/`.

## Mesure de base

Avant toute modification :

- `bash -n scripts/install-k1.sh scripts/test-install-k1-preflight.sh` : vert ;
- univers shell : un banc agrégé, `scripts/test-install-k1-preflight.sh` ;
- résultat brut : `gate installation Maicie: refus, paire finale --force et relève gardée vérifiés`.

## Arbitrage

Le déplacement automatique est exclu : il exige une fenêtre d'arrêt et un
protocole SQLite complet. Le refus est exclu : il couperait le greffe avant sa
migration. L'installation conserve donc toute configuration explicite et
avertit lorsqu'elle désigne le cache.

## Oracle rouge

L'univers session 049 a été listé avant le tir : cinq scénarios nommés.
Avant production, le résultat brut était **1 passé / 4 échoués / 0 ignoré** :

- `default_hors_cache` : rouge ;
- `xdg_absolu_honore` : rouge ;
- `xdg_relatif_replie` : rouge ;
- `cache_explicite_preserve_averti` : rouge ;
- `etat_explicite_preserve_silencieux` : vert.

Le contrôle durable déjà vert prouve que le montage savait préserver une
configuration explicite. Les quatre rouges portaient donc sur le défaut et les
diagnostics absents, pas sur une panne générale du banc.

## Correction

Le générateur distingue désormais les deux racines :

- `CACHE_DIR` reste réservé au socket, au catalogue et aux journaux existants ;
- `STATE_HOME` vaut un `XDG_STATE_HOME` absolu, sinon
  `${HOME}/.local/state` ;
- `MAICIE_STATE_DIR` vaut `${STATE_HOME}/maicie` et reste privé en 0700.

Une valeur XDG relative est ignorée conformément à la spécification et produit
un avertissement avec le repli choisi.

Après le préflight de la paire figée, une configuration existante est lue sans
être réécrite. Si son chemin canonique se trouve sous `${HOME}/.cache` ou sous
un `XDG_CACHE_HOME` absolu, l'installateur nomme la source, confirme que la
migration n'a pas été faite et donne la cible durable. Le diagnostic précède
tout arrêt de service.

## Nominal final

Le même univers relisté rend **5 passés / 0 échec / 0 ignoré** :

```text
univers session-049 (5 scénarios): défaut durable, XDG absolu, XDG relatif, cache préservé averti, état préservé silencieux
session-049 default_hors_cache ... ok
session-049 xdg_absolu_honore ... ok
session-049 xdg_relatif_replie ... ok
session-049 cache_explicite_preserve_averti ... ok
session-049 etat_explicite_preserve_silencieux ... ok
session-049 result: 5 passed / 0 failed / 0 ignored
```

Le banc agrégé historique termine ensuite par sa ligne nominale :

```text
gate installation Maicie: refus, paire finale --force et relève gardée vérifiés
```

## Mutants et restauration

### Mutant 1 — restaurer le chemin de cache

Le mutant remplace uniquement `MAICIE_DB_PATH` par
`${CACHE_DIR}/maicie-state/maicie.sqlite3`. La ligne mutée a été relue avant le
tir. Résultat : **1 passé / 4 échoués / 0 ignoré**. Les trois chemins neufs et
la cible durable de l'avertissement meurent ; le contrôle de configuration
durable explicite reste vert.

### Mutant 2 — supprimer l'avertissement

Le second mutant retire uniquement l'appel productif au diagnostic après le
préflight. Résultat : **4 passés / 1 échoué / 0 ignoré**. Seul
`cache_explicite_preserve_averti` meurt ; les quatre autres scénarios restent
verts.

Après restauration, les condensats reviennent exactement à :

- `scripts/install-k1.sh` :
  `96334c5191029e64ee0ae5dfbc87777e0a479c2f21b03a6bfdf8e6bc7a46ccfe` ;
- `scripts/test-install-k1-preflight.sh` :
  `04f0a90914de9cdfc6e3eab84f91e301894cd9991cbd767b7eba8f4454155b66`.

Le nominal final est de nouveau **5/0/0**.

## Comparaison base/tête

La base et la tête ont été jouées dans la même passe, depuis deux worktrees Git
distincts :

- base : le banc agrégé historique passe et rend sa ligne nominale ;
- tête : le même banc passe, avec l'univers interne 049 à **5/0/0**.

La base ne possède pas encore les cinq scénarios internes ; aucun cardinal
inventé ne lui est attribué. `bash -n` est vert des deux côtés avant le tir.

## Gates statiques

- Linux 6.8.0-94-generic x86_64 ;
- GNU bash 5.2.21 ;
- Python 3.12.3 ;
- `bash -n scripts/install-k1.sh scripts/test-install-k1-preflight.sh` : vert ;
- `git diff --check` : vert ;
- `shellcheck` : indisponible sur la machine, aucun outil installé.

Le delta fonctionnel touche uniquement l'installateur et son banc. Les quatre
autres fichiers sont la transaction documentaire de la session 049. Aucun
fichier Rust, schéma SQLite ou protocole n'est modifié.

## Non mesuré

- macOS et launchd ;
- exécution de l'installateur contre le HOME vivant ;
- arrêt, copie, déplacement, redémarrage ou retour arrière de la base vivante ;
- comportement d'un `XDG_STATE_HOME` absolu explicitement configuré par
  l'opérateur sous un support éphémère qui n'est pas une racine de cache XDG.
