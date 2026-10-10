Reçu de validation et de livraison — session 148 — 10 octobre 2026

**Statut : livré sur disque, sans activation.** Le code a passé les validations
ci-dessous. L'application T3 148 et le binaire Bridget signé sont installés.
Les gates de T010 sont remplies. Les dix tâches et les statuts SpecKit sont
mis à jour dans le même commit documentaire. Aucun redémarrage n'a été effectué.

**Code testé et intégré**

| Composant | Commit du code | État confirmé |
| --- | --- | --- |
| Bridget natif | `347d788510b4d64529bc29f768b3ac87394f0000` | Intégré par avance rapide sur main, construit, signé et installé. Push `github/main` réussi. |
| T3 | `33f6d04e116430bf7f0011902d6af3868163f2d1` | Intégré sur `local/main-20261009` et poussé vers le fork. Aucun push upstream ni PR. |

Source privée du build Bridget terminé :
/Users/moi/.cache/bridget-final148.wDyecn/source

Les commits ci-dessus identifient le code testé. Le présent reçu est une mise
à jour documentaire ultérieure. Son commit sera observable dans l'historique
Git. Il ne change aucune source du binaire ni les commits du code indiqués
dans le tableau. Il ne prouve pas un nouveau runtime actif.

Destination du push Bridget confirmé, code retour 0 :
https://github.com/guthubrx/bridget.git

Le push T3 reste limité au fork et à sa branche locale. Aucun push T3 upstream
ni aucune PR n'ont été effectués.

**Validations indépendantes**

| Périmètre | Résultat retenu | Portée et limite |
| --- | --- | --- |
| Bibliothèques Rust du workspace | 1496 PASS, 0 FAIL, 15 ignorés | Run de référence avec umask 077 et dossier temporaire privé court. |
| Derniers tests Rust ciblés | 45 PASS | Identité, transport MCP et délégation native. |
| Deux recettes Rust E2E | 2 PASS | Fournisseur factice et refus d'identité T3 partielle. Elles ne prouvent pas un appel de modèle réel. |
| Format et clippy Rust final | PASS | Clippy conserve l'exception explicite préexistante `too_many_arguments`. Aucun zéro-warning sans cette exception n'est revendiqué. |
| Régressions identité T3 | 389 PASS, 1 SKIP | Résultats R2/R3. Ces 389 tests n'ont pas été rejoués dans R4. |
| Interop HTTP réelle R4 | 6 PASS, 0 FAIL | 5 tests de l'observateur et 1 scénario Rust vers le vrai serveur HTTP MCP T3. |
| Revues identité R2 et moteur natif R2/R3 | APPROVE | Les validations runtime suivantes complètent ces revues. |

La recette native avec fournisseur factice couvre dix reprises de la même
requête, un seul enfant et une seule réponse corrélée. Elle couvre aussi
l'annulation d'une mission bloquée et la récupération du résultat durable
après redémarrage du daemon isolé.

L'interop R4 utilise le vrai serveur Node/Effect, le registre d'authentification,
le toolkit MCP, ses handlers et le client Rust. Deux conversations distinctes
partagent le même identifiant d'instance fournisseur. Chaque conversation
possède sa session fournisseur et son autorisation privées. Les assertions
valident les headers HTTP, les DELETE de sessions, le refus 404 d'une session
fermée, puis le refus 401 de la conversation A révoquée. La conversation B
reste utilisable et ferme sa nouvelle session. Les projections de conversations
restent des données de fixture. Aucun processus modèle n'est lancé par cette
recette. Le partage du processus fournisseur est couvert séparément par les
tests des adaptateurs.

Rapports de référence :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-tests-natifs-r2-complet.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-recette-reelle-r1.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-tests-identite-r2.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-tests-identite-r3.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-tests-identite-r4.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-revue-identite-r2.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-revue-native-r2.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-revue-native-r3.md

Les échecs des rondes antérieures restent archivés. Ils ne sont pas effacés par
les résultats finaux. Le run Rust complet de référence a corrigé l'environnement
de test. Les derniers ciblés ont validé les corrections de format et de lint.
L'interop R4 a validé la correction de l'observateur des headers.

**Recette réelle GLM R2**

Verdict indépendant : APPROVE. Trois tests secs passent. La recette réelle
passe en **55,19 secondes**. Deux tests restent ignorés dans le run sec ; la
recette réelle a ensuite été exécutée explicitement avec son nom exact.

Un seul appel de délégation produit une tâche, un enfant et un résultat
corrélé uniques. Le modèle demandé est `glm-5.3`. Les six événements fournisseur
annoncent aussi `glm-5.3`. L'effort conservé vaut `null`, car le catalogue
n'annonce pas d'efforts pour ce modèle. La recette confirme le nettoyage des
processus qu'elle possède.

