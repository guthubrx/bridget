# Carte de reprise — cartae0

> **TOUTE AFFIRMATION D'INTEGRATION DE CETTE CARTE EST DATEE DU 28/08 ET NON REVERIFIEE DEPUIS.**
> Avant de t'appuyer sur une integration annoncee ici : `git fetch` PUIS
> `git merge-base --is-ancestor <sha> origin/main`. Ne deduis pas, mesure.
> ET NOMME LE REMOTE : `origin` ne designe pas le meme depot selon les checkouts du parc —
> deux d'entre eux pointent un miroir local fige, pas github.
> AVERTISSEMENT DE rc7-flux, qui vaut pour les deux sens : une affirmation « X est integre »
> ne se defait pas, mais une affirmation « X n'est PAS integre » PERIME DANS L'AUTRE SENS —
> il suffit qu'on integre pour que la carte fasse croire a une prochaine incarnation qu'il
> reste du travail alors qu'il est fait. C'est le mensonge le plus probable d'une carte.
> Bandeau ajoute le 28/08 17h00 par le referent, sur signalement de rc7-flux.


> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit, RELEVE LA VRAIE VARIABLE D ENTREE — le cwd de TON PROCESSUS :
> `readlink /proc/<ton-pid>/cwd` puis `git -C <ce-cwd> rev-parse --show-toplevel`.
> C est le `basename` de cette racine git qui devient ton domaine. Ton cwd n est PAS celui que
> tu crois : le processus tourne souvent un cran plus bas que le repertoire que tu observes.
> NE PERDS PAS DE TEMPS a lire `agent-domains/<ton-nom>` : un nom neuf n a JAMAIS de fichier,
> la reponse sera toujours `absent`. Cette consigne, presente ici jusqu au 28/08 16h44, etait
> un rite — corrigee sur demonstration de rc5-flux, qui a invalide sa propre proposition.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS AU CHANGEMENT DE NOM — et non au respawn, requalification du 28/08 :
> il est persiste dans `agent-domains/<nom>`, RELU a chaque enregistrement y compris apres
> reconnexion, et un respawn du MEME nom retrouve donc son domaine. Un nom neuf n a pas de
> fichier : `derive_domain` s applique alors et rend le `basename` de la racine git du cwd,
> avec repli sur le cwd lui-meme. IL PEUT DONC VALOIR `bridget` NATIVEMENT si ce repertoire
> s appelle `bridget` — c est le cas de `rc7` et `essai-claude-distant`, verifie par leur PID.
> Source : `fn derive_domain` et `fn effective_domain`, `crates/bridget-daemon/src/wrapper.rs`,
> ligne 983 DANS L ARBRE `/home/moi/revue/rc7/bridget` — la ligne differe selon les checkouts,
> il y en a six sous /home/moi. `effective_domain` n a AUCUN repli code en dur vers `bridget`.
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
