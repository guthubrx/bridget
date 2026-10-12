# Dépendances de l’audit v14 159B

1. ROOT termine Promote et le correctifSC014 prévu. Il exécute les tests concernés. Les reçus des RED/GREEN et du FULL restent distincts. Un FULL avec échecs historiques n’est jamais appelé GREEN global.
2. ROOT annonce le gel final de code. Rafraîchir l’inventaire suivi/nouveau, les hunks, les hashes et l’état de documentation. La capture de préparation ne devient pas finale par renommage.
3. Charger module10-output-schema et chaque module thématique. Sept JSON sont requis : modules03,04,05,06,07,08,12. Lire réellement les hunks et les puits, dater les plages et leurs hashes. Qualifier explicitement les Register mécaniques.
4. Reprise possible des owners : protocole154→03/06, audit_performance151→04/08, git154→05/07/12. ROOT choisit l’affectation ; aucune équipe supplémentaire n’est lancée par cette préparation. Toute auto-revue doit être déclarée. Un C/H exige un défenseur indépendant et un verdict CONFIRMED/REFUTED prouvé.
5. Agréger avec dédup(file,line,category). Réemployer le script officiel : /Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/scripts/fingerprint.py --batch. Baseline : /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/baseline.json ; absente à la capture, pas modifiée. Aucun scanner ou scénario supplémentaire gratuit.
6. ROOT corrige dans le scope autorisé. Un cycle maximum, dix corrections maximum, deux tentatives par finding. Tracer fix-report.md et cycles.log ; aucun commit de reviewer, AUTO_COMMIT=false. Après correction, scoring ciblé final en lecture seule, avec héritage des plages seulement si leurs sources sont attestées identiques ; code modifié relu.
7. Produire cycle-1/aggregated-findings.json historique, cycle-scoring/aggregated-findings.json final, scoring.md et grade.json. Le score suit les poids et N/A du module10 ; il expose la couverture réelle et la mémoire négative. Potentiel minimalisme et Vertus LLM restent obligatoires. Aucune note avant ces preuves.
8. ROOT copie vers /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/audits/2026-10-12/session-2026-10-12-spec-159B-01 seulement après gel et clôture. Validateur officiel, stdlib Python, sans installation :

python3 /Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/scripts/validate_session.py /Users/moi/.cache/bridget-audit159B-preparation-20261012

La commande ci-dessus est une instruction future, pas un run effectué. Elle exige grade.json, scoring.md et les agrégats finaux. Elle doit sortir RC0 avant livraison. La validation de schéma ne prouve pas le comportement runtime. Aucun audits/latest, baseline, Git ou WT modifié par cette préparation.

## Références lues

/Users/moi/.agents/skills/audit/SKILL.md
/Users/moi/dotfiles/audit/commands/audit.md
/Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/orchestrateur.md
/Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/00-parametres.md
/Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/01-anti-hallucination.md
/Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/02-decouverte.md
/Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/11-regles-or.md

Lectures du protocole centrées sur phases0/0.5/0.75 ; aucun corps complet des modules thématiques revendiqué.