L'enfant GLM est réel. Le parent est une fixture native Codex ; aucun modèle
Codex réel n'a été exécuté. Aucun T3 ne participe au chemin de l'enfant.
Le fait que le testeur soit lancé depuis T3 est hors de ce chemin testé.

La route d'authentification utilisateur existante a été réutilisée en mémoire.
Aucune valeur de clé n'est publiée dans ce reçu. La commande, ses réglages et
le registre utilisateur n'ont pas été modifiés par la recette. Le modèle réel
ne prouve pas le fonctionnement des anciennes sessions encore chargées.

Cette validation GLM couvre la posture d'inspection `discovery`. Claude/GLM
n'ont pas de protocole natif de développement confiné. La posture `development`
reste refusée pour ces fournisseurs. Le catalogue annonce
`development_protocol_unavailable` ; l'appel peut rendre le message du registre
« posture développement réservée à Codex app-server ». Aucun droit global MCP
ni nouvelle permission humaine n'est déduit de cette recette.

Rapport et reçu archivés :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-recette-reelle-r2.md
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/glm-recette-reelle-r2-receipt.json

La preuve privée originale reste conservée :
/private/tmp/ng148-bfcc36f958/receipt.json

**Paquet T3 et installation sur disque**

Build privé avec Node 24.13.1 officiel et pnpm 11.10.0. Aucun avertissement de
version Node incompatible dans le build final. Le vérificateur passe avec
**2576 contrôles réussis, aucun échec**. Il compare 27711 fichiers source suivis,
les pointeurs exacts de deux sous-modules, 2377 fichiers compilés frais et les
118 fichiers natifs de l'application, de sa copie et du ZIP. Seuls les champs
de version des quatre manifests de livraison ont changé dans la source privée.

La signature ad hoc couvre 38 objets et conserve exactement leurs droits
existants. Les contrôles `codesign --verify --strict --deep` du stage et de sa
copie passent. L'identité reste `com.t3tools.t3code`. Le nom et l'exécutable
internes restent `T3 Code (Alpha)`.

Le smoke test SQLite sous Electron passe : écriture, fermeture, réouverture,
lecture persistante et `quick_check=ok`. Il utilise Electron 44.4.5, son Node
24.21.0 et SQLite 3.53.4. La base est privée. Aucun GUI, serveur T3 ou modèle
n'a été démarré par ce contrôle.

| Élément | Valeur confirmée |
| --- | --- |
| Version du paquet et de l'application installée | `0.0.45-local.148` |
| Commit embarqué | `33f6d04e1164` |
| SHA256 du ZIP signé | `1b3feb42bbfba8c162ca2c4e89d9ab10f5d5250c89152b17ca51374bcb05351c` |
| SHA256 de l'archive app.asar installée | `60d306764354e331156d634293afaa9d749cc92b6818cc8a4f98892cc2370486` |

Archive installée :
/Applications/T3 Code (Local).app/Contents/Resources/app.asar

ZIP signé :
/Users/moi/.cache/t3-final148.XnkCUZ/artifacts/T3-Code-0.0.45-local.148-arm64.zip

Copie de livraison contrôlée :
/Users/moi/.cache/t3-final148.XnkCUZ/staging/T3 Code (Local).app

Application installée par le principal :
/Applications/T3 Code (Local).app

Le principal confirme le contrôle de signature strict de l'application
installée, avec code retour 0. Il confirme aussi la comparaison complète de
l'arbre installé avec la copie contrôlée, par `diff -qr`, avec code retour 0.
Le reçu de build portait initialement `installed=false`. Il décrit le build
privé avant ce remplacement. L'installation sur disque est le checkpoint
ultérieur transmis par le principal.

Reçus et inventaire détaillé :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/t3-build-receipt.json
- /Users/moi/.cache/t3-final148.XnkCUZ/build-receipt.json
- /Users/moi/.cache/t3-final148.XnkCUZ/verification.json

Les avertissements du build restent visibles : fichiers web au-dessus du seuil
de taille, import Linux `x11` externalisé et `import.meta` remplacé dans une
sortie CommonJS. Ils ne sont pas présentés comme corrigés. Le ZIP, son blockmap
et la copie initiaux non signés ont été retirés après validation du paquet signé.
Le blockmap initial ne correspondait pas au ZIP signé. Aucun blockmap de mise
à jour n'est livré pour ce ZIP local. Les logs du défaut de signature restent
conservés. Le résultat précédent du vérificateur reste aussi archivé ; sa correction
traite les deux références de sous-modules comme des pointeurs Git, sans
exclusion large de dossiers source.

Journal du build avec ses avertissements :
/Users/moi/.cache/t3-final148.XnkCUZ/build-desktop.log

**Binaire Bridget construit et installé**

Le build release privé, la signature ad hoc, le contrôle strict de signature
et le contrôle de version passent avec code retour 0. Le clone est au commit
Bridget du tableau. Les sources restent propres et le fichier de verrouillage
des dépendances est inchangé. Le build utilise Rust et Cargo 1.92.0.

