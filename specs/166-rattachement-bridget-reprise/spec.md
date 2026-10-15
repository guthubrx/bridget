# Feature Specification: Rattachement Bridget après reprise T3

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 166-rattachement-bridget-reprise
Titre: Rattachement Bridget après reprise T3
Statut: In Progress
Priorité: P1
Tâches: 13/14 (92,9%)
Tests: 0/0 (0%)

Résumé:
- Contexte: un fil T3 repris peut rester visible dans Bridget tout en recevant `t3_session_unavailable` lors de chaque appel.
- Objectif: renouveler et charger l'identité privée du fil à chaque reprise qui l'exige, sans réinstallation globale ni contournement d'identité.
- Dépendances: SPEC-149, SPEC-163, SPEC-165

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-166-rattachement-bridget-reprise`  
**Created**: 2026-10-15  
**Status**: In Progress  
**Priority**: P1  
**Dependencies**: SPEC-149, SPEC-163, SPEC-165

## Problème

Une conversation T3 peut rester inactive assez longtemps pour que sa session fournisseur soit recyclée. À sa reprise, le fil réapparaît dans Bridget et son agent est vivant. Pourtant, les outils Bridget peuvent encore utiliser une ancienne preuve privée. Chaque appel échoue alors avec `t3_session_unavailable`.

La réinstallation du pont ou l'augmentation de limites système ne corrige pas cette incohérence. L'utilisateur doit pouvoir reprendre un fil ancien, une bifurcation ou une conversation principale sans réparer manuellement le connecteur.

## Histoires utilisateur

### US1 — Reprendre un fil sans réparer Bridget (P1)

Comme utilisateur de T3, je veux qu'un fil repris retrouve automatiquement ses outils Bridget afin de poursuivre le travail sans réinstallation du pont.

**Test indépendant** : un fil reçoit une identité, devient inactif, perd sa session fournisseur, puis reprend un tour. Son premier appel Bridget réussit avec l'identité du fil repris.

**Scénarios d'acceptation** :

1. Étant donné un fil inactif dont la session fournisseur a été recyclée, quand un nouveau tour démarre, alors les outils Bridget utilisent une preuve privée valide pour ce tour.
2. Étant donné un fil repris dans une session fournisseur partagée, quand une conversation voisine est déjà active, alors chaque fil conserve sa propre identité.
3. Étant donné une reprise réussie, quand plusieurs appels Bridget sont effectués pendant le même tour, alors ils restent autorisés sans rotation inutile.

### US2 — Préserver les refus de sécurité (P1)

Comme utilisateur, je veux que la réparation automatique ne permette jamais à un fil d'emprunter l'identité d'un autre fil.

**Test indépendant** : une preuve ancienne, révoquée ou appartenant à un autre fil reste refusée après la reprise.

**Scénarios d'acceptation** :

1. Étant donné une ancienne preuve privée, quand la reprise a chargé une preuve neuve, alors l'ancienne preuve reste refusée.
2. Étant donné deux fils attachés à la même session fournisseur, quand ils appellent Bridget, alors chaque attestation désigne son propre fil.
3. Étant donné une reprise sans fait de droits actif, quand une opération exige des droits hérités, alors elle reste refusée selon les règles existantes.

### US3 — Conserver la continuité des conversations (P1)

Comme utilisateur qui reprend un travail long, je veux conserver l'historique, le contexte et le processus courant pendant la réparation du rattachement.

**Test indépendant** : la reprise ne crée pas une nouvelle conversation et ne supprime aucun message, résultat ou lien Bridget existant.

**Scénarios d'acceptation** :

1. Étant donné un fil ancien avec un historique, quand son identité est renouvelée, alors son identifiant et son historique restent inchangés.
2. Étant donné un processus fournisseur partagé encore vivant, quand un seul fil reprend, alors les autres fils ne sont pas interrompus.
3. Étant donné un échec de renouvellement, quand le fil appelle Bridget, alors il reçoit un refus explicite et aucun repli vers une identité voisine.

## Exigences fonctionnelles

- **FR-001.** La reprise d'un fil doit vérifier que la preuve privée chargée est encore l'objet courant du registre privé pour ce fil et cette instance fournisseur.
- **FR-002.** Si la preuve chargée est absente, révoquée ou périmée, la reprise doit établir une preuve valide avant le premier appel Bridget du nouveau tour.
- **FR-003.** La preuve valide doit être chargée dans le processus fournisseur qui exécute réellement le tour repris.
- **FR-004.** Une session fournisseur partagée doit conserver une preuve distincte par fil.
- **FR-005.** Le mécanisme exécuté à chaque reprise ne doit pas redémarrer T3, réinstaller le pont ou recréer la conversation. La relance unique requise pour activer une nouvelle version installée reste une opération de livraison.
- **FR-006.** La réparation ne doit supprimer ni historique, ni message, ni résultat, ni preuve durable.
- **FR-007.** Une ancienne preuve doit rester refusée après son remplacement.
- **FR-008.** Une preuve d'un fil ne doit jamais autoriser un autre fil, même si les deux fils partagent le même fournisseur.
- **FR-009.** Les droits du tour doivent continuer à être attestés au moment de l'appel. La réparation d'identité ne doit pas élargir les droits.
- **FR-010.** Un échec de réparation doit produire un refus explicite et sûr. Il ne doit pas utiliser une identité de processus, de voisin ou de secours.
- **FR-011.** La solution doit réutiliser le cycle existant d'attachement, de reprise et de montage des outils. Elle ne doit ajouter ni service, ni stockage, ni dépendance.
- **FR-012.** Le scénario de session recyclée puis reprise doit être couvert par un test déterministe qui échoue avant le correctif.
- **FR-013.** Les scénarios existants de démarrage, bifurcation, reprise, détachement, rotation et partage de session doivent rester valides.

## Cas limites

- Le fil reprend pendant qu'une autre conversation utilise déjà la session fournisseur partagée.
- La preuve enregistrée existe encore, mais le registre actif ne la reconnaît plus.
- Le processus fournisseur conserve une configuration ancienne après une rotation.
- Deux reprises du même fil arrivent presque en même temps.
- Le nouveau tour se termine pendant la vérification de la preuve.
- La reprise échoue avant le chargement complet des outils.
- Le fil est supprimé ou archivé pendant la reprise.

## Critères de succès

- **SC-001.** Dans 100 % des scénarios déterministes de recyclage puis reprise, le premier appel Bridget du nouveau tour réussit avec l'identité exacte du fil.
- **SC-002.** Dans 100 % des scénarios à deux fils partagés, aucune identité ne traverse la frontière entre les fils.
- **SC-003.** Une preuve remplacée est refusée dans 100 % des scénarios de régression.
- **SC-004.** La reprise conserve le même fil et tout son historique dans 100 % des scénarios couverts.
- **SC-005.** Les tests ciblés du gestionnaire de sessions, de l'adaptateur fournisseur et de l'attestation Bridget passent avant livraison.
- **SC-006.** Une recette réelle sur un fil T3 repris obtient une attestation Bridget valide sans réinstallation du pont.

## Hypothèses

- T3 reste l'autorité qui émet et vérifie la preuve privée d'un fil.
- Le fournisseur peut partager un processus entre plusieurs conversations.
- Le cycle de reprise sait déjà recharger la configuration d'un fil lorsque son environnement change.
- Le pont Bridget actif de la session 165 reste compatible avec la correction T3.

## Hors périmètre

- Modification des règles de droits ou des modes d'approbation.
- Nouvelle identité de secours fondée sur le PID, le dossier ou le nom du fil.
- Réinstallation automatique de Bridget.
- Redémarrage automatique de T3.
- Correction du contenu de la session 141 du projet Opus2D.
- Modification du navrail ou de l'interface utilisateur.

## Dépendances

- **SPEC-149** : identité privée par session T3 et refus sûr des preuves périmées.
- **SPEC-163** : sessions fournisseur partagées, reprise et décisions d'admission.
- **SPEC-165** : cycle de vie borné des liens Bridget exposés par T3.
