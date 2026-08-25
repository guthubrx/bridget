# ARRÊT DE NUIT — FLOTTE BLOQUÉE (2026-08-24, ~22h)

## 🔎 CAUSE ÉTABLIE (enquête close le 2026-08-25) — **UNE SEULE CAUSE**

**Ni le disque, ni les processus. Le shell de connexion, écrasé par un agent.**
`dscl . -read /Users/moi UserShell` → `/opt/homebrew/bin/zsh`. Un agent
exécute ses commandes par `$SHELL -c "…"` : si ce shell ne peut pas démarrer,
la commande rend un code d'erreur **sans stdout ni stderr**.

**Chronologie horodatée, avec le `pid` comme discriminant** (captures de
terminal Cursor). *Un terminal qui porte un `pid` a démarré ; sans `pid`, le
spawn n'a jamais eu lieu.*

| heure | terminal | issue | pid | cwd |
|---|---|---|---|---|
| 18:56:02 | 511694 | succeeded | 49675 | `/private/tmp/bridget-c5-rejeu-…` |
| 21:47:45 | 214029 | failed | **aucun** | `/private/tmp/bridget-c5-rejeu-…` — **disparu** |
| 21:49:52 | 214030 | failed | **aucun** | idem, disparu |
| **21:49:55** | **575702** | **succeeded** | **79623** | worktree valide |
| 21:53:15 | 214031 | failed | aucun | idem, disparu |
| ~21:49:55–21:53:15 | — | **`Write` du shim** | — | *bascule de la machine* |
| 21:53:52 | — | réécriture du shim (inode actuel) | — | — |
| 21:57:06 | 424050 | failed | **aucun** | **cwd VALIDE** |

**⚠️ IL N'Y A PAS DEUX CAUSES. LE LIEN N'A JAMAIS « DISPARU » AVANT — IL A ÉTÉ
ÉCRASÉ PAR LE SHIM.** Deux versions successives de cette note ont dit le
contraire ; toutes deux sont **fausses** et retirées.

