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

Livraison : en cours, aucun nouveau client activé à ce stade.

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
