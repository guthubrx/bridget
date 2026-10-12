# Audit160 — reprise sur quota

Session12octobre2026. Modefix : un cycle de correction et une lecture de scoring distincte. Scope21Rust du diff ; noteA99,65. Lecture ciblée : 6.22% des hunks entièrement relus (12/193), 11partiels ; 1564lignes uniques et0fichier entier. Ce pourcentage n’est pas une couverture de tests. Les18SHAinchangées n’accordent aucun crédit frais.

## Résumé exécutif

0CRITICAL,0HIGH,1MEDIUM,1LOW. REL001 est corrigé et vérifié par lecture fraîche des champs partiels et le vrai T0351PASS. PERF001 reste un coûtO(T×P) sousState, borné et non mesuré. DUP001 reste un doublon froid de143lignes dans deux oracles distincts. Aucun refactor/scénario de confort.

| Domaine | Note | Numérique | Findings |
|---|---|---:|---:|
| security | A | 100 | 0 |
| llm_security | A | 100 | 0 |
| supply_chain | A | 100 | 0 |
| observability | A | 100 | 0 |
| reliability | A | 100 | 0 |
| tests | A | 100 | 0 |
| performance | A | 100 | 0 |
| complexity | A | 97 | 1 |
| quality | A | 100 | 0 |
| architecture | A | 100 | 0 |
| ai_hygiene | A | 100 | 0 |
| duplication | A | 99 | 1 |
| minimalism | A | 100 | 0 |

UX:N/A. Pondération officielle totale20 ; complexité et duplication pénalisées séparément sans compterPERF deuxfois. La note décrit les constats trouvés dans cet échantillon, pas le code non lu.

## Tendance

C1historique0C/0H/2M/1L→scoring0C/0H/1M/1L. REL001e7f5155c4440 résolu. PERF0014a596aab67b4 et DUP00144c679c11265 persistent. Extraits et ancres recalculés sur la capturefinale ; JSONproducteursC1 préservés. Pas de comparaison inter-session inventée.

## Tests et limites

FULLunique historiqueRC101,900,75s :2683PASS/6FAIL/76ignored sur97ciblesparentes. SeptPASS de sous-processus exclus. Les sixFAIL viennent de l’oracle154 sansquota_wait:null et de cinq tests exigeant encoreversion11. Deux fichiers de tests seulement corrigés ;rejeuxdistincts1541PASS/7,88s et migrations5PASS/4,99s,RC0,sourcesstables. Aucun deuxièmeFULL niGLOBALGREEN revendiqué. Les19sources présentes pendantFULL sont inchangées après ces correctifs de tests.

La migration12 est exigée par T011 et le test native160_execution_quota_creation. Les anciennes assertions restent strictes, seules les valeurs prévues ont changé. Pas de résultat de modèle réel, quota réel, T3/production ou mémoire d’un modèle. Couverture de tests/mutation/performance mesurées:null. Les zones non comprises dans les read_manifest restent non examinées.

## Minimalisme

Potentiel minimalisme : 0 ligne de production démontrée supprimable à comportement constant dans les plages relues. Le doublon froid reste consigné ; aucune suppression supplémentaire prouvée.

## Vertus LLM et responsabilité future

Les checkpoints accroissent la charge de lecture mais explicitent les frontières de reprise. Les volumes relus correspondent aux invariants crash/ACK et aux tests prescrits. Les extractions portent une règle documentée, sans framework ajouté. Identité, corrélation,ACK et erreurs restent explicables. Les zones horsplages et la production ne sont pas jugées. L’auteur deT019 a relu son helper : cela ne remplace pas les contre-revues externesROOT/C1/Claude.

## Points positifs

- crates/bridget-daemon/src/referent_control.rs:420 — Union triée des intervalles, Q≤2, chevauchement non additionné.
- crates/bridget-daemon/src/provider_limits.rs:81 — Chaque fenêtre est filtrée avant les deux maxima.
- crates/bridget-daemon/src/provider_limits.rs:91 — Petit prédicat157 dupliqué avec justification explicite.
- specs/160-reprise-quota/reuse-audit.md:124 — Reconstruction ciblée justifiée, sans nouveau DTO de rejeu.
- crates/bridget-daemon/src/wrapper.rs:5955 — Helper porte la corrélation avant retrait du binding.
- crates/bridget-daemon/src/execution_store.rs:1060 — Qualification par clés uniques et paramètres SQL liés.
- crates/bridget-daemon/src/referent_control.rs:420 — Union des pauses/quota par tri unique.
- crates/bridget-daemon/src/execution_store.rs:1456 — Enfant, continuation et lien résumés committés ensemble.
- crates/bridget-daemon/src/daemon.rs:6512 — Replayed relit enveloppe, lien et phase sans nouvelle gravure.
- crates/bridget-daemon/src/daemon/native_delegation.rs:2917 — Chaque champ présent divergent est refusé avant les règles de champ absent.
- crates/bridget-daemon/src/execution_store.rs:1428 — Rejeu AlreadyResumed avant nouvelles mutations ; transaction Immediate et identité liée.
- crates/bridget-daemon/src/delegation_mcp.rs:134 — Opt-in uniquement booléen ; forme étrangère refusée.
- crates/bridget-daemon/src/daemon/native_delegation.rs:6253 — Cas partiels divergents exécutent le vrai handler et comparent Task, suivi et silence.
- crates/bridget-daemon/src/daemon/native_delegation.rs:16697 — ACK réel précède Done et relecture du magasin.

## Baseline et suite

Baseline absente, aucune suppression écrite. AUTO_COMMITfalse. Aucun test/build/scanner/benchmark/Git/service/DB/modèle lancé par cet owner. ROOT assure livraison et cleanup après ses gates. PERF001 etDUP001 restent signaux nonbloquants, sans waiver de régression.