**La pièce qui tranche** : le terminal **575702 réussit à 21:49:55 avec un
`pid`**, donc *après* les deux premiers `ENOENT`. Le shell était vivant,
présent et exécutable à cet instant. Les échecs antérieurs ne peuvent donc pas
signifier qu'il était absent : ils venaient tous du **même répertoire de
travail disparu**, `/private/tmp/bridget-c5-rejeu-7fdf5c6-36812` (vérifié
absent, alors qu'il avait servi à 18:56:02). **Node rend `ENOENT` quand le
`cwd` n'existe plus, et libelle l'erreur du nom du binaire.**
**Contre-épreuve** : à 21:57, les terminaux 424050/424051 échouent avec un
**cwd parfaitement valide** — là, seul le shell peut être en cause. 575702 est
le **dernier spawn réussi de toute la machine**.

**Confirmation indépendante** : restaurer le lien a rendu les mains à
**vingt agents vivants, sans aucune relance de session**. Un seul fichier
remis en place ne ressuscite pas une flotte si la cause était ailleurs.

**La chaîne réelle, en cinq temps :**
1. un répertoire temporaire s'évapore — **anodin, ne casse rien** ;
2. les commandes de cette session rendent `spawn …/zsh ENOENT` — erreur **du
   `cwd`**, libellée du nom du shell ;
3. l'agent lit le message au premier degré : *« Le shell Cursor pointe vers un
   zsh manquant »* ;
4. il **écrase le lien Homebrew** par un fichier texte de 32 octets en `644` —
   **la flotte tombe ici** ;
5. il voit son erreur, trouve le vrai binaire par `Glob`… et **réécrit le même
   shim** au lieu de le copier, sans `chmod +x`. Il ne le pouvait plus : privé
   de shell, il n'avait ni `cp` ni `ln`, et son outil `Write` n'écrit que du
   texte — un binaire de 668 Ko ne s'écrit pas ainsi. `brew reinstall zsh`
   échoue ensuite, faute de mains.

**Auteur, établi sur pièce** : agent Cursor, session
`27839754-9b49-408c-9b44-9d99c26207d8`, transcript
`~/.cursor/projects/Users-moi-Nextcloud-10-Scripts-bridget/agent-transcripts/…`
(`tool_use` `Write`, chemin et contenu cités). Aucune écriture Claude Code sur
ce chemin. Aucun `brew` n'a tourné le 24. **Un seul fichier touché** dans
`/opt/homebrew/bin` et `/usr/local/bin` de toute la journée.

**Seul angle encore ouvert, et il est mineur** : qui a supprimé le répertoire
temporaire `/private/tmp/bridget-c5-rejeu-…`. **Cela n'a cassé personne** —
inutile d'y consacrer une mission.

## ⚑ LE PIÈGE D'INSTRUMENT — trois victimes successives

> **Un `ENOENT` de spawn ne nomme pas forcément le coupable qu'il affiche.**

Le même message a trompé, dans l'ordre : **l'agent Cursor**, qui a écrasé le
shell système ; **le relecteur** qui a mené l'enquête et conclu que la panne
précédait le shim ; puis **le référent**, qui a gravé cette conclusion dans
cette note alors que le relecteur l'avait déjà retirée.

**Le discriminant, gratuit et fiable** : chercher un **contre-exemple qui a
réussi *après* l'échec**. Un terminal avec `pid` a démarré. S'il en existe un
postérieur, le binaire n'était pas en cause.

**Leçon de dispositif** : la revue croisée n'a pas failli — la correction
était partie **avant** la gravure. Elle a été gravée quand même, parce que le
message n'avait pas été ouvert. *Un dispositif de revue ne protège que si la
correction remonte plus vite que la gravure.*

**Le binaire n'a jamais été touché** : `/opt/homebrew/Cellar/zsh/5.9.2/bin/zsh`,
668 352 octets, intact. C'est le lien qui manquait.
**Réparation appliquée** : `ln -sfn ../Cellar/zsh/5.9.2/bin/zsh
/opt/homebrew/bin/zsh`. Les mains sont revenues à tous les processus vivants
**sans aucune relance de session**.
**Pièce conservée** : `~/piece-zsh-2026-08-24-2153.bak`.

---

## ✅ RÉSOLU — le diagnostic d'origine était le bon, et ma « correction » était fausse

**Séquence complète, mesurée, à lire comme une leçon de méthode.**

1. Diagnostic d'origine : **disque plein**. Correct.
2. J'ai voulu le réfuter en constatant que `Write` fonctionnait toujours, et
   j'ai fait mesurer `df -h /` → **40 % de capacité**. J'en ai conclu que
   l'espace n'était pas en cause et j'ai réécrit cette note en accusant la
   table des processus. **Faux.**
3. `df /` mesure la **partition système en lecture seule**, pas le volume de
   données. La bonne commande est `df -h /System/Volumes/Data`, qui rendait :
   **99 % de capacité, 17 Gi libres sur 926 Gi.** Le disque était bien plein.

**⚠️ L'erreur que j'ai commise est exactement celle contre laquelle cette
note mettait en garde deux paragraphes plus bas** (« ne pas chiffrer
l'ampleur sur le mauvais répertoire, l'erreur a déjà été commise cette nuit
sur un autre sujet »). Écrire l'avertissement ne protège pas de l'erreur ;
seule la commande juste protège. **Sur macOS, `df /` ne veut rien dire.**

### Le nettoyage, exécuté et mesuré

| | avant | après |
|---|---|---|
| espace libre | 17 Gi | **28 Gi** |
| capacité | 99 % | 97 % |
| inodes libres | 179 M | 289 M |

**43 répertoires temporaires orphelins** dans `$TMPDIR` — pas un seul dans
`/tmp`. La cause déclarée par son auteur (bancs qui terminent par `panic!`
avant leur `fs::remove_dir_all`) est donc **confirmée par le comptage**.

### Ce qui reste cassé, et qui est autre chose

**Le shell du référent ne revient pas** après la libération des 11 Gi
(`echo vivant` rend toujours exit 1 sans sortie). Ce n'est donc pas l'espace :
c'est **le processus Claude Code lui-même qui est à bout** — descripteurs de
fichiers épuisés, très probablement, après des centaines de commandes.
**Remède : relancer la session.** Le daemon et les agents sont des processus
indépendants et survivent.

**RÉPONDU PAR relec5, et c'est le fait le plus important de cette note :**
**la panne survit au disque.** Testé deux fois après la libération des 11 Gi
(`df -h /System/Volumes/Data ; echo vivant`, puis `echo vivant` seul) →
exit 1, aucune sortie. Son canal MCP fonctionne parfaitement ; seul son shell
est mort.

**Ce que ça établit** : la saturation était la cause de la **chute**, pas la
cause de la **persistance**. Un processus qui a perdu sa capacité d'exécution
ne la récupère pas quand la ressource revient. **Le nettoyage était
nécessaire et il a marché sur le volume — il ne suffit pas à redémarrer les
agents.** Seule une relance des sessions rend la main.

**Conséquence pratique** : ne pas perdre de temps à chercher un autre
coupable. Nettoyer, puis relancer les sessions, dans cet ordre.

### ✅ REMÈDE CONFIRMÉ PAR MESURE — le respawn rend les mains

Un agent **neuf** (`relec6`) a été spawné après le nettoyage, sous un nom
libre, sans toucher aux anciens :

```
mkdir -p /tmp/relec6-lot
bridget spawn claude --name relec6 --cwd /tmp/relec6-lot --persistent
```

Son premier geste : `echo vivant` → **stdout `vivant`, exit 0.**

**Ce que ça établit définitivement** : la paralysie est un état persistant
des processus nés AVANT la saturation, pas une propriété courante de la
machine. Aucune machine distante n'est nécessaire. **Tout agent dont on a
besoin se respawne, et il a des mains.**

**Attention à la syntaxe** : `bridget respawn` **n'existe pas** — les rondes
de vigilance emploient un raccourci de langage. La vraie forme est
`bridget spawn <TYPE> [--name N] [--cwd CHEMIN] [--persistent]`, types
`codex | claude | gemini | gclaude`.

**Deux précautions** :
- spawner sous un **nom libre** plutôt que `stop` + réutilisation : les
  anciens agents sont inoffensifs (ils parlent sans mains) et un `stop` sur
  un équipier non géré est un risque gratuit ;
- **`--cwd` hors du dépôt** — le checkout principal est fermé aux agents ;
  chacun clone dans son propre répertoire.

**Bornez le nombre d'agents qui compilent.** À 97 % de volume, chaque
`CARGO_TARGET_DIR` pèse plusieurs Gi. Deux compilations simultanées au
maximum tant que l'espace n'est pas rendu.

### Résidus à supprimer — déclarés par leurs auteurs, qui ne peuvent pas le faire

```
rm -rf /tmp/tgt-relec5 /tmp/relec5-lot /tmp/relec5-perf.sqlite3 \
       /tmp/relec5-banc /tmp/relec5-banc2 /tmp/relec5-banc3
```
**À GARDER** : `/tmp/relec5-preuves/` et `/tmp/relec5-pty.py` (quelques Ko,
valent des heures). `/tmp/cursor8-12b.txt` a résisté au `rm` (permission
refusée) — le laisser, il est minuscule.

### La leçon matérielle, à graver dans les règles

Les 43 orphelins avaient **deux** causes, pas une : des bancs qui paniquent
*avant* leur `fs::remove_dir_all` (relec1), et des bancs qui n'ont **jamais
eu** de nettoyage du tout (relec5, déclaré par lui avant de connaître le
chiffre). Les bancs du premier ayant été repris par l'auteur puis par le
second, **ses résidus se sont multipliés par trois collèges**.

> **Une revue qui compile a un coût matériel, à borner comme le reste.**
> Un banc doit nettoyer par un garde qui survit au `panic!`, et le target
> d'un relecteur se supprime à la reddition du verdict.

---

## Le constat d'origine (conservé tel quel — il avait raison)

**Le shell est mort pour toute la flotte** — `echo test` et `rm` rendent
exit 1 sans sortie, chez le référent comme chez les sept agents interrogés.
Personne ne peut supprimer ses propres fichiers. **Intervention humaine
nécessaire.**

## La courbe, relevée avant la panne

| heure | libre sur `/System/Volumes/Data` |
|---|---|
| 21:23 | **33 Gi** |
| 21:35 | 22 Gi |
| 21:45 | **19 Gi** |
| 21:52:23 | dernière commande réussie |
| ~21:53 | **shells morts** |

Chute de ~14 Gi en 22 minutes, puis de 19 Gi à zéro en moins de sept
minutes. **Ce n'est pas une dérive lente : quelque chose a consommé
massivement après 21:45.**

## ⚑ LA CAUSE PROBABLE — des milliers de répertoires temporaires orphelins

Déclarée par son auteur : **les bancs de revue se terminent par un `panic!`
AVANT leur `fs::remove_dir_all`.** Chaque tir laisse donc sa base SQLite
derrière lui — séries de 10 tirs, contrôles appariés, plusieurs passages,
repris ensuite par deux autres agents. **Des milliers d'entrées et
d'inodes.**

```
/tmp/maicie-routines-*
/tmp/maicie-ui-projection-*
/tmp/r1t.*
```

**⚠️ ET LES MÊMES SOUS `$TMPDIR` RÉEL** (`/var/folders/…/T/`), qui est le
chemin par défaut et **n'est pas `/tmp`** — ne pas chiffrer l'ampleur sur le
mauvais répertoire, l'erreur a déjà été commise cette nuit sur un autre
sujet.

## ⚑ LE GESTE QUI DÉBLOQUE

```
df -h /
ls -d $TMPDIR/maicie-routines-* /tmp/maicie-routines-* 2>/dev/null | wc -l
rm -rf $TMPDIR/maicie-routines-* $TMPDIR/maicie-ui-projection-* $TMPDIR/r1t.* \
       /tmp/maicie-routines-* /tmp/maicie-ui-projection-* /tmp/r1t.*
rm -rf /tmp/tgt-relec5                     # le plus gros : debug + release
rm -rf /tmp/cursor4-viktor-ledger-target /tmp/tgt-relec1 /tmp/relec1-routines
rm -rf /tmp/cursor7-routines-target /tmp/cursor8-* /tmp/tgt-cursor3-* \
       /tmp/viktor-statut-honnete-target /tmp/bridget-jury-viktor-* \
       /tmp/bridget-target-ledger-emission-* /tmp/relec5-lot
df -h /
```

**NE PAS SUPPRIMER** : le `target/` du dépôt principal (le daemon en
dépend) ; ni `/tmp/relec1-preuves`, `/tmp/relec4-epreuves`,
`/tmp/relec5-preuves`, `/tmp/relec5-pty.py`,
`/tmp/cursor4-viktor-ledger-attacks` — bancs et instruments des jurys,
quelques kilo-octets qui valent des heures.

Ensuite : `cargo build --release`.

## ⚠️ TRAVAIL À DÉCOUVERT — NE RIEN RESTAURER

`.worktrees/fix-outcome-unknown-statut-honnete` porte **deux README modifiés
et NON COMMITÉS**. Un `git checkout --` ou un `git stash` les effacerait sans
trace — c'est déjà arrivé cette nuit à six éditions du même auteur.
**Premier geste au retour du disque : les faire committer par leur auteur.**

## ⚠️ FENÊTRE DE DOUTE SUR LES COMPTES — rétroactive

Une compilation sur disque saturé produit des erreurs qui **ressemblent à des
rouges de lot**. C'est rétroactif : **tout compte de gates pris après ~21:45
est suspect**, y compris ceux des relecteurs. À revérifier quand l'espace
sera rendu :

- le rouge `coordination_dispatch::notification_partage_une_echeance_absolue`
  (3/3 en suite complète, **5/5 vert en isolation**) : la pression disque est
  une cause au moins aussi plausible que la charge CPU, et elle expliquerait
  mieux l'écart suite/isolé ;
- les comptes `73/73` et `4/4` annoncés sur `9f7bcc6`, s'ils ont été pris
  dans cette fenêtre.

Les comptes **antérieurs à 21:45** restent fiables (35/35, 0/5, MA-1,
matrice six champs, 13/13).

---

## Les trois lots (rien n'est perdu, tout le reste est commité)

### routines v15 — tête `9f7bcc6` — BLOCKED sur un seul motif
Neuf SHA, **cinq motifs traités**, dont deux nés des correctifs eux-mêmes.
Reste **l'adoption hors borne** : voir `ETAT-NUIT-ANGLES-A-INSTRUIRE.md`,
qui porte le protocole en trois points et l'avertissement d'exécution.
**Ne jamais merger `061e773` ni `f837ed0`.**

### statuts distincts — tête `2415d0e` (`e1a58f0` est orphelin)
Les deux relecteurs ont rendu APPROVE_WITH_CHANGES ; dernière condition
livrée, **plus la nuance non commitée ci-dessus**. Vérifier avant merge que
le diff `106bb7f..2415d0e` ne contient que les README.

### ledger émis/vu — tête `235e874` — BLOCKED
Trois conditions livrées. Bloqué sur la **performance de la jointure** :
20 s de médiane à 10 000 messages contre 11 ms avec index. Correctif : index
sur `(operation_kind, idempotency_key)`, EXPLAIN qui passe de SCAN à SEARCH,
remesure sous 20 ms sur cinq rounds.

## Mergé cette nuit (déjà en production)

Correctif du boot (8 s), spawn-fantôme, hook StatusLine, checklist de
redémarrage, lot `outcome_unknown` complet, hotfix des rappels étrangers, et
le **consentement `--migrate`** — qui empêche désormais qu'une base soit
migrée silencieusement à l'ouverture, cause des deux pannes de la soirée.
