# Implémentation148

Statut : développement validé. Livraison en cours ; aucun processus148 activé.

Le daemon porte une saga native durable. Le registre fixe le fournisseur, le
modèle, l'effort et la posture avant lancement. La flotte gère les processus.
La mission et le résultat utilisent les remises idempotentes existantes.
Pour un enfant natif neuf, les faits durables d'identité et de Git sont joints
à sa mission dans un seul tour fournisseur. L'enveloppe et sa corrélation restent
inchangées. Une relance conserve la carte de reprise séparée déjà existante.
L'état durable distingue remise, travail, attente des enfants, résultat,
annulation et nettoyage. La définition du fournisseur est figée à l'acceptation.

Les droits sont natifs. Les demandes MCP ne choisissent jamais leur parent.
Un grant humain est inaccessible depuis le MCP. Sa révocation ferme les nouvelles
demandes et tout retour implicite aux droits hérités. La reprise d'une instance
gérée exige une preuve de propriété vivante. Les lectures restent sans mutation.

Le connecteur T3 fournit une preuve de session propre à chaque montage MCP.
Bridget vérifie cette preuve à chaque opération, puis son inscription vivante
au daemon. Une preuve invalide ferme l'appel avant la résolution PID. Les
restrictions des enfants internes restent conservées. Le transport stateful
capture et renvoie son identifiant de session HTTP, puis termine cette session.
Le moteur natif ne consomme aucun outil d'orchestration T3.

## Validation en cours

| Exigences | Scénarios associés |
|---|---|
| FR001–FR006, SC001/SC004 | Résolveur identité, sessions T3, vrai HTTP/Rust, preuve env partielle, révocation. |
| FR007–FR014, SC002/SC003/SC005 | Catalogue natif, sélection exacte, grants, rejeu10 et refus fermés. |
| FR015–FR019, SC006 | Capture avant ACK, attente des enfants, résultat stable, annulation et nettoyage. |
| FR020–FR024 | Mission seule, exclusion des credentials enfant, guides, ressources bornées, production intacte. |
| FR025–FR026 | Recette daemon/superviseur/wrapper sans serveur T3. |
| SC007 | Tests ciblés, régressions, revues GLM et builds à finir avant livraison. |

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

## Limites connues

La posture d'écriture Claude/GLM est refusée par le registre actuel. Le catalogue
annonce `development_protocol_unavailable`. La posture d'inspection reste
disponible sous les droits natifs du parent. Aucun confinement fictif n'est promis.
Les anciens processus continuent à utiliser leur code déjà chargé.
