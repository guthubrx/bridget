# Contre-revue de plan SPEC-163 — ronde 1

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

**Date :** 2026-10-13. **Agent/fournisseur :** Claude Opus 5.5 / claudeAgent.
**TaskId :** node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-plan-review-20261013-r1
**Verdict reçu :** APPROVE_WITH_CHANGES. **Statut initial du plan :** Draft ; après corrections et GO ROOT : Ready. ADR retenue par ROOT. Aucun APPROVE final déduit des corrections.

ROOT a reçu la revue, vérifié les six objections et transmis les choix ci-dessous. Le rédacteur a appliqué les corrections documentaires, sans nouvelle lecture de code présentée comme preuve fraîche de ces objections.

| Objection | Preuve vérifiée par ROOT / choix retenu | Limite conservée |
|---|---|---|
| O1 — fin réelle du travail minreq | Source /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:72, :111, :197–207, :351–380. Exclure with_timeout ; la présence de MINREQ_TIMEOUT même vide/invalide interdit activation et réseau du seul service, au démarrage et avant appel. Le daemon et le Delegate explicite continuent. Raison visible ; aucune suppression globale de variable. | DNS, connexion, TLS, écriture ou lecture serveur peuvent retenir le permis indéfiniment. Demandeur deux secondes, suivants repli/refus immédiat, aucun thread nouveau. Une IP littérale ne prouve pas qu’un thread détaché est terminé. HTTPS via même minreq. |
| O2 — reçus orphelins et refus ultérieurs | ROOT retient l’entretien existant comme propriétaire après réconciliation du redémarrage, indépendamment du demandeur : tous reçus d’époque précédente traités par lots/CAS/E/S hors mutex. pending probes figés puis unique/repli autorisé ou decision_interrupted ; decision_started repli/refus ; chosen admission idempotente + lien atomique. | Aucun HTTP de reprise ni nouveau scheduler. Tout refus après chosen terminal auto, même task_limit/pause/retention_saturated. Parent/droits indisponibles : aucun lancement. Erreurs techniques/SQL restent visibles, non résolues et entretenues ; aucune fausse clôture ni purge d’état ambigu. |
| O3 — rétention sous pression | ROOT retient UNION4 096 incluant refus auto. Seuls refus auto terminaux sans Task purgeables sous pression, du plus ancien au plus récent ; tombstone trente jours dans même transaction. Natif et auto tentent purge sûre avant refus au plafond. | Rejeu exact jusqu’à purge, request_expired après. Aucun pending/chosen/ambigu/vivant touché. Collecte existante réemployée ; cinq refus natifs hors compteur et inchangés. |
| O4 — compatibilité Lineage | ROOT confirme v3 → v2 seulement unsupported_version structuré et maintien historique v2 → v1 sur unsupported_version ou invalid_request structurés. | invalid_request en v3 n’autorise aucun repli. Pages de version homogène ; journal et cancel sans fallback. |
| O5 — collision native/auto | ROOT retient vérification du reçu auto avant by_agent_request par l’entrée native externe sous Immediate. Le chemin interne chosen écrit le refus uniquement dans le reçu auto, sans native_refusal sous la même clé. | Clé owner stable, autorité interne non forgeable et deux unicités historiques conservées. Les cinq codes natifs ne sont pas élargis. |
| O6 — orientation Cartae | Nouvelle orientation utilisateur intégrée : Rust/compilation/tests Linux sur Cartae dans namespace privé umask077/TMPDIR court/fixtures isolées ; Mac pour T3/tests OS/signature/installation. | Worker actif 471321/socket/state intacts. Recette décideur depuis maître Mac ; mêmes sources par commit SHA, preuves Linux et Mac distinctes. Une compilation active dans toute la flotte, jobs2/nice10/tests2 ; FULL unique seulement si gate prévu. Connecteur défaillant hors feature mais bloquant pour lancement distant. |

## Portée et prochaine étape

Les corrections portent sur spec, plan, modèle, recherche, contrats, quickstart, checklist, audit de réemploi et ADR. Le PASS de réemploi reste un résultat indépendant de la contre-revue. ROOT a ensuite relu et validé les corrections O1–O6, puis autorisé Tasks/Analyze. Le verdict externe R1 reste inchangé. Les tâches générées par la phase suivante ne prouvent ni migration, ni implémentation, ni activation.

**Aucun test ni build exécuté.** Ce reçu ne prouve pas le comportement de minreq en production, la résolution réelle des reçus, la purge ni les transitions Lineage. Il ne prétend pas rapporter une approbation finale du reviewer après correction.

## Contre-revue de CODE SPEC-163 — ronde 1,14 octobre 2026

