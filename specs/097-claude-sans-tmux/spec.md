# Spécification 097 — Claude natif et interactif sans tmux

**Branche** : session-097-claude-sans-tmux · **Date** : 2026-09-13 · **Statut** : Validée par l'utilisateur (périmètre deux volets)
**Demande** : qu'un agent Claude Code, géré par le daemon ou lancé par l'humain dans son terminal habituel, reçoive réellement les messages Bridget sans serveur tmux, avec la même honnêteté de présence et de livraison que Codex depuis la session 090.
**Tests** : à exécuter.

## Incident déclencheur

Le 2026-09-13, deux envois de l'agent Codex « calliope » vers l'agent Claude `e87d6bc4` sont restés en phase indéterminée puis ont expiré, alors que l'annuaire affichait Claude « connected » avec le transport tmux. La session Claude avait été lancée par `bridget claude --resume` dans iTerm, sans serveur tmux : le wrapper n'avait aucun pane où injecter et les rappels ont été ignorés en silence. Claude n'a lu la mission qu'après avoir été prévenu par l'humain et consulté le ledger. Depuis l'extraction 089, seule la voie Codex a été recettée ; la voie Claude reste le chantier non fait du noyau.

## Besoin et périmètre

Deux volets, un seul objectif : plus aucun agent Claude affiché joignable qui ne l'est pas.

- **Volet A — Claude géré recetté** : le Claude lancé par le daemon (parcours spawn, pilote natif) doit prouver sur un compte réel qu'il reçoit une demande suivie, répond de façon liée, tient un journal attachable et s'arrête proprement. Il hérite de la connexion déjà établie par l'humain sur son abonnement ; aucune bascule vers une API facturée n'est acceptée pour obtenir le vert.
- **Volet B — Claude interactif sans tmux** : `bridget claude` doit fonctionner dans un terminal ordinaire. Le wrapper devient propriétaire d'un pseudo-terminal dans lequel il relaie la frappe et l'affichage de l'humain, et où il remet les messages Bridget. tmux n'est plus ni requis ni supposé.
- **Garde-fous transverses** : une session sans voie de remise disponible n'est pas enregistrée comme joignable, ou l'est avec un état explicite ; l'annuaire ne montre plus « tmux » pour un agent qui n'a aucun pane.

Hors périmètre : GUI, orchestration métier, Maicie, nouveaux outils MCP, changement du protocole daemon, autres fournisseurs interactifs (Gemini, gclaude), acceptation automatique des permissions natives.

## Scénarios utilisateur et acceptation

### US1 — Un Claude géré répond réellement (P1)

L'humain lance un équipier Claude géré depuis Bridget sur sa machine où Claude Code est déjà connecté à son compte. Un autre agent lui envoie une demande suivie ; Claude la reçoit, travaille, répond par une réponse liée qui clôt la demande. L'humain observe l'échange par `bridget attach` et arrête l'équipier sans laisser de processus orphelin.

Acceptation : sur compte réel et sans variable d'API facturée, une demande suivie devient « answered » avec l'identifiant exact ; le journal contient la mission et la réponse ; l'arrêt est constaté sous 10 s hors requête fournisseur déjà partie, zéro enfant survivant.

Test indépendant : la recette des tâches T020/T021 de la session 089 est rejouée avec le binaire courant et consignée avec preuves expurgées.

### US2 — Une session Claude interactive joignable dans n'importe quel terminal (P1)

L'humain lance `bridget claude` (ou `--resume`) dans iTerm, Terminal ou un shell distant, sans tmux. Claude Code conserve son interface native : couleurs, redimensionnement, raccourcis, permissions. Un message Bridget arrive dans cette même conversation, visible par l'humain, et Claude peut y répondre par ses outils Bridget. L'expérience au clavier ne change pas par rapport au lancement direct de Claude Code.

Acceptation : sans tmux dans l'environnement, une saisie humaine puis une demande Bridget sont observées dans la même conversation ; la réponse liée clôt la demande ; le redimensionnement de la fenêtre et Ctrl-C se comportent comme avec Claude Code seul.

### US3 — Remise honnête pendant un tour et saisie préservée (P1)

Un message Bridget arrive pendant que Claude travaille ou pendant que l'humain tape. La saisie en cours n'est pas détruite, aucun tour concurrent invisible n'est créé, et le message n'est déclaré remis que lorsqu'il a effectivement été écrit dans la session. Les rappels et notifications de palier suivent la même règle.

Acceptation : corpus tour actif/inactif, saisie partielle, message long au-delà du plafond, séquence d'échappement dans le corps ; aucun succès de remise inventé, aucun rappel silencieusement perdu.

### US4 — Présence et refus explicites (P1)

Quand aucune voie de remise n'est possible (pas de terminal, pseudo-terminal indisponible, fournisseur absent), le lancement est refusé avec un diagnostic actionnable, ou la présence porte un état « injoignable » que l'annuaire et les émetteurs voient. L'annuaire décrit le canal réel : plus de « tmux » sans tmux.

Acceptation : `bridget who` et `bridget agents --json` montrent le canal réel ; un envoi vers un agent injoignable reçoit un refus ou un état indéterminé nommé, jamais un « délivré » ; un émetteur n'attend pas cinq minutes pour l'apprendre.

### US5 — Fermeture et pannes (P1)

Quitter Claude termine la session interactive et sa présence Bridget. La fermeture du terminal ne promet pas la persistance d'un agent géré. Une coupure du daemon ne recrée pas la conversation ; après rétablissement, identité, corrélations et garanties de rejeu restent celles du noyau.

