# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/validation/converge151-passage2.md](../../../../../../../specs/151-attente-fin-de-tache/validation/converge151-passage2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Repère | Chemin absolu |
|---|---|
| N | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon/native_delegation.rs |
| D | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon.rs |
| T | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation.rs |
| L | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation_lineage.rs |
| M | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/mcp.rs |
| F | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation_mcp.rs |
| C | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/communication/client.rs |
| R | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/ledger_requests.rs |
| I | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/idempotency/tests.rs |
| S | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/tests.rs |
| P | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-transport/src/protocol.rs |
| W | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/wrapper.rs |
| E | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/native_delegation_e2e.rs |
| G | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_skill_test.rs |
| B | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/t3code.rs |
| V | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/tests-151.md |
| A | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-ancien-daemon.md |
| DOC | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/docs/delegation-native.md |
| SK | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/skills/bridget/SKILL.md |
| CMD | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/skills/bridget/references/commandes.md |

<a id="extrait-2"></a>

## Extrait 2

| FR | Réalisation actuelle | Tests et preuves |
|---|---|---|
| 001 | F:41,F:98 ; N:469,N:493 | F:201,F:230 ; N:2535 ; P:7912 |
| 002 | N:523, tout le lot | N:2535 |
| 003 | N:535, ensemble prêt ordonné | N:2364 ; N:2622 |
| 004 | N:483, aucune mutation | N:2364, payload avant/après |
| 005 | N:510 ; T:15,T:25 | N:2647,100 répétitions ; T:410 |
| 006 | N:546 | N:2420 ; E:301 |
| 007 | N:551 ; D:315 | N:2496 ; N:2420 ; N:2273 |
| 008 | N:493 ; D:188,D:194 | N:2535 ; N:2679 ; M:2771 |
| 009 | N:511 ; D:211,D:270,D:204 | N:2580 ; E:749 |
| 010 | N:393, attente avant actions | N:2420, pause active |
| 011 | F:166 ; M:760 ; C:21 | M:3519 ; budgets vérifiés dans le code |
| 012 | M:33,M:60 | M:2815 ; M:2610 |
| 013 | M:83,M:96 ; C:953 | M:2858,M:2928,M:3449,M:3631 |
| 014 | N:109,N:119 | N:2893 ; N:2364 |
| 015 | N:102 | N:2822, neuf états |
| 016 | N:84,N:87 | N:2868, limites UTF-8 |
| 017 | N:117 | N:2822 |
| 018 | N:90,N:126 | N:2798 ; N:2922 ; E:454 |
| 019 | N:483 ; D:242 ; Status sans tick | N:2364 ; N:2922 ; E:454 |
| 020 | N:1243, remise normale indépendante | E:301 ; E:749 |
| 021 | DOC:75 ; SK:117 ; remise conservée | E:301 |
| 022 | L:35, delivery_acked conservé | N:3267 ; régressions150 dans V |
| 023 | N:535 | N:2622 |
| 024 | Classement T3 conservé | B:3632, famille Message et regroupement |
| 025 | M:760 ; D:242 ; Send inchangé | M:2771 ; M:3519 ; G:253 |
| 026 | R:272 ; D:275 | N:2192 ; S:144 |
| 027 | D:288,D:295,D:306 | N:2125 ; N:1958 |
| 028 | R:699,R:717 ; résumé et état atomiques | I:89,I:121,I:211,I:279 ; S:49,S:82,S:144 |
| 029 | D:242, aucun envoi | N:2192 ; G:253 |
| 030 | Injection normale avant clôture ; DOC:142 | G:221,G:253 ; limite explicitée |
| 031 | D:18221 ; C:24 ; F:156 | C:1237,C:1270 ; P:7938 ; A:31 |
| 032 | N:628,600/3600 | M:2470 ; M:3519 |
| 033 | W:5056,W:5272 | W:8130 ; M:2470 |
| 034 | N:84, filtre150 conservé | N:2893 ; P:7947 ; M:3193 ; W004 |
| 035 | DOC:75 ; SK:100 ; CMD:122 | G:253 ; E:454,E:707, exemples exécutés |
| 036 | N:796 ; T:190 | N:2976,N:3038,N:3076,N:3114,N:3157 |
| 037 | N:91 ; L:276 ; ACK sans projection | N:3226 ; N:3267 ; E:707 |
| 038 | M:190,M:96 | M:2985,M:3042,M:3193,M:3295 ; E:749 |
| 039 | Bumps après commit D:7287,D:7979,D:16408,D:18156,D:18476,D:18523 | N:1958 ; N:2243 ; N:2273 ; I:279 |