Cette section s’ajoute à la revue de PLAN R1 ci-dessus, sans la remplacer. Fournisseur claudeAgent, modèle claude-sonnet-5-5 (Claude Sonnet 5.5). TaskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r1. Verdict externe APPROVE_WITH_CHANGES : une objection moyenne et quatre mineures, aucun défaut externe déclaré bloquant. Aucun test/build ni écriture exécuté par le reviewer. Les objections et vérifications portent sur le snapshot R2, jamais sur les futures sources R3.

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r1-final.json, SHA-256 09b9227924e182aee635637d90f4a80af45e5fe7bc64e8a39c2df8ba438c0537 vérifié localement. Manifeste R2 /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r2.json, SHA-256 1901f295c99b8fec80e70d569e6ce9c8e57ef6527f6a3cdd27c21562f553bbae ; le reviewer rapporte 21 surfaces recalculées sans écart. Le rédacteur archive la revue, sans nouvelle contre-validation de code.

| Objection externe | Constat R2 du reviewer | Arbitrage ROOT et état |
|---|---|---|
| O1 — moyenne : owner hors ligne | auto_recovery_context ne résout pas sans route/fait de droits ; pending/started/chosen retiennent les places UNION jusqu’aux 128 actives. | Retenue pour T025/T054 : résolution autonome fail closed sans lancement ni restriction de FR015 ; distinguer absence owner et erreurs techniques/SQL. Correction et oracle hors ligne restent à prouver en R3. |
| O2 — mineure : worker provisoirement absent | Probe échoué enlève le candidat ; reprise pending/started peut refuser définitivement. | Conforme au contrat snapshot → unique/repli/refus, garde sûre. ROOT n’ajoute ni grâce worker, ni framework, ni scénario nouveau. |
| O3 — mineure : Explicit inutilisé | Enum/conversion existent, mais le résumé auto ne fabrique pas explicit pour Delegate historique ; v3 donne null. | null reste information indisponible/inconnue, pas une invention de choix explicite. ROOT conserve cette sémantique du contrat v1/legacy ; aucune suppression ou nouvelle projection future présumée. |
| O4 — mineure : chemin tests-only dupliqué | admit_auto_local représente environ 107 lignes et n’est atteint que par les tests, qui évitent le parcours configuré de production. | Retenue ROOT : supprimer cette duplication et déplacer les oracles vers le parcours de production ; préserver finalise_auto_local_choice réellement utilisé. Résultat R3 encore attendu. |
| O5 — mineure : diagnostic tardif | Après Task/lien Admitted, refuse_auto_admission retourne admission_pending et peut remplacer le code du parcours distant. | Point diagnostic à arbitrer sur types exacts et état durable ; ne pas confondre refus CAS et erreur SQL ni masquer une panne SQL. Aucune correction ou clôture annoncée avant preuve. |

Références statiques rapportées : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:6742 (O1), :611 (O2), :975 et :1890 (O4), :781 (O5) ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:13 (plafonds), /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs (résumé auto O3). Ces numéros appartiennent au snapshot R2 et deviennent historiques lors des corrections.

Observation séparée du reviewer : la collision explicite/auto distante arrive au puits après un premier by_agent_request/Prepare, alors que le contrat demande sa garde avant by_agent_request. Pas de double Task constatée par la lecture. ROOT demande la vérification de cette garde précoce en R3, sans lui attribuer une correction future ni transformer cette observation en sixième objection externe.

Le reviewer n’a pas relu T3 Reader/tooltip, CLI/MCP, remote_worker/fixture, les 27 tests Linux ou journal R2, ni la correction DTO santé. Ses conclusions ne sont donc pas des preuves d’exécution ou de compatibilité de ces surfaces. Il a lu les transitions SQL/CAS, la réservation/purge, l’admission/réseau/reprise, le probe maître et la projection v3 ; les garanties vérifiées statiquement restent distinctes des oracles à exécuter.

ROOT a retenu quatre compléments de convergence T052–T055. Total réel : 55 tâches/4 cochées ; header ancien 45 non édité ici. P2 n’est pas Converged, la nouvelle R3 est en cours. Santé réelle 271 octets incompatible avec le DTO R2 retenue T052, aucun POST. T050 : 30 PASS et format/lint/typecheck qualifiés séparément ; les 17 PASS R2 demeurent déclarés dans le journal, les 27 PASS R1 indépendamment attestés gardent leur provenance antérieure. Aucun T040/G8/G9, P3 ou livraison déduit des cinq objections externes non bloquantes.


## Suite R3/R4 de la contre-revue CODE R1 — 14 octobre 2026

