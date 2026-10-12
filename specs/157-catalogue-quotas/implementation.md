# Implémentation157 — relecture T026

État de clôture préparé le 2026-10-12 : 26/26 sous réserve de la validation finale T026 et de la capture du diff par le principal. T025 et sa reprise exacte sont prouvées. Ce reçu ne coche aucune tâche. Aucun Converge, audit final, build ou livraison157 n’est déduit de ce reçu.

## Comportement et limites

Le catalogue et l’admission réemploient le même verdict. Le contrôle porte sur le type, le modèle, l’effort, le préflight, la politique héritée, les options154/155 puis le quota. Le catalogue publie can_run, constraints, quota, checks, features et verdict_context. La garantie exige le même parent, la même posture, un fait de session, le même dossier et un état inchangé. L’authentification reste non vérifiée. SameProject sans racine donne cwd:null et des booléens faux ; une admission autorisée dans un projet concret reste possible.

La mémoire de quotas garde la source et l’instant d’observation sous le nom agent_type du registre. Elle conserve huit fenêtres par type. Les fenêtres périmées sont ignorées en lecture et purgées à l’écriture. Une neuvième clé évince le fait le plus ancien ; la clé lexicale départage les égalités. Un retour futur et rejected Claude stream-json, ou un pourcentage au moins égal à 100, attestent l’épuisement. Le statut global Codex atteint peut rester informatif. Le retour global est le plus tardif des seules fenêtres épuisées. La mémoire survit à une présence, mais pas au daemon. Deux types partageant un compte ne partagent pas leur quota.

Le parent peut fournir une liste explicite de trois alternatives au plus. Seuls les neuf motifs FR013 permettent de remplacer la cible initiale. Un modèle, un effort, un droit ou une option incompatible de cette cible ne sont pas contournés. Les refus des alternatives explicites sont conservés dans tried. La tâche porte la cible effective ; son canonical conserve la requête originale. Le rejeu conserve la substitution. En worktree, Probe choisit et épingle une seule cible. Reserve et Commit revalident cette cible sans resélection. Les gardes de dossier154 et de reviewer155 restent actives.

Provider_quota_exhausted et fallback_exhausted ne créent pas de tâche et gardent la clé libre. Le failure existant décode leur porteur code:JSON borné. Les codes nus et les préfixes brief/workspace sont conservés. Le pont MCP conserve code puis detail. Aucun remplacement après admission ni reprise sur quota n’est ajouté : ce travail reste en 160.

## Réemploi et nouveaux éléments

| Élément | Preuve actuelle | Nécessité |
|---|---|---|
| Préflight commun | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/lifecycle.rs:102 | Extraction des contrôles existants ; submit_spawn et prepare_spawn_parts le réemploient. L’erreur build_environment est propagée. |
| Verdict et préparation R8 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon/native_delegation.rs:895 | resolve_target reste l’entrée fraîche ; target_resolver prépare une seule entrée pour les trois postures du catalogue. |
| Politique et lectures R8 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/native_permissions.rs:457 | child_policy et child_policy_reader partagent leurs branches. Capture/recheck/source sans règles sont mémorisés pour une seule entrée, sans cache durable. |
| Mémoire nouvelle prévue T004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/provider_limits.rs:25 | Trois usages réels : écriture au handler, lecture du catalogue, garde d’admission. Presence.rate_limits ne survit pas à la présence. |
| Écriture du quota | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon.rs:7085 | Le handler et RateLimitFact/RateLimitSource existants sont réemployés ; la présence fournit agent_type. |
| Substitution durable prévue | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/delegation.rs:164 | Task et son payload existants, avec défaut None ; aucune table ou migration nouvelle. |
| Refus enrichi | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon/native_delegation.rs:74 | Extension de failure, sans failure_with_detail ni nouveau type d’erreur. |
| Frontière MCP | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/delegation_mcp.rs:84 | Liste blanche et InvalidParams existants, avant connexion pour les formes invalides. |

Le module provider_limits compte 289 lignes, tests compris. Son algorithme parcourt au plus neuf fenêtres à l’écriture et huit en lecture, avec recherche O(log p). Le remplacement parcourt au plus quatre cibles. Le modèle est épinglé par pin_delegation_model existant avant le préflight. Aucun nouveau service, framework, outil, dépendance, CLI ou état Lineage n’est créé.

Les tests réemploient fixture_with_reader149, delegate_request, les vrais gestionnaires et la fausse frontière MCP. Le raccord de registre T007 a six consommateurs prévus. Les trois raccords privés T017 ont chacun au moins trois consommateurs dans les quatorze tests prévus. L’arbitrage R8 réemploie les corps existants pour respecter SC007 ; il évite un verdict miroir. L’audit de réutilisation actuel a été relu après les deux paragraphes ajoutés par le principal.

## Chemins d’erreur relus

Les validations d’identité, d’enveloppe, de délai, de pause et de droits précèdent la sélection. Type/modèle/effort précèdent les motifs remplaçables. Le premier refus contractuel initial reste prioritaire. Les erreurs d’environnement remontent avant réservation. Les JSON enrichis malformés reviennent au code canonique fermé, sans fabriquer de détail. Les invalidations entre Probe/Reserve/Commit refusent sans sélectionner une autre cible ; la réservation se libère par le mécanisme existant. Le rejeu ne réévalue pas un quota mutable et une autre liste conserve envelope_mismatch.

