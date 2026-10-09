# Analyse de cohérence — SPEC145

Date : 2026-10-07. Verdict documentaire : PASS après deux passes initiales et une troisième passe ciblée I004. Ce résultat ne valide aucun code ni runtime. Le GO d'implémentation appartient au principal.

## Périmètre et méthode

Artefacts : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/spec.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/plan.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/tasks.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/contracts/human-thread-view.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/checklists/requirements.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/reuse-audit.md`.

La première passe a été effectuée par le principal en lecture seule, avec la constitution utilisateur. Le résultat est transmis à l'agent documentaire. Les corrections non ambiguës sont appliquées après cette passe, hors Analyze. La seconde passe relit les artefacts corrigés, sans aucune écriture pendant la lecture. Ce rapport est écrit après sa fin.

Le helper check-prerequisites attendu est absent, constat confirmé par le principal. Le protocole Analyze est appliqué par lecture documentaire. Aucun helper absent exécuté ni skip Analyze n'est prétendu.

## Passe1 — Constats transmis par le principal

Couverture :22 exigences FR145,8 critères SC145 et20 tâches. Checklist de spécification19/19 PASS. Gate réutilisation PASS. Aucun défaut CRITICAL ou HIGH.

| ID | Gravité | Constat | Correction hors Analyze |
| --- | --- | --- | --- |
| A1 | MEDIUM | Dépendance T011→T010 inutile pour les RED store/tabs/UI mockée ; couplage T010→T009 inutile pour la factory de query. | T007 et T011 marquées [P]. T011 dépend du GO Analyze/implémentation. T010 dépend de T008 ; T009 reste obligatoire avant T017. T012 distingue travail natif après T011 et montage query GREEN après T010. |
| A2 | MEDIUM | Converge final et mise à jour documentaire mélangés ; mini revue ne suffit pas au protocole audit demandé. | T018 première convergence en lecture seule avant audit. T019 audit v14 fix auto_commit=false, cycle final readonly et convergence après dernier fix/rejeu, tâches byte-identiques si aucun manque ; reçus hors phase Converge après comparaison. T020 reçu/journal/statuts hors Converge, sur preuves seulement. |
| A3 | LOW | Quelques libellés T3ThreadBinding et formulations de plusieurs méthodes RPC subsistaient. | Unifier T3ThreadBindingFact ; une seule RPC bridget.read porte trois actions fermées. |

Les corrections ne changent ni le besoin, ni le périmètre, ni l'autorisation de livraison. Le chemin `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/server.test.ts` a été confirmé par le principal.

## Passe2 — Vérification documentaire après corrections

Les six artefacts principaux, le gate réutilisation et le plan de revue ont été relus. A1–A3 sont corrigés. Aucun CRITICAL, HIGH ou MEDIUM nouveau dans le périmètre documentaire. La couverture22FR/8SC→20tâches reste complète. Les20 tâches sont non cochées.

- Autorisation : résolution serveur, racine canonisée, liaison primaire, appartenance à chaque lecture et refus fermé restent cohérents.
- Lecture seule : handler avant maintenance mutative, CLI avant initialisation/autostart, aucune émission/read/ACK/réveil/mission/génération sont planifiés et testables.
- Protocole : une capacité versionnée, enum de trois actions, enveloppe et erreurs fermées, limites de page/projection/transport restent alignés.
- Interface : plusieurs fils, pagination ASC avec snapshot, exactitude/copie, recherche locale, refresh manuel, contexte obsolète et clavier/Sheet/PR sont couverts.
- Validation : RED avant changements correspondants ; build/type/lint applicables ; audit v14 et convergence séparés de la clôture documentaire.

`git diff --check` est PASS. Il s'agit d'un contrôle de format documentaire, pas d'un test produit.

## Passe3 — Contrat d'interopérabilité I004

La découverte source réelle I004 montre que la sortie d'historique ne doit pas réutiliser le type notify d'une requête Post. Lecture directe des lignes600–655 et1005–1060 dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` : entry_json624 restitue notify_json ; ligne1035 stocke `{mode,targets}`. NotifySpec::mode36–50 confirme none, targets et all. Le contrat humain conserve cet objet fermé avec UUID[] borné16, sans créer un nouveau type Post.

ThreadRow84–85 et entry_json628 confirment dates i64/Option<i64>. La fonction unix_now_secs6997 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs` utilise as_secs. created_at/closed_at restent nombres Unix secondes ; aucune chaîne ISO fictive.

Le principal approuve cette correction fidèle à la source, sans nouveau besoin. Contrat, modèle, recherche et arbitrage de réutilisation ont été corrigés hors Analyze. La troisième passe relit en lecture seule contrat et modèle après correction. Le constat PASS porte uniquement sur cette cohérence documentaire. Les travailleurs Rust/TypeScript alignent le code ; leurs preuves GREEN restent attendues. Le reçu partiel se trouve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/validation.md`.

L'aperçu isolé est désormais autorisé par l'utilisateur. L'ouverture tab_5 about:blank n'est pas une recette visuelle. Aucune tâche d'implémentation n'est cochée par cette passe.

## Limites et suite

L'agent d'un autre fournisseur n'est pas joignable selon le constat same_project reçu. Aucune approbation indépendante n'est inventée. Les futurs résultats audit et tests devront être consignés sur le diff145 réel. Aucune case ne devient complète par la seule existence d'un artefact.

Les statuts de plan/tâches peuvent maintenant refléter Analyze PASS dans une phase documentaire distincte. Le principal a confirmé sa seconde passe et transmis le GO d'implémentation aux trois travailleurs après l'analyse. Ce GO ne prouve encore aucune tâche terminée. Aucun commit, merge, push, installation, déploiement, navigateur ou restart n'est autorisé par cette analyse.
