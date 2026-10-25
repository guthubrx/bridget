# Tâches183 — Interface mobile et suivi d’activité

Ordre strict ci-dessous. Responsable unique. Aucun arrêt à une tranche MVP. Les contre-revues sont des livrables distincts en lecture seule. Les chemins sont relatifs au worktree de cette feature.

## Préparation

- [x] T001 Produire et relire specs/183-interface-mobile/spec.md, plan.md, research.md, data-model.md, contracts/interface.md, quickstart.md et reuse-audit.md ; demande, cadrage, volet et interactivité confirmés, gate de réutilisationPASS. FR-001 à FR-012.
- [x] T002 Écrire les cas JVM de recherche/attention/compactage dans android/app/src/test/java/app/cartae/bridget/ConversationTextTest.kt et les cas de contenu hostile dans RichContentTest.kt ; préparer les attentes de non-régression. FR-002, FR-004, FR-007 à FR-009.

## US5 — Compactage attesté

- [x] T003 Étendre le journal du transport natif dans crates/bridget-transport/src/codex_app_server.rs pour context_compaction début/fin, avec mêmes identités de tour/item et tests du mauvais tour, doublons et fin ; lancer python3 scripts/build.py cargo test -p bridget-transport avec filtre183. FR-009, FR-011.
- [x] T004 Étendre crates/bridget-daemon/src/t3code_contract_v2.rs et t3code.rs pour les items structurels de compactage déjà exposés par l’adaptateur ; tests parser/projection sans texte inféré. Documenter les détails dans docs/adapters/t3.md. FR-009, FR-011.
- [x] T005 Étendre android/app/src/main/java/app/cartae/bridget/ConversationText.kt et MainActivity.kt pour l’état de compactage corrélé ; fin de tour, perte du suivi et changement d’agent ferment l’indication. FR-009.

## US1 — Volet et attention

- [x] T006 Étendre android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt et LocalStore.kt pour repères de lecture/attention par association ; questions prioritaires, faits attestés et ancienneté explicite ; aucun acquittement depuis le seul volet. FR-002, FR-003.
- [x] T007 Étendre la projection crates/bridget-daemon/src/mobile/client.rs uniquement avec les faits d’annuaire utiles déjà disponibles et tests de portée ; conserver autorisation avant pagination et compatibilité des clients anciens. FR-002, FR-011.
- [x] T008 Réutiliser AgentDirectory/ProjectAvatar de android/app/src/main/java/app/cartae/bridget/MainActivity.kt dans un volet gauche accessible du fil ; Tous/Favoris, recherche, icônes et marqueurs avec libellés, fermeture Retour et changement d’agent vérifiables. FR-001, FR-002.
- [x] T009 Étendre le rafraîchissement visible de android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt et la mémoire de lecture MainActivity.kt ; pas d’appel réseau par agent, brouillons/positions conservés et états anciens en cas de panne. FR-001 à FR-003.

## US2 — Conversation et saisie

- [x] T010 Étendre android/app/src/main/java/app/cartae/bridget/MainActivity.kt et ConversationText.kt pour rechercher dans les groupes chargés et atteindre les résultats ; compte, précédent/suivant, absence de résultat et couverture limitée explicites. FR-004.
- [x] T011 Réutiliser la reconnaissance système dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt/MobileViewModel.kt pour dicter un brouillon ; annulation/indisponibilité/changement d’agent/association testés, aucun envoi automatique. FR-005, FR-010.

## US3 — Pièces jointes et lecture

- [x] T012 Étendre android/app/src/main/java/app/cartae/bridget/LocalStore.kt/MobileViewModel.kt pour staging de fichier local, retrait avant réseau et transmission explicite avec commande/réconciliation existantes ; tests dans android/app/src/androidTest/java/app/cartae/bridget/QueueStoreTest.kt ou test ciblé de cette responsabilité. FR-006.
- [x] T013 Étendre android/app/src/main/java/app/cartae/bridget/MainActivity.kt : menu « + », photo/fichier selon mécanismes système existants, carte des pièces jointes et retrait avant envoi ; préserver les reçus séparés fichier/message. FR-006, FR-010.
- [x] T014 Étendre android/app/src/main/java/app/cartae/bridget/RichContent.kt et ses tests pour blocs de code colorés avec langage/Copier, carte HTML et rendu sans doublon ; vérifier texte copié exact. FR-007.

## US4 — HTML interactif isolé

- [x] T015 Documenter la frontière et le modèle de menace dans docs/decisions/183-apercus-html-isoles.md ; comparer iframe opaque et options existantes, recherche anti-doublon dans reuse-audit.md, règles d’accès et échecs testables. FR-008, FR-011.
- [x] T016 Étendre HtmlPreview/htmlPreviewDocument dans android/app/src/main/java/app/cartae/bridget/RichContent.kt : hôte de confiance distinct, iframe scripts sans même origine, réseau/fichier/contenu/permissions/navigation/téléchargements bloqués, modes statique/code conservés et moteur corrigé requis. FR-007, FR-008.
- [x] T017 Exécuter l’instrumentation d’isolation et de retour au fil dans android/app/src/androidTest/java/app/cartae/bridget : compteur0->1, parent/session/stockage/fichier/réseau/popup/permissions refusés, page défaillante fermable ; conserver les sorties réelles privées. FR-003, FR-008.

## Vérification et documentation

- [x] T018 Compléter android/app/src/main/res/values/strings.xml et values-fr/strings.xml ; contrôler libellés, cibles48dp, taille de police et lecture sans couleur seule, questions/notifications/file inchangées. FR-010.
- [x] T019 Lancer JVM, instrumentation ciblée et tests Rust touchés ; refaire les circuits US1 à US5 sur émulateur et produire preuves-corrections/2026-10-25-183-interface-mobile/preuves.html avec captures réelles et étapes vérifiées. FR-001 à FR-012.
- [x] T020 Mettre à jour docs/ et specs/183-interface-mobile/quickstart.md avec les parcours natifs, limites de recherche/dictée/attention et modèle d’aperçu ; aucune présentation par comparaison externe. FR-011, FR-012.
- [x] T021 Assembler les artefacts de livraison suivant scripts/android-build.sh et scripts/build.py, contrôler version et procédure d’installation avant le commit ; tracer tests et éventuels blocages dans specs/183-interface-mobile/validation/. FR-012.

## Dépendances et stratégie

T001 -> T002 -> T003/T004 -> T005 -> T006/T007 -> T008/T009 -> T010/T011 -> T012/T013 -> T014 -> T015/T016 -> T017 -> T018/T019/T020/T021. Aucun partage concurrent d’écriture. Le serveur porte les sources ; le Mac sert à la compilation Android et au pilotage si ses outils sont requis. Les tests CPU filtrés utilisent les caches existants et les gardes projet.

Analyze est exécuté avant T002. Converge compare le code final à toutes les exigences après T021. Contre-revue, audit puis livraisonGit/installation/nettoyage suivent sans clôture anticipée. Une case reste ouverte si sa preuve manque. Les ajouts de convergence sont append-only.

## Convergence

- [x] T022 Correction d’audit : retirer l’extracteur HTML inactif de android/app/src/main/java/app/cartae/bridget/RichContent.kt et remplacer son ancien test par le parcours CommonMark réellement utilisé dans RichContentTest.kt ; vérifier absence d’appel, JVM/assemblage puis seconde convergence. Comportement visible constant, FR-007 et frugalité.
