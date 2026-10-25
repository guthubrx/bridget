# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/plan.md](../../../../../../specs/165-cycle-vie-liens-t3/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le pont ouvre actuellement un lien Bridget complet pour chaque fil T3 considéré comme vivant. Le snapshot récent distingue pourtant les conversations principales des sous-agents. Le changement ajoute cette filiation à la vue interne, puis réutilise la fermeture existante pour ne conserver que les conversations principales et les sous-agents qui ont encore une activité non terminale.

<a id="extrait-2"></a>

## Extrait 2

- Langage : Rust, workspace Cargo existant.
- Périmètre principal : `crates/bridget-daemon/src/t3code.rs`, `crates/bridget-daemon/src/t3code_contract.rs`, `crates/bridget-daemon/src/t3code_contract_v2.rs`.
- Tests : modules de tests déjà présents dans les mêmes fichiers.
- Dépendances nouvelles : aucune.
- Persistance nouvelle : aucune.
- Interface publique nouvelle : aucune.
- Complexité : O(n) par snapshot, où n est le nombre de fils T3. La boucle existe déjà ; le prédicat ajoute seulement des lectures O(1).

<a id="extrait-3"></a>

## Extrait 3

- 1 530 fils T3 dans le snapshot ;
- 1 130 fils jugés vivants par le filtre actuel ;
- 1 100 de ces fils sont des sous-agents ;
- 834 sous-agents ont un état terminal ;
- 2 fils seulement ont une exécution active ;
- le pont a atteint environ 4 099 fichiers ouverts avec une limite de 4 096 ;
- chaque lien conserve typiquement trois sockets Unix et un verrou de reçus, plus un journal lorsqu’il est actif.

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

`is_live()` continue à dire si le fil existe encore dans T3. Une fonction pure dans `t3code.rs` décide si ce fil doit posséder un agent Bridget connecté. Elle réutilise `turn_is_final()` pour reconnaître les états terminaux.

<a id="extrait-6"></a>

## Extrait 6

- Conversation principale ou bifurcation : lien conservé tant que le fil T3 est vivant.
- Sous-agent : lien conservé si une activité non terminale ou une obligation persistée existe.
- Sous-agent inactif ou terminal sans lien et sans obligation : aucun lien.
- Sous-agent qui vient de terminer : lien conservé jusqu'au drain de la clé terminale.
- Filiation absente : comportement historique conservé.

<a id="extrait-7"></a>

## Extrait 7

Le snapshot T3 V2 contient `lineage.relationshipToParent`. Cette session étend `ThreadSummary` et l'adaptateur V2 pour porter ce champ. Une valeur absente représente soit une racine V2, soit un ancien contrat V1. Dans les deux cas, Bridget garde le lien vivant pour préserver la reprise.

<a id="extrait-8"></a>

## Extrait 8

Le premier snapshot réussi déclenche un scan unique. Il itère sur ses sous-agents inéligibles et appelle `load_thread_state(id)`. Il ne liste pas le dossier, car le nom compact ne permet pas de retrouver l'identifiant T3. Une obligation existe si `pending` n'est pas vide. Un dernier tour terminal déjà connu dans `turn_origins` mais absent de `ended_turns` est aussi une obligation de rattrapage. Seuls ces enfants reçoivent une tentative de drain.

<a id="extrait-9"></a>

## Extrait 9

Si un lien meurt sans fermeture planifiée, la boucle relit une fois son état. Elle réinscrit l'obligation si elle existe et si aucune tentative n'a été consommée. Après une perte du serveur T3, le prochain snapshot réussi refait cette sélection sous la même garde. Cela couvre une réponse créée après le scan initial et un redémarrage du daemon pendant le drain.

<a id="extrait-10"></a>

## Extrait 10

Si T3 rend un sous-agent actif dans un snapshot futur, la boucle recrée son lien avant le rafraîchissement des identités. Un message Bridget seul ne reprend pas un enfant fermé. Une obligation durable peut aussi ouvrir un lien borné pour finir sa remise.

<a id="extrait-11"></a>

## Extrait 11

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/t3code_contract.rs` | Porter la filiation compatible V1. |
| `crates/bridget-daemon/src/t3code_contract_v2.rs` | Lire la filiation V2 et tester les états enfant actifs/terminaux. |
| `crates/bridget-daemon/src/t3code.rs` | Centraliser le prédicat, drainer la clé terminale, reprendre les obligations persistées et joindre le lecteur à la fermeture. |
| `docs/decisions/165-cycle-vie-liens-t3.md` | Historiser la décision structurante. |
| `specs/165-cycle-vie-liens-t3/*` | Artefacts, preuves et reçu de livraison. |

<a id="extrait-12"></a>

## Extrait 12

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

<a id="extrait-13"></a>

## Extrait 13

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

<a id="extrait-14"></a>

## Extrait 14

Le changement sépare trois états : présent dans T3, connecté dans Bridget et en drain. Une fonction pure, une clé de quiescence et deux ensembles bornés portent les obligations et les tentatives. Aucun nouveau service, format de stockage, réglage ou mécanisme de nettoyage n’est introduit.
