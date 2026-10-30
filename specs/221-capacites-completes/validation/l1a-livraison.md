# L1a/C25 — livraison du 30 octobre 2026
Statut du programme : In Progress,8/176tâches closes. Premier lot technique livré pour les nouveaux superviseurs Linux ; aucune clôture globale des50catégories. Cadrage conservé, décision utilisateur : interfaces complètes web/bureau/iOS/Windows pilotant Mac/Linux, pas de moteurWindows.

## Git, code et contrôles
Commit fonctionnel86a88e3fb165b8a7099d0bc1806a604b48500c30, intégré à main Cartae et poussé vers origin GitHub ; ls-remote vérifié égal. Main Mac synchronisé depuis github. Les miroirs provisionnés seront synchronisés au commit documentaire de clôture ; HEAD de préparation détachés préservés.
Six tests distincts C25 et guides32pages/522liens/13paires PASS. Convergence L1a passage1 : T176ajoutée pour identité du journal ; passage2 : CONVERGED sans modification de tasks.md. Le programme global reste non convergé.
Audit V15 source : decision=pass/release_blocked=false,12/12contrôles applicables,32critères de qualité notés sur42dont10hors périmètre ; notes67–75/100. Ce résultat porte sur les empreintes des cinq fichiers Rust de l'instantané d'audit, identiques au commit fonctionnel. Les comptes rendus documentaires postérieurs ne sont pas présentés comme un nouvel audit du produit entier.
Contre-revue d'implémentation demandée, refus capability_unavailable/not_recorded/task_id=null. Aucun avis favorable d'implémentation externe ; relecture locale explicitement non indépendante. Deux contre-revues de conception antérieures conservées avec décisions et limites.

## Construction compatible et installation
Premier candidat construit sur Cartae refusé avant installation : demande GLIBC2.39, trop récent pour SYS2.36. Aucun service coupé pour ce candidat.
Archive immuable de86a88e3f transférée et vérifiée : SHA2566534a206b8c4d31a6e0a4d34425cc671c536a3c1f0faf1a533ce58ddf856493b. Construction Linux commune sur SYS avec Rust1.92 et manifest source221, cache partagé source204/target. Le scripts/build.py utilisé est strictement identique au script actuel (SHA25678f21ac00c2beb6ce0c5a474e7f5bf19c1e07380510fdb6d19192804a2c42775), entretien avant/après effectif ; aucun Cargo lancé directement.
Build221-drain-86a88e3f, version commerciale0.6.5 ;39804048octets ; exigence GLIBC maximale2.34.
SHA256ad7933fb7d74397e78949279ce04a604ff5d45cd579bd3ff4be82272a44abcd0.
Chemin installé sur Cartae/SYS/KS : /home/moi/.local/lib/bridget-communication/221-drain-86a88e3f/bin/bridget.
Contrôle clients local puis validation par le binaire installé précédent : compatible, inventaires non vides Cartae2/SYS2/KS1. Aucun compte, modèle ou programme fournisseur changé.

## Chargement physique
| Machine | Composant | PID chargé |
| --- | --- | --- |
| Cartae | maître |1822411|
| Cartae | service mobile |1822504|
| Cartae | worker cartae |1823070|
| SYS/sysle2 | worker sysle2 |563182|
| KS/ksstore | worker ksstore |953374|
Image /proc/PID/exe et SHA identiques vérifiés pour les cinq processus. L'updateur Cartae est Python, PID1822510 : son code inline est identique au script courant, SHA2564deb2743904e447e5cd022eefb73727e8234866960d7d5ec4051b463b7300493. Ne pas le présenter comme un sixième processus du binaire Rust.
Sauvegardes cohérentes avant/après arrêt, verrou de l'updater et unités contrôlées. Coupure du maître2,41s. Ancien actif220conservé comme retour ; aucun gel de délégations ajouté. session native et capabilities après retour PASS ; delegations_halted=false,45options déclarées (pas une preuve d'accès à tous les modèles).
Aucun service métier, compte ou ancienne admission relancé. La correction est Linux ; worker/pont Mac220non modifiés. MCP58296de ce fil reste214, relu par ps ; aucun nouveau schéma MCP n'est requis par C25.

