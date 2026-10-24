# Reprise après déconnexion du démon — 24 octobre 2026

Défaut confirmé par lecture : le client décodait Disconnect comme une réponse
normale, puis la classait daemon_protocol. Le protocole décrit cette trame
comme l’arrêt du démon. Le pont T3 de Bridget marque déjà ce lien inactif.
Les incidents signalés coïncident avec l’activation du maître ; leurs deux
tâches sont ensuite result_available, sans erreur. La trame réelle de ces
incidents n’a pas été capturée : cette corrélation ne constitue pas cette preuve.

Correction : traiter Disconnect comme une interruption du transport et fermer
le socket. Le code fourni par l’appelant reste intact : daemon_unreachable
pour une lecture, outcome_unknown pour une mutation. Le conseil conserve les
identifiants et n’affirme ni échec métier, ni annulation, ni absence de tâche.
Aucun rejeu ajouté aux lectures. La récupération d’admission existante reste
bornée et emploie exactement la même clé et la même enveloppe.

Le conseil de rattachement T3 distingue lien absent/inactif ou transport
indisponible des preuves invalides. Une nouvelle attestation ne prouve pas la
reconnexion du lien. Aucun droit, secret ni identité n’est assoupli.

Référence T3 : McpSessionRegistry.ts distingue la durée de vie des justificatifs
et leur révocation explicite ; aucune gestion du protocole Bridget dans ce
module. Le mécanisme directement réutilisé est celui de t3code.rs : Disconnect
marque le lien inactif. CodexSessionRuntime.ts appelle config/mcpServer/reload
avant sendTurn quand un MCP est configuré. Cela ne prouve pas le rechargement
d’un client déjà en cours de tour. Aucun redémarrage T3 n’est requis par le code.

Validation ciblée : 21 tests distincts réussis (22 exécutions). Les deux nouveaux
tests Disconnect passent par des sockets Unix ; un test couvre aussi le socket
fermé et l’absence d’un second envoi. Les filtres binding_ (11),
admission_recovery171_tests (2), spec172_transport_tests (6, dont un nouveau
Disconnect déjà compté) et confidentialité du diagnostic (1) passent.
La compilation initiale a signalé un import de trait manquant dans le nouveau
test ; corrigé avant ces résultats. Aucun test complet ni charge applicative.
Les guides passent le contrôle documentaire : 27 pages, 422 liens, 11 paires.

Livraison détaillée ci-dessous ; distinguer fichier installé et client déjà ouvert.

Revue indépendante : une lacune préexistante a été relevée sur Continue,
Respond, Cancel, Ack et AutoDelegate v1. Leur envoi passe désormais par
send_then_wait : une perte de réponse après écriture reste outcome_unknown,
sans reprise automatique ajoutée. Un test socket Cancel/Ack couvre ce point.
Le conseil spécifique aux tâches a été déplacé dans delegation_mcp ; le
transport commun conserve un conseil général. La revue ne vaut pas approbation
de ces ajustements ultérieurs ; ils sont vérifiés par ROOT et les tests ciblés.

Après ajustement, les 3 tests Disconnect et les 7 tests de transport attesté
passent, dont Cancel et Ack sans nouvelle émission. Les tests de preuve et de
récupération déjà passés ne sont pas relancés : ces fonctions sont inchangées.
Total : 22 tests distincts réussis sur le périmètre, sans suite complète.

## Livraison des clients

Source edeee50f72fff295aa9975c38f32d3c6767dd8df poussée sur main. Deux
compilations release optimisées réussies : Mac 193,627 s ; Linux 291,563 s.
Le client Mac a été signé et la signature vérifiée. Le client Linux est installé
dans une version distincte, et le lien utilisateur a été remplacé atomiquement.
Les anciennes versions utilisées par les processus restent disponibles.

Empreintes SHA-256 des exécutables installés :
- Mac : ddd5439d1fdd387e483cd76954f5669dcf7be97b22873d35c79c3a9693153a86.
- Linux : 59b82abd7345d2ca5c000398f4df928e45df10fb8e22f7efd1920b4633665c4e.

Chaque exécutable installé a reçu Disconnect depuis un socket Unix privé,
après deux trames du client public de catalogue. Le message rend désormais
la coupure et le conseil général. Ce contrôle ne lance aucun modèle et ne
contacte pas le démon réel. Les oracles Rust distincts couvrent le conseil
spécifique aux tâches, l’attente, le statut, Cancel et Ack. Ce test de binaire
ne constitue pas une recette d’exécution distante complète.

Aucun service ni modèle redémarré : maître, pont, processus T3 principaux et
worker conservés. Un processus de rendu T3 a disparu pendant la compilation,
sans signal envoyé par cette livraison ; ne pas prétendre que tous les
processus de l’application sont restés identiques. Les deux façades MCP des
parents incidents restent ouvertes avec leur ancien code. Le correctif client
prendra effet à leur rechargement ; installer le fichier ne prouve pas cette
activation. Le worker conserve également son exécutable chargé.

Entretien appelé avant/après les builds, puis installed sur les deux hôtes.
Le passage installed Linux refuse le retrait du staging car l’observation des
processus est incomplète ; aucun contournement. Le bilan de build annonce
6,77 Gio de caches reconnus, sous le budget commun de 10 Gio. Mac : 0,76 Gio.
Cette limite de nettoyage n’empêche pas l’installation du client vérifié.

Avis de livraison envoyés par Bridget aux deux parents, avec la limite de
rechargement explicite. Remise en vol constatée ; lecture non confirmée.
