# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/spec.md](../../../../../../specs/165-cycle-vie-liens-t3/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Spec: 165-cycle-vie-liens-t3
Titre: Cycle de vie des liens T3
Statut: Implemented
Priorité: P1
Tâches: 18/18 (100%)
Tests: 13/13 (100%)

<a id="extrait-2"></a>

## Extrait 2

Résumé:
- Contexte: le pont garde une connexion vivante pour presque chaque ancien sous-agent T3.
- Objectif: libérer les ressources des sous-agents inactifs sans perdre leur historique ni couper les conversations principales.
- Dépendances: SPEC-149, SPEC-164

<a id="extrait-3"></a>

## Extrait 3

**Feature Branch**: `session-165-cycle-vie-liens-t3`
**Created**: 2026-10-14
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-149, SPEC-164

<a id="extrait-4"></a>

## Extrait 4

Le pont Bridget garde aujourd'hui un agent connecté pour presque chaque fil T3 non rangé. Cette règle inclut des centaines de sous-agents qui ont déjà terminé leur mission. Chaque lien conserve plusieurs ressources système. Le nombre de ressources ouvertes augmente donc avec tout l'historique, jusqu'à bloquer les nouveaux fils.

<a id="extrait-5"></a>

## Extrait 5

1. Étant donné un sous-agent terminé, quand le pont a traité son dernier résultat et vidé ses obligations, alors son lien vivant disparaît au plus tard dans les dix secondes.
2. Étant donné un sous-agent en échec, annulé ou interrompu, quand son activité cesse, alors il libère les mêmes ressources qu'un sous-agent terminé.
3. Étant donné un historique de sous-agent libéré, quand l'utilisateur consulte T3, alors le fil, ses messages et ses preuves sont toujours présents.

<a id="extrait-6"></a>

## Extrait 6

1. Étant donné un sous-agent en préparation, en attente ou en cours, quand le pont actualise son inventaire, alors son agent Bridget reste joignable.
2. Étant donné un sous-agent récemment créé mais pas encore démarré, quand sa mission passe en préparation, en file ou en cours, alors il devient joignable au prochain rafraîchissement sans intervention humaine.
3. Étant donné un sous-agent précédemment inactif, quand T3 reprend son activité, alors son lien est recréé automatiquement au prochain rafraîchissement. Un message Bridget seul ne reprend pas un enfant fermé.

<a id="extrait-7"></a>

## Extrait 7

Comme utilisateur de T3, je veux pouvoir reprendre une conversation principale terminée ou annulée sans qu'un nettoyage de sous-agent la rende indisponible.

<a id="extrait-8"></a>

## Extrait 8

1. Étant donné une conversation principale terminée, annulée ou inactive, quand le pont nettoie les sous-agents, alors cette conversation reste joignable.
2. Étant donné un fil archivé, supprimé ou rangé par l'utilisateur, quand le pont actualise son inventaire, alors son lien est retiré comme aujourd'hui.
3. Étant donné une ancienne version de T3 qui ne fournit pas la nature parent/enfant du fil, quand Bridget lit son inventaire, alors le comportement compatible actuel est conservé.

<a id="extrait-9"></a>

## Extrait 9

