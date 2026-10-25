# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/reuse-audit.md](../../../../../../specs/163-decision-admission/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Type | Élément | Preuve | Décision |
|---|---|---|---|---|
| 1 | N | decision_services | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs:89 | Section privée distincte, absente du registre actuel. |
| 2 | N | admission_policy | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs:144 | Politique absente ; deux axes false par défaut. |
| 3 | R | Profils moteur/modèle/effort | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:2200 | Réutiliser matrice et validation. |
| 4 | P | Lecture de politique | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon.rs:9761 | Décision ROOT : lecture fraîche par admission, puis snapshot durable ; aucun watcher. |
| 5 | R | Filtre local | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:2170 | target_resolver réemployé. |
| 6 | N | Probe groupé distant | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs:2698 | Lecture nouvelle nécessaire car Prepare crée un worktree. |
| 7 | P | Capacité par époque | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs:2441 | Décision ROOT : Hello/AcceptedWithCapabilities après feature maître ; v1 inchangé. |
| 8 | R | Mapping/racines protégées | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs:1715 | Extraire les mêmes gardes. |
| 9 | R | Binaire/modèle | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/lifecycle.rs:102 | launch_preflight sans fournisseur. |
| 10 | R | Projection/recheck droits | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/native_permissions.rs:73 | Mêmes primitives ; recheck :141. |
| 11 | R | Préflight worktree | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs:384 | Git readonly borné ; création réservée à Prepare. |
| 12 | R | Admission du seul élu | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:1005 | delegate_remote, aucun reroutage après choix. |
| 13 | N | Reçu avant Task | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:409 | Table technique dans la même SQLite. |
| 14 | R | Clé owner/request | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:414 | Demandeur durable attesté ; conserver aussi unique instance/request. |
| 15 | P | Collision native/auto | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:878 | Décision ROOT : Task/refus/tombstone/auto ; autorité interne chosen non forgeable. |
| 16 | P | Task et lien atomiques | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs:445 | Décision ROOT : write-in-tx Immediate et publication après commit. |
| 17 | R | Compteurs actifs | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:1041 | Étendre sans double compte ; distinct de Fleet. |
| 18 | R | UNION 4096 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs:28 | Adapter comptage et pression de collecte, pas somme des tables. |
| 19 | R | Collecte/tombstones | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs:402 | Retirer reçu lié dans la même transaction ; garder gardes de vivacité. |
| 20 | R | Refus/rétention | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:913 | Cinq codes natifs inchangés ; refus auto séparés. |
| 21 | R | Canon/framing | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/jsonl.rs:12 | 512 Kio sous 16 Mio ; worker 2 Mio, enveloppe et LF inclus. |
| 22 | N | auto_delegate | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs:2157 | Variante native distincte, capacité requise. |
| 23 | R | MCP/capacité client | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs:315 | Probe feature avant variante fermée. |
| 24 | P | CLI automatique | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli.rs:269 | Décision ROOT : auto-delegate native, guichet historique intact. |
| 25 | N | decision_service | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/t3code_mcp.rs:136 | Module frontière HTTP spécialisé ; client existant, pas runtime neuf. |
| 26 | N | Adaptateur Laya | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/contracts/decision-service.md | Seul protocole étudié ; Jev non attesté. |
| 27 | R | HTTP borné | /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/request.rs:239 | send_lazy et bornes corps/en-têtes/statut/redirects. |
| 28 | P | Deadline/permis | /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:351 | Décision ROOT : sans with_timeout, tâche unique, permis jusqu’au vrai retour. |
| 29 | N | Observation de santé | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/data-model.md | Volatile et bornée, aucun cache décision existant. |
| 30 | R | TLS minreq | /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/Cargo.toml:52 | Feature https-rustls du même client ; validation certificat. |
| 31 | R | Lineage v3 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/lineage_protocol.rs:21 | List/show existants ; v1/v2 inchangés. |
| 32 | R | Lecteur T3 | /Users/moi/11.Repositories/t3code-local/apps/server/src/bridget/BridgetReader.ts:490 | Surfaces versions/replis :532, détails :617. |
| 33 | R | Infobulle | /Users/moi/11.Repositories/t3code-local/apps/web/src/components/chat/SubagentTooltipContent.tsx:41 | Même surface, aucun composant global. |
| 34 | R | CLI Lineage versionnée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli_lineage.rs:117 | Étendre le lecteur existant. |

<a id="extrait-2"></a>

## Extrait 2

1. rg --files sur registre/délégation/worker/workspace/protocol/Lineage/Cargo : surfaces retenues.
2. rg prepare/prepare_workspace/capabilities : effets du Prepare et gardes pures.
3. rg CREATE TABLE/Immediate/owner/tombstone/purge : unicités, transactions, compteurs et collecte.
4. rg minreq/auto_delegate/candidate_probe/response_version/reload dans sources et plans150–162 : nouveaux noms absents, HTTP et versions existants.
5. Cache minreq2.14.1 : send_lazy/timeout/max_headers/redirect ; lecture des garanties réelles.
6. AgentRegistry::load et affectations registre : chargement maître initial contre lectures worker par opération.
7. CLI/MCP delegate/delegation : absence de commande native delegate ; guichet distinct.
8. Plans150–162 : rétention, reprise, worktrees, droits, limites, worker et Lineage déjà livrés.
9. T3 : fichiers BridgetReader, contrats et infobulle, puis versions/replis.
10. Mémoires/ponts/scripts : absences initiales puis sync ROOT lu ; check-prerequisites absent, non exécuté par l’auditeur.

<a id="extrait-3"></a>

## Extrait 3

Les trois frontières nouvelles sont justifiées : admission avant Task, probe sans effets métier, réponse HTTP non fiable. L’ADR proposée explique leurs alternatives et leur coût de maintenance. Aucun axe de recherche supplémentaire ni test facultatif ajouté. Endpoint/checkpoint et profils réels restent des gates d’activation ; le plan et la spec sont Ready après GO ROOT sur les corrections ; la proposition de types demeure à figer par le développeur avant consommation T3.

<a id="extrait-4"></a>

## Extrait 4

| Point | Arbitrage transmis par ROOT et intégré |
|---|---|
| O1 | minreq sans with_timeout ; MINREQ_TIMEOUT présent interdit le seul service. Garde démarrage/préappel, raison visible ; blocages DNS/connexion/TLS/écriture/lecture peuvent conserver le permis indéfiniment. |
| O2 | Entretien existant propriétaire, lots/CAS/E/S hors mutex après réconciliation, sans demandeur et sans HTTP de reprise ; pending unique/repli/refus, decision_started repli/refus, chosen idempotent. Erreurs techniques/SQL visibles et entretenues ; refus après chosen terminal auto. |
| O3 | Refus auto terminaux sans Task purgeables sous pression du plus ancien au plus récent, tombstone trente jours atomique et request_expired après purge. Purge sûre avant refus natif/auto au plafond ; aucun état vivant ou ambigu sacrifié. |
| O4 | v3 → v2 uniquement unsupported_version ; v2 → v1 garde unsupported_version ou invalid_request structurés ; pages homogènes, journal/cancel sans repli. |
| O5 | Reçu auto consulté avant by_agent_request par le natif externe sous Immediate ; refus interne chosen seulement dans reçu auto, unicités inchangées. |
| O6 | Rust/Linux sur Cartae isolé du worker actif ; T3/macOS/signature sur Mac, recette décideur depuis maître Mac, commit source commun et compilation unique à l’échelle de la flotte. |
