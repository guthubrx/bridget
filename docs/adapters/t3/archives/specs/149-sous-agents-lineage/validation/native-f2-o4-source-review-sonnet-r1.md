# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Niveau | Verdict |
|---|---|
| Sources : delta F2 (racine retenue) et O4 (fournisseur orphelin), plus identité, héritage, sagas, lignée, autorité native Bridget | **SOURCE_ONLY_APPROVE** - zéro finding bloquant |
| Preuve d'exécution du delta F2/O4 | **Non faite ici.** Le testeur r8 la possède. |

<a id="extrait-2"></a>

## Extrait 2

Lecture faite : ADR149, `contracts/permissions.md`, `contracts/lineage.md`, `recovery149.md`, `review-final149.md`. Diffs complets de `native_delegation.rs`, `daemon.rs`, `wrapper.rs`, `codex_app_server.rs`, `managed_session.rs`, `t3code_mcp.rs`. Lecture intégrale de `native_permissions.rs`, `delegation_lineage.rs`, `communication.rs` (portée projet), `router.rs` (résolution), `managed_process.rs` (arrêt de groupe). Lecture ciblée de `execution_store.rs`, `delegation.rs`, `mcp_identity.rs`, `delegation_mcp.rs`. Les tests F2 de `native_delegation_permissions149_tests.rs` ont été lus comme spécification. Ils n'ont pas été exécutés.

<a id="extrait-3"></a>

## Extrait 3

