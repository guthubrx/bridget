# Spécification 099 — Fiabilité et identité des communications

**Branche** : session-099-fiabilite-communications
**Date** : 2026-09-16
**Statut** : Implemented — 16/16 tâches initiales + 6 tâches de convergence (reprise du 2026-09-16) ; recette complète `cargo test --workspace` exigée verte avant commit et déploiement.
**Demande** : « $my-specify-all corrige les problèmes que tu as trouvés ».
**Dépendances** : 089 (messagerie et garanties), 094 (clients MCP), 097 (sessions natives), 098 (pont t3code).
**Tests** : 26/26 nouvelles régressions passent (25 daemon, 1 protocole) ; baseline : 39 tests cœur et 13 tests t3code avant correction.

## Pourquoi

La revue a reproduit trois défauts opérationnels et une attribution indue :
un destinataire lent immobilise les autres communications ; un client auxiliaire
peut emprunter une identité qu'il connaît ; un travail t3code annulé peut démarrer
après l'annulation ; une réponse terminée est oubliée lorsque son destinataire est
déconnecté. La lecture montre aussi une sauvegarde intermédiaire omettant des
attentes et une observation coupant silencieusement les textes longs.

Le périmètre est la correction de ces défauts, pas une refonte du produit ni la
suppression d'adaptateurs, du journal, du suivi durable ou de la fédération.

## Scénarios utilisateur et tests

### US1 — Un destinataire défaillant n'immobilise pas les autres (P1)

Étant donné un agent connecté qui ne lit plus, quand un message lui est envoyé,
les autres utilisateurs consultent l'annuaire et échangent toujours. L'expéditeur
obtient un échec ou une issue indéterminée explicite si la remise ne peut aboutir.
Une réponse très rapide ne doit pas être perdue entre la remise et son suivi.
Test : vrais flux isolés, destinataire saturé, annuaire et échange témoin concurrents,
puis rupture du flux ; aucun succès de remise fictif.

### US2 — Une identité connue ne vaut pas autorisation (P1)

Étant donné un agent vivant, un autre client connaissant ses identifiants ne peut
ni envoyer en son nom ni modifier son nom ni annuler ses demandes. Le client
auxiliaire légitimement rattaché continue à fonctionner, y compris après une
reconnexion prise en charge. Les anciens clients incompatibles reçoivent un refus
clair plutôt qu'un accès implicite.
Test : tentative sans preuve, preuve incorrecte, preuve d'un autre agent,
auxiliaire valide et preuve périmée après nouvelle incarnation.

### US3 — Ne pas démarrer un travail déjà annulé ou périmé (P1)

Étant donné un fil t3code occupé, un travail reçu reste en attente. Si l'émetteur
l'annule avant son démarrage, si son échéance est passée, ou si la connexion
d'autorité disparaît, le travail n'est pas démarré ultérieurement en silence.
Le pont continue à traiter les commandes de contrôle pendant cette attente.
Test : faux fournisseur occupé, annulation attestée avant libération, aucune
commande de démarrage pour cette demande ; mêmes essais avec expiration et
déconnexion. Une autre demande valide peut être traitée ensuite.

### US4 — Une réponse prête reste récupérable (P1)

Étant donné une demande suivie dont la réponse est prête, la déconnexion de son
destinataire ne fait pas oublier cette réponse. Après retour du destinataire dans
la période de validité, elle est remise et clôt la bonne demande. Un redémarrage
du pont ne perd ni cette réponse ni les autres attentes du même fil.
Test : destinataire absent à la fin du tour, reprise et redémarrage du pont,
réponse liée reçue ; plusieurs attentes survivent à une sauvegarde intermédiaire.
Aucune seconde exécution fournisseur ne sert à refaire une réponse déjà produite.

### US5 — Une observation complète ou explicitement incomplète (P2)

Une sortie de plus de 4 096 caractères n'est pas présentée comme complète si elle
a été tronquée. Les portions conservées restent dans l'ordre et sans doublon
au redémarrage. Une saturation du journal ne marque pas silencieusement comme
observé un contenu qui n'a pas été écrit.
Test : texte Unicode long, écriture refusée, reprise ; contenu intégral ou lacune
explicite, sans succès mensonger.

## Exigences fonctionnelles

- FR-09901 : isoler l'effet d'une remise classique bloquée sur les autres agents ;
  borner l'attente d'écriture et de verrou de sortie à une seconde.
