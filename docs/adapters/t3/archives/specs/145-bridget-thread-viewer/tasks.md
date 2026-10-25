# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/tasks.md](../../../../../../specs/145-bridget-thread-viewer/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`.
T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`.
Les dépendances de tâches priment sur le parallélisme. Les groupes séparés peuvent avancer dans leurs périmètres sans modifier les fichiers d'un autre propriétaire. Aucune tâche n'autorise commit, fusion, push, installation ou restart. L'utilisateur a autorisé l'aperçu isolé le2026-10-07 ; une recette reste à prouver.

<a id="extrait-2"></a>

## Extrait 2

- [x] T001 — Ajouter et exécuter les tests RED du protocole Client et des liaisons dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-transport/src/protocol.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`. Couvrir enum humain fermé, capacité `human_thread_view_v1`, version inconnue, rôle incorrect, ancien daemon, liaison primaire seulement, projet autre/invalide, identité/instance divergente, ambiguïté, déconnexion et modèle dormant. FR03–07, FR17–18, FR21. Dépendance : Analyze PASS et GO implémentation.
- [x] T002 — Implémenter `HumanThreadViewV1`, `HumanThreadViewResult` et `T3ThreadBindingFact` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-transport/src/protocol.rs` ; publier le fait après Registered et contexte projet dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/t3code.rs` ; ajouter index en mémoire et preuve source T3 dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`. Ne pas réouvrir ThreadRequest agent au Client. Rejouer T001 GREEN. FR03–05, FR17–18. Dépendance : T001.
- [x] T003 — Ajouter et exécuter les tests RED des lectures humaines dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/threads.rs`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`. Comparer données métier avant/après lectures et refus, sans comparer les octets SQLite/WAL. Tester lecture/refus avant collect_closed_attach_views, fil forgé, révocation, racine Git canonisée différente du workspace, noms absents, projection sans own_acked_seq/wake/credentials, corps exact, limites, plafond128 Kio et trois pages ASC à snapshot stable. FR03–12, FR21–22. Dépendance : T001.
- [x] T004 — Implémenter la résolution et projection humaines en réutilisant list/show/history et Store::agent_display_name dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/threads.rs`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`. Membres/noms liste par SQL groupé,16 membres maximum ; early return avant maintenance ; enums d'erreur fermés. Aucun reçu, read, ACK, dispatch ni migration. Rejouer T003 GREEN. FR03–12, FR21–22. Dépendances : T002, T003.
- [x] T005 — Ajouter et exécuter les tests RED de CLI inspect et négociation dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/cli.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/communication/client.rs`. Vérifier arguments fermés, rôle Client, daemon absent sans autostart/namespace, retour avant initialize_process/resolve_current_identity, version incompatible, résultat JSON sur exit2 et absence d'accès stockage direct. FR05–07, FR11, FR16–18, FR21. Dépendance : T001.
- [x] T006 — Implémenter `bridget thread inspect` avec Namespace::from_environment sans prepare et DaemonConnection::connect direct dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/cli.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/communication/client.rs`. Intercepter inspect avant initialisation et identité. Négocier capacité/version, projeter résultats exacts et refus, puis rejouer T005 GREEN. FR05–07, FR11, FR16–18. Dépendances : T004, T005.

<a id="extrait-3"></a>

## Extrait 3

- [x] T007 [P] — Ajouter et exécuter les tests RED du service serveur dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/bridget/BridgetReader.test.ts`. Faux ProjectionSnapshotQuery et ProcessRunner : racine issue du projet, autre environnement, argument forgé, argv sans shell, exécutable config/binaire utilisateur/PATH, rejet cmd/bat, délai6s, cap256 Kio, projection128 Kio, invalid UTF-8/JSON/version, stderr non exposé et JSON métier exit2 décodé. FR03–07, FR11, FR16–19, FR21. Dépendance : Analyze PASS et GO implémentation.
- [x] T008 — Ajouter les schémas humains et une seule méthode `bridget.read` dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/contracts/src/bridget.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/contracts/src/rpc.ts` et l'export natif du paquet ; implémenter le service Effect dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/bridget/BridgetReader.ts`. Résoudre conversation/projet côté serveur ; aucune identité navigateur ; exécution bornée et erreurs fermées. Rejouer T007 GREEN. FR03–07, FR11, FR16–19. Dépendance : T007.
- [x] T009 — Ajouter tests RED d'autorisation/montage, puis intégrer AuthOrchestrationReadScope et service dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/auth/RpcAuthorization.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/auth/RpcAuthorization.test.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/ws.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/server.test.ts`. Vérifier connexion read autorisée, scope absent refusé et absence de génération/action. Rejouer GREEN. FR03–07, FR17, FR19, FR21–22. Dépendance : T008.
- [x] T010 — Ajouter la lecture d'environnement Bridget dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime/src/state/orchestration.ts` avec les factories natives existantes. Ajouter tests RED puis GREEN de routage et invalidation au même module ou à sa suite existante attestée. Aucun polling ni commande métier. FR04, FR14–15, FR17, FR19. Dépendance : T008. Le montage serveur T009 reste requis avant la validation T017.

