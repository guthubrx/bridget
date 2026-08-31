# Audit de réemploi - SPEC-081

Date : 2026-08-31
Statut : PASS

## Inventaire du plan

| Besoin proposé | Existant recherché | Décision | Preuve |
|---|---|---|---|
| Identité opaque et profil | AgentProfileStore | RÉUTILISER | crates/bridget-daemon/src/agent_profile.rs:173-226 crée agent_identities et agent_profiles, avec UUID, display_name, labels, instructions et préférences |
| Projection UI de profil | relais UI v1 | RÉUTILISER puis faire évoluer v2 | crates/bridget-daemon/src/ui.rs:3496-3535 joint déjà profil et agent |
| Routage | Router | RÉUTILISER et renommer ses clés | crates/bridget-core/src/router.rs:78-205 indexe aujourd'hui les connexions par name |
| Contrat annuaire | AgentInfo | RÉUTILISER et faire évoluer | crates/bridget-transport/src/protocol.rs:2647-2790 |
| Prompt fournisseur | wrapper | RÉUTILISER et modifier | crates/bridget-daemon/src/wrapper.rs:607-632 injecte encore name |
| Données de message | LedgerStore | RÉUTILISER et migrer | crates/bridget-daemon/src/store.rs:534-555, 3131-3150 |
| Flotte durable | DesiredStateStore, FleetSupervisor | RÉUTILISER et migrer les clés | crates/bridget-daemon/src/desired_state.rs, crates/bridget-daemon/src/fleet.rs:235-253 |
| Migration SQLite Maicie | MaicieStore | RÉUTILISER | plugins/maicie/src/store.rs:56-58, 8174-8837 |
| Publication protégée Maicie | install_publish | RÉUTILISER | plugins/maicie/src/install_publish.rs:146-220 |
| Profils Maicie | ProfileConfig et LoadedProfile | RÉUTILISER et remplacer agent_name | plugins/maicie/src/config.rs:264-293, plugins/maicie/src/profiles.rs:15-36 |
| Délégation/routine/outbox Maicie | Domain et store existants | RÉUTILISER et modifier | plugins/maicie/src/domain.rs:2326-2365, plugins/maicie/src/store.rs:8230-8247 |
| Commande de migration Bridget | aucun équivalent ciblé | CRÉER dans CLI existante | recherche migration dans crates/bridget-daemon/src/cli.rs : sauvegarde ponctuelle seulement, aucune migration d'identité |

## Risques constatés

1. AgentProfileStore possède déjà agent_id mais le remplit depuis une table d'alias de routage. Cette table doit être employée uniquement comme source d'import puis supprimée.
2. Le stockage de messages et de flotte est réparti entre SQLite, JSON et wrappers. Une migration en une seule transaction globale est impossible entre fichiers. Le comportement sûr est donc : préflight complet, sauvegarde de tous les fichiers, transaction par ressource, journal de progression et refus de redémarrage si le journal est incomplet.
3. Maicie stocke certaines délégations comme payload JSON avec des index SQL. Toute migration doit vérifier que payload et index restent cohérents, conformément aux migrations existantes.
4. Les principaux human et system ne sont pas des agents. Les convertir en UUID créerait un faux agent et violerait la décision de modèle.
5. Les clients et wrappers antérieurs au contrat v2 ne doivent pas être maintenus par une couche alias. Ils sont explicitement refusés après la migration.

## Arbitrages

| Sujet | Décision | Raison |
|---|---|---|
| Nouveau registre d'identité | Non | AgentProfileStore est déjà l'unique détenteur adaptée de agent_id et display_name. |
| Alias legacy permanent | Non | L'objectif utilisateur est de supprimer la confusion. |
| Migration globalement atomique | Non, par ressource sauvegardée | SQLite et fleet.json ne peuvent pas partager une transaction. Préflight, sauvegarde et journal offrent une reprise sûre sans prétendre à une atomicité impossible. |
| État des références supprimées | requires_retarget | Jamais de réattribution sur homonymie ou nouvel agent. |

## Gate avant tasks

- [x] Les tables/projections d'identité existantes ont été lues.
- [x] Les stores Maicie et leur mécanisme de migration ont été lus.
- [x] Aucun doublon de registre, store ou UI n'est proposé.
- [x] Le seul nouvel élément est une sous-commande CLI de coordination, nécessaire pour orchestrer des stores existants.
- [x] Aucun arbitrage utilisateur restant n'est nécessaire.
