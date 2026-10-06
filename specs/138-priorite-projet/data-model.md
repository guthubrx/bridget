# Modèle 138 — Faits vivants et mandat borné

## Fait de projet de communication

Fait attaché à une connexion et son instance déjà attestées. Champs minimaux:
hôte attesté, racine canonique commune, origine de preuve T3 ou wrapper.
La clé de comparaison dérive de l'hôte et de la racine. Aucun identifiant de
registre runtime retiré n'est accepté comme autorité.

L'absence est représentée explicitement comme inconnue. Un label domain ne
participe pas à la clé. Un message ne peut pas fournir son projet pour remplacer
ce fait. Une connexion auxiliaire hérite de son parent attesté.

Cycle: annonce vérifiée → fait connu ; absence/conflit → inconnu ; reconnexion
→ nouvelle annonce ; fin de connexion → retrait du fait. La présence durable
historique ne prouve pas à elle seule qu'une connexion est encore active.

## Portée d'annuaire

Valeurs: same_project et global. Défaut de communication: same_project.
Résultat: faits d'agents existants, appartenance connue/inconnue, warning éventuel.
ListAgents conserve sa projection globale de diagnostic.

Un émetteur inconnu obtient zéro candidat same_project. Aucun busy local
n'est remplacé automatiquement par un candidat extérieur.

## Motif interprojets

cross_project_reason est un champ optionnel dans BridgetMessage et ThreadRequest.
Chaîne trimée: 1 à 512 octets UTF-8. Refuser null explicite, NUL et contrôles.
La valeur est un choix déclaré. Elle n'est jamais déduite du corps du message.

Absence: canon historique exact. Présence: suffixe canonique versionné qui couvre
la valeur entière. Sous une même clé d'opération, un motif différent est un
conflit. Les résultats et avertissements font partie du résultat idempotent.

## Mandat de fil

Chaque create/post mixte porte son cross_project_reason. Aucune nouvelle colonne
ou table. Les membres immuables de discussion_members forment l'audience.
Le caller peut réutiliser le motif d'un mandat explicitement configuré pour elle.
Un motif présent dans l'historique ne crée aucun consentement implicite.

Le contrôle porte sur tous les membres lecteurs. ThreadNotify décide uniquement
de la sollicitation. notify=[] ne rend pas le corps privé.

Lecture/histoire/reçus ne changent aucun mandat et ne rejouent aucune notification.
Un refus ou un rejeu ne crée pas de mandat supplémentaire. Chaque canon nouveau
couvre son motif ; None conserve la forme historique.

## Demande suivie et réponse

La demande suivie existante conserve l'enveloppe autorisée. Sa réponse peut
réutiliser le motif si in_reply_to identifie une demande valide, si l'émetteur
de la réponse est la cible initiale et si sa cible est l'émetteur initial.
Un simple identifiant fourni ou un texte « réponse » ne suffit pas.

## Mandat Agent Loop

La racine attestée du run est distincte de run.domain. Une attribution explicite
porte un motif et les agents/rôles visés. Le run conserve le mandat pour les
rappels de ces attributions. Un changement de cible n'hérite pas de ce motif.
Les runs legacy sans preuve sont inconnus ; ils ne recrutent pas automatiquement.
Les missions existantes ne sont pas converties ni réaffectées.

Le client de fond porte la racine explicite sur sa propre connexion négociée.
Le daemon valide ce contexte sans créer d'agent et sans emprunter d'identité T3.
Un auxiliaire conserve le fait parent. Un run ancien sans racine reste inconnu.
L'outbox fige racine, motif et destinataire avant la première tentative ; un
rejeu ne les reprend pas depuis la configuration courante du run.
Le résultat publié, son result_path et son archive restent cohérents sous le
verrou commun existant. Une publication concurrente ne devient pas une archive
obsolète après une réservation CAS. T020 couvre cet interleave.

## Résultat émetteur

Warning structuré: code project_unknown ou cross_project, parties connues,
motif lorsqu'il est requis. Il est établi avant dépôt/notification et inclus
dans la réponse caller. Le corps remis et le résultat métier existant restent
distincts. Aucun warning n'est adressé comme un message au destinataire.

## Consigne injectée et résultat de contrôle durable

SteerCurrent.message utilise le fait de connexion et la garde partagée avant
injection. Aucun projet ni mandat n'est déduit du texte de la consigne.
La table existante execution_control_commands conserve project_warnings dans
une seule colonne TEXT JSON NOT NULL DEFAULT '[]'. La migration compatible
suit init_schema et les helpers d'introspection existants. Les anciennes lignes
produisent []. Aucune table, dépendance ou service supplémentaire.

Cette colonne est une exception explicite à l'absence de colonne nouvelle.
canonical_bytes reste immuable et comparé exactement. refusal_reason garde sa
responsabilité de refus ; aucun warning n'y est caché. Un résultat accepté est
restitué avant toute garde mutable, après changement de faits ou redémarrage,
sans réinjecter le contenu. Interrupt sans message et canon legacy restent exacts.