Le verdict externe CODE R1 sur R2 demeure APPROVE_WITH_CHANGES. Aucune seconde contre-revue ni APPROVE final n’est inventé. Les choix suivants complètent les arbitrages historiques, sans les remplacer.

ROOT rapporte R3 indépendamment attestée : 30 PASS ciblés, 28 bibliothèque plus un CLI et un worker ; source-manifest163-r3.json conserve sa provenance. Le journal producteur décrit la clôture sans owner avec permission_not_inherited, la suppression du chemin tests-only admit_auto_local au profit du parcours configuré, la conservation du diagnostic distant après admitted et la collision externe avant Prepare. Ces déclarations et les oracles ciblés ne valent pas une nouvelle contre-revue indépendante de toutes ces branches. O1/O4 et l’observation de collision attendent le rapprochement final ROOT avec R4.

O3 : l’arbitrage ROOT du 14 octobre est inscrit dans le plan et le contrat. AutoDelegate avec deux axes désactivés et tuple fixe réel validé conserve Explicit ; Fallback reste prioritaire ; hors repli, un axe automatique garde Automatic même avec candidat unique. Delegate162 historique garde decision:null si son profil est inconnu. Le rapport T3 en lecture seule confirme les trois modes consommés et la neutralité de null, sans attester à lui seul la projection Rust finale. O2 demeure conforme au refus sûr sans grâce worker nouvelle. O5 demeure un contrôle du diagnostic, pas une assimilation de CAS à une panne SQL.

R4 est transférée : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r4.json, SHA-256 0a9b6eb57f9cc2ad0948dca0ea1eb4b861c0dd7a47ec098d1ddf30c24255f015 ; 21 sources concordantes, cinq changements et seize inchangées depuis R3. Les trois PASS ciblés sont désormais indépendamment attestés, avec logs complets et RC d’exécution corroborés ; deux FAIL RC101 restent historiques. Cette preuve ne remplace pas une nouvelle contre-revue. Le compteur de concurrence observe un ManagedSupervisorCommand::Start émis, pas un processus fournisseur. Les échecs et replays sont conservés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

ROOT doit lire R4 avant clôture Converge P3. Le tableau préparatoire des 31 FR/SC et des 56 tâches est dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Progression actuelle 4/56 après ajout ROOT de T056 ; audit final, build de livraison, T040, G8/G9 et livraison ne sont pas présumés. Le rédacteur n’a exécuté aucun test/build, Git, service ou synchronisation distante.

Nuance attestée : R3→R4 final comporte cinq changements ; les 21 empreintes avant/après tests R4 diffèrent sur quatre fichiers, decision_service.rs étant déjà final avant tests. HASHES_BEFORE_AFTER_IDENTIQUES=1 est le RC1 de cmp, donc différence. Les 21 empreintes après tests concordent avec les sources finales. Aucun GREEN global ni rejeu des 30 tests R3 déduit des trois PASS R4.

### Limite spécifique T055 — oracle MCP non exécuté dans la campagne R3

attestation_cartae163 a recherché la trace complète de 960 lignes : aucun PASS visible pour delegation_mcp::tests::closed_arguments_reject_forged_parent_without_connecting. Les seules mentions sont des lectures de code. Ce test ne porte pas le préfixe spec163_ et ne fait donc pas partie des 28 PASS de bibliothèque R3. Ses nouvelles assertions sur l’entrée auto, les bornes et le parent forgé ne doivent pas recevoir un PASS par extrapolation. Le PASS CLI R3 est distinct. Cette preuve MCP prévue par T029/T055 reste à fournir avant clôture ; aucun test nouveau ajouté ni replay effectué par le rédacteur. Le code concerné est /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs.

### Écart réel P3 et poursuite autorisée — T056, 14 octobre 2026

ROOT a confirmé que preferences est configuré et validé mais n’est pas consommé dans la demande de décision R4. Ce manque concerne l’acceptation US2 et le choix de préférences prévu à T039. ROOT a ajouté T056 dans tasks.md : raccord des préférences figées aux seuls IDs opaques admissibles, dans les champs déjà bornés, et oracle HTTP existant. Inventaire relu : 56 tâches, quatre cochées seulement. Aucun nouveau banc, protocole ou case cochée par le rédacteur.

La couverture documentaire demeure 31/31 exigences associées à des tâches ; elle ne signifie pas 31/31 comportements prouvés. T056 et la preuve MCP de T055 restent ouvertes. P3 ne peut pas être déclaré CONVERGED avec ces manques. ROOT trace la poursuite de correction en phase d’implémentation sous le mandat utilisateur de terminer autonomement, avant contre-revue et audit finaux ; aucun quatrième passage de convergence déguisé ni gate de livraison contourné. Aucun succès futur, audit, build, recette T040 ou G8/G9 déduit.

