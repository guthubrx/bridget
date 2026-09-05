# T006 — Revue indépendante avant extraction

**Verdict : PASS pour la stratégie de coupe et la disposition des garanties.**

Date : 2026-09-05. Référence produit : `dfa2134dcfe2a2522e3ae77d93561e6ae72556b3`. Lecture indépendante de l'implémentation de l'extraction ; le relecteur n'a modifié aucun fichier de production, corpus, manifeste ou carte des tests. Il a rédigé le modèle de menace T004 : la présente revue ne remplace donc pas la revue adverse finale de sécurité T036.

Ce verdict constate qu'aucune garantie de communication n'est volontairement abandonnée par le **plan**. Il ne déclare pas l'extraction réalisée, la baseline réelle exécutée ni le corpus en cours de matérialisation validé. Les préconditions mécaniques T002/T003/T005 restent celles du plan ; une revue de conception ne les coche pas à leur place. Aucune nouvelle autorisation utilisateur n'est demandée pour poursuivre ces tâches déjà approuvées.

## Pièces lues

- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/spec.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/plan.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/tasks.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/baseline.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/data-model.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/contracts/communication.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/contracts/README.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/test-map.md`
- `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/specs/089-communication-core/threat-model.md`

Le nouveau golden de trames a également été lu dans `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/crates/bridget-transport/tests/core_089_wire_test.rs` : comparaison d'émission aux bytes figés puis consommation des bytes figés, et générateur de référence distinct ignoré par défaut. Le manifeste/verrou complet est encore en travail au moment de cette lecture : aucun résultat d'intégrité finale n'est imputé à cette revue.

## Points retenus

- [x] **Une seule autorité.** Canon commun, identité stable, store maître et transactions conservés ; les façades CLI/MCP ne recréent pas de métier. T008 corrige explicitement le scope importé depuis MCP.
- [x] **La réponse n'est pas une mission réussie.** ACK, answered, objectif métier et fraîcheur restent distincts ; aucune promesse d'exécution exactement une fois après une panne fournisseur ambiguë.
- [x] **Pas de perte par suppression de Maicie.** Son crate métier sort, mais le contrat service/capacités/claims/événements demeure et ses clients publics de test sont classés M, pas supprimés en bloc.
- [x] **Pas de perte par suppression du web.** Contenu, référence, provenance et accès restent conservés ; renderer/HTTP/desktop sont hors paquet. La carte distingue expressément un document HTML inerte de son exécution.
- [x] **Pas de perte par suppression de Docker.** Capacités/modèles/efforts opaques, garde de facturation, permissions fournisseur et définition figée restent dans les gates. Aucune autorité d'exécution nouvelle n'est inférée depuis la disponibilité d'un pilote.
- [x] **SSH reste obligatoire.** Les tests locaux ne sont pas promus en preuve de deux machines ; les scripts doivent cesser de cibler/écraser les sockets historiques. Aucun nouveau canal sémantique ni serveur public proposé.
- [x] **Préservation des tests.** Carte de 264 fichiers et douze critères ; fichiers mixtes M soumis à disposition de chaque oracle au moment de leur coupe. Aucun retrait motivé par un rouge ; les auxiliaires de crash et tests ignorés explicitement restent distingués.
- [x] **Le domaine de confiance est honnête.** Même UID et accès SSH autorisé ne constituent pas une sandbox inter-agents. Les credentials du tunnel ne sont pas présentés comme ceux de chaque processus distant.

## Points de coupe à surveiller — dans les tâches existantes

1. **T007 avant exécution de binaires extraits.** Le ramassage des temporaires globaux fait de HOME seul une fausse isolation. Fixer TMPDIR seulement autour de Cargo ne suffit pas lorsqu'un `Command::env_clear()` le supprime avant le daemon. Le modèle T004 nomme les harnais concernés. Ce point bloque un lancement non confiné, pas le travail de code.
2. **T010 : conserver la visibilité des documents.** L'autorisation actuelle utilise un contexte projet/conversation/tour attesté. Une référence opaque de contexte peut survivre sans runtime Docker ; remplacer ce contexte par « global » n'est pas une extraction. Si aucun équivalent conservant les droits n'existe, arrêter uniquement cette couture et faire arbitrer — ne pas supprimer le contrôle.
3. **T012 : conserver l'atomicité réelle.** Les initialisations ledger/requests existent dans plusieurs magasins, et le guichet partage ses transitions avec les événements. Extraire en callbacks post-commit créerait une fenêtre de crash. Les fautes injectées doivent contrôler toutes les écritures, pas seulement le record principal.
4. **T029 : les contrôles historiques ne valent pas preuve de la cible.** Fichiers privés seulement après chmod, PID/symlinks et allocation de ligne avant la borne sont des écarts concrets T004. Ne pas livrer la nouvelle sécurité en citant seulement un test de permissions finales ou une trame 64 Kio déjà allouée.
5. **T002/T003 : figer puis contrôler la disposition.** Le corpus ne doit pas devenir vert par réécriture simultanée d'un golden et de son hash ; sa provenance de référence doit rester vérifiable. La classification M d'un fichier n'autorise aucune suppression globale de ses tests. Le graphe Cargo et le paquet sans sources sorties constituent le verdict d'extraction, pas un garde textuel sur un manifest.

## Minimalisme et maintenabilité

PASS : conserver les trois crates et les mécanismes existants suffit au besoin décrit ; aucune dépendance A2A/LangGraph, couche de repository, autorité d'authentification ou canal UI ne devient nécessaire. Le découplage réduit les consommateurs obligatoires au lieu de fabriquer un protocole parallèle.

Potentiel de suppression : les modules UI/runtime et le plugin métier sont identifiés, mais le nombre de lignes supprimables à comportement constant n'est pas chiffré ici. Un chiffre sans mesure du paquet serait trompeur ; T013/T033 le mesurent avec les dépendances et performances. Le risque principal n'est pas le manque d'abstraction : c'est la suppression d'un contrôle de sécurité avec son ancien consommateur.

## Limite exacte du verdict

Cette revue ouvre le travail de découplage **lorsque les préconditions P0 indépendantes sont attestées** ; elle ne dispense ni des tests par tranche ni de la recette SSH/fournisseur ni de la revue finale. L'installation historique et les données utilisateur restent hors périmètre. Aucun daemon, réseau, registre, fournisseur ou test de crash n'a été exécuté par le relecteur.