Acceptation : fermeture normale, terminal coupé, fournisseur arrêté, daemon indisponible puis revenu : aucune présence fausse, aucun enfant oublié, même identité après reconnexion.

### Cas limites

Terminal non interactif (pipe), TERM inconnu, terminal étroit, redimensionnement pendant une remise, message arrivant pendant une demande de permission native, corps contenant des séquences de contrôle, deux sessions interactives Claude simultanées, tmux présent mais non requis, `--resume` sur un fil inexistant, déconnexion daemon pendant l'injection, arrêt reçu pendant le démarrage du fournisseur.

## Exigences fonctionnelles

- FR-09701 : le Claude géré lancé par le daemon utilise la connexion au compte déjà établie par l'humain sur cette machine ; aucune clé d'API n'est lue, injectée ni exigée pour la recette. Un compte non connecté produit un refus nommé, pas un blocage muet.
- FR-09702 : la recette réelle du Claude géré couvre demande suivie, réponse liée, journal attachable, arrêt propre et redémarrage du daemon sans double remise, avec preuves consignées et expurgées.
- FR-09703 : `bridget claude` hors tmux ouvre l'interface native de Claude Code dans un pseudo-terminal possédé par le wrapper, avec les options utilisateur relayées ou explicitement refusées, jamais ignorées.
- FR-09704 : le wrapper relaie fidèlement frappe, affichage, taille de fenêtre et signaux entre le terminal de l'humain et Claude Code ; aucune altération visible de l'interface native.
- FR-09705 : un message Bridget est remis en l'écrivant dans la session comme une saisie humaine, sans détruire une saisie en cours ni créer de tour concurrent ; l'accusé de remise n'est émis qu'après écriture effective, sinon l'état reste indéterminé et nommé.
- FR-09706 : les rappels de palier et les notifications système empruntent la même voie et les mêmes règles que les messages ; aucune remise ignorée sans trace visible pour l'émetteur.
- FR-09707 : quand aucune voie de remise n'est disponible au démarrage, le wrapper refuse de démarrer avec un diagnostic, ou s'enregistre avec un état injoignable explicite ; il ne se déclare jamais joignable par défaut.
- FR-09708 : la présence publiée décrit le canal réel de la session (pseudo-terminal, tmux, pilote natif) et l'annuaire l'affiche ainsi ; un agent sans canal de remise est distinguable en un coup d'œil.
- FR-09709 : envoi, suivi, réponse liée, canon, ledger, idempotence et corrélation existants sont réutilisés sans nouveau protocole ni nouvel outil MCP.
- FR-09710 : les permissions natives de Claude Code restent la décision de l'humain ; aucun bypass, aucune approbation automatique, aucune modification de configuration globale du fournisseur.
- FR-09711 : le journal relayé couvre les tours humains et interagents ; `bridget attach` fonctionne sur la session interactive comme sur un agent géré.
- FR-09712 : arrêt et nettoyage ne touchent que les ressources de la session ; perte du daemon ne relance pas le fournisseur ; identité conservée après reconnexion.
- FR-09713 : la voie tmux n'est plus un prérequis documenté ; aide, README FR/EN, skill et inventaire des commandes sont alignés sur les canaux réels.

## Entités

Session interactive Claude (identité Bridget, pseudo-terminal, fil de conversation, canal déclaré), équipier Claude géré (identité, pilote natif, journal), remise (message, état dispatching/acked/indeterminate, voie employée), présence publiée (canal, mode, localisation, état de joignabilité).

## Critères de réussite

| Critère | Preuve exigée |
|---|---|
| SC-09701 | Recette réelle Claude géré : demande suivie répondue avec identifiant exact, journal attachable, arrêt sous 10 s, zéro enfant survivant ; aucune API facturée. |
| SC-09702 | `bridget claude` dans iTerm sans tmux : message Bridget reçu et réponse liée au ledger en moins de 60 s après envoi, sans intervention humaine. |
| SC-09703 | Interface native inchangée : redimensionnement, couleurs, Ctrl-C et permissions se comportent comme Claude Code seul, vérifié par corpus de terminal. |
| SC-09704 | Message pendant tour actif ou saisie partielle : saisie préservée, une seule remise, état exact au ledger ; retry même clé = une remise. |
| SC-09705 | Aucun agent affiché « connected » sans canal de remise ; l'annuaire montre le canal réel ; un envoi vers un injoignable échoue nommément en moins de 5 s. |
| SC-09706 | Rejeu de l'incident du 2026-09-13 : le même envoi Codex → Claude aboutit en « answered ». |
| SC-09707 | fmt, clippy et suite complète du workspace verts en environnement privé (umask 077) ; aucun test supprimé pour obtenir le vert. |

## Hypothèses et limites

macOS et Linux, Claude Code installé et connecté par l'humain sur la machine ; GLM via Claude Code reste admissible mais n'est pas requis pour le verdict. Le pseudo-terminal devient l'unique voie interactive Claude : la voie tmux n'est pas maintenue en parallèle, une session lancée dans tmux passe aussi par le pseudo-terminal. Aucune promesse de survie à la fermeture du terminal : la persistance reste le parcours géré. Codex conserve son parcours 090 ; Gemini et gclaude ne sont pas traités ici. Les quotas et l'identité commerciale du compte ne sont pas garantis par le produit.
