# Audit de réutilisation de l'existant - 157-catalogue-quotas

## Decision

Statut: PASS
Date: 2026-10-11 (audit historique ; réattestation ciblée du2026-10-12 ci-dessous)
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/specs/157-catalogue-quotas

Conclusion courte: Le plan réutilise le préflight de `submit_spawn`, les codes de
`refusal_record`, `child_policy`, les sources de `rate_limits`, l'enveloppe
`canonical`. La reprise sur quota est sortie (160). Un seul module nouveau (`provider_limits.rs`),
justifié par 4 usages. Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 11 |
| Items audites | 11 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Préflight de disponibilité | contrôles de `submit_spawn` (capacités, binaire) ET de `prepare_spawn_parts` (facturation, protocole, environnement) | `crates/bridget-daemon/src/lifecycle.rs:209-226`, `:342-380`, `command_exists` `:542` | extraction des deux, pas de nouvelle vérification |
| Codes de refus de disponibilité | `refusal_record` | `lifecycle.rs:561` | mêmes noms (`command_missing`, `billing_guard`, `env_unfit`, `unsupported_capability`) |
| Politique de l'enfant | `child_policy` | `crates/bridget-daemon/src/native_permissions.rs:206` | appelée par le verdict, aucune règle de droits neuve |
| Sources de quota | `handle_rate_limit` et ses 3 sources | `daemon.rs:6600-6660`, `claude_stream_json.rs:1355`, `codex_app_server.rs:2643`, `cli.rs:4999` | une écriture ajoutée au point unique |
| Faits de limite | `RateLimitFact` | `bridget-transport/src/protocol.rs:4843` | structure réutilisée telle quelle |
| Enveloppe de rejeu | `canonical` stocké à part de `request` | `crates/bridget-daemon/src/delegation.rs:114-130` | permet de réécrire la cible effective |
| Ordre des refus | contrat 150 | `specs/150-socle-delegation/contracts/delegation150.md` | insertion de la sélection de cible |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Mémoire de limites | `Presence.rate_limits` | `daemon.rs:566` | Non réutilisable seul : disparaît avec la présence, sans instant d'observation. Module neuf justifié (research R4) |
| Détail dans un refus | `failure(code)` | `native_delegation.rs:34` | Étendre le `failure` existant par code:JSON, sans helper supplémentaire |
| Sonde d'authentification | aucune | `rg` credential, login, unauthenticated : rien de pertinent | Non créée (question 3) |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/reuse-audit.md#extrait-1).

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id` | enveloppe étendue sans changer les octets sans nouveau champ |
| 149-sous-agents-lineage | droits hérités, Lineage à états fermés | aucun nouvel état Lineage |
| 150-socle-delegation | refus nommés, ordre des refus, `reply_timeout` | ordre étendu |
| 116 (entretien) | faits republiés par les sources | justifie une mémoire volatile |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/reuse-audit.md#extrait-2).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Authentification | non vérifiée, dite dans `checks` | aucune sonde sans appel fournisseur | 2026-10-11 |
| Où mémoriser les limites | module `provider_limits.rs` | 4 usages, `daemon.rs` trop gros | 2026-10-11 |
| Persistance des limites | volatile | les sources republient | 2026-10-11 |
| Cible effective | écrite dans `record.request` | `canonical` garde l'enveloppe | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Reporté en 160

Le réemploi des primitives de reprise de 152 (`latest_in_chain`, `may_resume`,
`schedule_execution_recovery`, `native_child_env`) est audité en 160, quand 152
est livrée et ses primitives prouvées (`spec.md`, section « Session future 160 »).

## Réattestation ciblée au GO157 — 2026-10-12
La base livrée est 8b5b313694cf05eabfe6a86a970b96f03b015f49, sources156 cf498755dc871b51eb8c0add5f8a882d67651d2a.
Dix sources ont été relues par plages et sont restées stables. Cette lecture ne valide ni une baseline157, ni un incident réel, ni le dépôt entier.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/specs/157-catalogue-quotas/validation/readiness157-after156.json
SHA256 : 9de506e51b9d3b00a7e2b7cf3ab76c35453b29181e091f7e24560e5ea5fb7654.
Les trois passes Probe, Reserve et Commit conserveront le candidat et la définition épinglée. Commit refusera une invalidation sans nouvelle sélection.
Le refus du parent child_requests_reviewer_unsupported reste hors fallback. Les contrôles de dossier et la politique héritée restent requis.
Le détail code:JSON étendra le failure existant. Les diagnostics154 de chemin et de branche resteront conservés.
Le seul helper privé proposé pour T007 construit le registre de quatre alias. Ses six consommateurs sont les six scénarios déjà prévus. Il réemploie la fixture149 ; aucun verdict miroir, nouvelle API ou infrastructure de test.


## Arbitrage R8 et raccords des tests157 — 2026-10-12
- R8 : réutiliser les corps de resolve_target et child_policy, puis extraire target_resolver/child_policy_reader. Les trois politiques gardent leurs branches ; seules les lectures/préflight liés aux mêmes entrées sont préparés une fois. Deux consommateurs de contexte (admission fraîche et catalogue trois postures), avec la contrainte architecturale écrite R8/SC007. La première mesure dépassait le seuil ; aucun cache durable, DTO ou verdict parallèle créé. Preuves : manifest R8 en cache, mesure-catalogue.json et matrice T007. Les équivalents sont les fonctions existantes, conservées comme entrées fraîches.
- T017 : réutiliser fixture_with_reader149, delegate_request et les gestionnaires réels. Les trois raccords privés fallback157_definition/registry/request ont chacun au moins trois consommateurs dans les quatorze tests déjà prévus ; aucun scénario ni infrastructure supplémentaire.
