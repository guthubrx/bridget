# Audit de Code — SPEC141

Date : 2026-10-07, gel final vérifié à 07:54:51 CEST ; complément visuel reçu à 07:56:47 CEST.
Mode : readonly. Grille : pre-merge avec sécurité et performance frontend ciblées.
Durée de revue après le premier gel : 8 min 52 s, corrections et nouveau gel inclus.
Score du diff corrigé : A. Aucun défaut ouvert prouvé. La note ne constitue pas une note du monorepo.

Note établie sur un échantillon de 0,0990099 % des fichiers source suivis : quatre fichiers sur 4 040. Toutes les régions modifiées sont relues, avec le contexte d'appel utile. Les grandes régions historiques inchangées ne sont pas auditées intégralement. Le compteur 532 désigne les 489 lignes ajoutées et 43 lignes retirées du diff ; il ne désigne pas une couverture de tests.

## Tendance

Première session pour le scope SPEC141. Aucune tendance de note n'est inventée depuis un autre scope.

La comparaison déterministe contre le commit 9706acbde648e581d4c41302a0f4ce8e4ac20a9d donne : JSCPD 3,694910 % avant, 3,662741 % après. Les 35 clones hérités persistent. Deux fragments de tests de sept lignes apparaissent. Aucun nouveau clone de production. Le plus grand bloc demeure de 22 lignes. Les totaux cumulatifs passent de 498 à 510 lignes dupliquées ; ils ne décrivent pas douze lignes de production régressées.

## Résumé exécutif

| Domaine du diff | Note | CRITICAL | HIGH | MEDIUM | LOW |
|---|---|---:|---:|---:|---:|
| Sécurité de présentation | A | 0 | 0 | 0 | 0 |
| Qualité et architecture | A | 0 | 0 | 0 | 0 |
| Tests et fiabilité ciblée | A | 0 | 0 | 0 | 0 |
| Performance et complexité | A | 0 | 0 | 0 | 0 |
| Duplication | A | 0 | 0 | 0 | 0 |
| Minimalisme et hygiène | A | 0 | 0 | 0 | 0 |
| Accessibilité des commandes | A | 0 | 0 | 0 | 0 |
| Fournisseurs, supply chain, observabilité | N/A | — | — | — | — |

Les domaines N/A sont des exclusions explicites du périmètre, non des domaines jugés sains. Les domaines évalués partent de 100 et ne subissent aucune déduction pour défaut ouvert. Leur moyenne est 100, donc A selon le module 10. Ce calcul est une synthèse du diff vérifié, pas une preuve exhaustive d'absence de défaut.

## Findings par sévérité

Aucun CRITICAL, HIGH, MEDIUM ou LOW ouvert dans le diff corrigé. Aucun finding supprimé par baseline. Le fichier de suppressions est absent et reste non écrit.

Deux défauts confirmés pendant la convergence ont été corrigés avant le gel final :

- T009 : une expression de terminaison optionnelle CR/LF pouvait consommer le CR appartenant à un corps comme A\r. Le parseur retire maintenant exactement le séparateur déduit de l'en-tête canonique. Des séparateurs structurels mélangés imposent le repli sûr. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:129 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:334.
- T010 : un en-tête canonique au-delà de 1024 caractères sortait de la projection compacte, mais sa copie suivait alors le chemin qui remplace les références par leurs labels. La copie reconnaît désormais ce seul en-tête canonique séparément, sans activer un rendu compact. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:101 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2268 ; test /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:396.

Le principal a rapporté les tests RED puis GREEN. Cet audit n'a pas rejoué les anciennes versions rouges. Il a lu la correction, les assertions et la contre-revue locale APPROVE. Il a rejoué la version finale.

## Validation exécutée

Audit : commande ciblée vp test run sur les deux suites de timeline. Résultat réel : 294 tests PASS, deux fichiers PASS, exit 0, durée 2,95 s, début 07:52:09 CEST. git diff --check retourne exit 0. Les quatre hashes SHA256 restent identiques au gel final.

Principal et implémentation : format, lint ciblé, types web et build web PASS. Les 22 warnings lint sont identiques à la baseline selon la comparaison du principal. Ils ne sont pas présentés comme 22 nouveaux défauts.

