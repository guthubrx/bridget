# Plan d’implémentation — SPEC-165 Cycle de vie des liens adaptateur de bureau

## Résumé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-1).

La solution ne supprime aucune donnée. Elle ne modifie ni le protocole Bridget, ni le daemon, ni le stockage. Elle réduit le nombre de sockets, verrous et journaux vivants en fermant les connexions devenues inutiles.

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-2).

## Preuve du problème

Observation du 14 octobre 2026 sur le runtime réel :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-3).

La hausse temporaire de la limite à 16 384 a confirmé le diagnostic. Elle a permis d’exposer le fil courant, mais elle ne constitue pas le correctif.

## Constitution Check

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-4).

Aucune violation constitutionnelle n’est nécessaire.

## Architecture retenue

### 1. Conserver deux notions distinctes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-6).

Cette séparation évite de transformer la rétention d’historique en rétention de ressources.

### 2. Utiliser le contrat V2 déjà reçu

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-7).

Les états non terminaux reconnus sont ceux déjà validés par l’adaptateur : préparation, file, démarrage, exécution et attente. Un identifiant d’activité en cours conserve aussi le lien.

### 3. Drainer avant de fermer

Un enfant terminal qui possède déjà un lien reçoit encore ses résumés. Le worker publie `drained_key` seulement quand la clé a été rafraîchie avec succès et que sa file, son journal, ses corrélations et ses réponses acquittables sont vides. Toute remise et tout ajout dans `pending` invalident cette publication. La boucle ferme si cette clé égale la clé terminale. `turn_wait` borne une confirmation absente.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-8).

Le propriétaire du `Link` garde `drain_since`. L'horloge commence à la première observation inéligible avec un lien. Un nouveau `change_key()` terminal ne la remet pas à zéro. Seule une activité qui rend le sous-agent de nouveau éligible annule le drain. Une tentative qui atteint `turn_wait` est consommée pour ce processus : l'état reste sur disque, mais le lien ne boucle pas.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-9).

Le pont rouvre au plus 32 liens de drain simultanément pour les obligations persistées. Les obligations suivantes restent dans l'ensemble borné et passent sur un cycle ultérieur. Ce plafond empêche un ancien stock d'états de recréer un pic de descripteurs pendant `turn_wait`. Il ne ferme pas prématurément des liens déjà actifs qui terminent ensemble. La mesure réelle initiale compte zéro obligation parmi 1 121 sous-agents terminaux.

Une fonction pure produit les actions de réconciliation à partir du résumé, de la présence du lien, de sa clé drainée, de son horloge et des ensembles d'obligations et de tentatives. La boucle `serve()` exécute seulement ces actions. Mille enfants terminaux peuvent ainsi être vérifiés sans daemon réel.

Le lien drainé passe ensuite par `Link::close()` :

1. envoi de la désinscription ;
2. arrêt du relais et du journal par le worker ;
3. fermeture du flux par le worker, puis fin du worker ;
4. fermeture de secours du clone possédé par `Link` ;
5. attente explicite de la fin du lecteur réseau, même après une panique du worker ;
6. libération du verrou lors de la destruction du tracker.

`Link` conserve les handles du worker et du lecteur. Tous les chemins explicites retirent un lien par une même finalisation. Son `Drop` ferme aussi le clone de socket comme filet de sécurité. Un constructeur interne à partir de morceaux permet de tester le cas où le worker panique sans ouvrir un vrai daemon.

Aucun nettoyeur de fichiers n’est ajouté. Les reçus et journaux restent sur disque.

### 4. Réouverture automatique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-10).

## Reutilisation de l’existant

- `ThreadSummary::is_live()` garde la règle archive/suppression/rangement.
- `turn_is_final()` garde la liste existante des états terminaux.
- `Link::close()` garde la séquence de désinscription et de libération.
- La `BTreeMap` de liens reste l’unique état de connexion. Trois `HashSet` bornés aux identifiants du snapshot portent les obligations à reprendre, les tentatives déjà consommées et les drains de reprise ouverts.
- `latest_turn` et `session.active_turn_id` portent déjà l’activité courante.
- `thread_provider()` reste la garde fournisseur.
- Les tests unitaires des contrats V1/V2 et les fixtures `summary()` sont étendus.
- SPEC-149 établit qu’un fil de sous-agent est une projection historique et ne doit pas devenir un agent permanent.

## Divergences volontaires

- La limite locale de 16 384 n’est pas intégrée au générateur de service. Le runtime conserve son garde-fou existant. La recette finale revient à la limite normale pour prouver que la croissance est corrigée.
- Le multiplexage de plusieurs agents sur une connexion serait plus large et modifierait le protocole. Il est exclu.
- Les conversations principales restent connectées. Elles sont reprenables par l’utilisateur et leur nombre observé reste faible.

## Fichiers prévus

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-11).

## Phases d’implémentation

1. Étendre le contrat et ses fixtures sans changer le comportement V1.
2. Écrire les oracles de décision avant la modification de la boucle.
3. Appliquer le prédicat au statut et à la réconciliation sans modifier `drive_queue()`.
4. Ajouter le drain par clé et la reprise bornée des obligations persistées.
5. Vérifier la fermeture complète avec les tests ciblés et une inspection hostile du code concurrent.
6. Intégrer et pousser la source validée.
7. Construire le binaire signé, préparer le rollback et l'installer selon la procédure existante.
8. Revenir à la limite normale, relancer seulement le pont et comparer les descripteurs avant/après.
9. Finaliser les preuves et nettoyer le worktree.

## Stratégie de test

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-12).

## Observabilité et preuve

La recette conserve :

- nombre de fils du snapshot par relation et état ;
- nombre de liens publiés par le pont ;
- nombre et type de fichiers ouverts du PID ;
- extrait de journal après la ligne de démarrage ;
- présence du fil courant ;
- absence d’erreur d’authentification et de limite de fichiers.

## Risques et protections

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-13).

## Charge cognitive future

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/plan.md#extrait-14).
