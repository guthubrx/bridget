# Analyze SPEC169 — Rapport et corrections autorisées

Date : 2026-10-15. Portée : cohérence documentaire avant GO code. Les tâches d'exécution restent ouvertes.

## Passes et résultat

ROOT a analysé les17FR,9SC,5 histoires, le plan, le modèle, le contrat, le réemploi et les11 tâches initiales. Les passes ont couvert doublons, ambiguïtés, précision, constitution, couverture, cohérence et minimalisme. Deux findings non ambigus ont été retenus. ROOT a autorisé leur correction documentaire séparée ; le responsable documentaire a appliqué cet autofix, ajouté T012 et revérifié les compteurs et dépendances.

Résultat après corrections : **0 CRITICAL restant ;17/17FR et9/9SC couverts par12 tâches et le jalonJ5 ;2 findings corrigés documentairement.** Aucune application, correction de code, suite de tests ou activation n'est déclarée réussie par ce rapport.

Les scripts projet sont absents. Le protocole Analyze a été lu dans /Users/moi/.agents/skills/speckit-analyze/SKILL.md puis appliqué manuellement. La passe d'analyse est en lecture seule ; les retouches et ce reçu relèvent de l'autorisation d'autofix explicite de ROOT. Aucun hook ou script manquant n'a été simulé comme exécuté.

## Findings et traitement

| ID | Catégorie | Sévérité | Constat | Correction documentaire |
|---|---|---|---|---|
| A1 | Ambiguïté | LOW | FR015 pouvait laisser croire que les textes de consignes étaient stockés dans le JSON. | FR015 dit désormais que seule la préférence est stockée ; les consignes utilisent cette source unique. |
| A2 | Couverture causale | MEDIUM | La reprise de ce long fil échoue `champ response limit` ; le prérequis réel de FR017/SC009 manquait au plan. | Exception bornée FR016, plan/recherche, réemploiA15 et T012 dans le handler HTTP existant. T011 dépend de T010 ET T012 ; IDs antérieurs conservés. La preuve réelle reste au jalonJ5. |

A2 n'est pas une correction de code déjà réalisée. ROOT a vérifié une réponse complète de33 868 206octets, supérieure à16Mio, dont environ27,5Mo de `visibleTurnItems` redondants non consommés par Bridget. Sa simulation donne6 372 704octets en vidant ce seul champ. La correction prévue vise uniquement le sujet du principal authentifié `bridget`, après contrôle de scope ; `thread`, `runs`, `nodes`, `messages`, `turnItems`, les autres champs et les autres sujets restent intacts.

Diagnostic : /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md. SHA-256 : `25df7f44115ddcb43cb38e89a1ba49214bf1eaafc3292b2c5d3309506334b434`.

Le handler et le banc sont réutilisés : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235 (`withHttp`). La casse `http.ts` est requise sur Linux. Aucun Rust, helper produit, endpoint, RPC ou contrat public nouveau n'est demandé.

## Couverture des exigences

| Exigence | Tâches ou jalon | Preuve attendue |
|---|---|---|
| FR001 | T001, T005 | Préférence par fichier, sans UI. |
| FR002 | T001, T005, J4–J5 | Absence OFF, ON cible explicite. |
| FR003 | T001, T005 | Cache fiable, refus avant secrets/écriture. |
| FR004 | T002–T005, T007–T008 | Mode courant avant chaque tour. |
| FR005 | T003, T006–T008 | Exception T3 et outils hors délégation. |
| FR006 | T003, T006 | Contraintes explicites conservées. |
| FR007 | T003, T006 | Choix et exécution gardés par Bridget. |
| FR008 | T002, T004, T006, T009 | Identité et droits inchangés. |
| FR009 | T003–T004, T007–T009 | Blocage explicite, aucun repli d'hôte. |
| FR010 | T009–T010 | Rejeu et suivi sans doublon. |
| FR011 | T005, T010 | Tâches en cours gardées sur leur moteur. |
| FR012 | T010 | Projection Lineage et lectures sans effet. |
| FR013 | T006, T010 | Autonomie et résultat natifs conservés. |
| FR014 | T002–T004, T007–T009 | Six raccords avec limites explicites. |
| FR015 | T001–T003, T006, T008, T011 | Source unique et mécanismes bornés. |
| FR016 | T002–T009, T011–T012 | Périmètre minimal et correctif causal HTTP. |
| FR017 | T012, J3–J5 | Installation, activation et reprise réelles. |

## Couverture des critères de succès

| Critère | Tâches ou jalon | Preuve attendue |
|---|---|---|
| SC001 | T002–T005, T007–T008 | Matrice des modes et raccords. |
| SC002 | T001, T005 | Erreur explicite et fichier byte-identique. |
| SC003 | T003–T004, T006, T009, J5 | Tests ciblés puis recette réelle et compteurs séparés. |
| SC004 | T002, T004–T006, T009 | Contraintes, droits et identité. |
| SC005 | T003–T004, T007–T010 | Refus/rejeu sans secours ou doublon. |
| SC006 | T002, T004–T005, T010, T012, J5 | Transitions, suivi et reprise du gros fil. |
| SC007 | T003–T004, T006–T008 | Exception T3 et commandes natives intactes. |
| SC008 | T001, T005–T006, T008, T011 | Aucun mécanisme public ou stockage supplémentaire. |
| SC009 | T012, J3–J5 | Source installée/chargée, ON et reprise prouvés. |

## Contrôles de clôture documentaire

| Mesure | Résultat |
|---|---:|
| Histoires couvertes | 5/5 |
| Exigences couvertes | 17/17 |
| Critères couverts | 9/9 |
| Tâches d'implémentation | 12, toutes ouvertes |
| Items de réemploi | PASS15/15 |
| Findings corrigés documentairement | 2 |
| Ambiguïtés ou doublons actifs | 0 |
| CRITICAL restant | 0 |
| Tâches sans lien FR/SC | 0 |

Les jalons de livraison restent séparés pour éviter un commit circulaire. Le correctif HTTP réutilise handler et banc existants ; la commande Pi reste la seule création privée arbitrée. Les contrôles T3 restent ciblés. Pi réel demeure non attesté, binaire hors PATH ; seuls contrat/fixtures sont prévus, sans installation de fournisseur.

Artefacts stables : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget. Prochaine action : copie Cartae puis GO code par ROOT. La couverture décrit le travail prévu, pas des critères déjà réalisés.

Aménagement d'exécution autorisé ensuite par ROOT : seule T012 est marquée `[P]`, sous ownership séparé des deux fichiers HTTP. T001–T010 restent séquentielles. Le principal garde le permis unique de contrôles lourds ; le second n'exécute aucun contrôle. T011 attend toujours T010 et T012. Cette répartition ne change ni les totaux, ni la couverture, ni les dépendances ; la matrice Analyze reste valable.

## Relecture finale ROOT après recette (2026-10-15)

Relecture manuelle des artefacts et des preuves d’exploitation :14/14 cases inchangées,17FR/9SC reliés aux preuves existantes, FR017/SC003 réel/SC009 couverts par J5. Les deux corrections Analyze restent closes ; aucun manque nouveau ni changement de code. Source T3 58ddcf3a16205a24f940164cba500089f49ec6c0 inchangée ; P2 reste le dernier Converge. Scripts projet absents : même substitut manuel documenté, sans script simulé. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md.