| Champ | Valeur confirmée |
| --- | --- |
| Version Bridget | `0.1.3` |
| Identifiant de build embarqué | `347d788510b4` |
| SHA256 du binaire signé, privé puis installé | `8104a63b3e16784280a2da1aeef0a0632158f31b0bd08312db3f17ec8bd9f56e` |
| Build, signature, vérification et version | Codes retour 0 |
| Installation, signature, hash et version installés | Contrôles identiques, codes retour 0 |

Binaire privé contrôlé :
/Users/moi/.cache/bridget-final148.wDyecn/target/release/bridget

Binaire installé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget

Le principal a préparé une copie voisine, vérifié sa signature et son hash,
puis effectué le remplacement atomique avec `os.replace`. La sauvegarde du
binaire précédent a été comparée à l'original avant le remplacement.

Reçu archivé et original du build :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/bridget-build-receipt.json
- /Users/moi/.cache/bridget-final148.wDyecn/receipt.json

Le reçu du build porte `no_install=true`. Il décrit le build privé avant
l'installation ultérieure effectuée et vérifiée par le principal.

Les LaunchAgents ont été contrôlés en lecture seule. Ils pointent vers le
binaire installé :

- /Users/moi/Library/LaunchAgents/com.bridget.daemon.plist
- /Users/moi/Library/LaunchAgents/com.bridget.t3.plist

Le lien CLI pointe vers ce même binaire :
/Users/moi/.local/bin/bridget

Les liens des skills pointent vers les fichiers à jour de main.

**Sauvegardes et processus actifs**

Sauvegarde de l'application précédente, version locale `.20261009.7` :
/Users/moi/.cache/bridget-install148.KlIN2m/T3 Code (Local).app.rollback

Le principal confirme sa signature valide et son arbre identique à celui de
l'application précédente.

Sauvegarde du binaire Bridget précédent :
/Users/moi/.cache/bridget-install148.KlIN2m/bridget.rollback

Signature valide. SHA256 confirmé :
`0db040b8dc7b1a0892d1357614b326c68abb0ea8ef2afa0238ddb5f5d464d8b2`.
Cette empreinte décrit la sauvegarde précédente, pas le nouveau binaire 148.

Aucun redémarrage n'a été effectué lors du remplacement de l'application et
du binaire Bridget. Après les deux installations, le principal confirme que
les quatre PID et leurs dates de démarrage restent inchangés :

| Processus | PID conservé |
| --- | --- |
| Serveur T3 | 58468 |
| Codex | 57109 |
| Daemon Bridget | 58394 |
| Serveur de pont T3 Bridget | 58396 |

Les sources intégrées et les fichiers installés sont distincts du code chargé
par ces processus. Aucun fonctionnement actuel de la version 148 n'est déduit
de leur présence. Une relance explicite de T3 et des services Bridget sera
nécessaire pour charger les nouveaux fichiers. Aucune activation différée
n'est programmée. Aucun réglage utilisateur, aucune authentification et aucune
base de production n'ont été modifiés dans cette livraison.

**Nettoyage confirmé et preuves conservées**

Le principal a retiré les trois worktrees propres et leurs branches déjà
ancêtres du code intégré :

| Dépôt | Branche de session retirée |
| --- | --- |
| /Users/moi/Nextcloud/10.Scripts/64.bridget | `session-147-t3-v2` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget | `session-148-identite-delegation` |
| /Users/moi/11.Repositories/t3code-local | `session-148-identite-delegation` |

Bridget conserve seulement son worktree principal. Les autres worktrees et
branches T3 étrangers sont intacts. Les anciens builds debug et dépendances
des worktrees retirés ont été supprimés.

La copie temporaire de l'ancienne application sur /Applications a été retirée
après comparaison avec sa sauvegarde identique. Le dossier temporaire Node26
suivant a été retiré :
/private/tmp/b148.E39YGn

La copie non signée suivante a été retirée, ainsi que le ZIP et le blockmap
initiaux non signés :
/Users/moi/.cache/t3-final148.XnkCUZ/staging.unsigned-original

Les logs et reçus des défauts restent conservés. Le paquet final signé et le
cache final T3 restent conservés :
/Users/moi/.cache/t3-final148.XnkCUZ

Les preuves archivées, les deux sauvegardes rollback 148, la sauvegarde de
l'application `.20261009.7` et la sauvegarde 146 sont conservées. Cette dernière
reste dans :
/Users/moi/.cache/bridget-install147.VTg5jQ

Les chemins de worktrees présents dans les rapports restent des chemins de
preuve historiques après leur retrait. Aucun nouveau test, build, commit,
remplacement de fichier produit ou redémarrage n'est exécuté par la rédaction
de ce reçu.