## Suite R5 attestée — aucune nouvelle contre-revue finale

Le correctif de préférences T056 est codé et dispose de dix PASS ciblés indépendamment attestés : T054 un et service neuf ; format RC0. Les trois FAIL des campagnes initiales et le FAIL MCP distinct restent conservés. Ce dernier compare le nombre d’outils MCP, neuf contre huit attendus ; les trois gardes auto traversées avant cette assertion ne valent pas un PASS global T055. ROOT a lancé le replay exact R6 après correction de la seule attente ; aucun résultat R6 n’est anticipé.

Le snapshot R5 final est /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r5.json, SHA-256 048f38bd7fcdbbec0893f0756886a65fef8d036e06c1d10f1d6ebc85dd3c32d5. L’acceptation en lecture seule de T056 reste en cours. Le verdict externe CODE R1 APPROVE_WITH_CHANGES et le checkpoint P3 NONCONVERGED gardent leur provenance historique. Aucune nouvelle approbation, P4, clôture, audit, build, recette ou livraison déduite ; progression inchangée 4/56.

## R6 attestée et contre-revue CODE R2 en attente

Le test exact MCP de T055 est maintenant attesté : un PASS, zéro FAIL, 2036 filtrés, RC0/6,38 s. La correction huit→neuf porte seulement sur le nombre d’outils, pas les champs handoff ; les gardes auto demeurent inchangées. Le FAIL R5 et ses autres preuves restent historiques. Source R6 /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r6.json, SHA-256 09d25daa87620ce6aada5d1132babeee1f0d7510800c908a76133551396ceefe ; log /Users/moi/.cache/bridget-delivery163-20261013/t055-r6-mcp-closed-parent.log, SHA-256 02f70366e4b3b46e06f85c18c0bf2eca191b3f418db29e4012a23c111ab5b043.

ROOT a demandé la contre-revue CODE Claude R2 à 01:51 CEST le 14 octobre, en lecture seule. Son verdict reste en attente ; aucun APPROVE R2 inventé. P3 NONCONVERGED et l’exception autorisée de poursuite restent les checkpoints historiques. 56 tâches, quatre cochées ; aucune clôture de convergence, audit, build, recette T040 ou G8/G9 déduite du seul PASS MCP.


## Contre-revue de CODE SPEC-163 — ronde 2, source R6

Fournisseur claudeAgent, modèle claude-sonnet-5-5. TaskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r2. Verdict reçu APPROVE_WITH_CHANGES. Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r2-final.json, SHA-256 8477c4e88a67f0ce5d3d65680d5475c6b4a1aff38247de14870889c0f3c71c3c, vérifié localement. Les 21 sources R6 concordent avec le manifeste selon le reviewer. Aucun test, build, Git en écriture ou service exécuté par lui. Les sections R1 et l’attente R2 antérieures restent historiques ; aucun verdict sur R7 déduit.

| Finding R2 | Constat et preuve rapportés sur R6 | Arbitrage ROOT / état |
|---|---|---|
| F1 — moyen | admit_delegate sort avant la fermeture du reçu aux lignes 3401–3473 ; finalise_auto_local_choice propage l’erreur aux lignes 704–715. Les refus de preflight Worktree (2902) et create (2939) sont aussi propagés. chosen reste actif après reply_timeout=10 ou base_ref inconnue, puis le rejeu reste admission_pending. | Confirmé ROOT, T057 retenue avant clôture. Validation/refus CAS locaux et reprise, oracles existants seulement. R7 admise ; aucune correction encore attestée. |
| F2 — faible à moyen | Entretien synchrone en fin de tick, lot jusqu’à 128 ; Prepare jusqu’à trente secondes, probe cinq secondes ; erreur technique inchangée réentretenue. Le reviewer évoque starvation des lignes suivantes par tri époque/date et LIMIT 128. | Réserve conservée sur cadence et reprises techniques. ROOT rappelle le plafond global 128 sous Immediate, incluant pending/started/chosen : LIMIT 128 seul ne prouve pas un affamement SQL permanent. Condition examinée par audit 04/06 ; aucun chrono ni correctif de scheduler prétendu. |
| F3 — faible | Résultat de probe après cinq secondes rejeté CommandMismatch puis WorkerRejected ; le lecteur worker rompt sur ce message inattendu, permettant des reconnexions répétées en dépôt lent. | Réserve exacte conservée dans Limites. Pas de correction de connexion, API ou protocole nouveau revendiqué. |

Références de source du reviewer : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:3401, :3473, :704, :715, :2902, :2939, :7878, :2169, :651, :6985 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs:1353 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon.rs (CandidateProbeResult) ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs:3206. Numéros historiques R6, non présentés comme nouvelle lecture de code du rédacteur.

