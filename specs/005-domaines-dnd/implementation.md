# Journal d'Implémentation — Domaines et « ne pas déranger »

## Métadonnées

- **Spec** : 005-domaines-dnd
- **Branche** : `session-05-domaines-dnd`
- **Démarré / terminé** : 2026-08-17

## Progression

### T101 — Domaine dans le protocole et dérivation

- **Statut** : ✅ Complété
- **Fichiers** : `protocol.rs` (`Register.domain`, `AgentInfo.domain`, messages
  `Domain` et `Availability`), `daemon.rs` (`Presence.domain`,
  `Presence.derived_domain`, `Presence.dnd_until`, `presence_of_agent`,
  `handle_domain`, `handle_availability`), `wrapper.rs` (`derive_domain`,
  `effective_domain`, `domain_state_path`)
- **Notes** : `derived_domain` est conservé séparément du domaine effectif, sinon
  `--reset` n'aurait rien sur quoi retomber. Les quatre sites d'appel de
  `connect_and_register` passent le domaine effectif, y compris les deux boucles
  de reconnexion — le domaine surchargé survit donc à une coupure pour la même
  raison que le nom depuis la session 004.

### T102 — Colonne, filtre et sortie JSON

- **Statut** : ✅ Complété
- **Fichiers** : `cli.rs` (`cmd_who` réécrit, `cmd_agents`,
  `extract_domain_filter`, `cell`)
- **Notes** : `cmd_who` accumulait huit calculs de largeur de colonne identiques,
  copiés-collés. La neuvième colonne a justifié de les remplacer par un helper
  `column()` : la fonction perd une trentaine de lignes répétitives et la
  prochaine colonne coûtera une ligne. `runtime_cell` devient `cell`, son rôle
  n'étant plus limité au runtime.

### T103 — `bridget domain`

- **Statut** : ✅ Complété
- **Fichiers** : `cli.rs` (`cmd_domain`, `domain_state_path`)
- **Notes** : **gate anti-doublon déclenché.** J'avais écrit une fonction
  `send_control_to_daemon` pour porter le message ; le compilateur a révélé
  qu'une fonction du même nom existait déjà (`cli.rs:378`), utilisée par
  `send_to_daemon`. La mienne a été supprimée et l'existante réutilisée telle
  quelle. Aucune ligne de doublon n'a survécu.

### T104 — Statut de disponibilité

- **Statut** : ✅ Complété
- **Fichiers** : `daemon.rs` (`dnd_until`, `is_dnd`, `dnd_minutes_left`),
  `cli.rs` (`cmd_dnd`, `parse_duration`, `DND_DEFAULT_MINUTES`)
- **Notes** : le statut est une échéance (`Option<Instant>`) et non un booléen.
  L'expiration devient une comparaison à la lecture : aucune tâche de fond, rien
  à balayer, et une échéance déjà passée équivaut naturellement à une absence de
  statut. `parse_duration` accepte `90s`, `30m`, `2h`, et un nombre nu vaut des
  minutes.

### T105 — Refus de routage et suspension des rappels

- **Statut** : ✅ Complété
- **Fichiers** : `daemon.rs` (contrôle avant routage, filtre dans la boucle
  d'escalade)
- **Notes** : le contrôle est placé après la quarantaine et les hops, avant la
  résolution du routeur. Les rappels des paliers 1 et 2 sont retenus pour un
  destinataire en statut, mais **pas** la notification d'échec du palier 3 :
  celle-ci s'adresse à l'émetteur et ne dérange personne d'autre.

### T106 — Documentation et vérification

- **Statut** : ✅ Complété
- **Fichiers** : `README.md` (commandes, sections « Domaines de travail » et
  « Ne pas déranger »), ce journal

## Vérifications réelles

Daemon redémarré sur la version compilée avec l'accord de l'utilisateur, sept
agents reconnectés automatiquement en conservant leurs noms.

| Critère | Résultat | Preuve |
|---|---|---|
| FR-003 — colonne DOMAINE | ✅ | présente pour tous les agents, alignée, `—` si inconnue |
| FR-004 — filtre | ✅ | `bridget who --domain bridget` n'a rendu que `agent-2`, avec l'en-tête « Agents du domaine » |
| FR-006 — surcharge | ✅ | `bridget domain bridget` → visible immédiatement |
| FR-006 — réinitialisation | ✅ | `bridget domain --reset` → domaine à `—`, fichier d'état supprimé |
| FR-007 — trace de persistance | ✅ | `~/.cache/bridget/agent-domains/agent-2` contient `bridget` |
| FR-008, FR-009 — refus hors agent | ✅ | `dnd` et `domain` rendent 1 avec un message explicite |
| FR-009, FR-010 — activation | ✅ | `bridget dnd --duration 15m` → état `dnd` dans l'annuaire |
| FR-011 — refus motivé | ✅ | envoi refusé : « « agent-2 » ne souhaite pas être dérangé (encore 15 min) », code 1 |
| FR-013 — visibilité | ✅ | colonne ÉTAT à `dnd` |
| FR-014 — levée | ✅ | `bridget dnd off` → `connected`, et le message suivant a bien été livré dans le terminal de l'agent |
| SC-006 — non-régression | ✅ | 86 tests verts (28 core + 33 daemon + 6 intégration + 19 transport), `cargo build` à 0 warning |

## Reste à vérifier

**Le domaine dérivé sur un agent réel.** Les wrappers actuellement connectés ont
été lancés avec le binaire précédent : ils n'envoient pas de domaine, d'où les
`—` dans l'annuaire. La dérivation est couverte par le code et vérifiable dès le
prochain lancement d'un agent — `bridget who` doit alors afficher `bridget` pour
un agent lancé dans ce dépôt, sans aucune commande.

**La persistance du domaine à travers un redémarrage du daemon**, pour la même
raison : elle repose sur le domaine renvoyé par le wrapper à son
réenregistrement. La trace disque, elle, est vérifiée.

**La suspension des rappels d'escalade** est couverte par un test unitaire : la
décision a été extraite dans `should_remind()`, fonction pure vérifiée sur les
six combinaisons de palier et de statut. Ce qui n'est pas reproduit en réel, c'est
la boucle complète — il faudrait une demande `--reply` dont l'échéance de rappel
tombe pendant un statut actif, sur un agent volontairement muet.

Deux tests manquaient à l'appel des observables de `tasks.md` et ont été ajoutés
avant de cocher : la dérivation du domaine depuis la racine git, et la priorité
du domaine surchargé sur le domaine dérivé.

## Findings hors périmètre

Aucun nouveau. Les quatre findings de la session 004 restent ouverts
(`managers.rs` mort, `purge_if_too_large` vide, absence de délai de lecture dans
`send_rename_to_daemon`, avertissements `clippy` préexistants), auxquels s'ajoute
le finding de sécurité transverse sur l'authentification de l'émetteur des
messages de contrôle — que cette session étend mécaniquement à `Domain` et
`Availability`, sans l'aggraver dans son principe.
