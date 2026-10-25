# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/spec.md](../../../../../../specs/166-rattachement-bridget-reprise/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Spec: 166-rattachement-bridget-reprise
Titre: Rattachement Bridget après reprise T3
Statut: Implemented
Priorité: P1
Tâches: 14/14 (100%)
Tests: 0/0 (0%)

<a id="extrait-2"></a>

## Extrait 2

Résumé:
- Contexte: un fil T3 repris peut rester visible dans Bridget tout en recevant `t3_session_unavailable` lors de chaque appel.
- Objectif: renouveler et charger l'identité privée du fil à chaque reprise qui l'exige, sans réinstallation globale ni contournement d'identité.
- Dépendances: SPEC-149, SPEC-163, SPEC-165

<a id="extrait-3"></a>

## Extrait 3

Une conversation T3 peut rester inactive assez longtemps pour que sa session fournisseur soit recyclée. À sa reprise, le fil réapparaît dans Bridget et son agent est vivant. Pourtant, les outils Bridget peuvent encore utiliser une ancienne preuve privée. Chaque appel échoue alors avec `t3_session_unavailable`.

<a id="extrait-4"></a>

## Extrait 4

Comme utilisateur de T3, je veux qu'un fil repris retrouve automatiquement ses outils Bridget afin de poursuivre le travail sans réinstallation du pont.

<a id="extrait-5"></a>

## Extrait 5

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

<a id="extrait-6"></a>

## Extrait 6

- **SC-001.** Dans 100 % des scénarios déterministes de recyclage puis reprise, le premier appel Bridget du nouveau tour réussit avec l'identité exacte du fil.
- **SC-002.** Dans 100 % des scénarios à deux fils partagés, aucune identité ne traverse la frontière entre les fils.
- **SC-003.** Une preuve remplacée est refusée dans 100 % des scénarios de régression.
- **SC-004.** La reprise conserve le même fil et tout son historique dans 100 % des scénarios couverts.
- **SC-005.** Les tests ciblés du gestionnaire de sessions, de l'adaptateur fournisseur et de l'attestation Bridget passent avant livraison.
- **SC-006.** Une recette réelle sur un fil T3 repris obtient une attestation Bridget valide sans réinstallation du pont.

<a id="extrait-7"></a>

## Extrait 7

- T3 reste l'autorité qui émet et vérifie la preuve privée d'un fil.
- Le fournisseur peut partager un processus entre plusieurs conversations.
- Le cycle de reprise sait déjà recharger la configuration d'un fil lorsque son environnement change.
- Le pont Bridget actif de la session 165 reste compatible avec la correction T3.

<a id="extrait-8"></a>

## Extrait 8

- Modification des règles de droits ou des modes d'approbation.
- Nouvelle identité de secours fondée sur le PID, le dossier ou le nom du fil.
- Réinstallation automatique de Bridget.
- Redémarrage automatique de T3.
- Correction du contenu de la session 141 du projet Opus2D.
- Modification du navrail ou de l'interface utilisateur.

<a id="extrait-9"></a>

## Extrait 9

- **SPEC-149** : identité privée par session T3 et refus sûr des preuves périmées.
- **SPEC-163** : sessions fournisseur partagées, reprise et décisions d'admission.
- **SPEC-165** : cycle de vie borné des liens Bridget exposés par T3.
