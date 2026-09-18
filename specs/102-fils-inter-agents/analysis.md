# Analyze102 — cohérence documentaire

Date : 2026-09-16. Périmètre : spec/plan/tasks, modèle, contrat, recherche,
audit de réutilisation et recette prévue ; lecture du socle réel, aucun code102.
Primitives appliquées manuellement : scripts check-prerequisites/setup absents
constatés par ls ; les instructions des skills ont été lues et suivies.

L'analyse elle-même est une lecture ; les corrections ci-dessous sont effectuées
ensuite par l'étape Auto-Fix de my-specify-all, autorisée pour les documents.

## Première passe : findings et traitement

| ID | Catégorie | Sévérité | Emplacement | Constat | Correction ciblée |
|---|---|---|---|---|---|
| A01 | Cohérence | HIGH | plan§7 / tasksT021 | Recherche du reçu actif avant last_ack pouvait refuser le rejeu après suppression | Contrôler identité puis last_ack avant reçu actif |
| A02 | Bornes | HIGH | contrat Bornes / modèle opérations | Nouvelle clé close après clôture pouvait faire croître operations sans limite | thread_closed sans insertion ; seul rejeu exact réussi renvoie closed |
| A03 | Pagination | HIGH | contrat Bornes | Entrée pouvant occuper tout60Kio sans place pour enveloppe de page | Entrée sérialisée≤48Kio, pagecomplète≤60Kio,12Kio réservés |
| A04 | Compatibilité | HIGH | plan§8 / contrat alerte | « capacité négociée » ne disait pas où la transmettre | Champs facultatifs Register/Registered, connexioncourante, aucun ClientCapability auxiliaire détourné |
| A05 | Disponibilité | MEDIUM | plan§6 | Refus prouvé avant injection et issue inconnue confondus pour reprise | Nouvelle tentative après changement de cause seulement si refus prouvé ; autre règle pour nouvelle mention après unknown |
| A06 | Maintenabilité | MEDIUM | contrat résultats | Statuts et formats de sortie trop laissés à l'implémenteur | Discriminants fermés, sorties page/ACK/erreur exemplifiées, types de notify explicités |
| A07 | Traçabilité | LOW | fiche synthèse | Compteurs et présence des artefacts restés provisoires | 0/32 tâches et0/36 scénarios, statut cycle In Progress sans code commencé |
| A08 | Réutilisation | HIGH | plan projection099 / data-model | Scope interne, instance figée et délai120s pas assez distingués de l'horizon0997jours | Patron supervisor_scope/reserve/begin_send_delivery explicité ; identités transport persistées ; délai d'injection distinct du TTL de clé |
| A09 | Compatibilité/sécurité | HIGH | communication.rs:64, idempotency/send_delivery.rs:176 | Canon099 champ par champ ignore nouveau champ ; réaffectation générique peut déplacer une notice | Extension canon conditionnelle préservant golden DM ; exclusion typée des notices du reroutage inter-instance, oracles V20/V25/V27 |

Les objections bdget avaient été reçues avant tasks et traitées séparément dans
adversarial-review-bdget.md : nouvelle mention après issue inconnue, ACK tardif,
reply implicite, métadonnées visibles et lecture humaine. Elles ne constituent
pas une validation de code. La borne de départ5/s reste justifiée et l'ajout d'un
scheduler indépendant est explicitement exclu.

## Seconde passe

Réexamen des changements contre les invariants :

- ACK sans ligne active après confirmation : last_ack reconnu en premier.
- ACK tardif courant accepté ; reçu remplacé refusé ; aucune avance arbitraire.
- Post+ACK en transaction ; rejeu exact post contrôlé avant validation du vieux reçu.
- Nouvelle mention strictement supérieure peut repartir après échéance unknown ;
  la même borne ne repart pas aveuglément, et un ACK ancien ne touche pas la suivante.
- Quotas entrées/fils/opérations bornés ; pages ne coupent pas un message.
- Identité aux frontières, aucune entrée privée copiée au ledger ; absence de
  capacité ne devient jamais un DM, catalogue MCP vérifié séparément.
- ThreadNotice distinguée des observations et des DM ; reply implicite protégé.
- Huit actions, six tables,21 items audités ; aucune dépendance/service nouveau.
- Toutes les tâches restent non cochées, aucune commande future exécutée.

**Issue : aucun finding CRITICAL/HIGH documentaire non résolu identifié à la
seconde passe.** Cela n'atteste pas le futur comportement runtime.

## Couverture

La matrice détaillée exigence→tâches figure à la fin de tasks.md ; la matrice
scénario→exigence dans test-plan.md. Contrôles statiques exécutés avec Node en
lecture seule (script éphémère, aucun fichier de test produit) :

| Mesure | Résultat |
|---|---:|
| Exigences fonctionnelles | 20 |
| Critères de succès | 7 |
| Exigences/critères avec au moins une tâche | 27/27 (100 %) |
| Tâches numérotées sans trou ni doublon | 32 |
| Références à un ID de tâche absent | 0 |
| Scénarios de vérification planifiés | 36 |
| Tâches d'implémentation cochées | 0 |
| Exemples JSON de contrat analysables | 8/8 |
| Marqueurs de clarification restant dans les artefacts normatifs | 0 |

Tâches transversales non rattachées à une seule story : T001–T007 et T026–T032,
justifiées par socle, tests, sécurité, compatibilité et livraison. Aucun ajout
de fonctionnalité sans besoin ; le guide donne les premières commandes de reprise.

