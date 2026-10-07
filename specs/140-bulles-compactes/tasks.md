# Tâches SPEC140 — Bulles Bridget compactes

Statut : Implemented (code et vérifications, sans livraison production). Gate reuse-audit : PASS validé par le principal.
Primitive speckit-tasks lue. setup-tasks.sh, template local et extensions.yml absents, constatés par ls : génération manuelle suivant le même contrat. Aucun commit automatique. Chaque case nécessite une preuve observable.

## Phase1 — Préparation

- [x] T001 Vérifier les deux worktrees, la checklist et les points de réutilisation avant création ; consigner le périmètre et les recherches dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md. Résultat : liste de fichiers autorisés et isolation confirmées, aucune base active utilisée.

## Phase2 — Fondation

Les décisions de compatibilité et de réutilisation sont prises dans les artefacts. T001 est le prérequis commun. Aucun framework, store, API ou dépendance à installer. Les tests de chaque story précèdent son code.

## Phase3 — US1 : bulle compacte accessible (P1)

Test indépendant : cinq enveloppes complètes se replient sur une ligne et s'ouvrent ; messages ordinaires inchangés.

- [x] T002 [US1] Ajouter les tests RED des cinq projections et faux positifs dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Vérifier l'ancrage et la borne1024, puis enregistrer l'échec attendu avant code.
- [x] T003 [US1] Étendre la projection pure existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts. Résultat : fixtures T002 GREEN, aucune mutation du texte ou attestation de provenance.
- [x] T004 [US1] Ajouter ou étendre les tests de rendu proches de /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx. Résultat RED avant code pour repli initial, clic, Entrée/Espace, état accessible et focus. Si aucun harnais DOM existant adapté, consigner l'arbitrage dans reuse-audit puis employer la recette réelle isolée, sans prétendre avoir un test automatique.
- [x] T005 [US1] Modifier la branche existante de CollapsibleUserMessageBody dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx et importer localement le logo source /Users/moi/Nextcloud/10.Scripts/64.bridget/assets/branding/bridget-logo.svg. Résultat : bulle droite bleu-gris, une ligne initiale, commande clavier et détails bruts ; aucun chargement réseau ni renderer parallèle.

## Phase4 — US2 : libellés sans invention (P1)

Test indépendant : reply, nom absent et compteurs ont des libellés fiables ; aucun mot du corps ne devient un type de mission.

- [x] T006 [US2] Étendre les tests dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts pour noms présents/absents, reply, lots, notification et corps contenant action/blocker/decision/history. Résultat : la nature n'est jamais inférée du contenu ; conserver RED avant les changements nécessaires.
- [x] T007 [US2] Compléter les libellés dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts et leur affichage dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx. Résultat : T006 GREEN, aucun faux nom de destinataire ou titre de fil.

## Phase5 — US3 : vrai titre de fil compatible (P1)

Test indépendant : destinataire membre reçoit le titre ; absent, ancien format ou non membre ne reçoit aucun titre indu ; canon et journal sont identiques.

- [x] T008 [P] [US3] Ajouter les tests Rust RED dans les modules de test existants de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-core/src/message.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/t3code.rs. Résultat : champs absents/lecteur ancien, membre/non-membre, remise classique/idempotente, canon/journal, titre hostile et borne200 couverts en environnement isolé.
- [x] T009 [US3] Étendre le champ racine facultatif de BridgetMessage dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-core/src/message.rs, sa projection de remise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/daemon.rs et la ligne JSON de titre dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/t3code.rs. Résultat : tests T008 GREEN ; ThreadNotice, project_thread_wake et octets persistés inchangés.
- [x] T010 [US3] Ajouter les tests de ligne titre JSON et de fallback ancien dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Résultat RED pour titre valide, hostile, absent, vide, invalide ou hors position ; texte brut original toujours identique.
- [x] T011 [US3] Consommer la métadonnée titre uniquement en présentation dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts. Résultat : T010 GREEN, « Fil partagé » si aucune valeur valide ; aucun lookup, backfill ou texte inconnu retiré.

## Phase6 — US4 : conservation et virtualisation (P1)

Test indépendant : anciennes et nouvelles entrées gardent texte, copie, pièces jointes et actions ; changer de fil ne transporte pas l'état ouvert.

- [x] T012 [US4] Vérifier et corriger si nécessaire la conservation dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx et les tests proches. Étendre les tests logiques dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Résultat : copie brute exacte, suffixe inconnu conservé, pièces jointes/actions existantes, longs messages, état isolé fil/message, recyclage et ancrage. Consigner la recette réelle du composant en thèmes clair/sombre et largeur320 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md.

## Phase7 — Vérification transversale

- [x] T013 Lancer les tests Rust concernés, les tests MessagesTimeline, format/lint/TypeScript ciblés et build web. Consigner commandes exactes, codes retour et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md. Résultat : aucun succès global supposé, aucune écriture production ni fournisseur payant.
- [x] T014 Confronter toutes les FR au code et aux tests après la dernière correction ; demander la contre-revue inter-fournisseurs si disponible puis exécuter l'auditv14 ou son fallback documenté. Consigner preuves et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md et les rapports voisins. Résultat : Analyze documenté dans ce tour, Converge CONVERGED sans mutation de tasks hors ajout de manques, audit sans finding bloquant ; sinon statut In Progress et première tâche exacte restante.

## Dépendances et parallélisme

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T010 → T011 → T012 → T013 → T014.
Branche Rust indépendante des fichiers frontend : T001 → T008 → T009 → T013.
US3 s'intègre au rendu US1 après T011. US2 et US4 utilisent le même composant frontend : ne pas faire écrire deux agents sur ses fichiers en parallèle. Les tests RED Rust peuvent avancer pendant les tests/rendu US1. Les deux branches doivent avoir fini avant T013.

## Stratégie de mise en œuvre

Incréments testables, mais pas de livraison MVP présentée comme complète : les quatre stories sont obligatoires. Les composants, la copie et l'ancrage existants réduisent la maintenance. Le seul nouvel asset représente le logo officiel. Le champ titre et sa ligne JSON portent une compatibilité nécessaire, pas une abstraction de convenance. Aucun commit, paquet installé ou service redémarré automatiquement.

## Convergence

Réservé aux éventuelles tâches ajoutées en append-only si un écart à la spec est démontré. Une tâche cochée n'est pas une preuve de conformité à elle seule.
