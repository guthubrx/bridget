# Comparaison adaptateur de bureau Code / Bridget - communication et délégation entre agents

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research-t3-bridget.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research-t3-bridget.md#extrait-2).

## Notes actuelles (sur 5)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research-t3-bridget.md#extrait-3).

Bridget est déjà devant sur l'annuaire, les fils partagés, les agents hors
outil et les garde-fous (profondeur 8, 16 enfants, droits hérités). Ne pas
régresser sur ces points.

## adaptateur de bureau : mécanismes à reprendre

**A1 Attente.** `delegate_task` `mode: "async"` (défaut) ou `"wait"`,
`timeoutMs` 10 min par défaut, 60 min max. Expiration : l'enfant n'est pas
annulé, `waitTimedOut: true`, la politique de réveil du parent passe à
`always`. Attente sans polling : abonnement aux événements stockés depuis un
curseur de séquence pris avant la première lecture. Résultat structuré :
`taskId`, `status`, `workState` (`working` / `waiting_for_children` /
`result_available`), `summary` stable, `latestTerminal*` pour les tours
suivants.

**A3 Annulation.** Vérifie d'abord récursivement que toute la descendance peut
être arrêtée, puis arrête l'enfant et la descendance, puis abandonne la remise
au parent. Un nouvel essai avec la même clé ne rejoue que les arrêts échoués.

**A4 Idempotence et reprise.** Identifiants déterministes dérivés de
`clientRequestId`, espace de clés propre à l'appelant. Reçus de commande qui
rejouent aussi les refus. Au redémarrage, rejeu des événements persistés,
reprise du tour racine interrompu de l'enfant ; un enfant non repris « doit
encore un résultat ». Outbox : 5 essais, bail 30 s.

**A5 Contexte.** L'enfant reçoit seulement le prompt. Historique complet par
fork (natif ou transfert portable budgété à 16k tokens, avec références vers
l'historique omis).

**A6 Isolation.** Par `t3_thread_launch` `workspaceStrategy` : `root`,
`existing_worktree{worktreePath}`, `worktree{baseRef, branch?,
startFromOrigin?}`. Un enfant `delegate_task` partage le worktree du parent.

**A7 Droits.** Rangs de modes ; refus de toute élévation ; contrôle refait sous
le verrou de la cible.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research-t3-bridget.md#extrait-4).

**A9 Rondes.** Doctrine : une nouvelle tâche par ronde avec tout le contexte
(brief, constats, réponses, objections), clé distincte par ronde, jamais de
relance par envoi dans le fil enfant. `t3_thread_merge_back` : transfert du
delta de contexte.

**A10 Quotas.** Reprise sur limite d'usage à l'heure de réinitialisation, même
fournisseur, identité `fil:run:reset` contre les boucles. Pas de bascule
automatique de fournisseur.

**A11 Demandes de l'enfant.** `t3_pending_request_*` ne répond qu'aux
questions `user_input`, pas aux approbations. Les approbations restent à
l'humain (alerte push).

**B1 Envoyer et attendre.** `t3_thread_wait` : réveil sur fin de run, rend
`timedOut` sans interrompre.

**B2 Steer et file.** `t3_thread_send` modes `auto` / `queue` / `steer` /
`restart` ; fournisseur sans steer natif → interruption puis relance.
`t3_queue_list/read/edit/cancel/reorder/promote_to_steer`.
`t3_thread_interrupt`.

**B3 Réveils.** Boîte aux lettres durable de fin d'enfant, injectée en steer si
le parent est actif, sinon mise en file, résultats groupés. Tâches planifiées
`interval` / `fixed_time` / `webhook` liées au fil appelant ; exécution manquée
ignorée et reprogrammée. Webhooks HMAC-SHA256, 60 requêtes/min, dédup par
identifiant de livraison. Surveillance de PR avec limite de réveils.

**B4 Livraison.** Événements, projection et reçu dans une seule transaction.
Outbox à identifiants déterministes. Remise au parent au moins une fois,
états `pending → claimed → delivered → acknowledged → disposed` ; « remis »
distinct de « lu ».

**B7 Secrets.** `request_secret` : la valeur va au magasin, l'agent ne reçoit
qu'un `secretRef` à usage unique, limité au projet, 24 h.

**C Contrôle.** Panneau d'agents, journal agrégé, arrêt d'un fil qui arrête sa
descendance. Pas d'arrêt global.

**D Sécurité.** Jeton par environnement, fil, fournisseur et session, 24 h,
révoqué à la fin de session. Agents extérieurs : OAuth, plafond lecture seule
par défaut. Chaque outil MCP déclare ses portées (`reads`, `writesThreads`…),
vérifiées à la compilation.

## Bridget : écarts constatés (base 1f2c661d)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research-t3-bridget.md#extrait-5).
