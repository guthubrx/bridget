# Session 101 — Abonnements utilisables depuis T3

Date : 2026-09-16. Statut : Implemented. Branche : session-101-abonnements-t3.
Tests : 31/31 ciblés101 et régressions vertes ; détail dans implementation.md.
Recette réelle T3 réussie : fin observée15:02:04Z, remise15:02:05Z,
présence dans le fil15:02:08Z ; T011 close. Diff non commité, adoption autorisée.
Dépendances : 098 (adaptateur T3), 099 (identité attestée), 100 (journal et observations).

## Contexte et périmètre

Bridget est le produit de communication et d'observation inter-agents. T3 est
un adaptateur, pas son centre. La session 100 apporte des abonnements mais
l'usage réel « préviens-moi ici quand Horizon-3D termine » échoue : identité
appelante non reconnue et événements T3 non raccordés aux observations.
La demande validée est de rendre ce parcours utilisable, sans formulaire,
profil, workflow imposé, permission automatiquement acceptée ni verrou de fichier.

## Scénarios utilisateur et tests

### US1 — Demander une notification depuis sa conversation (P1)

L'utilisateur demande à son agent T3 d'observer un autre agent. Bridget reconnaît
la conversation appelante sans lui demander de copier une identité ou un secret.

- Une conversation reconnue crée un abonnement lui appartenant ; une autre
  conversation ne peut ni le consulter ni le supprimer comme sien.
- Deux conversations du même projet ne sont pas confondues.
- Une identité impossible à prouver produit un refus explicite, sans usurpation.

### US2 — Recevoir une fin de tour réellement observée (P1)

Après abonnement à Horizon-3D, un tour achevé produit une notification dans la
conversation demandeuse. La notification dit « tour terminé », pas « projet fini ».

- Les faits reçus après l'abonnement comptent, même produits juste avant.
- L'historique connu à l'installation du pont n'est pas rejoué.
- Un tour actif, un état inconnu ou une simple absence de sortie ne valent pas fin.
- Une notification ne déclenche pas une boucle de notifications de notifications.
- Une répétition du même fait ne produit pas de seconde notification.

### US3 — Connaître ce qui est réellement surveillable (P1)

Les types d'événements disponibles et leurs sources sont consultables. Une
demande précise impossible est refusée immédiatement avec une explication.

- Agent inexistant, source sans événement compatible ou source indisponible :
  aucun succès trompeur.
- Les écritures et permissions ne sont proposées que si elles sont attestées,
  jamais déduites du texte d'une réponse ou d'une commande shell.
- Une surveillance interrompue n'est pas présentée comme active ; après
  reconnexion, son état reste vérifiable et les limites sont explicites.

### US4 — Préserver les usages quotidiens (P2)

Le partage d'extrait T3 reste utilisable et les écritures concurrentes restent
signalées sur les sources qui les attestent. Aucun signal ne bloque le travail.

- Deux sources compatibles écrivant le même fichier produisent un avertissement.
- Une source sans preuve d'écriture n'annonce pas une couverture des collisions.
- Les sources non T3 conservent leur fonctionnement autonome.

## Exigences fonctionnelles

- **FR-001** : attribuer chaque appel privé à la conversation réelle, sans identité
  choisie à partir d'un nom, du projet, du répertoire courant ou d'un PID partagé.
- **FR-002** : conserver les contrôles de preuve et d'appartenance de la session 099.
- **FR-003** : convertir les fins de tours T3 attestées en événements Bridget,
  incluant les tours humains, les erreurs et interruptions explicitement terminées.
- **FR-004** : dédupliquer les faits et exclure les tours provoqués par les
  notifications système afin d'éviter les boucles.
- **FR-005** : maintenir la règle validée « faits reçus après abonnement » ; ne pas
  rejouer l'historique à l'installation et signaler toute lacune de surveillance.
- **FR-006** : annoncer les capacités effectives des sources ; refuser les
  abonnements sans source compatible connue ou visant un agent introuvable.
- **FR-007** : rendre visible l'indisponibilité d'une source et la perte de
  surveillance après reconnexion/redémarrage, sans annoncer une continuité fictive.
- **FR-008** : conserver abonnement ponctuel, expiration, désabonnement propre,
  limites de charge et respect du mode ne-pas-déranger.
- **FR-009** : préserver partage du journal, avertissements de collision et
  fonctionnement sans T3 ; aucun composant Maicie requis.
- **FR-010** : valider les parcours avec tests isolés et une preuve de notification
  dans un vrai fil T3, sans redémarrer les conversations actives pour les tests.

## Entités principales

Conversation appelante attestée ; source observable et ses capacités ; fait de
tour terminé ; abonnement appartenant à une conversation ; état de surveillance.
Les titres restent des libellés, jamais des preuves d'identité.

## Cas limites et hypothèses

T3 indisponible, preuve périmée, plusieurs fils sur un même processus, fin sans
texte assistant, événement inconnu, pagination tronquée, reconnexion, redémarrage
du daemon, source retirée, destinataire occupé ou en ne-pas-déranger.
La garantie porte sur une fin de tour observée, pas sur l'achèvement métier.
Pas de reconstruction d'événements non fournis par la source. Une impossibilité
de liaison sûre exige un arbitrage, pas une réduction silencieuse de FR-001.
Pas de modification de T3 lui-même sans accord explicite sur ce changement de périmètre.

## Critères de succès

- **SC-001** : une demande dans un fil T3 compatible reçoit une notification unique
  dans ce même fil en moins de 10 secondes après observation de la fin de tour,
  lorsque le destinataire est disponible et la file n'est pas saturée.
- **SC-002** : tous les cas d'identité ambiguë ou de source incompatible testés
  sont refusés sans abonnement privé créé sous une identité d'autrui.
- **SC-003** : aucune boucle ni notification historique dans les scénarios de
  démarrage et de traitement d'une notification.
- **SC-004** : interruption et reprise de surveillance ont un état explicite et
  vérifiable ; les tests des trois fonctionnalités 100 restent satisfaits.
- **SC-005** : aucun redémarrage de T3 ou fournisseur actif, aucun verrou de
  fichiers et aucune dépendance d'orchestration introduits.
