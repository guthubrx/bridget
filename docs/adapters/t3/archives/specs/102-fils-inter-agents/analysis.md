# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/analysis.md](../../../../../../specs/102-fils-inter-agents/analysis.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Exigence | Réalisation | Preuve de test |
|---|---|---|
| FR-001 | T:337 create, T:408 list, T:435 show ; S:742, S:1042, S:1079 | I:221 V01 |
| FR-002 | S:819 allocation `seq = last_seq+1`, aucune API d'édition ou de suppression | I:266 V02, I:605 V31 concurrence |
| FR-003 | S:819 intentions seulement pour les cibles ; T:474 aucun scan du corps | I:310 V03 |
| FR-004 | S:819 `NotAMember` annule tout le dépôt ; `All` résolu sur les membres | I:903 V04, I:433 V21 |
| FR-005 | T:474 cibles structurées seules ; cli.rs:902 refus inconnu/ambigu | I:903 (citation @all), cli `spec102_v07` |
| FR-006 | S:1298 candidats, S:1338 réservation figée, `pending_seq` watermark | I:1014 V08, I:1055 V09 |
| FR-007 | S:116 `WakeRow::to_json`, motifs `offline/dnd/capability_unavailable/rate_limited` | I:1105 V10, I:1232 V25, débit I (v10_debit) |
| FR-008 | S:1100 read, S:623 `read_range` (nombre + octets, jamais mi-entrée) | I:1409 V11, I:1490 V13, I:1513 V14 |
| FR-009 | S:683 `apply_ack` (dernier reçu, CAS `base_seq`, suppression du reçu) | I:1450 V12, I:1595 V16, I:1675 V17 |
| FR-010 | S:1246 history, borne figée, sans reçu | I:1716 V18 |
| FR-011 | S:542 `lookup_operation` (rejeu exact / `EnvelopeMismatch`) | I:341 V19, I:400 V20 |
| FR-012 | D:10678 `live_connection_identity` ; matrices Service/Client ; attach refus par défaut | I:736 V22, I:433 V21 |
| FR-013 | D:7392 `dispatch_thread_wakes` : différé offline/dnd/capacité, jamais de lancement | I:1105 V10 |
| FR-014 | S:279 schéma durable ; D:7516 reprise même clé/instance ; S:1446 échéance → inconnue | I:1160 V24, I:1232 V25, I:1767 V26, I:650 V34 |
| FR-015 | cli.rs:860 `cmd_thread`, mcp.rs:457 `bridget_thread` ; capacité annoncée wrapper.rs:1648 | I:1879 V33, I:1295 V27 |
| FR-016 | commandes.md « Synthèse demandée » ; aucun code de résumé | relecture (V30 documentaire) |
| FR-017 | T:37 `LIMITS`, S:742/819 contrôles transactionnels | I:2051 V31 quotas, S `spec102_quota_tests`, I:1513 V14, I:2132 V35 |
| FR-018 | S:980 close (créateur, annulation des intentions non parties) | I:482 V32 |
| FR-019 | SKILL.md « Fils partagés », commandes.md « Fils partagés (102) » | core_089_skill_test (exemples), relecture (V36 documentaire) |
| FR-020 | D:7988 et D:13179 notice client neutralisée ; t3code.rs:2413 enveloppe sans attente ; cli.rs:838 marqueur | I:1801 V23, t3code `spec102_v28`, cli/wrapper `spec102_v29`, DM témoins I:2132 |
| SC-001 | zéro trame chez C/D/A sur 20 échanges | I:903 V04, I:310 V03 |
| SC-002 | seulement 101–110 après confirmation 1–100 | I:1409 V11 |
| SC-003 | coupures : arrêt coopératif du daemon, réponse perdue, reçu durable | I:1450 V12, I:341 V19, I:1160 V24, I:1767 V26 |
| SC-004 | dix mentions → une génération ; mention concurrente détectée | I:1014 V08, I:1055 V09 |
| SC-005 | p95 3,5 ms sur 200 opérations, 8 DM témoins remis | I:2132 V35 |
| SC-006 | codes précis, aucune mutation partielle | I:433, I:531, I:736, I:1595 |
| SC-007 | documentation auto-portante (skill + référence) | relecture (V36 documentaire) |
