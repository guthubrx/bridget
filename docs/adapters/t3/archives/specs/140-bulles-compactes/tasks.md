# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/tasks.md](../../../../../../specs/140-bulles-compactes/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T002 [US1] Ajouter les tests RED des cinq projections et faux positifs dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Vérifier l'ancrage et la borne1024, puis enregistrer l'échec attendu avant code.
- [x] T003 [US1] Étendre la projection pure existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts. Résultat : fixtures T002 GREEN, aucune mutation du texte ou attestation de provenance.
- [x] T004 [US1] Ajouter ou étendre les tests de rendu proches de /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx. Résultat RED avant code pour repli initial, clic, Entrée/Espace, état accessible et focus. Si aucun harnais DOM existant adapté, consigner l'arbitrage dans reuse-audit puis employer la recette réelle isolée, sans prétendre avoir un test automatique.
- [x] T005 [US1] Modifier la branche existante de CollapsibleUserMessageBody dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx et importer localement le logo source /Users/moi/Nextcloud/10.Scripts/64.bridget/assets/branding/bridget-logo.svg. Résultat : bulle droite bleu-gris, une ligne initiale, commande clavier et détails bruts ; aucun chargement réseau ni renderer parallèle.

<a id="extrait-2"></a>

## Extrait 2

- [x] T006 [US2] Étendre les tests dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts pour noms présents/absents, reply, lots, notification et corps contenant action/blocker/decision/history. Résultat : la nature n'est jamais inférée du contenu ; conserver RED avant les changements nécessaires.
- [x] T007 [US2] Compléter les libellés dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts et leur affichage dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx. Résultat : T006 GREEN, aucun faux nom de destinataire ou titre de fil.

<a id="extrait-3"></a>

## Extrait 3

- [x] T008 [P] [US3] Ajouter les tests Rust RED dans les modules de test existants de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-core/src/message.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/t3code.rs. Résultat : champs absents/lecteur ancien, membre/non-membre, remise classique/idempotente, canon/journal, titre hostile et borne200 couverts en environnement isolé.
- [x] T009 [US3] Étendre le champ racine facultatif de BridgetMessage dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-core/src/message.rs, sa projection de remise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/daemon.rs et la ligne JSON de titre dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/t3code.rs. Résultat : tests T008 GREEN ; ThreadNotice, project_thread_wake et octets persistés inchangés.
- [x] T010 [US3] Ajouter les tests de ligne titre JSON et de fallback ancien dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Résultat RED pour titre valide, hostile, absent, vide, invalide ou hors position ; texte brut original toujours identique.
- [x] T011 [US3] Consommer la métadonnée titre uniquement en présentation dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts. Résultat : T010 GREEN, « Fil partagé » si aucune valeur valide ; aucun lookup, backfill ou texte inconnu retiré.

<a id="extrait-4"></a>

## Extrait 4

- [x] T012 [US4] Vérifier et corriger si nécessaire la conservation dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx et les tests proches. Étendre les tests logiques dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Résultat : copie brute exacte, suffixe inconnu conservé, pièces jointes/actions existantes, longs messages, état isolé fil/message, recyclage et ancrage. Consigner la recette réelle du composant en thèmes clair/sombre et largeur320 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md.
