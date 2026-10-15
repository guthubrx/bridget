# Contre-revue du plan168 — Claude Sonnet5.5

Date :2026-10-15. Fournisseur/moteur :Claude ; modèle demandé :Sonnet5.5.
Portée :documents de conception et contrats, pas de code168 ni preuve d’exécution.
Provenance de cette trace :verdicts et objections transmis par ROOT au propriétaire documentaire.
Les sorties originales restent conservées par ROOT ; ce document synthétise sans prétendre les citer intégralement.
Repères de rondes :168-plan-review-20261015-r1 et168-plan-review-20261015-r2.
Aucun identifiantTask complet non fourni n’est inventé.

## Rondes et décision

- R1 :REQUEST_CHANGES. Les objections ont été vérifiées sur les contrats et aides locales,
  puis arbitrées par ROOT et corrigées dans les artefacts, sans réduction des septUS/35FR/11SC.
- R2 :APPROVE communiqué par ROOT ; aucun défaut bloquant restant sur le plan corrigé.
- Audit de réutilisation :PASS, gate5/5 ; SHA d36e094c11cdda16d1575c1450efe98e7450c4bcbeeec169da493b4e97b785cd.
- Tasks sont générées après ces gates ; Analyze est requis avant toute implémentation.

## Questions et objections retenues ou arbitrées

| Question ou objection | Réponse retenue et preuve documentaire/native | Décision |
|---|---|---|
| Le petit décideur peut-il garder OAuth avec bare ? | L’aide locale Mac2.1.300 et Cartae2.1.251 limite bare à APIkey/helper. safe-mode conserve auth/politiques administrées ; restricted, outils/MCP vides, settings/contextes isolés et schéma restent requis. | Objection retenue ; bare interdit, recette OAuth réelle prévue avant livraison. |
| La liste présentée à Laya permet-elle un vrai choix sans perdre des faits ? | Format compact complet FR021, IDs v2 et modèle≤32octets ; si tout ne tient pas, niveau indisponible explicite. Recette service sur ensemble entier compatible≤8 ;12+via LLM réel et score. | Objection retenue ; aucune promesse Laya128 ni top8 caché. |
| La conservation stderr et les limites de lecture sont-elles nécessaires/suffisantes ? | inherit_stderr=false forcé, aucun tampon stderr ; stdout64Kio et sortie4Kio avant parsing, deadline dès spawn et permis jusqu’à arrêt réel. | Correction retenue ; preuve de sûreté de transport reste à exécuter. |
| Les sources de décision cassent-elles le CHECK SQL ? | Modes SQL explicit/automatic/fallback conservés ; source détaillée dans metadata v2, correspondance explicite. | Objection retenue ; pas de nouveau mode SQL. |
| La migration couvre-t-elle base neuve et données anciennes ? | DDL168 neuf directement ; migration transactionnelle de table163 connue, copieBLOB exacte et trois index ; rollback et réouverture prévus. | Correction retenue ; testsSQL persistants requis. |
| Le plafond128 peut-il masquer des modèles ou gonfler les profils ? | Plafond statique global sur toutes connexions×modèles déclarées, désactivées comprises, avant filtres ; registre129refusé ; efforts en liste, pas de profil par modèle. | Arbitrage ROOT retenu, complet pour flotte prévue sans expansion illimitée. |
| Le catalogue prouve-t-il les droits fournisseur de chaque modèle ? | Non :noms/efforts et provenance déclaratifs ; entitlement inconnu après lecture. Aucun27appels requis ; refus fournisseur final explicite, sans bascule. | Limite conservée, pas de faux succès d’accès. |
| CPU nécessite-t-il un nouveau service ou une fausse mesure initiale ? | Échantillon mémorisé, heartbeat/probe existants, collecteurB réemployé au ticklocalC ; premierCPU=null et une collecte par machine/lot. | Correction retenue, pas de daemon/thread supplémentaire. |
| Un rollback ancien binaire est-il sûr lorsque168estOFF ? | Toute lignev2 interdit cet ancien lecteur ; garder un binaire compatible+OFF ou restaurer cohérent sans perte. | Objection retenue et procédure corrigée. |
| Peut-on garantir que le brief ne contient jamais de secret ? | Non :garantie limitée aux secrets/config/auth automatiquement injectés ; le brief utilisateur reste inerte et borné, sans scanner parfait promis. | Objection retenue ; périmètre de sûreté corrigé. |
| Un modèle sans réglage d’effort serait-il masqué ? | default_effort présent et résoluble :niveau compatible, sinon null/liste vide pour défaut natif ; effort imposé incompatible refuse. | Arbitrage ROOT retenu, aucun medium inventé. |
| Peut-on ajouter workload dans HandoffV1 ? | Non :schéma fermé. workload=general|compilation, défautgeneral, est un champ AutoDelegateV2 horsbrief, figé au canon/snapshot/payload. | Écart d’audit corrigé sans changer153. |
| Le catalogue humain impose-t-il T3 ? | routing_catalogue_v1 sur clientcontract existant, feature dédiée, Role::Client et peerUID=sameUID ; projection publique allowlistée, jamais human_authority_guard. | Écart d’audit corrigé, nouvelle lecture justifiée. |
| Shared garantit-il la base source exacte ? | expected_base_commit v2 pour tout candidatGit, propagation snapshot/probe/Prepare et verify_base_commit. AbsenceSHA exclut le candidatGit ; technique≠outsideGit ; SHA fourni horsGit refuse mismatch ; Worktree exigeGit. | Arbitrage ROOT retenu ; aucun HEAD distant implicite. |
| Les changements réintroduisent-ils un moteur ou une dépendanceT3 ? | Réemploi des stores, transports, maintenance, façades et guards ; ownershipB publie tous DTO avant consommateursD/C/E. | AuditPASS et R2APPROVE sur la conception, pas de verdict d’implémentation. |

## Questions de contrôle finalR2

La revue finale devait vérifier les réponsesR1, la cohérence inter-artefacts, la couverture complète,
les limites explicites, la compatibilité163, les contraintes de temps/processus et la livraison prévue.
ROOT a rapporté APPROVE sans défaut bloquant. Cette trace n’attribue aucun succès de test ou d’activation168.

Les preuves d’exécution appartiennent aux futures Tasks et au reçu de livraison ; les docs initiales
ne permettent pas de cocher du code absent.