<a id="extrait-4"></a>

## Extrait 4

- [x] T011 [P] — Ajouter et exécuter tests RED du store/onglets/panneau dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/rightPanelStore.test.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/RightPanelTabs.test.tsx` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.test.tsx`. Vérifier surface singleton native par conversation, migration, icône b neutre, titre, ouverture/fermeture, liste autorisée, membres, noms/IDs, vide, liaison absente/ambiguë et réponse A tardive après B. FR01–04, FR08, FR10, FR15–16, FR19–22. Dépendance : Analyze PASS et GO implémentation. Tests mockés indépendants du service et de T010.
- [x] T012 — Étendre `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/rightPanelStore.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/RightPanelTabs.tsx`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/ChatView.tsx` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/routes/_chat.pull-requests.tsx` ; créer `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.tsx`. La partie store/onglets native peut commencer après T011. Monter le panneau avec référence environnement/conversation, sans persister les corps. Ne déclarer le montage des queries GREEN qu'après T010. Rejouer T011 GREEN. FR01–04, FR08, FR10, FR15–16, FR19–22. Dépendances : T011 pour la surface ; T010 pour son montage complet.

<a id="extrait-5"></a>

## Extrait 5

- [x] T013 — Ajouter et exécuter tests RED de détail/historique dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.test.tsx`. Trois pages ou plus ASC, progression snapshot, aucun trou/doublon sur données stables, changement entre plusieurs fils, noms absents, types History/Action/Blocker/Decision et legacy « Message », dates, texte Unicode/multiligne exact et copie source, refus après révocation et plafonds. FR03, FR07–12, FR15–16, FR20–21. Dépendance : T012.
- [x] T014 — Implémenter détail, membres, rendu exact, copie de chaîne source et pagination explicite dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.tsx`. Isoler pages par fil/contexte/génération ; réutiliser snapshot sous to_seq ; invalider données révoquées ; rejouer T013 GREEN. FR03, FR07–12, FR15–16, FR19–20. Dépendance : T013.

<a id="extrait-6"></a>

## Extrait 6

- [x] T015 — Ajouter tests RED puis implémenter la recherche locale et le rafraîchissement dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.tsx` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.test.tsx`. Tester portée « données chargées », recherche titres/membres/auteurs/corps, zéro appel à la frappe, effacement, relire liste/fil courant avec nouveau snapshot, absence de polling et zéro mutation. FR06, FR10, FR13–15, FR20–21. Dépendance : T014.

<a id="extrait-7"></a>

## Extrait 7

- [x] T016 — Ajouter tests RED puis compléter les états et interactions dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.tsx`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/BridgetPanel.test.tsx` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/RightPanelTabs.test.tsx`. Vérifier clavier/focus/noms accessibles, chargement/vides/erreurs/retry, daemon absent/incompatible/délai, fermeture et révocation pendant requête, A → B → A sans résurrection, Sheet/panneau étroit et montage Pull requests. FR01–02, FR14–16, FR19–22. Dépendances : T012, T014, T015.