| Point du brief | Conclusion | Preuve dans les sources |
|---|---|---|
| Capacité typée, privée, non forgeable par un client socket | Conforme. | `bridget-daemon/src/daemon/native_delegation.rs:665-667` : structure à champ privé, visible seulement du module parent. Une seule construction : `:1136`, dans `tick`. Aucune variante de `WrapperToDaemon` ne la porte. Le chemin client appelle `handle_idempotent_send` avec `None` (`daemon.rs:8583-8584`). Les identifiants `conn-N` sont fabriqués par le daemon (`daemon.rs:3157-3160`). |
| Faits durables exacts : état, corps, de, à, instance de l'enfant, propriétaire courant, route | Conforme. | `native_delegation.rs:669-714` : état `result_available`, corps égal au résultat durable (non vide, 256 Kio au plus), id `native-result-<tâche>`, `from` = enfant, `to` = propriétaire, corrélation = mission, ni `reply` ni `intent`, connexion exactement `native-delegation:<id>`, absente de `connections`, auxiliaire, nom et instance de l'enfant, portée d'émetteur de la tâche. Puis route primaire du propriétaire vivante avec la bonne instance, `can_own_task`, liaison T3 si l'hôte est T3. |
| Autorité avant réservation et idempotence, rejeu compris | Conforme. | `daemon.rs:8608-8611` renvoie `Nack` avant la négociation (`:8618`) et avant `idempotency.reserve` (`:8649`). L'ancien raccourci `replayed` de `tick` est supprimé : le rejeu passe maintenant par l'autorité (`native_delegation.rs:1134-1146`). |
| Aucune confiance par préfixe | Conforme. | Le préfixe `native-delegation:` n'accorde rien. L'autorité exige l'égalité exacte (`:684`). La seule autre lecture du préfixe sert à ne pas capturer la remise comme une réponse (`:776`). |
| Révocation propriétaire, racine, instance, tombstone | Conforme. | Révocation directe : `delegation_grant_required` (`:40`). Racine révoquée : `permission_not_inherited` (`:42`, `:535`). Relais d'un résultat retenu : `native_result_authority_revoked` (`:693-696`), la tâche passe `failed`. Le rejeu d'une requête admise précède la révocation (`:413-419` avant `:420`), comme en 148. |
| Identité, liaison T3, projet croisé | Conforme. | `handle_attested` revérifie identité et garde humaine hors verrou puis au puits (`:267-312`). Le cwd reste sous le cwd du fait et `project_scope` joue (`:421-434`). Le relais revérifie la liaison T3 (`:707-712`). Un projet inconnu ne donne qu'un avertissement (`communication.rs:36-62`). |
| CAS au démarrage : `working` et ancien `failed/unreachable` | Conforme. | `prepare_restart` (`:829-868`) : `starting`, `mission_pending`, `working` deviennent `failed/unreachable`. Pour tout `failed/unreachable`, y compris l'ancien, l'exécution `execution-<mission>` est fermée par `transition_if_current` (`execution_store.rs:1227-1285`) seulement si elle est dans un état actif. Un état terminal n'est jamais rouvert. L'agent cible doit être l'enfant, sinon `native_execution_mismatch`. Un CAS refusé donne `native_execution_changed`. |
| Erreurs fermées, pas de doublon de fournisseur | Conforme. | Toute erreur de `prepare_restart` remonte par `reserve_managed_recoveries(...)?` (`daemon.rs:4620`). Le daemon refuse de démarrer. Aucune reprise ne part. Choix assumé, à documenter en exploitation. Les enfants natifs sont exclus de la reprise ordinaire (`daemon.rs:3914`, `:3946`) et leur entrée désirée est arrêtée (`:863-865`). |
| Résultat durable, parent hors ligne puis reconnecté, remise une fois | Conforme. | `tick` fait sortir la racine de `waiting_for_children` AVANT le contrôle `!parent_live` (`:1005-1034`). Le résultat reste durable. La clé idempotente est la même qu'en 148 (portée de la tâche, `native-result-<tâche>`). Un rejeu `Accepted` ou `OutcomeUnknown` avec remise vaut « envoyé » (`:1147-1160`). |
| Résultat capturé, pas inventé | Conforme. | `capture_reply` fixe le résultat avant l'ACK (`:771-808`). `waiting_for_children` sans corps valide devient `failed/native_result_invalid` (`:1022-1028`). Un descendant fermé ne fabrique aucun texte. |
| Cause F2 : l'exécution du petit-enfant restait `running` | Traitée. | `prepare_restart` ferme l'exécution. `descendants_busy` ignore une exécution résiduelle seulement si la tâche enfant est `failed/unreachable` ET l'exécution est `failed`, `unreachable` ou `interrupted` (`:1264-1274`). Un descendant ordinaire (agent lié de la flotte) reste compté actif (`:1275-1285`). |
| Cause F2 : la route primaire de l'expéditeur disparue | Traitée. | Le relais ne dépend plus de `sender_is_authorized`. `Router::resolve` n'utilise pas l'expéditeur (`bridget-core/src/router.rs:129-148`). `prepare_dispatch` exige la route de l'expéditeur seulement si `reply` est vrai (`daemon.rs:8421`). Le projet de l'expéditeur devient inconnu, donc avertissement seulement. |
| Reprise ordinaire inchangée | Conforme. | Reprise de flotte : seuls les noms des tâches natives sont sautés. Remises incertaines : seul un `message.id` égal à une mission native est marqué indéterminé (`daemon.rs:4195-4207`). Reprise d'exécution : seule `execution-<mission>` de l'enfant est sautée (`:4252-4264`). Wrapper : tout est conditionné par `NATIVE_MISSION_BOOTSTRAP_ENV` valide et `managed_reporter` (`wrapper.rs:3942-3951`) ou par `BRIDGET_NATIVE_CHILD_POLICY`. |
| Aucun vidage accidentel d'une ancienne mission | Conforme. | `mission_pending` devient `failed` au redémarrage et n'est jamais renvoyé. Une remise native incertaine n'est jamais redéfférée. Le wrapper natif ne se reconnecte plus : `Ok(0)` et erreur de lecture font `break` (`wrapper.rs:4282`, `:4494`). Il ne relance plus son fournisseur (`:4542`). |
| Gestionnaire SIGTERM sûr | Conforme. | `signal_hook::flag::register` n'écrit qu'un booléen (`wrapper.rs:408-429`). Créé seulement pour une mission native liée à l'instance du wrapper. Testé au début de boucle (`:4246`), après la lecture (`:4277`) et avant le lancement (`:4014`). La lecture a un délai de 1 s (`:1773`), donc le drapeau est vu en 1 s au plus. |
| Arrêt effectif borné | Conforme, preuve d'exécution ailleurs. | Après la boucle, `stop_native_mission` est appelé pour toute sortie (`wrapper.rs:4684-4686`). Codex : `turn/interrupt` du tour exact (message = mission) borné à 3 s, puis `shutdown` (`codex_app_server.rs:1117-1140`). Claude : `cancel_delivery` puis `shutdown` avec SIGKILL du groupe propre (`claude_stream_json.rs:721-726`). |
| Groupes de processus du fournisseur | Conforme. | Le fournisseur est créé avec `process_group(0)` (`codex_app_server.rs:361`, `claude_stream_json.rs:572`). `shutdown` signale `-pid`. Le SIGTERM du daemon vise le groupe du wrapper (`managed_process.rs:402-420`), qui n'atteignait pas le groupe du fournisseur. Le wrapper l'arrête maintenant lui-même. |
| Annulation du tour Codex actif | Conforme. | Admission fermée avant le contrôle (`:1124-1130`). L'interruption ne vise que le tour dont `message_id` égale la mission. |
| Pas de privilèges en plus | Conforme. | Aucune fonction du delta n'élargit un droit. `child_policy` ne produit jamais plus que le fait du parent (`native_permissions.rs:206-277`). Un enfant imbriqué hérite de son snapshot figé (`native_delegation.rs:46-52`). |
| Bridget natif Bridget | Conforme. | Après admission, ni `tick`, ni `prepare_restart`, ni le relais, ni `cancel_tree` n'appellent T3. T3 n'est joint que par `reattest` à l'admission (`:270`) et par la garde humaine de la lecture. |

