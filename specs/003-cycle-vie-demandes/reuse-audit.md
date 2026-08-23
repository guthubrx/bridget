# Audit de réutilisation — cycle de vie des demandes

**Date** : 2026-08-14  
**Statut** : PASS

## Éléments proposés et existant réutilisable

| Besoin du plan | Existant local | Décision |
|---|---|---|
| Transport des commandes | `crates/bridget-transport/src/protocol.rs` | Étendre les enums existantes ; pas de second canal. |
| Routage et rappels | `crates/bridget-daemon/src/daemon.rs` | Remplacer la liste volatile `pending_replies` par les demandes ouvertes chargées depuis le store. |
| Persistance | `crates/bridget-daemon/src/store.rs` | Ajouter la table et les transitions dans la base SQLite existante. |
| Commandes opérateur | `crates/bridget-daemon/src/cli.rs` | Ajouter `cancel` et `requests` au parseur existant. |
| Lien avec la réponse | `crates/bridget-core/src/message.rs` et `crates/bridget-core/src/envelope.rs` | Ajouter une référence optionnelle au message plutôt qu'un second type de message métier. |
| Validation | `crates/bridget-daemon/tests/integration_test.rs` | Étendre les faux wrappers et les scénarios socket existants. |

## Recherche effectuée

- `rg` a confirmé que `pending_replies` n'existe que dans le daemon : aucune duplication de scheduler à fusionner.
- `Store` possède déjà l'initialisation de schéma, le ledger et la purge : aucune nouvelle base ou couche repository ne se justifie.
- `last-sender-*` est déjà écrit par le wrapper et consommé par `bridget reply` : il doit être enrichi du lien de demande, pas remplacé par une nouvelle mécanique de contexte.

## Gate avant tâches

- [x] Aucun service, composant, table ou endpoint parallèle ne doit être créé.
- [x] Les responsabilités proposées réutilisent des modules existants.
- [x] Aucun refactor structurel ambigu n'est requis.
- [x] Le plan respecte le périmètre coopératif défini par la spec.

## Risques

- Le daemon actuel corrèle une réponse par la paire expéditeur/destinataire. Le lien explicite `in_reply_to` est nécessaire pour que deux demandes parallèles restent indépendantes.
- Les rappels actuels utilisent `Instant`, non persistant. Les dates murales persistées doivent devenir la référence après redémarrage.
