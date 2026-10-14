# Plan d’implémentation — SPEC-165 Cycle de vie des liens T3

## Résumé

Le pont ouvre actuellement un lien Bridget complet pour chaque fil T3 considéré comme vivant. Le snapshot récent distingue pourtant les conversations principales des sous-agents. Le changement ajoute cette filiation à la vue interne, puis réutilise la fermeture existante pour ne conserver que les conversations principales et les sous-agents qui ont encore une activité non terminale.

La solution ne supprime aucune donnée. Elle ne modifie ni le protocole Bridget, ni le daemon, ni le stockage. Elle réduit le nombre de sockets, verrous et journaux vivants en fermant les connexions devenues inutiles.

## Contexte technique

- Langage : Rust, workspace Cargo existant.
- Périmètre principal : `crates/bridget-daemon/src/t3code.rs`, `crates/bridget-daemon/src/t3code_contract.rs`, `crates/bridget-daemon/src/t3code_contract_v2.rs`.
- Tests : modules de tests déjà présents dans les mêmes fichiers.
- Dépendances nouvelles : aucune.
- Persistance nouvelle : aucune.
- Interface publique nouvelle : aucune.
- Complexité : O(n) par snapshot, où n est le nombre de fils T3. La boucle existe déjà ; le prédicat ajoute seulement des lectures O(1).

## Preuve du problème

Observation du 14 octobre 2026 sur le runtime réel :

- 1 530 fils T3 dans le snapshot ;
- 1 130 fils jugés vivants par le filtre actuel ;
- 1 100 de ces fils sont des sous-agents ;
- 834 sous-agents ont un état terminal ;
- 2 fils seulement ont une exécution active ;
- le pont a atteint environ 4 099 fichiers ouverts avec une limite de 4 096 ;
- chaque lien conserve typiquement trois sockets Unix et un verrou de reçus, plus un journal lorsqu’il est actif.

La hausse temporaire de la limite à 16 384 a confirmé le diagnostic. Elle a permis d’exposer le fil courant, mais elle ne constitue pas le correctif.

## Constitution Check

| Règle | Décision |
|---|---|
| Processus SpecKit | Session 165 isolée avec spec, plan, tâches, analyse et livraison. |
| Worktree dédié | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3`. |
| Diagnostic avant modification | Données runtime, snapshot T3 et inventaire `lsof` observés avant tout code. |
| Réutiliser avant de créer | Réutiliser `ThreadSummary`, la boucle de réconciliation et `Link::close`. |
| Minimalisme | Trois fichiers Rust modifiés, tests colocalisés, aucune dépendance et aucun service. |
| Complexité | Maintenir O(n) par snapshot. Aucun lookup linéaire imbriqué. |
| Responsabilité future | Une méthode de décision pure porte l’invariant. L’ADR explique le choix et ses limites. |
| Tests | Oracles V1/V2, transitions terminales, compatibilité racine et mesure runtime après activation. |

Aucune violation constitutionnelle n’est nécessaire.

## Architecture retenue

### 1. Conserver deux notions distinctes

`is_live()` continue à dire si le fil existe encore dans T3. Une fonction pure dans `t3code.rs` décide si ce fil doit posséder un agent Bridget connecté. Elle réutilise `turn_is_final()` pour reconnaître les états terminaux.

- Conversation principale ou bifurcation : lien conservé tant que le fil T3 est vivant.
- Sous-agent : lien conservé si une activité non terminale ou une obligation persistée existe.
- Sous-agent inactif ou terminal sans lien et sans obligation : aucun lien.
- Sous-agent qui vient de terminer : lien conservé jusqu'au drain de la clé terminale.
- Filiation absente : comportement historique conservé.

Cette séparation évite de transformer la rétention d’historique en rétention de ressources.

### 2. Utiliser le contrat V2 déjà reçu

Le snapshot T3 V2 contient `lineage.relationshipToParent`. Cette session étend `ThreadSummary` et l'adaptateur V2 pour porter ce champ. Une valeur absente représente soit une racine V2, soit un ancien contrat V1. Dans les deux cas, Bridget garde le lien vivant pour préserver la reprise.

Les états non terminaux reconnus sont ceux déjà validés par l’adaptateur : préparation, file, démarrage, exécution et attente. Un identifiant d’activité en cours conserve aussi le lien.

### 3. Drainer avant de fermer

Un enfant terminal qui possède déjà un lien reçoit encore ses résumés. Le worker publie `drained_key` seulement quand la clé a été rafraîchie avec succès et que sa file, son journal, ses corrélations et ses réponses acquittables sont vides. Toute remise et tout ajout dans `pending` invalident cette publication. La boucle ferme si cette clé égale la clé terminale. `turn_wait` borne une confirmation absente.

Le premier snapshot réussi déclenche un scan unique. Il itère sur ses sous-agents inéligibles et appelle `load_thread_state(id)`. Il ne liste pas le dossier, car le nom compact ne permet pas de retrouver l'identifiant T3. Une obligation existe si `pending` n'est pas vide. Un dernier tour terminal déjà connu dans `turn_origins` mais absent de `ended_turns` est aussi une obligation de rattrapage. Seuls ces enfants reçoivent une tentative de drain.

Le propriétaire du `Link` garde `drain_since`. L'horloge commence à la première observation inéligible avec un lien. Un nouveau `change_key()` terminal ne la remet pas à zéro. Seule une activité qui rend le sous-agent de nouveau éligible annule le drain. Une tentative qui atteint `turn_wait` est consommée pour ce processus : l'état reste sur disque, mais le lien ne boucle pas.

Si un lien meurt sans fermeture planifiée, la boucle relit une fois son état. Elle réinscrit l'obligation si elle existe et si aucune tentative n'a été consommée. Après une perte du serveur T3, le prochain snapshot réussi refait cette sélection sous la même garde. Cela couvre une réponse créée après le scan initial et un redémarrage du daemon pendant le drain.

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

Si T3 rend un sous-agent actif dans un snapshot futur, la boucle recrée son lien avant le rafraîchissement des identités. Un message Bridget seul ne reprend pas un enfant fermé. Une obligation durable peut aussi ouvrir un lien borné pour finir sa remise.

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

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/t3code_contract.rs` | Porter la filiation compatible V1. |
| `crates/bridget-daemon/src/t3code_contract_v2.rs` | Lire la filiation V2 et tester les états enfant actifs/terminaux. |
| `crates/bridget-daemon/src/t3code.rs` | Centraliser le prédicat, drainer la clé terminale, reprendre les obligations persistées et joindre le lecteur à la fermeture. |
| `docs/decisions/165-cycle-vie-liens-t3.md` | Historiser la décision structurante. |
| `specs/165-cycle-vie-liens-t3/*` | Artefacts, preuves et reçu de livraison. |

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

