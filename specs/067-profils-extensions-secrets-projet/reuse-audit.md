# Audit de réutilisation de l'existant - Profils, extensions et secrets bornés par projet

## Decision

Statut: PASS contre `main` dda4ec2198b941cc38915a00f847df435d80934d
Date: 2026-08-30
Feature dir: /home/moi/bridget-referent/.worktrees/session-067-profils-extensions-secrets-projet/specs/067-profils-extensions-secrets-projet

Conclusion courte: les profils agents, approbations, définitions résolues,
capabilities, gardes d'environnement et runtime Docker 066 existent déjà sur
la tête livrée. Aucun registre de profil projet, ExtensionRef, SecretRef,
catalogue de ressources ni redaction streaming n'existe dans le code. Le plan
étend ces points d'extension sans créer de broker ou service externe.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 18 |
| Items audites | 18 |
| Reutilisations deja prevues | 14 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| AgentProfile déclaratif | `ProfileConfig` | `plugins/maicie/src/config.rs` | Référencer/étendre, pas dupliquer. |
| Profil chargé | `LoadedProfile` | `plugins/maicie/src/profiles.rs` | Conserver agent/model/effort/tools. |
| Vue approbation | `ApprovalProfileView` | `plugins/maicie/src/profiles.rs` | Étendre à toutes les ressources projet. |
| Digest définition | `definition_digest_matches` | `plugins/maicie/src/profiles.rs` | Réutiliser comparaison binaire. |
| Proposition/approbation | app Maicie | `plugins/maicie/src/app.rs` | Étendre le flux local existant. |
| Approbation durable | store Maicie | `plugins/maicie/src/store.rs` | Conserver command_id et octets. |
| Registre agents | `AgentDefinition` | `crates/bridget-daemon/src/registry.rs` | Autorité commandes/capabilities/env. |
| Env interdit/allowlist | `forbidden_env`, `pass_env` | `crates/bridget-daemon/src/registry.rs` | Secret env ne contourne pas ces gardes. |
| Construction env | `build_environment` | `crates/bridget-daemon/src/lifecycle.rs` | Injecter seulement après validation. |
| Définition résolue | contrat protocol | `crates/bridget-transport/src/protocol.rs` | Étendre digest, pas second format. |
| Greffe autorisation | gate/audit privé | `crates/bridget-transport/src/greffe_authorization.rs` | Réutiliser style d'audit local. |
| Runtime mounts | ProjectRuntimePolicy 066 | `specs/066-environnement-partage-projet/data-model.md` | Ajouter refs au digest et attestation. |
| Runtime wrapper | spawn provider | `crates/bridget-daemon/src/wrapper.rs` | Lire process-env dans le wrapper. |
| Profils Codex/Claude/Cursor | native registry | `crates/bridget-daemon/src/registry.rs` | Même contrat provider-neutre. |
| Cursor | transport ACP | `crates/bridget-daemon/src/registry.rs` (`AcpTransport`) | Pas d'adaptateur spécial. |
| State root projet | ProjectEnvironment 066 | `specs/066-environnement-partage-projet/spec.md` | Écritures et mémoire restent par projet. |
| Projections | UI Maicie/Bridget | `plugins/maicie/src/ui_projection.rs` | Étendre sans valeur. |
| Incidents délégués redacted | `ManagedEventKind::Diagnostic` et `DelegatedRuntimeEventFrame` | `crates/bridget-transport/src/managed_session.rs`, `crates/bridget-transport/src/protocol.rs` | Réutiliser code/référence, interdire tout octet secret. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| ProjectProfile | ProfileConfig | `plugins/maicie/src/config.rs` (`ProfileConfig`) | Composer les profils existants, nouvelle agrégation justifiée. |
| ExtensionRef | personality_ref/tools | `plugins/maicie/src/config.rs` | Généraliser les références versionnées sans plugin manager. |
| SecretRef | pass_env | `crates/bridget-daemon/src/registry.rs` (`pass_env`) | Garder allowlist, déplacer la valeur hors env daemon. |
| Secret audit | GreffeAuthorizationGate | `crates/bridget-transport/src/greffe_authorization.rs` | Réutiliser les invariants d'audit, pas l'autorité métier. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucune | Aucun profil projet ou SecretRef 1:1 | N/A | Maintenir le gate avant nouvelle abstraction. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Privacy gate et minimalisme | Valeurs hors revue et aucun broker. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Validation frontières et dépendances | Sources, targets et capabilities validés. |
| `/home/moi/.speckit/ref/adversarial-review.md` | Review hostile secrets/infra | Fuite, permissions et montages testés. |
| `/home/moi/.speckit/ref/standards-tests.md` | Gherkin et tests comportementaux | Feature 067 et scanners réels. |
| `/home/moi/.speckit/ref/standards-observability.md` | Redaction et métriques | Valeurs exclues, raisons bornées. |
| `/home/moi/.speckit/research/01-ai-agents-agentic-ai.md` | Éviter l'autonomie non prouvée | Aucun plugin manager agentique. |
| `/home/moi/.speckit/research/05-knowledge-management.md` | Freshness et contamination | Mémoire globale différée. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-011 | Profils et délégations Maicie | Autorité métier. |
| SPEC-015 | Guichet et fraîcheur | Pas d'approbation distante. |
| SPEC-026 | Greffe central autorisé | Audit local et raisons structurées. |
| SPEC-064 | Capabilities fournisseurs | Admission avant secret. |
| SPEC-065 | Identité projet | Portée stable. |
| SPEC-066 | Runtime et mounts | Recreate et attestation. |
| SPEC-068 | Incident runtime délégué redacted | Scanner frames et store; aucun octet fournisseur ou secret. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "ProfileConfig|LoadedProfile|ApprovalProfileView" plugins/maicie/src` | profils | Modèle et vue existants. |
| `rg -n "pass_env|forbidden_env" crates/bridget-daemon/src` | env | Gardes et allowlists existantes. |
| `rg -n "secret|credential|plugin|skill" crates plugins` | codebase | Pas de registre project-scoped 1:1. |
| `rg -n "GreffeAuthorizationGate|append_audit" crates/bridget-transport` | audit | Pattern privé et fail-closed existant. |
| `rg -n "cursor|AcpTransport" crates` | provider | Cursor déjà sous ACP commun. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Profils agents | réutiliser | Autorité et approbation déjà présentes. | 2026-08-29 |
| Stockage valeurs | refuser | Évite clé maître et service secret maison. | 2026-08-29 |
| Secret par agent | différer | Impossible à garantir dans conteneur partagé. | 2026-08-29 |
| Mémoire globale | différer | Autorité/confidentialité non spécifiées. | 2026-08-29 |
| Plugin téléchargement | refuser | Provenance et reproductibilité exigées. | 2026-08-29 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Complément cible RC8

Les profils, AgentRegistry, allowlists d'environnement, wrapper et politique
runtime restent les abstractions à étendre. Aucun resolver hôte de références
de ressources n'existe actuellement: un catalogue de configuration fermé est
donc justifié, sans daemon, store partagé ou service de secrets nouveau.

Le runtime 066 doit fournir le même UID/GID numérique que celui attesté pour
les secrets privés. La vérification d'un fichier fixture `0600` est ajoutée au
contrat commun afin d'éviter une conception correcte sur le papier mais
illisible dans le conteneur réel.

SecretSourceStamp réutilise les métadonnées filesystem et le digest canonique
déjà nécessaires aux attestations. Il ne crée ni hash de valeur secrète, ni
broker, ni mécanisme de rotation automatique.

## Rejeu sur main dda4ec2 après SPEC-066

`ProjectRuntimePolicy` et les montages attestés sont productifs dans
`crates/bridget-daemon/src/project_runtime.rs:292`, `:689`, `:785` et `:1147`.
`ProjectMount` porte déjà le caractère read-only dans `:548`, mais il n'existe
pas de catalogue ni de validation de ressource projet. `AgentDefinition` reste
l'autorité des capabilities et de `forbidden_env`/`pass_env` dans
`crates/bridget-daemon/src/registry.rs:44`; `build_environment` est le seul
constructeur d'environnement historique dans `crates/bridget-daemon/src/lifecycle.rs:506`.

`ProfileConfig`, `LoadedProfile`, `ApprovalProfileView` et la comparaison de
digest existent dans `plugins/maicie/src/config.rs:259` et
`plugins/maicie/src/profiles.rs:17`, `:62`, `:168`. La proposition et
l'approbation locales existantes sont dans `plugins/maicie/src/app.rs:963` et
`plugins/maicie/src/store.rs:4416`; aucun second registre ne sera créé.

SPEC-068 ajoute `ManagedEventKind::Diagnostic` et
`DelegatedRuntimeEventFrame` dans
`crates/bridget-transport/src/managed_session.rs:104` et
`crates/bridget-transport/src/protocol.rs:1272`. Ces sinks sont inclus dans les
scanners T023/T029, sans permettre les octets fournisseur ou secret.
