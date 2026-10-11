# Référence avant code151

Date : 2026-10-11. Branche session-151-attente-fin-de-tache. Base main
1f2c661d39a9ce106723a93ac883f76ab9fe52c6 + import non commité du socle150
(empreintes dependency150.json). Recette quickstart, env privé et umask077.
Commande par filtre : cargo test -p bridget-daemon --features test-support FILTRE -- --test-threads=4.
Cache Cargo 150 réutilisé ; aucune source150 modifiée.

| Filtre | PASS | FAIL | ignorés | code |
|---|---:|---:|---:|---:|
| native148 | 27 | 0 | 1 | 0 |
| native149 | 78 | 0 | 0 | 0 |
| native150 | 51 | 0 | 0 | 0 |
| delegation149 | 10 | 0 | 0 | 0 |
| spec_087 | 11 | 0 | 0 | 0 |
| batch | 0 | 0 | 0 | 0 |

Le filtre batch ne sélectionne aucun test existant : ce fait est conservé,
aucune couverture n'en est déduite. T014 teste directement le regroupement.

## Faits T001

- send_delivery_phase : idempotency/send_delivery.rs:323, Result<Option<String>>.
- task_view : 13 clés historiques ; test native148_cut_after_admission... compare
  la vue avant/après réouverture. Catalogue fermé native149 (9 clés), MCP24 outils,
  outils sûrs20, TOML complet spec091. Les trois nouveaux outils donnent 23 sûrs.
- DaemonIdentityReport : 5 constructions (daemon3, protocole1, core_089_status_test1) ;
  3 motifs exhaustifs. Ils seront adaptés sans changer host/db_path/instance_id.
- Codex : 60s par outil MCP par défaut, tool_timeout_sec par serveur (source officielle
  https://developers.openai.com/codex/config-reference/). Claude Code :
  MCP_TOOL_TIMEOUT100000000ms, stdio idle1800000ms selon version (source officielle
  https://code.claude.com/docs/en/env-vars). Montage Bridget Codex fixé à3700s ;
  hôtes externes documentés, sans changement de configuration globale.
- Inactivité réelle : idle-probe.json prouve 15.011s sans trame, puis réponse
  DaemonIdentityReport sur le socket isolé. Aucun Register utilisé.
- RequestWait : Client autorisé, Service refusé dans les matrices exhaustives ;
  wrapper implicite conserve l'authentification auxiliaire habituelle.
- Écrivains tracked_requests : ledger_requests create251, deadline305,
  cancelled332, timed_out376, escalation394, migration sender615/target621,
  answered686 ; send_delivery insert151 et Store purge454. Clôtures : ACK
  idempotent, wrapper mark_answered héritée, annulation, claim_timeout, guichet.
  Le compteur sera incrémenté après les commits des cinq clôtures.
- Aucune mémoire Codex Bridget trouvée dans les deux racines de mémoire annoncées.
  Le contexte transmis et les preuves150 sont les références de reprise.
- check-prerequisites.sh absent, commande réellement tentée (exit127) ; contrôle
  manuel : tous les artefacts existent, 26 tâches uniques, gate reuse-audit PASS.

## Self-review ArticleXIX/XX T001

Nécessité : prouver le socle et les hypothèses avant code. Solution simple :
réutiliser les filtres et le binaire de test, sans dépendance. Hypothèses : recettes
privées des tests existants. Vérifié : commandes, compteurs, probe15s, sources
officielles et exploration unique. Non vérifié : runtime de production, qui est
hors de cette livraison. Code évité : aucun outil de test permanent ajouté.
