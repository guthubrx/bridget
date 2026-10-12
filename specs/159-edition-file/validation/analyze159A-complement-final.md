# Analyze159A — complément final unique de conception
Date UTC : 2026-10-12T13:45:36.604327+00:00.

Les six findings initiaux sont résolus dans la conception. Aucun CRITICAL, HIGH ou MEDIUM restant dans ce complément ciblé. Deux références LOW restent corrigeables sans nouvelle tâche, test ou passe Analyze. **GO code fermé : T001 non exécutée.**

| Finding initial | Statut | Preuve |
|---|---|---|
| C01 | Résolu conception | ADR initiale A adoptée avant code ; T012 complète les preuves et T025 étend le même fichier B. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/docs/decisions/159-file-retenue-et-edition.md:3; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/tasks.md:46 |
| C02 | Résolu conception | Libellés en français ; les 16 cases demeurent. Aucune exigence modifiée. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/checklists/requirements.md:1 |
| I01 | Résolu conception | AUTOINCREMENT remplace MAX+1. Le watermark SQLite survit aux purges. delivery_id UNIQUE préserve les FK. B ne crée que permutations de rangs existants. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/data-model.md:21; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/tasks.md:29 |
| U01 | Résolu conception | Échéance Instant contrôlée à chaque boucle même sous trafic continu, ≤30s, réemploi heartbeat sans scheduler supplémentaire. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/tasks.md:35; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/plan.md:84 |
| I02 | Résolu conception | Le SELECT MAX O(N) disparaît du chemin prévu. Admission indexée O(log n), balayage global O(n+h log n) déclaré ; coûts SQLite non mesurés et T001/T010 maintenus. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/research.md:121 |
| D01 | Résolu conception | Même idx_send_deliveries_recipient_phase étendu non unique à (recipient_instance_id,phase,queue_rank), sans index recipient doublon ; préfixe conservé. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/data-model.md:32; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/plan.md:86; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/tasks.md:29 |

Mapping documentaire vérifié : 26FR,15SC,7US→27IDs inchangés, A13/B14 séparés. T001–T013 sont A ; T014–T027 B différé. FR021 documentation/ADR partagé est couvert dès A par T012. Matrice complète et provenance dans le JSON. Aucun pourcentage de couverture code déduit.

Réemploi courant PASS5/5 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/reuse-audit.md. 33items A,23réemplois/10créations motivées. Les 13sources du reçu T002 correspondent aux octets courants. Leurs plages sont héritées, pas présentées comme relecture code exhaustive.

Atomicité : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/atomicity-readonly.md:6–16 permet une réalisation State→transaction Immediate dans config.db_path commun. Lecture pause et interruptions dans cette transaction, pas seconde connexion writer ni socket. Push hors State après commit. La pause traitée après commit ne rétracte pas une remise déjà libérée. Cette faisabilité ne vaut pas réalisation159.

SC001 maintenu : chemin message/remise fermé identique, suites vertes et zéro nouvelle régression. La réattestation auxiliaire FR023 demeure explicite. Aucune dérogation legacy ni GREEN global déclaré. T001 référence/tests/latence reste avant code ; T011 backup/migration/refus vrai ancien binaire/restauration reste avant livraisonA.

Résiduels LOW :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/research.md:126 — R6 renvoie à T023 pour ADR/doc alors que T023 est E2E B. La responsabilité A est désormais T012. Correction : (T011, T023) → (T011, T012).
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/plan.md:160 — Le libellé pipeline152–158 ne comprend pas la présente session159. Le périmètre utilisateur autorisé est152–160. Correction : Remplacer seulement pipeline152–158 par pipeline152–160 dans ces deux phrases.

Limites : complément ciblé après patchROOT. Aucune troisième boucle, code/test/service/DB/Git ou tâche modifié. Les neuf fichiers capturés au début sont restés identiques à la fin : True. SHA des autres artefacts sont la capture finale ; pas affirmation de gel exhaustif au-delà de ces gardes.
Tasks SHA256 : 726f5ea5d2d1fcdc39aa55e35e3be926fc2dba28ac52463a97f7e50af9703810.

Complément reçu ROOT après la lecture unique : R2 Claude APPROVE_WITH_CHANGES, aucun défaut logique bloquant annoncé. Corrections documentaires attendues après Cargo/gel : repère connexions plan3058→4806–4818 ; lectures control_state/control_pause_interruptions explicitement APRÈS BEGIN IMMEDIATE dans la même transaction ; SQLITE_BUSY conserve held sans refus ni push et diffère au prochain déclencheur. Le risque de garder State acquis pendant busy_timeout2s est déclaré et couvert par les cas concurrents T007 déjà prévus, sans nouveau scénario. Ces corrections ne sont pas réputées appliquées ni relues. T001 compile selon ROOT ; réussite non acquise. Aucune troisième Analyze/revue engagée.
