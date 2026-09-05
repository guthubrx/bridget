# Recherche et décisions — 089

Consultation : 2026-09-05. Les liens externes sont des sources d'idées, pas une nouvelle autorité filaire pour Bridget. La référence locale est le commit dfa2134dcfe2a2522e3ae77d93561e6ae72556b3 ; les constats ci-dessous sont une première carte, pas l'inventaire exhaustif T001.

## Ce que le code impose déjà

- `crates/bridget-daemon/Cargo.toml:33` dépend directement de `../../plugins/maicie`. Les quatre membres du workspace sont confirmés par `cargo metadata --no-deps --format-version 1 --offline`. Ce n'est pas seulement un couplage d'affichage.
- `crates/bridget-daemon/src/lib.rs` expose à la fois ledger/MCP et ui/mission_projection/project_runtime. Les frontières sont donc à couper dans les appels, pas uniquement dans les manifests.
- `crates/bridget-transport/src/managed_session.rs` contient déjà événements avec raw, source et origine, un terminal typé et un trait de session. Réutiliser cette correction éprouvée.
- `crates/bridget-daemon/src/mcp.rs` différencie in_flight, outcome_unknown et orphaned. La skill globale consultée dans `/Users/moi/.codex/skills/bridget/SKILL.md` décrit encore outcome_unknown comme un accusé en vol. La nouvelle source de skill doit être alignée ; aucune modification globale automatique.
- `scripts/federate-ssh.sh` utilise un transfert reverse Unix par SSH mais choisit la socket locale dans le home historique. Le nouvel environnement exige des chemins explicites et ne doit jamais effacer la socket distante d'un autre daemon.

## Emprunts externes retenus, sans framework supplémentaire

| Source primaire | Concept utile | Application 089 et preuve |
|---|---|---|
| [A2A — spécification](https://a2a-protocol.org/latest/specification/) | Séparer message, tâche et découverte de capacités. Un identifiant de contexte n'est pas une mémoire partagée ; l'envoi n'impose pas une garantie universelle d'idempotence. | Conserver identité/capacités attestées et distinguer ACK de mission accomplie ; FR-08902/03/06, SC-08901/02. Pas de serveur A2A ni de conversion des messages en tâches. |
| [LangGraph — persistance](https://docs.langchain.com/oss/python/langgraph/persistence) | Points de reprise durables distincts d'un état en mémoire ; récupération à partir des faits sauvegardés. | Renforcer les oracles d'outbox/curseur et d'effets idempotents existants ; SC-08903/05. Aucun moteur de graphe dans Bridget. |
| [ACP — session](https://agentclientprotocol.com/protocol/v1/session-setup) | Protocole entre un client et une session d'agent, avec capacités explicites. | Conserver ACP comme un pilote à côté des natifs, pas comme remplacement de l'inter-agents ; SC-08909. |
| [MCP — version épinglée](https://modelcontextprotocol.io/specification/2025-06-18/basic) | Outils comme façade négociée, indépendante de la présentation. | Garder le MCP existant, JSON stdout pur et appels par le même contrat que CLI ; SC-08902/10. Pas d'upgrade implicite. |

Ces protocoles ne règlent ni le tarif des fournisseurs, ni la fiabilité d'un CLI particulier, ni la preuve qu'une mission est bien faite. GLM via Claude Code demeure le chemin d'abonnement déclaré fonctionnel par l'utilisateur ; aucune remise en cause commerciale déduite d'une fiche de protocole.

## Alternatives explicitement écartées pour cette session

1. **Tout convertir en A2A :** ajoute une nouvelle surface réseau, une correspondance d'états et de nouvelles questions d'autorisation sans résoudre le découplage local. Une passerelle pourra être étudiée pour un consommateur A2A réel, après le noyau.
2. **Installer LangChain/LangGraph comme bus :** ce sont des abstractions applicatives différentes ; notre dette est dans les dépendances et coutures de communication existantes. Les idées de reprise sont utiles, la dépendance n'est pas justifiée.
3. **Réécrire le daemon :** perd les preuves accumulées sur les courses ACK/ledger/replies/crash. Extraction graduelle préférable, avec réduction mesurée.
4. **Revenir à tmux pour simplifier :** contraire au besoin confirmé ; les pilotes natifs restent. L'ancien comportement peut servir de comparaison historique, jamais de dépendance requise.
5. **Forker T3 maintenant :** une UI ne prouve pas la stabilité du cœur. La preuve T3 devra ultérieurement lire un état unique, sans lancer une seconde flotte ni posséder une deuxième base de missions.
6. **Supprimer toutes les protections :** confond sandbox de rendu web et contrôle d'accès aux messages/processus. On enlève le premier du produit extrait, on maintient le second.

## Décisions locales

- D089-1 : extraction avec historique, aucun remote vers l'ancien dépôt dans le nouveau clone.
- D089-2 : trois crates conservés ; modules neutres ciblés avant tout ajout de crate.
- D089-3 : messages et requests sont le cœur ; les politiques de mission demeurent à l'extérieur. Guichet et faits de lifecycle publics restent un contrat négocié, non une dépendance au code Maicie.
- D089-4 : SSH = extension d'accès à un daemon maître unique, pas deux autorités qui réconcilient leur base.
- D089-5 : aucun stockage actuel importé par défaut ; configuration indépendante précède le premier essai de binaire.
- D089-6 : code, tests et skill doivent décrire la même garantie, notamment l'incertitude durable. Tout écart découvert est tranché par un oracle et documenté, non lissé par une prose plus optimiste.

## Questions de validation, pas choix d'architecture manquants

La disponibilité des comptes natifs/GLM et d'une seconde machine SSH sera vérifiée lors des gates ; ne pas les déclarer acquis depuis les journaux historiques. Les numéros de schémas, canons et fixtures exacts doivent être épinglés par T001/T002 avant refactor. La nouvelle baseline ne vaut pas permission de reprendre les données ou de redémarrer les agents existants.