Objections R1 : O1 résolue pour absence de route seulement, F1 en prolonge le cas local ; O2 documentée conforme ; O3 modes résolue ; O4 chemin tests-only supprimé ; O5 diagnostic/collision résolus. Le reviewer valide par lecture T056 : ordre des préférences sur snapshot filtré, POST réel à sept critères et bornes à huit par construction, sans ID privé. Aucun PASS nouveau déduit.

Limites du reviewer : tests T025/T3, fixtures MessagesTimeline/CLI/worker et documents spec/plan/tasks largement survolés ; parties hors 163 et advisories non relues. Les gates futurs T040/build/installation/Git/nettoyage ne sont pas comptés comme défauts. F1 doit être corrigée avant clôture ; F2/F3 restent réserves, sans prétendre les avoir corrigées. Audit R6 readonly en cours puis delta R7 frais avant grade. Total courant 57 tâches/quatre cochées ; P3 NONCONVERGED historique conservé, correction de contre-revue en phase 5, pas P4 formel ou approbation finale anticipée.

## Checkpoint préparatoire T057 R8 — 14 octobre 2026

ROOT et attestation_cartae163 confirment une fausse clôture SQL R7 : INSERT Task échoue avec fixture_task_sql, mais l’UPDATE de refus réussit. T057 reste ouverte. R8 est en cours depuis 00:25Z ; ownership exclusif native_delegation.rs et workspace_git.rs, 22 sources à attester. Le discriminant Git interne sépare absence de base et erreur E/S/timeout/troncature ; whitelist métier fermée aux trois fermetures, aucune interprétation libre des erreurs. Aucun résultat R8 encore attesté.

Preuves R7 et détails : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Les drafts d’audit R6 doivent lire le delta R8 des deux sources avant scoring final. P3 NONCONVERGED, phase 5 autorisée et 4/57 conservés ; aucun PASS R8 ou gate futur déduit.

## Contre-revue CODE R3 sur R8 — APPROVE_WITH_CHANGES

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r3-final.json, SHA-256 2dc3d9443fb275f1a06d2d7d1ea0955c5a784b55cf312d0cb746cf0c22f5ed12 ; taskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r3, Claude Sonnet 5.5/claudeAgent, lecture seule. Le reviewer a vérifié le hash du manifeste R8 et du delta, pas recalculé les trois hashes source ni relu les 23 fichiers. Aucun test exécuté par lui.

SQL INSERT Task : oracle R8 lu, chosen/noTask/rx vide puis reprise après retrait du trigger avec même c01/une Task/sans HTTP ; exécution attestée séparément. Le CAS Admitted conserve le diagnostic primaire. Base inconnue et sorties Git tronquées/délai se distinguent par les codes retenus.

Défaut restant confirmé : preflight_classified retourne Technical, mais into_legacy conserve une String workspace_path_unsafe acceptée par la liste métier au puits ; même risque dans create après worktree add. Un échec transitoire E/S peut donc devenir un refus durable. ROOT retient R9 dans T057 : conversion auto Technical vers un préfixe technique absent de la liste métier, refus purement déterministes Refused, E/S de création/Commit nonterminales avec diagnostic original. Code R9 seulement préparé ; aucun test/R9 final attesté encore. Le prochain oracle EIO typé traverse la méthode de production et le puits, sans prétendre forcer une syscall OS EIO.

Réserve faible non bloquante : rev-parse code128 ne prouve pas uniquement une base inconnue ; stderr non analysé et locale non figée. Reviewer ne la qualifie pas de défaut ; aucun durcissement Git/API/banc nouveau demandé ici. F2/F3 précédentes non relues par cette revue. R9 doit être relu dans le delta d’audit avant grade final ; aucun APPROVE final, Converge nouveau, build ou G8/G9 déduit.

## Checkpoint R9 final ciblé — 14 octobre 2026

ROOT atteste le transfert final R9 : 23 sources concordantes Mac/Cartae/snapshot, native_delegation.rs et workspace_git.rs changées, 21 inchangées R8. Trois oracles existants exacts passent avec RC0 : T024 local 6,918 s, T025 reprise 5,859 s, Git worktree emboîté 5,909 s. Format canonique des deux sources et diff-check RC0. Preuves exactes et erreurs historiques : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

