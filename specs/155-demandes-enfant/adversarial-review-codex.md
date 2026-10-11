# Contre-revue adverse - Session155

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, mode plan (lecture seule)
- **Canal** : délégation par le coordinateur
- **Question** : le plan garantit-il l'absence de gain de droits ?
- **Verdict reçu** : BLOCKED (« le plan ne garantit pas encore l'absence de gain de droits »)
- **Méthode de vérification** : chaque objection est une hypothèse, relue dans le
  checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`).

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | L'accord humain contourne le plafond (US2 accepte réseau ou commande hors droits figés) | relu `spec.md` US2 : « l'humain tranche accept ou decline » pour toute demande non couverte ; ADR 149 : « aucun gain de droits » | oui | défaut réel. Nouvelle règle : trois verdicts (`Human`, `Parent` pour les questions, `Forbidden`). Une limite absolue (bac à sable figé du parent) ne se lève pas, même par l'humain |
| 2 | Comparaison de chemins : `..`, liens symboliques, fichiers neufs ; une fonction pure ne résout pas le système de fichiers | relu `data-model.md` (fonction « pure ») ; `spawn_task` canonicalise déjà le `cwd` (`native_delegation.rs:540-545`), donc une base attestée existe | oui | séparer la résolution (impure, base attestée, refus de `..` et de tout lien symbolique, fichier neuf résolu par son dossier) du comparateur (pur). Zones de départ : `cwd` et `writableRoots` seuls, sans `/tmp`, sans `TMPDIR`, sans `.git` ni dossiers de l'outil |
| 3 | Plafond incomplet : `item/permissions/requestApproval` oublié ; réduction `discovery` de l'enfant ignorée | lu `codex_app_server.rs:2176-2181` (`is_approval_request` accepte tout `item/<x>/requestApproval`), faux serveur `:4864` et test `:4194` ; lu `native_permissions.rs:213` (`readOnly` si posture discovery) | oui | la méthode `permissions` est exclue en v1 (refus `approval_method_unsupported`, durée d'octroi inconnue). Un enfant `readOnly` ne reçoit aucun accord d'approbation (`child_restricted`). Détail incomplet, tronqué ou champ d'effet inconnu : `Forbidden` |
| 4 | `never` vers `on-request` peut activer un décideur automatique concurrent (`approvals_reviewer`) | lu `codex_app_server.rs:1373` (`approvalsReviewer` transmis) et `native_permissions.rs:208` (politique du parent copiée telle quelle) | oui | le passage `never` vers `on-request` est supprimé (il ne servait qu'à refuser). En mode `forward`, un parent dont `approvals_reviewer` n'est pas `user` rend `child_requests_reviewer_unsupported` à l'admission. Tests `auto_review` et `guardian_subagent` |
| 5 | Le refus de secours « première décision autre que accept » peut choisir `acceptForSession` | relu `contracts/child-requests155.md` (phrase de repli) ; le code actuel `approval_response` retombe sur la première décision proposée (`codex_app_server.rs:2209-2213`) | oui | le repli n'écrit que `decline` ou `cancel` selon la méthode, sinon une erreur JSON-RPC. Le chemin `deny` actuel n'est pas modifié (hors périmètre) |
| 6 | Corrélation sans incarnation fournisseur : clé `(task_id, provider_request_id)` | relu `data-model.md` ; les identifiants JSON-RPC sont courts et réutilisables entre tours ; l'identité de connexion est disponible (`conn_instances`, comme `capture_reply` `native_delegation.rs:771-808`) | oui | la clé inclut l'instance attestée, la session et le tour fournisseur ; empreinte du contenu ; un doublon au contenu différent est refusé (`request_conflict`) ; accusé accepté seulement de la connexion de l'instance |
| 7 | Un accord enregistré mais non remis survit à l'annulation, la pause, la révocation, le redémarrage | relu `data-model.md` (renvoi tant que `delivered_at` manque) | oui | séparer la décision (historique) et la validité de remise (`delivery_state`). Tout changement de cycle invalide l'accord non appliqué. Accusé négatif prévu |
| 8 | « Texte non fiable » n'empêche pas l'injection | relu le contrat (avis au parent) | oui, en partie | le risque de l'approbation par le parent disparaît : le parent ne répond plus qu'aux questions (voir réduction). Pour l'humain : faits vérifiés par Bridget d'abord (commande citée, chemins résolus, réseau), texte du modèle cité en dernier ; la demande doit venir du tour de la mission autorisée ; cas ambigu : refus |
| 9 | Décision humaine tardive reste dans la relève ; `close_self` ne ferme que les items ouverts ; filtre par préfixe sensible à 200 anciens items | lu `human_inbox.rs:281` (`clamp(1, 200)`), `:342-346` (`state = 'open'`), `:275-277` (`ORDER BY resolved_at ASC`) | oui | la relève par préfixe est supprimée. Le daemon lit chaque item par son identifiant exact (au plus 512 demandes ouvertes). Une décision tardive est acquittée sans exécution (abandon durable). Test à 250 décisions obsolètes |
| 10 | `daemon_deposit_human_inbox` ne rend rien ; fenêtre de panne entre dépôt et liaison | lu `daemon.rs:9412-9440` (retour `()`) | oui | la fonction rend le résultat (les deux appelants actuels l'ignorent sans effet). Recherche par clé exacte avant dépôt (réconciliation) |
| 11 | `bridget_task_respond` absent de la liste fermée des outils du parent natif | lu `wrapper.rs:5035-5057` (`BRIDGET_SAFE_MCP_TOOLS`, 20 outils), `:5061` (Claude `--allowedTools`), `:5265-5270` (Codex `approval_mode="approve"`), tests `:8116-8172` et `tests/claude_native_permissions_test.rs:400` | oui | ajouter l'outil précisément (21 outils) ; mettre à jour les 4 tests de liste ; tester un parent natif réel et la chaîne à 3 niveaux |

## Réduction de périmètre décidée (Article XIX, refus fermé)

1. **Le parent ne répond plus aux approbations.** Pour un enfant Codex hérité, la
   politique de l'enfant égale celle du parent, sauf la posture `discovery`. Toute
   approbation vise donc ce que le parent ne peut pas faire seul. La seule zone
   « couverte par le parent » est l'enfant `discovery`, que les objections 3 et 8
   interdisent d'élargir. Le parent n'a donc plus de route d'approbation en v1. La
   question 2 de `spec.md` permet de la rouvrir.
2. **Le parent répond aux questions** (`user_input`, élicitation en formulaire sans
   secret). Aucune question ne va à l'humain (la boîte ne porte pas de texte libre).
3. **L'humain répond aux approbations**, dans la limite absolue du bac à sable figé du
   parent. Au-delà : refus automatique `Forbidden`, jamais d'item.
4. **Retirés** : la table de couverture pour le parent, le passage `never` vers
   `on-request`, l'escalade, la route humaine des élicitations en mode URL (refusées),
   la relève par préfixe.
5. **Note de critère** : la note visée pour « Demandes de l'enfant » est 4 sur 5 dans
   cette session. 5 sur 5 demande les réponses « oui » aux questions 2 et 3.

## Points restant ouverts

- T002 doit lire sur Codex 0.161.0 la liste des champs d'effet de chaque méthode et la
  durée des octrois. Tant que T002 n'est pas fait, la liste blanche des champs est
  une hypothèse.
- Un accord humain donné à une commande dans un parent `dangerFullAccess` reste
  non borné par nature. C'est le droit du parent, pas un gain.
- Le comportement exact de Codex quand le client refuse après un accord déjà écrit
  n'est pas vérifiable ici (fenêtre entre écriture et accusé).

## Ronde 2 (plan révisé)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : les 11 objections sont traitées ; 2 et 7 restent ouvertes ; 5 est
  fermée seulement en mode `forward` ; 3 est conditionnée à T002.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| R2-1 | (obj. 2) Course sur les chemins : un dossier vérifié peut devenir un lien avant l'écriture ; la seconde résolution ne ferme pas la fenêtre | relu `research.md` R13 et `tasks.md` T008/T009 ; Bridget ne contrôle pas l'instant d'écriture de Codex ; `spawn_task` ne fait que figer le `cwd` (`native_delegation.rs:540-545`) | oui | refus fermé : toute approbation d'écriture de fichier sous un parent `workspaceWrite` donne `file_confinement_unproven`. Sous `dangerFullAccess` le chemin n'est pas une limite. L'humain garde réseau et commande dans la limite absolue. `resolve_request_paths` est retiré |
| R2-2 | (obj. 7) `void` ne retire pas un accord déjà envoyé | relu `data-model.md` : le renvoi et `void` sont posés côté daemon seul ; le wrapper écrit la réponse sans verrou commun avec la fermeture (`wrapper.rs`, boucle des trames du daemon `:4485`) | oui | `sent` écrit avant l'envoi ; réponse et fermeture sérialisées par un verrou du wrapper ; `ChildRequestCancel` / `ChildRequestClosed` ; `unknown` sans accusé ; anneau de 64 demandes réglées. Test à course forcée par barrière |
| R2-3 | (nouveau) L'échéance n'est pas dans le contrôle de remise | relu `tasks.md` T029 et FR-027 : `open` et FR-027 vérifiés, pas `deadline_at` ; le tick est à 1 s (`daemon.rs:4700`) | oui | échéance contrôlée à la décision et avant chaque envoi, sans attendre le tick ; `pending` à l'échéance devient `void`, `sent` suit la fermeture |
| R2-4 | (préexistant) Repli du mode `deny` vers la première décision offerte | lu `codex_app_server.rs:2209-2213` : `.or_else(\|\| choices...next())` ; avec une offre `["acceptForSession"]`, le mode actuel écrit `acceptForSession` | oui | défaut réel en production, indépendant de 155. Tâche T000, livrable seule : `accept` seulement si proposé, sinon `decline`, `cancel`, ou erreur JSON-RPC |
| R2-5 | (obj. 5) fermée seulement en `forward` | idem R2-4 | oui | fermée par T000 dans les deux modes |
| R2-6 | (obj. 3) conditionnée à T002 | relu T002 | oui | T002 ajoute : toute méthode dont les effets ou la durée ne sont pas établis par le schéma est refusée ; T003 ne démarre pas avant la liste |

**Conséquences** : 41 tâches (T000 à T040) ; une seule exception volontaire au
comportement 149 du mode `deny` (T000, FR-003, ADR 155) ; la note visée reste 4 sur 5.

**Reste ouvert** : la preuve de confinement à l'exécution de Codex (session future) ; la
liste blanche des champs d'effet (T002) ; un accord `unknown` ne peut pas être prouvé
annulé.
