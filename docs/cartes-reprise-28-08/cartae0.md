# Carte de reprise — cartae0

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> Executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.


- Agent : `cartae0`
- Emise : 2026-08-28T14-35-53Z
- Identifiant ledger : `mcp-1468209-6a919cc9-76`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE — cartae0

1 ETAT
Réserve nommée, aucune mission en cours. Arbre propre sur session-ui-correctifs, aucun processus ni cible temporaire en cours connus.

2 FAIT ATTESTE
Branche distante GitHub `session-ui-correctifs` = 1801c0fdaa998523be37c6739b3c9e66f3a30a57.
- 72cb449 : UI, trois codes distincts et diagnostic conservé ; relecture indépendante favorable.
- 3523492 : rapprochement timeline par `delivery_id = record.message_id`, messages identiques distincts ; oracle nominal et mutant causal vérifiés.
- 10bfff1 : témoin réel `post_ui_message` avec deux connexions Unix ; mutant human_sender_unregistered→daemon_unavailable tué.
- 1801c0f : `capture_reason` distingue négociation refusée, liaison fermée, réponse illisible, timeout et résidu inconnu ; nominal 1/0/22, mutant 0/1/22 ; intégré ensuite par Bridget en 49b7241.
Attesté : tests ciblés et mutants ci-dessus. Non attesté par moi : suite complète après intégration, ni intégration de la branche courante.

3 RESTE
Aucune reprise autonome. Si l’humain mandate une suite UI, prochaine étape précise : rebaser/partir de la tête distante courante, vérifier les changements de main, puis recompiler et refaire les oracles ciblés. Les décisions d’intégration appartiennent à l’humain.

4 CHEMINS ABSOLUS
/home/moi/revue/cartae0
/home/moi/revue/cartae0/crates/bridget-daemon/src/ui.rs
/home/moi/revue/cartae0/crates/bridget-daemon/assets/ui/app.js
/home/moi/revue/cartae0/plugins/maicie/src/main.rs

5 PIEGES
- Un filtre Cargo trop strict (`--exact` sans chemin complet) peut rendre 0/0/filtrés : ce n’est pas une mesure.
- Un oracle littéral ne garde aucun chemin ; le témoin doit appeler la fonction réelle.
- Ne pas réintroduire le rapprochement par contenu : deux messages identiques doivent rester distincts.
- `app.js:2867 join("")` est correct ; la perte de frontières supposée dans attach n’a pas été démontrée sur un flux continu via details.
- Ne pas toucher à attach.rs sans mandat ; cette surface a été rendue à un autre agent.

6 DELEGATIONS / IGNORÉ
Objectif UI `f71880e9-5e81-48ef-a159-51ba0bfba733` : volets livrés, premier et second attestés ; dernier état connu fermé par Bridget. Objectif provenance `859aed43-48fd-4bcb-9fb0-4fd1a850bd38` : arrêt volontaire sur impossibilité mesurée de construire HumanRequest/attestation dans le protocole actuel ; aucune production modifiée. Je ne connais pas l’état actuel des autres objectifs, des arbitrages humains, ni la suite complète post-49b7241. Carte envoyée explicitement : livraison identifiant renvoyé par Bridget.
