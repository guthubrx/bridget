# Audit de reutilisation de l'existant — 102-fils-inter-agents

## Decision

Statut: PASS
Date: 2026-09-16
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents

Conclusion : aucune reconstruction d'un fil collectif existant détectée. Le plan
étend le daemon, son stockage et ses façades ; il garde un état métier propre
pour les obligations que les observations et le ledger ne couvrent pas.
L'intégration101 reste une précondition future de code, pas un doublon à arbitrer.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 21 |
| Items audites | 21 |
| Reutilisations deja prevues | 11 |
| Existants potentiellement pertinents | 10 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

Preuves sous la racine /Users/moi/Nextcloud/10.Scripts/64.bridget ; `D/` désigne
crates/bridget-daemon/src/ ; les lignes101 désignent explicitement le worktree101.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

Les dix candidats proches sont désormais tous mentionnés et tranchés dans le plan.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/reuse-audit.md#extrait-2).

## Duplications evidentes

Aucune. Le module SQL et les tables nouvelles ne sont pas des copies du journal,
du ledger ou des abonnements : les invariants et droits diffèrent. Aucun package,
daemon, broker, outil de résumé ou nouveau framework n'est proposé.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | isolation, français, pas d'interruption fournisseurs | worktree102, docs seules |
| /Users/moi/.speckit/constitution.md XVIII | complexité | index, pages et fanout bornés |
| même fichier XIX–XX | réutilisation, responsabilité | état spécialisé justifié, aucun wrapper décoratif |
| .specify/memory/constitution.md | pont global | constitution lue, aucune règle locale inventée |
| .specify/memory/standards.md et standards-tests.md | tests traçables | BDD + Rust natif, pas pytest hors technologie |
| skill my-specify-all + demande utilisateur | pas de commit ; sans implémentation | arrêt après Analyze, tâches non cochées |

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/reuse-audit.md#extrait-3).

## Journal de recherche

Exploration unique déléguée à reuse_102 en lecture seule, résultats reçus avant
tasks. Lecture ciblée principale du protocole, Store et CLI en complément.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/reuse-audit.md#extrait-4).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Fil dans ledger | créer tables spécialisées | ledger lisible plus largement et schéma1:1 | 2026-09-16 |
| Idempotence post | créer thread_operations | Send écrit ledger et une seule livraison ; pas même opération | 2026-09-16 |
| Mention comme abonnement | créer thread_wakes | once consommé avant livraison, reprise interrupted | 2026-09-16 |
| Queue nouvelle | réutiliser remise099 | seules intentions métier nouvelles, aucun broker | 2026-09-16 |
| Résumé | réutiliser agent/history/post | pas de fonction de génération supplémentaire | 2026-09-16 |
| Négociation de la capacité d'alerte | créer la variante `ThreadNoticeCapability` (fait post-enregistrement) au lieu de champs `Register`/`Registered` | même sémantique (connexion, renégociation), ~105 littéraux évités ; preuve : `wrapper.rs` connect_and_register_at, `daemon.rs` thread_notice_versions | 2026-09-17 |
| Isolation des tests | réutiliser `tests/support/idempotent.rs` (racine, daemon, clients, preuves) ; ajouter seulement `stop_cooperatively` et `spec102_root` | arrêt coopératif sans SIGKILL exigé par T002 ; le reste existait | 2026-09-17 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Le plan a été rédigé directement sur ces résultats ; aucun refactor de réutilisation
postérieur nécessitant arbitrage utilisateur. Gate PASS avant création tasks.md.
