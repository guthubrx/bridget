Contrôle indépendant du correctif cwd : PASS_TARGETED_CWD.

Ce contrôle est limité à FR-019, aux deux contrats et aux gardes SC-002/SC-014. Les trois passages Converge sont clos et restent historiques. Aucun quatrième passage global n’est exécuté.

Le transport conserve cwd comme chaîne ou null. Absence donne null. Les autres types et les chaînes de plus de 1024 octets UTF-8 donnent details_incomplete. Le daemon affiche la chaîne entière avec Debug, garde cwd_verified=false et refuse les faits au-delà de 4096 octets. Il ne fabrique aucun dossier de repli ni preuve de confinement.

Preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-transport/src/codex_app_server.rs:2440,2603
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/native_permissions.rs:73,201
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:529,542

Les oracles existants Human/T032/transport couvrent cwd hostile complet, contrôles visibles, null/absence et dépassement UTF-8. Les autres types invalides et le budget total des faits sont prouvés ici par les gardes sources ; aucun nouveau test spécifique n’est revendiqué.

214 sources correspondent au reçu post-cwd : 5 changées et 209 inchangées depuis P3. Le scope compte 13 fichiers changés (5 Rust et 8 documents), 48 inchangés. Tasks reste à 42/42, SHA 8d63662e3d278cdbb5f04d725344c27ab250b324c5e99bd64ef17ae35b0225bb. Les documents ont été relus dans leurs passages cwd et provenance, pas intégralement.

Le reçu du principal atteste 71 PASS, RC0, 64.81 s et 214 sources stables. Son log SHA est vérifié : 2be93808d3211bfe126877fec072bae9eba5c2d8652f575ac3c353a531d7ba16. Aucun test n’est exécuté par cet auditeur. La recette réelle 4/4, les quatre régressions canal087/STDIO et la suite complète antérieure restent historiques pré-cwd. Aucun total 75 PASS sur le dernier snapshot ni GREEN global n’est déduit.

H3 et q² sont relus et maintenus. PERF-082 reste LOW/THEORETICAL. Aucun nouveau finding. Plages, SHA et limites :
/Users/moi/.cache/bridget-delivery155.um4xccv4/audit-delta155-cwd.json
