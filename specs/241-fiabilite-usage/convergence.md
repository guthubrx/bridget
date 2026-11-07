# Convergence 241 — passage1

Résultat : CONVERGED pour la réalisation du code, aucune tâche ajoutée.
Spec, plan, tâches et checklist relus ; code réel et tests confrontés.
SHA256 tasks.md avant/après :
51a799e022f3f248fd312ea98bfba56b8fe3988a943776b5d9d70fa9a6b88318.
Ce résultat ne coche ni l’audit, ni les assemblages, ni la livraison.
T010/T012/T021–T026 conservent les qualifications opérationnelles restant à faire.

| Exigence | Réalisation et preuve lues |
|---|---|
| FR-001 | communication/client.rs:1234, daemon.rs:7064/44463 : phases fermées et collecteur hors réponse ; vue lente reproduite puis réponse indépendante. Les causes historiques non attribuées restent dans validation.md. |
| FR-002 | communication/client.rs:1369/1430/1471/1493 et mcp.rs test spec241_cancelled_registration : une reprise avant mutation, budget partagé, refus et annulation terminaux. |
| FR-003 | daemon/native_delegation.rs:29960, delegation_mcp.rs:957 : vraies sockets, réponse perdue et deux replays, une Task/un lancement simulé ; reçu Auto existant récupéré sans nouvelle clé. |
| FR-004 | delegation.rs:1160/2832 et daemon/native_delegation.rs:9118 : marqueur produit par admission provider_turn, validation/réouverture sans promotion d’historique ; tests réels d’admission et Release. |
| FR-005 | daemon/native_delegation.rs:7020/7430, daemon/origin_proof.rs:355 et native_start_recovery198_tests.rs:225/1706/1711 : propriétaire, instances/racine révoqués, Task relue après E/S, sources et politique conservées. |
| FR-006 | lifecycle.rs:445, native_permissions.rs:302, remote_worker_read.rs readiness_at, prepare-project.py:511 : racines attestées et origine=cible authentifiée ; tests locaux, étrangers, alias, hors-Git, protections et publication inchangée au refus. |
| FR-007 | limits.md, workspace_git.rs:861, workspace_commit_view.rs validate_manifest/audit_inventory et storage_compatibility.rs identifiers : retrait32Gio/128WT/64mappings/16formats, compteurs et espace réels ; tests33Gio/u64/128checkouts/35formats. |
| FR-008 | delegation.rs:2577, delegation_lineage_tests.rs:629 et scénario sockets4097lignes : lecture par clé, curseurs/pages, état et annulation indépendants du total retenu. |
| FR-009 | daemon/native_delegation.rs:1002/1067/1213, remote_worker_read.rs:1604/1719, protocol.rs:2899/3339/3430 : exigences par commande, capacité négociée, outillage et politique d’écriture ; états fermé/absent/inutilisable/inconnu/disponible exercés. |
| FR-010 | mêmes contrôles de source/profil et project_probe_constraints : pas de seconde autorisation de dossier déjà couvert ; contrainte humaine conservée et décision ciblée seulement si besoin nouveau. Qualification native finale déjà portée par T012/T025. |
| FR-011 | late_worker_prepares240.rs inchangé contre04ad ; cache et branche de refus tardifs réutilisés, huit preuves240. Aucun nouveau rejeu métier. |
| FR-012 | fixtures Unix/SQLite/Git/Python et validation.md : essais directs/récupérés/refusés et limites de mesure conservés. Assemblages et parcours chargés encore dans T021/T025. |
| FR-013 | mécanismes existants build.py/update.py/nativeunits et storage_compatibility.rs : contrats de retour implémentés ; activation, publication, refs et nettoyage restent explicitement T024–T026. Pas d’annonce de livraison avant ces portes. |

Les7critères de réussite sont couverts par les mêmes scénarios. SC-007 demeure
une porte opérationnelle à vérifier après activation, avec exclusion de relance
T3. Cette distinction n’ajoute ni ne supprime une exigence de la spécification.

Auto-relecture XIX/XX : une méthode Task commune, un collecteur de vues, mêmes
stores et writers natifs. Pas de nouveau service/table/dépendance. Le coût du
préflight d’espace est un inventaire borné pour le checkout complet ; compact
réutilise son plan. La complexité historique des gros modules reste une limite.

## Passage2 — après delta R3

CONVERGED, aucune tâche ajoutée. Spec13FR, plan, tâches et checklist relus.
Les réalisations FR-001–008/010–013 n'ont pas changé depuis le passage1.
FR-009 : fonctions workload_command_segments/workload_command_writes et
identified_workload_requirements relues ; tests spec241_chained_commands et
spec241_known_mutations, puis vrai routage lecture/écriture, 24cas241 verts Mac.
Le contrat garde les limites des constructions inconnues ; aucun shell exécuté.
La seule nouvelle dépendance directe shlex2.0.1 était déjà verrouillée via cc ;
recherche et décision de réutilisation PASS avant édition. Ceci corrige la
mention « pas de dépendance » du passage1 : pas de nouveau paquet/version,
mais une arête directe justifiée. Aucun nouveau store/service.
SHA tasks.md avant/après identique :
51a799e022f3f248fd312ea98bfba56b8fe3988a943776b5d9d70fa9a6b88318.
R4 demandée sur ce seul delta ; audit/assemblages/activation restent les portes
T010/T012/T021–026 existantes, jamais transformées en succès par Converge.

## Passage3 — disposition R4

CONVERGED ; aucune tâche ajoutée, même SHA tasks.md. Seuls un commentaire de
contrat dans CandidateProbeV2, un test du programme non analysé et le retrait de
l'arête build-shlex redondante ont changé depuis le passage2. Comportement du
parseur de production inchangé. FR-009 porte sur les besoins identifiés ;
required_tools vide n'atteste pas un outil inconnu, requires_workspace_write=false
ne certifie pas l'absence d'effets. Code, consommateur et contrat relus ensemble.
Les inconnus conservent le sandbox ; les refuser tous ajouterait un périmètre
nouveau. Les portes T010/T012/T021–026 conservent leur sens opérationnel.
Fin des trois passages de convergence, aucun manque de code ajouté ni masqué.
