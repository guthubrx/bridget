# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/spec.md](../../../../../../specs/099-fiabilite-communications/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Branche** : session-099-fiabilite-communications
**Date** : 2026-09-16
**Statut** : Implemented — 16/16 tâches initiales + 6 tâches de convergence (reprise du 2026-09-16) ; recette complète `cargo test --workspace` exigée verte avant commit et déploiement.
**Demande** : « $my-specify-all corrige les problèmes que tu as trouvés ».
**Dépendances** : 089 (messagerie et garanties), 094 (clients MCP), 097 (sessions natives), 098 (pont t3code).
**Tests** : 26/26 nouvelles régressions passent (25 daemon, 1 protocole) ; baseline : 39 tests cœur et 13 tests t3code avant correction.

<a id="extrait-2"></a>

## Extrait 2

La revue a reproduit trois défauts opérationnels et une attribution indue :
un destinataire lent immobilise les autres communications ; un client auxiliaire
peut emprunter une identité qu'il connaît ; un travail t3code annulé peut démarrer
après l'annulation ; une réponse terminée est oubliée lorsque son destinataire est
déconnecté. La lecture montre aussi une sauvegarde intermédiaire omettant des
attentes et une observation coupant silencieusement les textes longs.

<a id="extrait-3"></a>

## Extrait 3

Étant donné un fil t3code occupé, un travail reçu reste en attente. Si l'émetteur
l'annule avant son démarrage, si son échéance est passée, ou si la connexion
d'autorité disparaît, le travail n'est pas démarré ultérieurement en silence.
Le pont continue à traiter les commandes de contrôle pendant cette attente.
Test : faux fournisseur occupé, annulation attestée avant libération, aucune
commande de démarrage pour cette demande ; mêmes essais avec expiration et
déconnexion. Une autre demande valide peut être traitée ensuite.

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

Une annulation après acceptation effective d'un tour par t3code n'est pas une
garantie d'arrêt fournisseur ; cette course reste visible et documentée.
Aucun changement de permissions fournisseur, aucun nouvel outil MCP, aucune
migration destructive, aucun déploiement et aucun commit automatique.

<a id="extrait-6"></a>

## Extrait 6

- Instance isolée /tmp/bg-audit-2jgfzw9l : annuaire 0,21 ms, attente bloquée
  1 201,14 ms, reprise 1,33 ms après fermeture du seul destinataire.
- Même instance : client MCP auxiliaire déclaré sous l'identité d'un autre agent,
  inscription et envoi acquittés, destinataire reçoit l'identité empruntée.
- Instance isolée /tmp/bg-t3-audit-dpekg5ti : RequestCancelled confirmé puis
  démarrage fournisseur constaté ; réponse oubliée (pending vide), destinataire
  reconnecté sans réponse et demande audit-lost-842d2f11 encore open.
- Scripts reproducteurs conservés dans
  /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-09-16/session-2026-09-16-bridget-global-01.
