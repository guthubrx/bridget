# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/plan.md](../../../../../../specs/164-conversation-t3-distante/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**D164-01 — Acceptée.** La présentation appartient au connecteur T3. Une projection pure dans `packages/client-runtime` convertit les événements du journal en blocs de lecture, communs aux deux clients. Elle ne crée aucun événement d'orchestration. Cette frontière réduit les doublons web/mobile et conserve l'indépendance Bridget.

<a id="extrait-2"></a>

## Extrait 2

**D164-04 — Acceptée.** Aucune migration, table, RPC, contrat public ou dépendance. Le seul delta partagé d’état est un historique éphémère borné de lacunes dans le réducteur client existant, nécessaire pour conserver toutes les frontières de lecture pertinentes. Les artefacts SpecKit vivent dans Bridget 164, pas dans le dépôt T3. Les décisions sont conservées ici, sans catalogue documentaire redondant dans T3.

<a id="extrait-3"></a>

## Extrait 3

Racine T3 Mac : `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante`.
Racine du développement Cartae : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`. ROOT y a préparé une copie T3 indépendante issue de Git archive de base 174a ; aucun fichier source Rust 163 ni droit courant n’est modifié. La localisation temporaire ne crée aucune dépendance de Bridget envers T3. Le développeur lit seulement jusqu’au GO code. Aucun install, test ou build avant autorisation ROOT ; chercher d’abord les toolchains et caches Linux adaptés, sans binaires Mac.

<a id="extrait-4"></a>

## Extrait 4

1. Préflight du checkout Cartae et des surfaces réutilisables. Plan, audit de réemploi, tasks et analyse. Contre-revue du plan ; GO ROOT explicite vers DEV après clôture des objections.
2. Projection pure avec fixtures des formes réellement observées. Self-review du regroupement, des frontières et des limites.
3. Rendu web/desktop, puis mobile, sur la même projection. Préserver les contrôles et le suivi existants.
4. Campagne ciblée unique sur la source finale ; après une correction, rejouer seulement les scénarios affectés. Inspecter le diff complet et l'absence d'effets fournisseur.
5. Converge puis contre-revue code inter-provider et audit ciblé. Corriger uniquement les objections réelles. Ne pas annoncer un succès global à partir des ciblés.
6. Après validation, intégration et push de la source T3 de163, intégrer les seules sources164 validées par T013/T014. Ne pas attendre les gates produit G8/G9 de Bridget163 pour rapprocher ces sources T3. Contrôles concernés par le rapprochement des branches uniquement.
7. ROOT commit/intègre/pousse d’abord la source164 validée vers le fork après la source T3 de163 validée/intégrée/poussée, puis vérifie les références. ROOT prépare ensuite le paquet signé dans un stage Mac natif, vérifie la présentation isolée, installe sans relancer T3 et nettoie164 avec archivage des preuves. Si164 est prête à temps, un seul paquet final163+164 évite une double compilation. Les validations et les gates produit G8/G9 des deux périmètres restent distinctes. Aucun code164 non validé par ses gates n’entre dans le paquet commun. Si164 n’est pas prête, le paquet163 seul reste possible. Les métadonnées finales suivent la recette et le nettoyage.
8. Avant activation du daemon/worker163, attendre la clôture des tâches natives productrices, dont la mission code164. Ne pas arrêter le développeur164 pendant son code. Après son retour stable et la clôture des producteurs, ROOT peut exécuter les contrôles164 isolés directement sans enfant actif.
9. Conserver le worktree163 distant tant que le cwd de l’enfant164 y est lié. À la fin164, fermer les producteurs propres, archiver les preuves, vérifier les gardes lsof et retirer la copie T3 puis le worktree163 seulement après intégration/push et contrôles de propreté. Une garde indisponible justifie une conservation ; elle ne se contourne pas.

<a id="extrait-5"></a>

## Extrait 5

L’inventaire ci-dessus comprend douze sources. Le reçu de transfert R3 est `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r3-final.json` (SHA-256 `a6546dd14b6135ed58374b6282105fe906ce01ce8eca2a9f7fcba6dd3314be56`). Le delta de `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` est une entrée et son commentaire dans une liste existante, pas une baisse globale de lint.

<a id="extrait-6"></a>

## Extrait 6

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

<a id="extrait-7"></a>

## Extrait 7

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

<a id="extrait-8"></a>

## Extrait 8

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-9"></a>

## Extrait 9

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

<a id="extrait-10"></a>

## Extrait 10

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.