Le correctif préserve la classe technique jusqu’au puits auto. Les refus déterministes de chemin au préflight restent métier ; après réservation, toute erreur de create en mode auto garde chosen avec diagnostic original et éventuels résidus de branche/chemin. Les erreurs E/S de la passe Commit restent non terminales ; ses gardes pures conservent leur traitement métier. L’oracle EIO typé traverse le mapper de production et la fermeture/store : chosen/noTask/rx0. Il ne force pas une syscall système EIO. Les résultats R8/R9 ne sont pas des suites globales ; aucun replay des succès inchangés.

Compteur 4/57 sans cochage, 31/31 couverture documentaire. ROOT relit le delta R9 et demande CODE R4 ; audit32 refresh en cours, aucun verdict/grade final anticipé. P3 NONCONVERGED historique et phase 5 corrective autorisée conservés, aucun P4 formel. T039 configuration off/on non appliquée, T040 service réel, builds/paquet/signatures, Git/livraison/G8/G9 encore ouverts. Quota CLI100 % et children ne prouvent pas des profils opérationnels en production.

### Réponse ROOT à CODE R3 — correctif R9 testé, verdict final attendu

Le delta R9 fait parvenir la classe Technical au puits auto avec un préfixe technique absent de la liste de refus métier. Les gardes de chemin déterministes au préflight restent Refused ; toute erreur create auto reste non terminale, comme les E/S Commit, avec diagnostic conservé. Les gardes pures Commit gardent leur traitement métier. T024 étendu vérifie une EIO typée vers le mapper de production puis fermeture/store avec chosen/noTask/rx0 ; T025 et le worktree emboîté sont rejoués. Trois PASS exacts attestés, aucun test de syscall OS EIO forcée revendiqué.

R3 et sa limite de code128 Git restent des faits historiques, pas un faux APPROVE corrigé. ROOT relit le delta final et demande CODE R4 ; audit32 rafraîchit son périmètre avant score final. Références SHA et logs R9 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Verdict R4/grade final encore attendus ; T057 non cochée, G8/G9 et livraison ouverts.

## Qualification CODE R4 — 14 octobre 2026

CODE R4 conclut APPROVE_WITH_CHANGES, sans changement de code ni test supplémentaire demandé. F1 faible est traité documentairement : la garde déterministe au préflight ferme le reçu ; le wrapper auto de create garde chosen pour toute erreur, même une garde pure. Une modification entre préflight et create, par exemple .worktrees devenu symlink, peut conserver la place jusqu’à la reprise sous une autre époque du coordinateur. Cette reprise refait le préflight. Un nouvel identifiant de Task et un worktree résiduel supplémentaire restent possibles après échec de création ; diagnostics et résidus de branche/chemin sont conservés. ROOT accepte cette limite, sans correctif R10 ni campagne nouvelle.

F2 faible de couverture : T024 exerce l’EIO typée via into_admission puis fermeture/store et la vraie garde .worktrees fichier vers Refused. Les map_err de create et les erreurs E/S de la passe Commit sont vérifiés par compilation et lecture, sans oracle direct. Une NotFound de canonicalize en Commit est non terminale ; aucun PASS direct de ce cas ni syscall EIO forcée revendiqué. Les trois PASS R9 restent les trois exécutions nommées, pas une preuve supplémentaire de ces chemins.

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r4-final.json, SHA-256 5cb0c98f361e7079eb59085ec2988b39636c4378a1fc5f4cab8f1544b7f8a5f0. Reviewer Claude Sonnet 5.5/claudeAgent ; taskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r4. Deux hashes source, manifeste et delta R9 vérifiés ; 21 sources inchangées, T3, logs et résultats de tests non relus par le reviewer. R1/R2 non relus exactement ; aucun test exécuté par cette revue. Audit32 encore en cours, aucun grade/cochage/livraison déduit ; 4/57, P3 NONCONVERGED historique et exception phase5 conservés, aucun P4 formel.

## Checkpoint T021 final — delta de tests seulement, 14 octobre 2026

Le manifeste final R9+T021 porte sur 23 sources : deux corps de tests changés dans native_delegation.rs/decision_service.rs, 21 sources inchangées ; les deux préfixes de production restent identiques à R9. L’oracle T054 existant, étendu à la réponse tardive après reprise réelle, donne 1 PASS/RC0, 7,718489 s commande/0,07 s banc. Premier FAIL de fixture conservé ; aucun test supplémentaire ni replay des succès antérieurs. Preuves exactes : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

L’entrée de build existante decision-request-max8.json, 709 octets/objet questions choice, a été copiée byte-identique du serveur au miroir Mac. Elle doit être versionnée avant le build Mac T038 ; ce n’est pas une preuve de recette de production. L’ancien artefact privé861 octets/questions tableau reste historique et obsolète. T040 doit utiliser la requête réellement capturée et ses tokens ; aucun fixture ne vaut réussite d’un POST de production. Audit32 et validation finale encore en cours, aucune acceptation49/57 anticipée : 4/57 inchangé. P3 NONCONVERGED historique/exception phase5 intactes, aucun R10 de production ou P4 formel ; builds/G8/G9 encore ouverts.