## Qualification native après activation
Tâche af2882a8-e5d0-428b-9c22-04fbd43e0004 ; clé bridget221-qualification-drain-20261030-01 ; base86a88e3f ; sélection automatique Claude/Haiku Cartae, discovery. Aucun outil, fichier ou mission métier demandé.
Résultat BRIDGET221_DRAIN_OK réellement reçu ; ACK puis une annulation propriétaire ; état stable result_available/error=null, guarded=false, cleanup_done=true. guard_released=false ne désigne pas une garde active ici, guarded=false.
Reçu worker terminal, commande et tâche af2882a8-e5d0-428b-9c22-04fbd43e0004, instance01b894ea-b031-4643-8b1a-cf937eb07965,génération1536. Marqueur PID1829884/naissance287684216 absent après clôture. Preuve .drained existante : worker_digest, command_digest et marker_digest recalculés depuis le reçu et comparés exactement ; SHA2569288236e0369a6d58274394a6e0b533ee11744577447d024824b5e56e3f5e9c8.
Le PID éphémère avait déjà terminé avant la lecture de son /proc/exe : aucune image directe de ce PID inventée. Le worker démarre le wrapper depuis current_exe ; le worker chargé est vérifié221. La preuve de commande et la disparition du PID sont directement observées.

## Limites physiques conservées
Trois superviseurs déjà vivants sont préservés, PID/naissance inchangés pendant les passages workers :
- Cartae2500061, tâche historique1cc44f4d, binaire183.
- SYS492283, tâche56b587e5, binaire217.
- SYS544924, tâche18f5c5a1, binaire220.
Ils ne chargent pas le correctif par remplacement du fichier. Leur travail et leur issue ne sont pas dupliqués ni forcés. Ne pas annoncer tous les superviseurs ou clients chargés221. Le correctif qualifié s'applique aux nouveaux lancements.
Aucune égalité comparative globale ne résulte du test C25 ; référence comparative de ce lot NON ÉTABLIE.

## Nettoyage borné
Résultats des deux revues lus/ACK et clôture native effectuée ; contrôle maître : guarded=false,cleanup_done=true. Leurs rapports sont conservés.
Retirés après Git propre, aucun fichier ignoré et observation système complète : worktree de qualification af2882a8Cartae et revue8dcf6395KS, avec leurs deux branches. La branche KS a été supprimée par git branch -d contre main via upstream local temporaire ; HEAD détaché16304623préservé, aucun forçage.
Un exécutable candidat Cartae refusé et cinq copies de transfert/archives retirés : trois fichiers Cartae, un KS, deux Mac. Aucun gain physique chiffré revendiqué.
Conservés : worktree de revue e2d6ed90SYS (observation processus inconnue : droits/délai), workspace de développement221actif, source/cache/archive SYS de construction pour reproductibilité, preuves privées et exécutables sous usage. Actif221et retour220conservés sur Linux ; Mac actif220/retour antérieur et MCP214sous usage préservés.
installed appelé après vérification hors staging sur Cartae/SYS : caches1,22/0,98Gio, sous budget10Gio. Aucun cache par test ni copie de target créée.

## Information et preuves publiées
Les quatre responsables monuments, Politique sol, trajets bus et Roadie ont reçu des messages avant/après, reply=false. IDs bridget221-l1a-avant-* et bridget221-l1a-apres-*, reçus in_flight : pas de lecture déduite ni de demande d'accusé.
Bilan HTML vérifié à728px clair et360px sombre, console sans erreur et sans débordement. Il présente les faits natifs et les limites, sans fabriquer d'interface produit. Preuve technique, aucune interface graphique modifiée.
Fichier : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/221-capacites-completes/validation/l1a-preuves.html.
Publication :4ffa692b-bf9f-4dc2-884e-ed57b74e0565-4c4c7b1f-171c-4a02-9138-384f2b64b0ae-html.
Reçus détaillés privés : /Users/moi/.cache/bridget221/activation-verified.json ; cleanup-receipt.json ; notices-before.json ; notices-after.json. Audit source : /home/moi/.cache/bridget221/audit-l1a-r1.
Première tâche suivanteT008/C26 : éprouver les rejeux de mutations sans doublon. Le reste du programme est autorisé et non terminé.