Recette native rapportée par le principal : clic/Tab/Entrée/Espace, focus visible de 2 px, choix indépendants conservés après repli, ancrage 905→905, repli de 4 annoncés/3 réels affichant les trois corps sans section attribuée, zéro Sources, zéro script exécuté. Le lot de notifications conserve ses détails historiques. À 320 pixels CSS réels, les rendus clair et sombre mesurent chacun 320 pixels de largeur et 320 de contenu, sans débordement. Un nom de 300 caractères Unicode reste complet et revient à la ligne. Le corps long complet garde ses dix paragraphes.

La capacité native preview_resize a expiré deux fois. La recette a donc utilisé un iframe de 320 pixels dans le même aperçu T3, sans navigateur alternatif. Cet audit a inspecté les deux captures finales ; il n'a pas exécuté les interactions du principal. Captures :

- /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp14lx-25fd3477.png
- /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp1vid-2e2033d4.png

Complément à 07:57:35 : le principal a cliqué le vrai bouton de copie et comparé les 3421 caractères à la fixture SQLite, texte identique. Le spy local writeText a été restauré ; cette recette ne revendique pas une écriture dans le presse-papiers système.

La contre-revue finale est locale et indépendante : APPROVE sur CR/LF, copie longue et absence de régression du préambule/texte ordinaire. Aucun fournisseur alternatif joignable n'a été identifié.

## Complexité algorithmique

Complexité algorithmique : périmètre vérifié, aucun anti-pattern détecté.

La reconnaissance compacte reste bornée à 1024 caractères. Le prédicat de copie parcourt au plus l'en-tête canonique, sans activer de nouvelle présentation. Le parseur examine les candidats une fois et les tranches de corps ne se recouvrent pas : O(n) en longueur du texte. L'aperçu parcourt au plus 120 points de code. Sa concaténation est donc bornée par une constante. Les sections utilisent des indices uniques, des recherches Set, et un état local. Aucun tri répété, N+1, I/O dans une boucle, récursion ou copie défensive en cascade n'est ajouté.

## Minimalisme & Frugalité

Potentiel minimalisme : ~0 lignes suppressibles à comportement constant dans le diff vérifié.

Les checklists 1–6 du module 12 sont appliquées. Aucun paramètre futur, dépendance, store global, couche de service ou wrapper sans règle n'est ajouté. La règle de nom local sert deux chemins réels et porte une compatibilité explicite. Le petit prédicat canonique sépare volontairement copie et projection compacte. Les tests répètent deux fragments courts pour deux chemins de régression distincts ; les factoriser maintenant ajouterait une abstraction de test sans réduire la règle métier.

## Vertus LLM & Responsabilité Future

Le changement réduit la charge de lecture des lots : les noms et les corps restent séparés sans inventer de sujet. Le volume du code correspond au parseur conservateur, aux replis imbriqués et aux tests des limites. Les abstractions servent des contraintes présentes de format et de copie, pas un futur hypothétique. Les invariants sont décrits dans le contrat141 et peuvent être expliqués : pas de séparation certaine, pas d'attribution ; pas de transformation de source ; état local ; pas de Sources dans les lots directs. Aucun enchaînement de couches opaque, stub ou test tautologique n'est détecté dans le diff.

## Points positifs

- Corps HTML traité comme texte par le renderer existant, parseRawHtml=false.
- Copie originale complète conservée, même avec lien t3-context et en-tête canonique long.
- Repli entier sur ambiguïté, sans expéditeur ou message inventé.
- Pas de nouveau transport, stockage, fournisseur ou modification d'Agent Loop.
- Réutilisation du renderer, de la copie, de la clé fil/message et de l'ancrage existants.

## Limites et dépendances vulnérables

Aucun scan CVE ou licence n'est lancé. Aucune nouvelle dépendance n'est ajoutée. Cela ne signifie pas que toutes les dépendances existantes sont exemptes de vulnérabilité. Aucun Lighthouse, métrique Core Web Vitals, mesure instrumentée de contraste, couverture ou mutation testing n'est revendiqué. Aucune donnée active ni production n'est touchée.

## Roadmap de remédiation

Aucune correction supplémentaire prouvée pour ce diff. Préserver le gel jusqu'à la décision de livraison. Toute nouvelle modification exige de refaire tests, hashes, convergence et audit concernés. Cet audit n'autorise ni commit automatique ni déploiement.