### Raccord T021 à la revue finale

CODE R4 reste la revue du code de production R9. Le delta T021 final modifie seulement les deux corps de tests ; les deux préfixes de production sont identiques, donc aucune nouvelle revue globale ou correction R10 de production annoncée. TEST-001 est fermé par l’attestation fraîche03/06 de l’oracle existant tardif/reprise/CAS ; premier FAIL de fixture conservé. Source/logs/hashes détaillés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Audit32/validator final et acceptation ROOT encore ouverts, aucune case cochée.

## Acceptation ROOT du développement — 49/57, 14 octobre 2026

GO ROOT après lecture du delta T021 final et du reçu FINAL_T021_READONLY_CONFORMANT : T005–T037 et T046–T057 acceptées/cochées, ajoutées aux quatre préparatoires déjà prouvées. T038–T045 restent ouvertes ; aucune recette, build Mac, source poussée/installée, activation ou G8/G9 anticipée. P3 NONCONVERGED reste historique ; l’exception utilisateur de poursuite corrective en phase5 est conservée, aucun P4 formel ou faux CONVERGED ajouté.

Audit final readonly : grade A/99,047619 ; 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; TEST-001 fermé par l’oracle T054 existant exécuté. Couverture4,655325 % du snapshot et0,355837 % du contexte global, donc audit partiel et aucun GREEN global. Reçu /Users/moi/.cache/bridget-delivery163-20261013/audit-preparation/audit163-final-t021-receipt.json, SHA-256 057c0b4349388fa46b3cc1c843a7c352c738260d01434161ade6eb5f4810fe87, relu et hash vérifié par le rédacteur. Aucun audit, source, test, Git, build ou service modifié/exécuté par ce cochage documentaire.

## Checkpoint installation et provenance — 53/57, 14 octobre 2026

ROOT accepte T038/T040/T041/T042 sur leurs preuves réelles. Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 installé : Mac SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063, Linux SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0. Maître 44553 né 04:55:03 CEST et pont 44667 né 04:55:05 utilisent /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; worker 1411115, naissance 146166371 utilise /home/moi/.local/lib/bridget-communication/163-5105378bf4f0/bin/bridget.

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

Sauvegarde retry : phase_complete/activate, projection173→174 de même génération et 33 autres tables exactes, schéma/count inchangés ; halt humain complet préservé. Migration additive reçusauto, quick_check ok/FK0, schémas 7/12/Fleet5. Sauvegarde jointe vérifiée ; après activation, toute rétrogradation exige une nouvelle sauvegarde jointe sans perte. La sauvegarde préactivation ne couvre pas les nouveaux reçus. L’incident171→172 et les anciens PID 86807/86830/1403968 restent uniquement historiques.

Le reçu d’installation conserve handshake_and_cancel_recipe_not_proven=true : poignée de main et annulation opérationnelles restent à vérifier pour G8.

T039 : /Users/moi/.cache/bridget-core/agents.json est appliquée au SHA 5dead7198c0220477d88229d1b0ab0f4b3fe5858a0391bde1cfb687c3265c93b ; fixed/fallback local-codex-pro, agents/machines inchangés. auto_engine=false, auto_placement=false, service_enabled=false. L’application off est réelle, mais l’activation des axes/service et les preuves G8 restent ouvertes : T039 non cochée, comme T043/T044/T045.

Ordre réel : sources committées/intégrées/poussées avant laboratoire T040, pour obtenir un checkpoint déterministe compilé identique sur Mac/Linux ; validation T040 exécutée avant activation. Ce commit de source préalable n’est pas une clôture de livraison ni un hash futur inventé. ROOT accepte T040 sur observations R4 et supplément corrigé 54056f…8d49 ; RC1/course F historique conservé, aucun nouveau RC0 ou POST ajouté. P3 NONCONVERGED historique/exception utilisateur phase5 conservés, aucun P4 formel.

Reçus :

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

## Correctif G8 — façade MCP T058, 14 octobre 2026

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

Compteur courant53/58 : T039/T043/T044/T045/T058 seules ouvertes. La preuve brute est conservée : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log ; SHA-256 03e823bb6a993a7f1b8b546edc53f6aeded6e163bcd9ed507b22a68dc28f2f47. P3 NONCONVERGED historique et poursuite corrective sous mandat utilisateur restent explicites ; aucun P4 formel inventé.


## Checkpoint source T058 — tests et contre-revue, 14 octobre 2026