- Contrat V1 : filiation absente, comportement historique inchangé.
- Contrat V2 : racine terminale conservée ; sous-agent non terminal conservé ; sous-agent terminal ou inactif retiré.
- Réactivation : un enfant terminal puis actif redevient éligible.
- Archivage : continue à retirer toutes les catégories.
- Réconciliation : un lien devenu inéligible reçoit le dernier Tick, puis ferme sur la même clé drainée.
- Durabilité : une obligation sur disque rouvre un enfant terminal, se réexpédie et disparaît après ACK.
- Reprise : une obligation apparue après le scan initial est réinscrite si le lien meurt avant sa fermeture planifiée.
- Borne : une confirmation absente ferme après `turn_wait`, conserve l'état durable et ne relance pas en boucle dans le même processus.
- Horloge : un changement de clé terminale ne repousse pas `turn_wait`; une vraie réactivation remet l'horloge à zéro.
- Course : une remise reçue après publication de `drained_key` invalide cette clé et interdit la fermeture sur cette publication.
- Panique : le propriétaire ferme la socket et le lecteur termine même si le worker panique.
- Volume : la décision pure conserve 30 racines et écarte 1 000 enfants terminaux sans ouvrir de daemon.
- Plafond : 100 obligations n'ouvrent que 32 drains simultanés ; les 68 autres restent planifiées.
- Régression : tests ciblés `t3code_contract`, `t3code_contract_v2` et `t3code`.
- Suite workspace adaptée au risque avant livraison.
- Runtime réel : absence de `Too many open files`, fil courant exposé, nombre de descripteurs très inférieur à 4 096 avec la limite normale.

## Observabilité et preuve

La recette conserve :

- nombre de fils du snapshot par relation et état ;
- nombre de liens publiés par le pont ;
- nombre et type de fichiers ouverts du PID ;
- extrait de journal après la ligne de démarrage ;
- présence du fil courant ;
- absence d’erreur d’authentification et de limite de fichiers.

## Risques et protections

| Risque | Protection |
|---|---|
| Couper une conversation principale terminée | La filiation prime sur l’état terminal. |
| Couper un enfant entre création et démarrage | Les états préparation/file/démarrage sont non terminaux et recréent le lien au prochain snapshot. |
| Perdre l’historique | Aucune suppression de fichier ou de donnée. |
| Rompre T3 V1 | Filiation absente = comportement historique. |
| Course pendant la fermeture | Dernier Tick conservé, clé drainée et réactivation qui annule la fermeture. |
| Réponse persistée après redémarrage | Scan initial unique et lien de drain borné. |
| Worker paniqué | Clone de socket possédé par `Link`, fermeture puis jointure du lecteur. |
| Daemon redémarré pendant le drain | Relecture ciblée de l'état du lien mort et une nouvelle tentative si elle n'a pas été consommée. |
| Dernier tour devenu terminal pendant l'arrêt du pont | Rattrapage si son origine était connue et si `ended_turns` ne contient pas ce tour. |
| État de quiescence périmé | Toute remise ou nouvelle attente efface la clé drainée avant une décision ultérieure. |
| Stock ancien d'obligations | Plafond constant de 32 liens de drain simultanés et report des suivants. |
| Masquer le bug par la limite | Recette finale sous la limite normale. |

## Charge cognitive future

Le changement sépare trois états : présent dans T3, connecté dans Bridget et en drain. Une fonction pure, une clé de quiescence et deux ensembles bornés portent les obligations et les tentatives. Aucun nouveau service, format de stockage, réglage ou mécanisme de nettoyage n’est introduit.

