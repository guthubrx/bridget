# Contre-revue de code SPEC168 — Sonnet R1

Date :2026-10-15. Fournisseur Claude, modèle Sonnet5.5 selon la passation ROOT.
Verdict transmis : **APPROVE_WITH_CHANGES**, aucun défaut bloquant établi.
Cette revue de code est distincte des revues de conception consignées dans adversarial-review-claude.md.
Le texte intégral reçu a été lu et son empreinte contrôlée :
/home/moi/.local/state/bridget-delivery168/review/code-r1/review-sonnet-r1.md
SHA-256 :fb92e6459eaf288948b667f84642e0214525bcfafc78976eeb146aa2867ced98.

## Questions et réponses aux objections

La revue examine fraîcheur intermachines, score, motifs de repli, rejeu Git, aide CLI,
immuabilité du choix, mesures inconnues, capacité Laya, isolement et durabilité.
Les statuts ci-dessous distinguent arbitrage documentaire et correction exécutée.

| ID | Constat de la revue | Réponse ROOT et état de preuve |
|---|---|---|
| M1 | Un verdict worker daté dans le futur disparaît sans diagnostic. | Corrigé : fraîcheur du verdict corrélé par réception maître/TTL ; garde source≤réception conservée pour les mesures. Tests score et service : chacun 1 PASS, RC 0. |
| L1 | Poids fixes/troncature divergent du contrat configurable/arrondi. | Corrigé : sept poids bornés de la coordinatrice, arrondis, pénalité 100 fixe et défauts omis au canon pour ancien digest. Registre et score : chacun 1 PASS, RC 0. Contrat/plan/modèle/quickstart alignés. |
| L2 | Le permis occupé force le score sans note durable. | Corrigé : note decision_capacity_occupied durable, permis physique conservé, rejeu sans nouvel essai. Test busy : 1 PASS, RC 0. |
| L3 | Un nouveau HEAD change le canon CLI malgré la même request_id. | Documenté dans spec, contrat et quickstart : envelope_mismatch attendu ; lire bridget task status pour retrouver la tâche. Le rejeu exact exige la même enveloppe/SHA. |
| L4 | L’aide auto omet quatre options acceptées. | Corrigé et relu : --engine, --connection-id, --workload, --expected-base-commit présents dans AUTO_DELEGATE_USAGE. Compilation bibliothèque RC 0 ; aucun test CLI additionnel revendiqué. |
| R1 | Ambiguïté entre choix durable et refus si saturation ultérieure. | Choix immuable, aucun décideur rappelé. Revalidation uniquement de l’élu ; refus opérationnel possible pour droits/capacité/base, sans autre cible. Trois documents précisés. |
| R2 | CPU bruité lors de sondes rapprochées. | Risque théorique non établi, à observer en recette. Aucun nouveau test global demandé. |
| R3 | Alternative disponible exige CPU/RAM/charge frais et connus. | Arbitrage conservé : inconnu ne prouve pas disponible ; pénalité100 sans alternative attestée. |
| R4 | Laya peut dépasser son état1536 dès peu de candidats. | Conforme :1..8 et état1536 ; indisponibilité de capacité explicite, aucune troncature silencieuse ni promesse Laya128. |

## Vérification documentaire ciblée

Lecture directe sur la source Cartae, sans exécution :
- cli.rs : construction delegate et insertion de expected_base_commit depuis la source ;
  lifecycle.rs::source_base_commit résout HEAD^{commit}.
- native_delegation.rs : contrôle du canon avant politique ; refresh_routing_selected réduit
  le plan au seul candidat choisi ; route_selected_admission utilise metadata.decision ;
  recover_routing_admission reprend Chosen sans consulter service ou petit modèle.

Ces lectures étayent L3/R1. Elles ne constituent pas une exécution des correctifs M1/L1/L2/L4.
Les repères de lignes du rapport original décrivent sa source de revue et peuvent changer ensuite.

## Limites et gates

Le reviewer n’a exécuté aucun test, build ou fournisseur. Ses lectures excluent notamment les corps
de nombreux tests/fixtures, daemon/lifecycle/reprise, détail registre/migration, MCP/protocoles et probe worker ;
ces limites restent celles du rapport original. Il s’appuie sur la qualification CLI pour safe-mode/restricted.
Il a retenu les raccords crédits/fraîcheur, inconnus, snapshot/CAS, plafond128, isolement et migration
dans les seules zones effectivement lues. Aucun audit global ni nouvelle validation n’en découle.

État final après reçus ROOT : 43 tâches sur 56 cochées ; T037 clôturée sur les quatre contrôles
affectés et la lecture des corrections. 70 noms uniques à dernière issue PASS, 102 exécutions PASS
brutes et 9 échecs historiques conservés, sur leurs empreintes respectives.
Aucune FULL, recette réelle168, activation ou release déduite. L’audit168 à venir suit la skillV15 ;
aucun grade ou auditV14 n’est revendiqué.

## Clôture T037 autorisée par ROOT

Les reçus t037-sonnet-r1-registry/score/service/busy portent chacun RC 0 et 1 PASS.
Le build daemon bibliothèque --no-run porte RC 0 ; il n’ajoute aucun test.
Le manifeste t037-sonnet-r1-hashes atteste 56 entrées avant/après identiques.
Commandes, empreintes, assertions et limites sont consignées dans :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md.

ROOT a relu fraîcheur, arrondis et permis. Le verdict Sonnet reste APPROVE_WITH_CHANGES :
les objections M1/L1/L2/L4 sont corrigées, L3 et R1 documentés ; R2/R3/R4 gardent leurs limites.
Ce n’est pas un nouveau verdict Sonnet ni une seconde revue intégrale.
Converge P3 reste CONVERGED ; aucune passe P4. T038 audit et T036 FULL restent ouvertes.
GEL final documentaire pour l’audit V15 ; aucune autre écriture après publication sans GO ROOT.