<a id="extrait-4"></a>

## Extrait 4

- Déclencheur : crash du daemon entre l'insertion de la tâche (`:503`) et le tick qui suit (`:316-318`). L'insertion et le tick sont dans le même appel. La fenêtre est de l'ordre de la milliseconde. O6 de `recovery149.md` n'a pas pu l'exposer en réel (6 essais sur 6 en `starting`).
- Effet : `prepare_restart` ne touche pas `queued`. Le propriétaire (enfant natif) ne se reconnecte jamais. `tick` fait `continue` (`!parent_live`). La racine reste `waiting_for_children`. Seul `cancel` la libère.
- Non bloquant : pour un propriétaire externe (T3, autonome), la reprise voulue joue (`native149_apres_redemarrage_le_tick_admet_le_queue...`).

<a id="extrait-5"></a>

## Extrait 5

1. Chaque appel MCP Bridget résout d'abord l'identité : `mcp_identity.rs:279-283` appelle `t3code_mcp::resolve_current`.
2. `attest` traite toute erreur de T3 comme `T3SessionUnavailable`, y compris `isError` (`t3code_mcp.rs:193-220`). Le refus nommé de T3 est donc aplati.
3. Cette erreur ferme l'appel entier avant toute autre lecture (« Une preuve explicite refusée ferme l'appel avant toute lecture PID », `mcp_identity.rs:280`).
4. `permission_attestation_unavailable` n'apparaît côté Rust qu'à la réattestation privée au puits (`handle_attested`, `native_delegation.rs:267-278`) ou pour une admission inherit ou development avec la seule enveloppe v1 (`:426-428`).
5. Les voies humaine Lineage et d'annulation native ne passent pas par ce credential T3.

<a id="extrait-6"></a>

## Extrait 6

> G-P-07(b). Pour un credential dont le fait de permissions a été retiré (fin, échec ou fermeture du tour), révoqué ou tourné, la fonction `bridget_session` de T3 rend le refus nommé `permission_attestation_unavailable`, jamais l'enveloppe v1. La façade MCP privée de Bridget traite ce refus comme un échec de preuve de session. Tout appel de ce credential est fermé avant effet avec `t3_session_unavailable` : `bridget_delegate` (nouvelle requête et rejeu d'une requête admise), `bridget_task_status`, `bridget_task_cancel` et `bridget_who`. Il n'y a ni repli v1, ni repli discovery, ni repli par PID. Aucun lancement n'a lieu.
> « Lecture et rejeu inchangés » s'entend ainsi : (i) la lecture humaine Lineage native (gardes 147), l'annulation humaine native et la saga native continuent, car elles ne dépendent pas du credential T3 ; (ii) un credential neuf qui n'a jamais eu de fait reçoit l'enveloppe v1 : identité, status, cancel et rejeu 148 d'une requête déjà admise restent possibles, sans nouvelle exécution. Une admission inherit ou development avec la seule enveloppe v1 rend `permission_attestation_unavailable`.
