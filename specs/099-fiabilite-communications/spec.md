# Spécification 099 — Fiabilité et identité des communications

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-1).

## Pourquoi

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-4).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-5).

## Preuves initiales

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/099-fiabilite-communications/spec.md#extrait-6).
