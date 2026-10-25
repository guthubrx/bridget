# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/implementation.md](../../../../../../specs/148-identite-delegation/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le connecteur T3 fournit une preuve de session propre à chaque montage MCP.
Bridget vérifie cette preuve à chaque opération, puis son inscription vivante
au daemon. Une preuve invalide ferme l'appel avant la résolution PID. Les
restrictions des enfants internes restent conservées. Le transport stateful
capture et renvoie son identifiant de session HTTP, puis termine cette session.
Le moteur natif ne consomme aucun outil d'orchestration T3.

<a id="extrait-2"></a>

## Extrait 2

| Exigences | Scénarios associés |
|---|---|
| FR001–FR006, SC001/SC004 | Résolveur identité, sessions T3, vrai HTTP/Rust, preuve env partielle, révocation. |
| FR007–FR014, SC002/SC003/SC005 | Catalogue natif, sélection exacte, grants, rejeu10 et refus fermés. |
| FR015–FR019, SC006 | Capture avant ACK, attente des enfants, résultat stable, annulation et nettoyage. |
| FR020–FR024 | Mission seule, exclusion des credentials enfant, guides, ressources bornées, production intacte. |
| FR025–FR026 | Recette daemon/superviseur/wrapper sans serveur T3. |
| SC007 | Tests ciblés, régressions, revues GLM et deux builds finaux réussis. |

<a id="extrait-3"></a>

## Extrait 3

Les rapports indépendants sont conservés dans le dossier validation de cette
session. La première revue GLM a trouvé le header HTTP manquant et le défaut
de nettoyage ; le principal les a corrigés. La revue identité R2 et la revue
moteur R2 approuvent ces réponses. La recette réseau R4 réussit ses six tests,
dont la révocation de A sans perte de l'identité de B. La recette native R2
réussit le rejeu, le résultat, l'annulation et le redémarrage. Les régressions
bibliothèques totalisent 1496 tests réussis et 15 ignorés. La revue du premier
tour R3 approuve le dernier changement du wrapper.
Les recettes synthétiques ne sont pas annoncées comme preuve d'un
fournisseur réel. La validation finale réussit 45 tests ciblés et deux recettes
E2E. Le format et Clippy réussissent avec l'exception préexistante explicite
`too_many_arguments`.
La tentative GLM réelle R1 échoue avant le démarrage du modèle : le trousseau
ne se résout pas sous le HOME isolé. Le rapport indépendant conserve les sondes
HOME réel RC0 / HOME isolé RC44. La correction de la recette est gelée :
authentification existante en mémoire seulement, nettoyage sans panique et
sélection exacte du test et chemin CLI borné. Trois tests secs réussissent :
préconditions d'authentification, fin du flux MCP et nettoyage pendant un échec.
La revue R2 approuve ces corrections. La recette réelle R2 réussit en 55,19 s :
un appel, un enfant GLM 5.3, une tâche durable et un résultat corrélé unique.
Le modèle annoncé est exactement GLM 5.3 et le nettoyage est confirmé.
Le parent est une fixture Codex, sans modèle Codex réel. Aucun serveur T3
n'intervient dans le parcours natif. Le rapport et le reçu R2 sont archivés.

<a id="extrait-4"></a>

## Extrait 4

Bridget est construit depuis le commit fusionné `347d788510b4` et installé avec
sa signature valide. T3 `33f6d04e1164` est installé en `0.0.45-local.148` après
2576 contrôles du paquet et un test SQLite sous Electron. Les sources sont
poussées. Le reçu final précise les empreintes, sauvegardes et processus conservés.