## Constitution, minimalisme et responsabilité future

Isolation et absence d'implémentation vérifiées par git status/diff. Aucun code,
manifeste ou test exécutable nouveau ; modifications limitées aux artefacts et
contexte SpecKit de la branche. Pas de commit ni de service relancé.

L'exception utilisateur arrête volontairement Implement/Converge/audit du code.
La préparation est terminée, le cycle logiciel reste In Progress (0/32), non
Implemented. Le statut Ready n'est pas utilisé comme une autorisation implicite
de développement ou un verdict de recette réelle.

Articles XVIII–XX : index et bornes de parcours ; pas de routeurLLM, ni broker,
ni nouvel orchestrateur. Complexité utile explicitée : reçus, intentions durables
et négociation. Potentiel minimalisme code : non applicable, zéro ligne de code
produite ; aucun chiffre de lignes supprimables inventé. La charge de maintenance
augmente avec l'état fiable, mais les invariants et tests permettent au mainteneur
de l'expliquer sans relire le chat. Les pages séparées history/read ont des effets
différents justifiant leur maintien.

## Limites et prochaine action

- Le worktree102 est basé sur1738a072 ; les changements101 non commitées ne sont
  pas intégrés. T001 exige une base commune établie avant édition du code partagé.
- L'accès humain est via un agent membre ; pas d'APIadmin historique nouvelle.
- Conservation sans purge automatique : à256fils, création refusée ; limite
  assumée et visible, pas une rétention infinie cachée.
- Ancien catalogue MCP : l'alerte de transport ne prouve pas l'accès à l'outil.
- Absence de garantie de cache, compréhension ou exactement-une-lecture.
- Aucun test Rust, benchmark, recette fournisseur, audit de code ou Converge
  exécuté pendant cette préparation. Ils sont volontairement hors commande.

Prochaine tâche après autorisation de coder : **T001**. Pas de blocage de conception
restant ; ne pas confondre ce constat avec l'intégration101 déjà réalisée.

## Bilan d'exécution de la préparation

| Phase | Issue réelle |
|---|---|
| Sync | script utilisateur exécuté avec succès dans le worktree102 |
| Specify / Plan | réalisés par protocoles des skills ; scripts/modèles projet absents |
| Audit Existing | PASS,21 items examinés, zéro duplication évidente non arbitrée |
| Contre-revue plan | bdget, fournisseur déclaré Claude, APPROVE_WITH_CHANGES ; traitement tracé |
| Tasks | 32 tâches générées, toutes non cochées |
| Analyze | première passe, corrections ciblées, seconde passe après les dernières corrections ; contrôles statiques verts |
| Implement | exclu explicitement par l'utilisateur ; zéro tâche tentée |
| Converge | exclu, zéro passage, aucune implémentation à confronter |
| Audit/revue de code | exclus ; la contre-revue ne portait que sur le plan |
| Git / production | aucun commit, merge, déploiement ou redémarrage |

ETA initiale annoncée à20:41 :27–45min, milieu36min, fin~21:18CEST.
Après Tasks, à21:06:56 :10–15min restantes, fin~21:19CEST.
Préparation validée vers21:19 :environ37min (écart au milieu initial~+3 %,
dans la fourchette). Dernières précisions : canon099 et réaffectation de remises,
trouvés par lecture réelle, sans changement du périmètre ni tâche supplémentaire.

Contrôle final :12 artefacts obligatoires présents,27/27 exigences couvertes,
32 tâches séquentielles,36 scénarios,8 exemplesJSON valides, zéro erreur du
validateur documentaire éphémère. git diff --check sans erreur ; arbre principal
toujours1738a072 avec les mêmes quatre modifications préexistantes. Le diff102
contient seulement AGENTS.md, .specify/feature.json, ADR038 et dossier de spec.
Les fichiers sont non commitées : transmettre le chemin du worktree, pas seulement
le nom de branche, au prochain agent.

## Converge — passage 1 (2026-09-17, après implémentation)

Confrontation du code réel et des tests aux exigences, preuves `fichier:ligne`
(racine crates/bridget-daemon ; `T` = src/threads.rs, `S` = src/store/threads.rs,
`D` = src/daemon.rs, `I` = tests/spec102_threads_test.rs).

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

Manques rouverts au passage 1 : **1 tâche ajoutée** (T033 : plafonds d'octets et
d'entrées avec limites réduites ; motif `rate_limited` et reprise au tick). Elle est
implémentée et cochée dans ce même cycle (tests `spec102_v31_plafonds…` et
`spec102_v10_debit_borne…`). Passage 2 : `tasks.md` inchangé octet pour octet →
**CONVERGED**.

Limites assumées et visibles : V30/V36 sont des contrôles documentaires (recette
agent), pas des tests automatisés ; la remise réelle aux wrappers Codex/Claude avec
faux fournisseur n'est pas rejouée (chemin commun `connect_and_register_at` +
tracker 099 existants) ; les plafonds de 16 Mio/128 Mio sont prouvés par la même
transaction avec limites réduites, pas à l'échelle réelle ; la mesure SC-005 est en
build debug sur le poste de recette.

Écart de contrat documenté : capacité d'alerte par fait `ThreadNoticeCapability`
(voir contracts/thread-api.md, reuse-audit.md). Minimalisme : les méthodes de
comptage non utilisées ont été retirées de S ; aucune option de configuration ;
un seul module métier et un sous-module SQL ; pas de scheduler séparé (tick existant).