- FR-09902 : ne pas confirmer comme réussie une remise dont l'écriture a échoué ;
  garder une issue honnête si une remise partielle ne peut être prouvée.
- FR-09903 : conserver la corrélation des réponses immédiates et les contrôles
  de doublons, de sauts et de débit pendant la correction.
- FR-09904 : exiger une preuve de rattachement pour les clients auxiliaires ;
  les seuls identifiants, le type déclaré ou l'identité du compte ne suffisent pas.
- FR-09905 : révoquer cette preuve à la fin de l'incarnation autorisée ;
  ne l'exposer ni dans l'annuaire, ni dans les journaux, ni dans les erreurs.
- FR-09906 : maintenir les clients légitimes et nommer les incompatibilités ;
  documenter qu'il ne s'agit pas d'un cloisonnement entre processus hostiles
  disposant du même compte système.
- FR-09907 : annulation avant démarrage, expiration et perte du daemon empêchent
  un démarrage différé ; traiter le contrôle sans attendre la fin du tour occupé.
- FR-09908 : conserver durablement les réponses non confirmées et permettre leur
  reprise, sans relancer le travail fournisseur ni fermer la mauvaise demande.
- FR-09909 : une sauvegarde de décision ne doit omettre aucune autre attente ;
  les anciennes données de pont restent lisibles.
- FR-09910 : conserver le contenu observable long ou annoncer sa lacune ; ne pas
  avancer un repère d'observation après un échec d'écriture.
- FR-09911 : utiliser les protections et formats existants quand ils couvrent le
  besoin ; aucune nouvelle dépendance ni service.
- FR-09912 : tests et recettes utilisent uniquement des états explicitement
  isolés ; aucune modification de l'installation historique ou de t3code.

## Entités

Agent et incarnation ; connexion auxiliaire autorisée ; demande suivie ; remise
en attente ; réponse prête non confirmée ; repère d'observation du fil.

## Critères de succès

- SC-09901 : pendant le blocage provoqué, dix consultations d'annuaire et un
  échange témoin aboutissent chacun en moins d'une seconde en recette isolée ;
  la remise défaillante obtient une issue en moins de deux secondes.
- SC-09902 : toutes les tentatives d'emprunt d'identité couvertes sont refusées ;
  le scénario auxiliaire valide reste utilisable.
- SC-09903 : zéro démarrage fournisseur pour un travail annulé avant démarrage,
  périmé ou laissé en attente lors de la perte du daemon.
- SC-09904 : après reconnexion et redémarrage du pont, une réponse déjà produite
  rejoint sa demande sans réexécution du travail ; aucune autre attente perdue.
- SC-09905 : une sortie de 10 000 caractères Unicode est observable intégralement
  ou accompagnée d'une lacune explicite, jamais coupée silencieusement.
- SC-09906 : nouveaux tests de régression, tests existants pertinents, formatage
  et analyse statique réussissent ; toute impossibilité de recette est nommée.

## Hypothèses et limites

L'accès au compte système et au répertoire privé reste une frontière de confiance :
un processus malveillant du même compte peut lire la mémoire ou les fichiers selon
les protections du système. La correction empêche l'usurpation par simple
déclaration de protocole, pas toutes les attaques locales.

Une annulation après acceptation effective d'un tour par t3code n'est pas une
garantie d'arrêt fournisseur ; cette course reste visible et documentée.
Aucun changement de permissions fournisseur, aucun nouvel outil MCP, aucune
migration destructive, aucun déploiement et aucun commit automatique.

## Preuves initiales

- Instance isolée /tmp/bg-audit-2jgfzw9l : annuaire 0,21 ms, attente bloquée
  1 201,14 ms, reprise 1,33 ms après fermeture du seul destinataire.
- Même instance : client MCP auxiliaire déclaré sous l'identité d'un autre agent,
  inscription et envoi acquittés, destinataire reçoit l'identité empruntée.
- Instance isolée /tmp/bg-t3-audit-dpekg5ti : RequestCancelled confirmé puis
  démarrage fournisseur constaté ; réponse oubliée (pending vide), destinataire
  reconnecté sans réponse et demande audit-lost-842d2f11 encore open.
- Scripts reproducteurs conservés dans
  /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-09-16/session-2026-09-16-bridget-global-01.