- **FR-001.** Le nombre de connexions vivantes doit dépendre des conversations principales joignables et du travail enfant actif. Il ne doit pas dépendre du nombre total de sous-agents terminés conservés dans l'historique.
- **FR-002.** Un sous-agent dont le travail est terminé, échoué, annulé, interrompu ou sans activité doit libérer sa connexion, ses verrous et ses ressources de journalisation après le drain de ses obligations.
- **FR-003.** La libération d'un lien ne doit supprimer ni conversation, ni message, ni résultat, ni reçu, ni preuve T3 ou Bridget.
- **FR-004.** Un sous-agent en préparation, en file, en attente ou en cours doit rester joignable.
- **FR-005.** Un sous-agent sans activité ne doit pas ouvrir de lien. Son premier état non terminal doit ouvrir le lien au prochain rafraîchissement.
- **FR-006.** Un sous-agent redevenu actif doit retrouver automatiquement un lien au prochain rafraîchissement, sans réinstallation du pont.
- **FR-007.** Les conversations principales et les bifurcations reprenables doivent rester joignables tant qu'elles ne sont ni archivées, ni supprimées, ni rangées.
- **FR-008.** Les fils archivés, supprimés ou rangés doivent continuer à fermer leur lien selon le comportement existant.
- **FR-009.** Si T3 ne fournit pas l'information qui distingue une conversation principale d'un sous-agent, Bridget doit conserver le comportement compatible existant.
- **FR-010.** La fermeture doit traiter le dernier état, vider le journal et attendre les réponses acquittables. Elle doit ensuite fermer le worker, le lecteur, les connexions et les verrous. `turn_wait` constitue la borne dure si une confirmation ne vient pas. Cette horloge commence à la première observation inéligible avec un lien. Un changement de clé terminale ne la repousse pas. Après cette borne, l'obligation reste sur disque mais ne rouvre plus en boucle dans le même processus.
- **FR-011.** La solution doit réutiliser le mécanisme actuel de fermeture et de désinscription. Elle ne doit pas ajouter de service, de dépendance ou de stockage.
- **FR-012.** La limite de fichiers ouverts reste une protection secondaire. Elle ne peut pas servir de critère de réussite du correctif.
- **FR-013.** Le pont doit publier un nombre de fils cohérent avec les liens réellement vivants.
- **FR-014.** Les décisions de conservation et de fermeture doivent être vérifiables par des tests déterministes, sans dépendre d'un fournisseur réel.
- **FR-015.** Après le premier snapshot réussi, un sous-agent terminal qui porte un `pending` persistant, ou un dernier tour connu mais non projeté, doit recevoir une tentative bornée de drain. Une mort imprévue de son lien doit réévaluer cette obligation. Une nouvelle activité T3 ou un redémarrage futur peut donner une nouvelle tentative.
- **FR-017.** Une remise ou une nouvelle attente reçue pendant le drain doit invalider immédiatement la quiescence publiée. Elle ne doit pas être perdue par une fermeture fondée sur un état périmé.
- **FR-018.** Le pont ne doit pas rouvrir plus de 32 liens de drain simultanément. Les obligations suivantes restent en attente d'un cycle ultérieur.
- **FR-016.** Une relation V2 inconnue ou mal formée doit être refusée comme contrat invalide. Elle ne doit jamais provoquer une fermeture par inférence.

<a id="extrait-10"></a>

## Extrait 10

- **SC-001.** Avec 1 000 sous-agents terminés et 30 conversations principales, le pont garde au plus 40 liens inactifs, hors travail réellement actif.
- **SC-002.** Après confirmation du dernier résultat et quiescence du lien, la fermeture libère le lien en moins de dix secondes dans 100 % des scénarios déterministes.
- **SC-003.** L'ajout de 1 000 sous-agents terminés n'augmente pas durablement le nombre de fichiers ouverts de plus de 20.
- **SC-004.** Une conversation principale terminée ou annulée reste joignable dans 100 % des scénarios de reprise.
- **SC-005.** Un sous-agent actif reste joignable, puis peut être recréé automatiquement après une reprise valide.
- **SC-006.** Tous les tests ciblés de cycle de vie et les régressions T3 existantes concernées passent avant livraison.
- **SC-007.** Une observation réelle après activation montre un pont opérationnel sans erreur de limite de fichiers, avec un nombre de descripteurs proportionnel aux liens retenus.

<a id="extrait-11"></a>

## Extrait 11

- T3 fournit la filiation des fils dans son inventaire récent.
- Les conversations principales doivent rester adressables après leur dernier tour.
- Les sous-agents terminés sont des preuves historiques. Ils ne sont pas des destinataires permanents.
- Le rafraîchissement existant reste la source de vérité pour ouvrir ou fermer un lien.

<a id="extrait-12"></a>

## Extrait 12

- Suppression ou archivage automatique des fils T3.
- Modification du contenu des journaux, reçus ou résultats.
- Nouveau protocole de transport ou multiplexage de plusieurs agents sur une connexion.
- Nouvelle commande utilisateur de nettoyage.
- Augmentation permanente présentée comme solution principale.
- Modification de l'interface du navrail T3.

<a id="extrait-13"></a>

## Extrait 13

- **SPEC-149** : exposition des fils T3 comme agents Bridget et identité stable.
- **SPEC-164** : application T3 actuellement installée et contrat récent d'inventaire des fils.
