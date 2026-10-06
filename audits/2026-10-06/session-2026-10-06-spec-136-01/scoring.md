# Audit de Code — Bridget SPEC136

Date : 2026-10-06T02:25:11Z. Grille all, mode fix, scope diff contre main@43afebbf.
Baseline propre cd1c3435, suite fraîche PASS avant cycle. Analyse préparatoire
en lecture puis cycle1fix sans candidat CRITICAL/HIGH, cycle-scoring readonly.
Aucune correction de code appliquée pendant l'audit. Mode parallèle sequentiel.
Coverage :100% des hunks136 et de leur contexte immédiat dans six fichiers ;
pas100% des178 sources. Pas de couverture de lignes calculée.
Score global A, calcul par domaines, pas une certification absolue.

## Tendance
Première session de ce scope136. Baseline absente = vide, jamais écrite.

## Résumé exécutif
| Domaine | Note/numeric | CRIT | HIGH | MED | LOW |
|---|---|---:|---:|---:|---:|
| Qualité | A/97 |0|0|1|0|
| Sécurité/architecture/tests/performance/fiabilité/observabilité/chaîne/IA/LLM/duplication/complexité/minimalisme | chacun A/100 |0|0|0|0|
| UX | N/A, pas d'UI modifiée |0|0|0|0|
Un warning non bloquant, aucun CRITICAL/HIGH. Release non bloquée.
Notes calculées uniquement sur les findings de ce périmètre.

## Findings
QUAL-001 MEDIUM : read_range, crates/bridget-daemon/src/store/threads.rs:649–736,
88 lignes, seuil50 dépassé. Lecture SQL/projection/budget réunis. Correctif
recommandé : extraction locale de projection dans une tâche de lisibilité
distincte. Ne pas perdre snapshot, bornes ou corps exacts. Pas de bug prouvé.
Les autres fonctions touchées longues, majoritairement héritées :
parse_thread_args1078–1223=146l ; tools1820–2157=338l ; post485–617=133l ;
thread_post885–1089=205l ; thread_read1211–1329=119l ; ensure_schema284–413=130l.
Pas de refactorisation globale pour satisfaire une règle de style.
JSCPD --complexity mesure422/134 par fichier, pas par fonction ; aucun seuil
cyclomatique par fonction déclaré satisfait sans outil approprié.

## Performance et complexité
Complexité algorithmique : périmètre vérifié, aucun anti-pattern détecté.
Les huit patterns ont été cherchés. Pages≤200/60Kio et membres≤16 ; une jointure
indexée, pas de N+1. EXPLAIN asserté par test, pas une supposition.
Post O(log E+M), read O(log E+P log E). Annotations corrigées/cohérentes.
Pas de gain p99 ou productivité revendiqué sans mesure.

## Tests et reprise
Suite workspace --no-fail-fast sortie0,1603 exécutions PASS affichées dont
une auxiliaire enfant ;52 ignorées prévues, aucun test136 ignoré,10 nouveaux PASS.
Ancien test de version ajusté1→2 sans retirer assertions de conservation.
Ancien test socketDarwin EINVAL intermittent consigné : retest4/4, suite fraîche
complète PASS. Son fichier n'a pas changé. Aucun faux PASS de revue papier.
Fmt/clippy final/diff PASS. Release isolée construite avec cd1c3435.
Résultats, commandes et limites dans specs136/validation/results.json.
Couverture de lignes/CVE/p99 non mesurées. Gherkin descriptif, Rust exécutable.

## Minimalisme & Frugalité
Checklists1–6 exécutées : pas de service, table, bus, modèle ou dépendance ajouté.
Deux colonnes+index/enum partagés justifiés ; corps immuables et état dérivé par
jointure. Décodeur Option à deux usages justifié par absence/null et canon legacy.
Le helper de test réutilise le harnais102, pas de framework nouveau.
Potentiel minimalisme : ~0 lignes suppressibles à comportement constant.

## Vertus LLM & Responsabilité Future
Charge future réduite pour le lecteur : lecture actuelle au lieu de long historique.
Volume justifié par migration, refus, reprise et tests des deux façades.
Abstractions servent le contrat réel ou une contrainte de compatibilité écrite.
Intention et limites expliquables : aucune inférence du texte, aucun verdict changé.
Aucune couche nouvelle non comprise après lecture des call-sites.
Un message déjà livré ne peut être retiré du contexte ; adoption du canal requise.

## Duplication et points positifs
JSCPD :25933 lignes,87 clones,927 lignes dupliquées,3.5745960744996723%,max29l.
Top blocs29/25/25/23/23 dans anciens testsMCP, aucun>100.
Neuf lignes de préparation136 proches dans deux tests : garder(Type3), scénarios
indépendants ; le harnais, lui, est partagé. Pas de second bus ni journal de mission.
SQL paramétré ; identités/membres vérifiés ; silence history durci ;
ancien snapshot stable ; preuves relisibles ; WAL/busy_timeout existants.
Tous les points positifs sont détaillés par thème dans cycle1.

## Roadmap et limites de livraison
Immédiat : rien de bloquant. Moyen terme : QUAL-001, sans changer le contrat.
Aucune baseline supprimée. Source/test inchangés depuis la validation.
Sauvegarde à la bascule ; pas de restauration d'une ancienne DB sur des messages
nouveaux. Rollback de binaire : suspendre le canal structuré avant lecture legacy.
Diff non committé traité par la livraison autorisée, pas par la skill d'audit.
