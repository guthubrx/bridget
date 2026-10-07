# Validation — SPEC143

Date : 2026-10-07. Statut : Implemented — non installé, non activé. T001–T008 vérifiées ; deux comparaisons Converge principales sans écart.

| Contrôle | Preuve reçue du principal | Verdict |
|---|---|---|
| Baseline | 294/294 PASS, 4,92 s, deux suites existantes | PASS |
| RED UI | 7 échecs / 4 PASS | Risque reproduit |
| RED logique | 4 échecs / 24 PASS | Risque reproduit |
| RED revue | 9 cas logique + 3 UI avant corrections | Risques reproduits |
| GREEN final | 346/346 : 252 logique + 94 UI | PASS |
| Format | Quatre fichiers | PASS |
| Lint | 0 erreur, 22 warnings existants | PASS sans erreur nouvelle |
| Types web | Contrôle frontend | PASS |
| Build web final | 32,44 s ; avertissement de chunk existant | PASS |
| Diff | Vérification du diff | PASS |
| Recette réelle | Navigateur clone isolé, corps/JSON, clavier/focus/copie, refus/unknown, ordinary, 320px et thèmes | PASS |
| Revue finale | Interne APPROVE après corrections RED/GREEN ; huit cas manuels réellement exécutés en mémoire et lecture seule, distincts des346 tests Vitest | PASS, pas revue inter-fournisseur |
| Converge1 | Principal, 10FR/5SC, aucun écart, tâches byte-identiques | CONVERGED passe1 |
| Audit/scoring final | 509 s ; grade A/100 sur le diff quatre fichiers ; deux MEDIUM corrigés, zéro résidu ; tests346 PASS en2,97s | PASS |
| Validateur de session | Auditeur puis principal, exit0, zéro erreur/zéro warning | PASS |
| Arrêt clone | Ports13916/5876 sans listener, services isolés arrêtés ; T3 actif inchangé | PASS |
| Converge2 | Principal à09:22:07 CEST,10FR/5SC sans écart, tâches byte-identiques939891be… et quatre sources inchangées | CONVERGED passe2 |

Les cinq captures inspectées dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/validation/` restent intactes. Recette détaillée dans ui-recipe.json au même dossier de spec. Aperçu natif indisponible, navigateur isolé utilisé ; aucun fournisseur actif, E2E de livraison ni visuel in_progress exécuté. Audit limité au diff, pas à l'ensemble du produit. Aucun restart T3 actif, commit, déploiement ou appel modèle payant.

Clôture technique à09:22:07 CEST :1879s,31min19s depuis08:50:48. Finition documentaire exclue de cette durée outil. Aucune tâche modifiée après le gel de comparaison.

## Révision US4 — Validation encore ouverte

Le complément autorisé à11:05:59 CEST rouvre la session en In Progress. Les tableaux ci-dessus, empreintes et captures restent des preuves historiques du socle US1–US3, pas une validation de la révision US4.

Gate et Analyze documentaires du complément : PASS avant code sur les formats confirmés transmis par le principal. Lecture intégrale principale et GO vers11:12 CEST : T009 cochée. Baseline346 PASS en4,39s, puis16 nouveaux échecs RED (14logique/2UI),346 tests historiques PASS ; preuve reçue vers11:10 CEST : T010 cochée. GREEN, contrôles, recette isolée, revue et convergence du complément restent attendus. T011–T013 restent ouvertes jusqu'à leurs preuves.

Contre-revue intermédiaire APPROVE_WITH_CHANGES : citations sur texte répété et références Markdown cross-boundary dégradées. GREEN366 antérieur non final. RED supplémentaire :6 échecs (4logique/2UI),367 PASS/373. Correctifs acceptés et coût documentés dans le contrat ; attendre GREEN373, contrôles et recette après correction. Aucune approbation finale ni clôture à ce stade.

## Révision finale US4 — Gel392

Les attentes du paragraphe historique précédent sont remplacées par les preuves finales suivantes.

| Contrôle | Preuve reçue du principal | État |
|---|---|---|
| RED ultérieurs |9 références→381 ;6 notes malformées→387 ;4 NBSP/tabs→391 ;1 focus réel391PASS/1RED→392 | Défauts reproduits puis corrigés |
| GREEN final |392 PASS,283 logique/109 UI,46 cas supplémentaires au socle346 | PASS |
| Rejeux | Principal5,40s à11:32:57 ; relecteur6,08s à11:32:20 |392 PASS indépendants |
| Contrôles | Format1,361s PASS ; lint0erreur/22warnings baseline ; types PASS ; build49,45s/6141modules PASS, chunk warning existant ; diff PASS | PASS |
| Revue finale | APPROVE, aucun résidu | PASS interne, pas inter-fournisseur |
| Recette native | Focus après deux RAF, trois toggles32px sans cadre, notes visibles, lien conservé, citation DOM sur texte répété | PASS avec limites documentées |
| Conteneur étroit |320px, toggle233,65625×32, zéro overflow, style restauré | PASS conteneur, pas viewport mobile |
| Arrêt et protection | OwnPIDs83544/83608 arrêtés, ports5876/13916 libres, T3 PID85017/start07:21:41 inchangé | PASS |
| Clone | SQLite quick_check ok, projection_thread_sessions0/provider_session_runtime0 ; fixtures/tempdir/onglet masqué conservés | PASS |
| Convergence principale finale | Lecture seule terminée11:39:54,16FR/8SC, aucun écart, tâches12/13 SHA64f824f0… avant/après identique et quatre sources inchangées | CONVERGED |

Copie exacte couverte unitairement ; pas de clipboard hôte E2E. Citation DOM/capture/résolution réelle, pas navigation de citation E2E. Resize1280 demandé mais non appliqué ; viewport2212×1382. Aucun fournisseur ou livraison E2E. L'ancien audit A reste historique et ne donne pas un nouveau score US4. Sources gelées392 et hashes dans results.json, T011/T012 cochées. Aucun commit, merge, push, installation, activation ou restart T3.

T013 cochée après verdict dans une phase documentaire distincte :13/13, statut Implemented non installé/non activé. La comparaison certifiée porte sur les tâches12/13 avant cette coche, sans modification pendant la lecture. Aucun second passage de convergence US4 déclaré.