## Articles XIX et XX

1. Nécessité : chaque ajout correspond au catalogue, à la mémoire bornée ou à la sélection explicite prévus. Le module de mémoire a trois chemins de production réels.
2. Simplicité : une seule boucle bornée, Task/canonical existants, mêmes politiques et préflight extrait. Aucun état partagé supplémentaire pour R8.
3. Hypothèses : quotas observés seulement, temps de retour fourni, contexte de session inchangé pour la cohérence. Aucune authentification ni incident réel corrélé n’est supposé.
4. Vérification : tous les hunks du patch capturé et le module neuf entier sont lus. Les erreurs, la compatibilité et les mesures sont confrontées aux reçus ciblés.
5. Non vérifié : aucun fournisseur ou modèle réel, aucun login, aucune activation157. La suite complète T025 conserve huit échecs historiques ; la relecture ne vaut pas résultat de test global vert.
6. Éléments conservés : cache local R8 justifié par la mesure, raccords privés justifiés par leurs consommateurs, règles154/155 inchangées. Aucune ligne supplémentaire supprimable à comportement et scénarios constants n’est prouvée dans le diff relu. Les répétitions de constructeurs fallback vide/substitution None assurent la compatibilité ; elles ne justifient pas un framework.

## Périmètre de lecture et provenance

Lecture intégrale du patch /Users/moi/.cache/bridget-plan157-20261012-r1/final-source-capture/diff-157.patch : 3628 lignes, 29 fichiers, 243502 octets. SHA256 : e2ada299007e7f2dee379923e7a2022a72cc92f031d2b4a9cc6502e3bc22ef90. Base : 8b5b313694cf05eabfe6a86a970b96f03b015f49. Les deux fichiers nouveaux sont inclus explicitement ; les reçus validation sont exclus pour éviter une capture récursive. Tous les hunks et leur contexte sont lus ; les corps historiques inchangés de tous les fichiers ne sont pas prétendus intégralement relus.

Module provider_limits entier, 289 lignes, SHA256 c4bf39bdc15ec588a10fa855e203daba9bc56d647c1d8d53b36d81fc09ecf592. Reuse-audit actuel SHA256 : 179a75b67c93456f7b9b353183e18f7f9906db20a6a0be27c1d5b25b37b26a80. Source native_delegation SHA256 : 038ce196f2c7f4ed0ef0cb2f0e8882a5d8fd41a90a1032ccfcd333a2079dd04d. Le principal poursuit seul la capture finale et la validation T026.

Les 18 reçus ciblés antérieurs ont été relus. Les deux reçus T025 ajoutés à la clôture sont vérifiés séparément ci-dessous. Les 18 logs correspondent à leur SHA annoncé. Chaque reçu indique sources_stable:true ; ces faits restent attachés à leur exécution. Les comptes et la mesure sont dans validation/tests-157.md. Ils ne sont pas additionnés en tests uniques.

## Clôture T025 et delta final de fixture

La suite complète unique T025 est terminée : 2396 PASS Cargo, 9 FAIL, 74 ignored, 95 cibles, RC101 en 747,24 s. Trois PASS de processus enfants sont exclus du total Cargo. Les huit échecs anciens concordent avec FULL156 ; le neuvième était un oracle154 désormais aligné sur le contrat157. Sa reprise exacte passe 1/0 en 14,28 s, RC0. Les 44 tests native157 exécutés passent dans FULL ; le banc de latence reste ignored et est prouvé séparément. Il n’y a ni deuxième FULL ni GREEN global.

Le principal a seulement ajouté substitution:null à l’oracle complet native154_partage_discovery_termine_et_statut_compatibles150, champ additif exigé par T021, et ajusté son commentaire/assertion de version. La plage actuelle 411–431 a été relue. Aucun code de production n’a changé entre FULL et replay : les 217 empreintes des deux reçus concordent sauf ce fichier de tests. Son SHA final est 015db2e65365e27c9bb7aa6bc6027252fffbe7e0211b0576f8baeac9c73fb587. La capture complète précédente reste une preuve de lecture antérieure à ce delta ; le principal reste responsable de la nouvelle capture finale.

Reçus : /Users/moi/.cache/bridget-plan157-20261012-r1/T025-full157-once.json (SHA : 442d5b063c06fd01697bf32dfc5631356ebb053088c02b06ed2f0bedc1fa398c) et /Users/moi/.cache/bridget-plan157-20261012-r1/T025-replay154-compat157.json (SHA : 74fd117b4e4c09da91b34fdb2935b93a3e45b7efa931b2e0103356639552ee52). Les 217 sources sont stables avant/après chaque exécution. Logs SHA : 0a85192a923d137ecc0471306658d170980bb4717c826e23c3cdda92f676191e et 8775c1d2ad3c3a4f18cfb9e3f7d7c99d7092576c2bfacc8b9b1136fd136fe121. Les comptes ciblés antérieurs ne sont pas additionnés avec FULL : leurs ensembles se recouvrent.
