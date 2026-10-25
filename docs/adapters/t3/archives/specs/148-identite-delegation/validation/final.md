# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/final.md](../../../../../../../specs/148-identite-delegation/validation/final.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Statut : livré sur disque, sans activation.** Le code a passé les validations
ci-dessous. L'application T3 148 et le binaire Bridget signé sont installés.
Les gates de T010 sont remplies. Les dix tâches et les statuts SpecKit sont
mis à jour dans le même commit documentaire. Aucun redémarrage n'a été effectué.

<a id="extrait-2"></a>

## Extrait 2

| Composant | Commit du code | État confirmé |
| --- | --- | --- |
| Bridget natif | `347d788510b4d64529bc29f768b3ac87394f0000` | Intégré par avance rapide sur main, construit, signé et installé. Push `github/main` réussi. |
| T3 | `33f6d04e116430bf7f0011902d6af3868163f2d1` | Intégré sur `local/main-20261009` et poussé vers le fork. Aucun push upstream ni PR. |

<a id="extrait-3"></a>

## Extrait 3

Le push T3 reste limité au fork et à sa branche locale. Aucun push T3 upstream
ni aucune PR n'ont été effectués.

<a id="extrait-4"></a>

## Extrait 4

| Périmètre | Résultat retenu | Portée et limite |
| --- | --- | --- |
| Bibliothèques Rust du workspace | 1496 PASS, 0 FAIL, 15 ignorés | Run de référence avec umask 077 et dossier temporaire privé court. |
| Derniers tests Rust ciblés | 45 PASS | Identité, transport MCP et délégation native. |
| Deux recettes Rust E2E | 2 PASS | Fournisseur factice et refus d'identité T3 partielle. Elles ne prouvent pas un appel de modèle réel. |
| Format et clippy Rust final | PASS | Clippy conserve l'exception explicite préexistante `too_many_arguments`. Aucun zéro-warning sans cette exception n'est revendiqué. |
| Régressions identité T3 | 389 PASS, 1 SKIP | Résultats R2/R3. Ces 389 tests n'ont pas été rejoués dans R4. |
| Interop HTTP réelle R4 | 6 PASS, 0 FAIL | 5 tests de l'observateur et 1 scénario Rust vers le vrai serveur HTTP MCP T3. |
| Revues identité R2 et moteur natif R2/R3 | APPROVE | Les validations runtime suivantes complètent ces revues. |

<a id="extrait-5"></a>

## Extrait 5

L'enfant GLM est réel. Le parent est une fixture native Codex ; aucun modèle
Codex réel n'a été exécuté. Aucun T3 ne participe au chemin de l'enfant.
Le fait que le testeur soit lancé depuis T3 est hors de ce chemin testé.

<a id="extrait-6"></a>

## Extrait 6

**Paquet T3 et installation sur disque**

<a id="extrait-7"></a>

## Extrait 7

La signature ad hoc couvre 38 objets et conserve exactement leurs droits
existants. Les contrôles `codesign --verify --strict --deep` du stage et de sa
copie passent. L'identité reste `com.t3tools.t3code`. Le nom et l'exécutable
internes restent `T3 Code (Alpha)`.

<a id="extrait-8"></a>

## Extrait 8

Le smoke test SQLite sous Electron passe : écriture, fermeture, réouverture,
lecture persistante et `quick_check=ok`. Il utilise Electron 44.4.5, son Node
24.21.0 et SQLite 3.53.4. La base est privée. Aucun GUI, serveur T3 ou modèle
n'a été démarré par ce contrôle.

<a id="extrait-9"></a>

## Extrait 9

Archive installée :
/Applications/T3 Code (Local).app/Contents/Resources/app.asar

<a id="extrait-10"></a>

## Extrait 10

ZIP signé :
/Users/moi/.cache/t3-final148.XnkCUZ/artifacts/T3-Code-0.0.45-local.148-arm64.zip

<a id="extrait-11"></a>

## Extrait 11

Copie de livraison contrôlée :
/Users/moi/.cache/t3-final148.XnkCUZ/staging/T3 Code (Local).app

<a id="extrait-12"></a>

## Extrait 12

Application installée par le principal :
/Applications/T3 Code (Local).app

<a id="extrait-13"></a>

## Extrait 13

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/t3-build-receipt.json
- /Users/moi/.cache/t3-final148.XnkCUZ/build-receipt.json
- /Users/moi/.cache/t3-final148.XnkCUZ/verification.json

<a id="extrait-14"></a>

## Extrait 14

Journal du build avec ses avertissements :
/Users/moi/.cache/t3-final148.XnkCUZ/build-desktop.log

<a id="extrait-15"></a>

## Extrait 15

- /Users/moi/Library/LaunchAgents/com.bridget.daemon.plist
- /Users/moi/Library/LaunchAgents/com.bridget.t3.plist

<a id="extrait-16"></a>

## Extrait 16

Sauvegarde de l'application précédente, version locale `.20261009.7` :
/Users/moi/.cache/bridget-install148.KlIN2m/T3 Code (Local).app.rollback

<a id="extrait-17"></a>

## Extrait 17

| Processus | PID conservé |
| --- | --- |
| Serveur T3 | 58468 |
| Codex | 57109 |
| Daemon Bridget | 58394 |
| Serveur de pont T3 Bridget | 58396 |

<a id="extrait-18"></a>

## Extrait 18

Les sources intégrées et les fichiers installés sont distincts du code chargé
par ces processus. Aucun fonctionnement actuel de la version 148 n'est déduit
de leur présence. Une relance explicite de T3 et des services Bridget sera
nécessaire pour charger les nouveaux fichiers. Aucune activation différée
n'est programmée. Aucun réglage utilisateur, aucune authentification et aucune
base de production n'ont été modifiés dans cette livraison.

<a id="extrait-19"></a>

## Extrait 19

| Dépôt | Branche de session retirée |
| --- | --- |
| /Users/moi/Nextcloud/10.Scripts/64.bridget | `session-147-t3-v2` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget | `session-148-identite-delegation` |
| /Users/moi/11.Repositories/t3code-local | `session-148-identite-delegation` |

<a id="extrait-20"></a>

## Extrait 20

Bridget conserve seulement son worktree principal. Les autres worktrees et
branches T3 étrangers sont intacts. Les anciens builds debug et dépendances
des worktrees retirés ont été supprimés.

<a id="extrait-21"></a>

## Extrait 21

La copie non signée suivante a été retirée, ainsi que le ZIP et le blockmap
initiaux non signés :
/Users/moi/.cache/t3-final148.XnkCUZ/staging.unsigned-original

<a id="extrait-22"></a>

## Extrait 22

Les logs et reçus des défauts restent conservés. Le paquet final signé et le
cache final T3 restent conservés :
/Users/moi/.cache/t3-final148.XnkCUZ
