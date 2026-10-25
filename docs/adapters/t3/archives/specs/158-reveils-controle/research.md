# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/research.md](../../../../../../specs/158-reveils-controle/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Recherche historique du2026-10-11 : checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`) et T3
`origin/main` par `git show`. Ces preuvesT3 et anciennes lignes restent historiques.
Préparation documentaire158A du2026-10-12 : les points de réemploi sont relus
sur la source157 gelée en préparation. ROOT doit vérifier la livraison157 et
renseigner son reçu canonique et ses empreintes avant le GO ; aucun SHA livré
ni test158 frais n'est revendiqué ici.

<a id="extrait-2"></a>

## Extrait 2

- **Constat** : `bridget lineage` exige `--t3-thread` (`cli_lineage.rs:19-20`) ; le daemon rattache la requête à un fil T3 (`daemon/native_lineage.rs`, `authorize`).
- **Constat sécurité** : `ControlStateV1` est une capacité négociable. Le daemon l'accorde à tout client qui la demande (`daemon.rs:16168-16180`). Les mutations et la boîte humaine ajoutent `control_client_actor` (`daemon.rs:9573`, `9772`, `9802`). `ControlStateRead` seul ne le fait pas.
- **Décision** : `DelegationOverviewRead` exige la négociation ET `control_client_actor`. Pour une requête correctement formée : `negotiation_required`, puis `capability_not_negotiated`, puis `human_actor_required` selon la première garde en échec. La négociation ne donne pas une autorité humaine. La lecture ouvre les métadonnées de tous les parents.
- **Borne** : `issuer_scope` est déclaré par le client (ADR 011). Un processus hostile du même compte n'est pas arrêté.
- **Complexité** : O(n log n), n ≤ 4096 tâches conservées (borne 150).

<a id="extrait-3"></a>

## Extrait 3

- **Constat T3** : `McpToolAccess.ts` rend la déclaration obligatoire par les types.
- **Constat Bridget** : `tools()` est un `Vec<Value>` (`mcp.rs:1891`), l'exécution un `match` sur le nom (`mcp.rs:470-598`). Une garantie de compilation obligerait à réécrire ~30 déclarations.
- **Décision** : `TOOL_SCOPES` + test exhaustif dans les deux sens + cohérence `readOnlyHint`. Garantie de CI, déclarée comme telle.
- **Fils** : `bridget_thread` `read` crée un reçu qui rejoue la même page tant que l'ACK manque (`store/threads.rs:1549-1565`). `read` et `ack` sont donc de l'état propre. `post`, `create`, `add_members`, `close` sont des écritures de fil.
- **Surface** : le test existant refuse toute mutation par `bridget_control_status` (`mcp.rs:2707-2708`). La règle « aucun outil ne pose l'arrêt » le généralise.

<a id="extrait-4"></a>

## Extrait 4

- **Constat T3** : plafond `read-only` par défaut pour un client externe.
- **Constat Bridget** : la tâche garde `effective_posture` (`delegation.rs:40`) ; la sortie d'admission utilise `task.effective_posture.or(posture)` (`native_delegation.rs:83`). Une posture omise est résolue à l'admission. Spec 153 FR-013/014 : un enfant inactif reste vivant entre rondes.
- **Décision** : plafond `read_only` s'il existe une tâche dont l'enfant est l'appelant, sans `cleanup_done`, de posture effective `discovery`. Il ne dépend pas de `terminal()`.
- **Non vérifié** : coût de la recherche par nom d'enfant. T017 mesure avant le code. Repli : cache par nom d'enfant.
- **Borne** : ADR 011 inchangée.

<a id="extrait-5"></a>

## Extrait 5

- Boucle de 1 s du daemon (`daemon.rs:4697-4710`) : `native_delegation::tick`, relances, `dispatch_thread_wakes` (`daemon.rs:7812`). Un planificateur s'y greffe.
- `WakeRow` (`store/threads.rs:102`) gère hors ligne, DND, débit, reprise : un réveil lié à un fil peut l'utiliser.
- `scripts/bridget-ronde.py` : patron d'un planificateur externe qui appelle la CLI.
- Règles T3 (`scheduledTasks/Schedule.ts`) : minimum 1 minute, grâce de 10 minutes, rattrapage unique d'un intervalle.
- Un réveil est un effet autonome : nouvelle variante dans la garde commune de `referent_control.rs`.
- Programmer un réveil vers un autre agent ou un fil est un effet extérieur (portée `sends`), avec droits de projet et de fil vérifiés avant admission.

<a id="extrait-6"></a>

## Extrait 6

- `queue_observation_notifications` abandonne la notification si l'abonné n'a pas de route, est en DND, ou si la file de 64 est pleine, et incrémente `observation_lost` (`daemon.rs:1427-1495`). Seul l'abonnement est persisté.
- Décision visée : garder les seuls faits non remis dans une table bornée. Un fait jamais vu par le daemon n'est pas récupérable ; la lacune est datée.
- Secret (T3) : référence `secret-ref:<hex32>`, 24 h, consommée par l'outil qui la demande (`SecretRequests.ts`). Bridget n'a aucun outil consommateur. Reporté.