Les douze oracles ciblés réussissent : onze tests de bibliothèque et un test d’intégration exact. Le reçu /Users/moi/.cache/bridget-delivery163-20261013/auto-mcp163-g8-validation-final.json porte SHA-256 bcbe4d808ea71c47c11690ef9570c282ed8ba62e324c186de1176e41cf60aa78. Les six empreintes source sont identiques avant et après la campagne. Le filtre initial qui avait exécuté zéro test reste archivé et ne compte pas parmi les PASS. Son nom a été corrigé ; les six premiers succès n’ont pas été rejoués.

La contre-revue indépendante R5 reste APPROVE_WITH_CHANGES. Elle valide les cinq empreintes du premier périmètre par lecture, sans exécuter de test. F1 a été traité ensuite : une seule ligne ajoute bridget_auto_delegate à l’attendu du test d’intégration existant. spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises donne 1 PASS/0 FAIL, RC0, 9,584249 s de commande. Les cinq sources déjà revues sont inchangées. Reçu R5 : /Users/moi/.cache/bridget-delivery163-20261013/t058-review-r5.json ; SHA-256 51c128704ecad1610eae22c50038e9c35f7a6deff8a7e8943279d9b22cc36c4d. Aucun nouveau verdict APPROVE final n’est inventé.

L’audit T058 readonly conclut A/100 sur le delta lu, avec 0C/0H/0M/0L actif. Sa couverture est partielle : 1 561/23 127 lignes physiques des six fichiers, soit 6,749687 %. Le validateur canonique donne RC0, zéro erreur et zéro avertissement. L’audit163 précédent conserve ses 3 MEDIUM et 2 LOW ; aucun finding historique n’est effacé ou réputé corrigé. Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-14/session-2026-10-14-spec-163-t058-01. read_manifest SHA-256 58538c71eeab71f5fcebd97b4fc19bb68f43b137c6e02025359ec6791c17014a.

Limite conservée : le refus du hook AutoDelegate sans request_id est vérifié par lecture de la garde, sans oracle négatif auto spécifique. L’oracle public tools/call prouve le raccord du wire sur un daemon simulé ; il ne prouve pas une Task durable ni un processus fournisseur réel.

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.


## Acceptation opérationnelle ROOT — historique57/58 intégré dans c57, 14 octobre 2026

ROOT accepte T039/T043/T044/T058 sur les preuves de configuration effective, de recette G8 et de nettoyage G9. T045 reste seule ouverte pour la clôture documentaire, le commit et le push finaux. La source active est 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 ; les métadonnées de clôture ultérieures auront leur propre commit. Le correctif T058 est fermé par son code, douze oracles exacts, la revue R5 avec F1 aligné, le delta d’audit et G8 réelle.

Reçus : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json ; SHA-256 5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c. /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les faits et limites sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/163-decision-admission/validation/livraison163.md. Les anciens chemins de worktrees Mac retirés désignent des lieux historiques, sans nouvelle copie de travail active.

P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 restent conservés, sans P4 formel inventé. Les anciennes FULL avec échecs, l’audit163 partiel 3M/2L, le delta T058 partiel sans finding et le delta T3 R6 sans finding restent distincts ; aucun GREEN global déduit.


## Clôture T045 — 58/58, 14 octobre 2026

ROOT a intégré et poussé les documents de livraison G8/G9 au commit c57cfd35efa1338cda5256a02c57d71258d8b8d4 sur github/main. Le reçu vérifie HEAD égal à la référence distante et dépôt propre à ce checkpoint. T045 est acceptée sur cette preuve : 58/58, Implemented ; Pipeline : Livré avec conservations justifiées. Le checkpoint 57/58 précédent, intégré dans c57, reste historique.

Preuve : /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json ; SHA-256 56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390. Le build actif reste 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5. Le commit documentaire c57 n’est pas un nouveau binaire. Les marqueurs Implemented/58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5. ROOT a vérifié github/main égal HEAD propre ; les sources suivies de Cartae main/worktree sont alignées sur ce commit documentaire, sans changement du build3efa. Preuve d’alignement Cartae : /Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json ; SHA-256 a8566f602017d52ea0ca9ab3706871038275aa04477b67077fb572b74d638620.

Les conservations G9 sont acceptées et nommées, sans suppression forcée. Anciennes FULL non GREEN, audits partiels et leurs findings, P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 sont maintenus. Aucun nouveau test, audit global, service ou P4 formel n’est déduit. L’ETA initiale 300–540 min et l’ETA après Tasks 360–600 min restent historiques ; dernière fenêtre 05:30–06:00 CEST. Aucun temps écoulé exact de développement ou CPU n’est ajouté.
