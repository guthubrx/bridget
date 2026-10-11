# Implémentation153 — T026

État : Implemented,29/29. Converge, audit et livraison restent des phases séparées.
Base : ee03ebdcfbba85028f0ba3962dde439919b48d29 (152 livrée).

Task, enfant, contexte, registre/droits, magasins, arrêt152, entretien150 et
attente151 sont réutilisés. Sept champs et missions passées portent garde/rejeu/ACK.
Aucun module Rust, dépendance, moteur ou outil de diagnostic neuf. Une seule
fixture cfg(test), quatre usages prévus ; schéma handoff partagé sans helper neuf.

## Relecture et couverture

Revues spécialisées T026 :38/38 hunks de production des quatre sources critiques,
24 tests N, neuf tests de magasin, deux cas de politique (12 combinaisons et
trois refus), quatre réponses tardives et adaptateurs. Le correctif final N
possède sa self-review et les contrôles ciblés ci-dessous.
Le propriétaire documentaire contrôle raccordements et reçus ; il ne revendique
pas une nouvelle lecture personnelle intégrale des hunks. La confrontation
31FR/17SC/5US complète appartient au passage Converge séparé.

| Source | Partie vérifiée |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs | Présence null/omission, Continue, octets150 conservés |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs | Sept champs, migration gardée, index/quotas, holds_slot test seulement |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs | CAS/horloge/commit, libération monotone, ACK/purge atomique |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage_tests.rs | Neuf cas SQL réels, rollback et signaux |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs | Admission/brief/garde/Continue/vues/quatre ID/ACK ;24 tests N relus |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs | Quatre vrais handlers T028, une fixture cfg(test)/quatre usages |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs | 12 combinaisons complètes et trois refus sources/droits/attestation |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_lineage_tests.rs | Constructeurs adaptés mécaniquement, projection149 intacte |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_mcp.rs | Outil fermé, schéma handoff partagé, aucun alias |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/mcp.rs | Catalogue28, schéma partagé, matrice15 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/wrapper.rs | Liste sûre24, approbations existantes |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer.rs | PreToolUse/fait courant/ACK réutilisés |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer149_tests.rs | Dix tests existants, trois adaptés, contrôles conservés |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/tests/native_delegation_e2e.rs | Trois rondes/instance/PID,16 enfants/17e refus, fin explicite |

## Invariants et corrections

FR-019 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:277 pur ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:467 compare
WaitEpoch sous le même verrou et appelle tick seulement après nouvelle admission
committée. Deux cas existants traversent le vrai handler et protègent
owner_instance/payload/epoch sur refus/rejeu. Deux RED réels R2, puis GREEN40+1.
Constructeur round1 ; legacy0 normalisé1. Matrice MCP27→28 = oracle seul.
holds_slot cfg(test) =17 octets, inverse byte-exact, neuf assertions/quatre tests.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:323 : transaction Immediate unique, CAS et heure relue avant
écriture ; I5 complet, compteur de reprise et identités de vie entière conservés,
signaux après commit uniquement. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1449 : ancienne mission
authentifiée purement, sans recover_child/save/dispatch. Quatre consommateurs
d’ID courant et ACK anciens communs ; unacked vide exigé hors OR cancelled/
delivery_acked ; historique supprimé dans la même transaction que l’arbre.
Politiques comparées complètes, posture demandée, sources et grant legacy revalidés.

## Preuves finales et limites

GREEN :40 lib +1 E2E PASS,0 FAIL/ignoré,RC0,69,14s, sources stables.
E2E57,56s : même processus/instance, trois missions/résultats et16/17 limite.
T021 :179 lib PASS,179/179 baseline identiques,RC0,65,33s, binaire/source stables.
16 intégrations (15 PASS/1 ignoré) confirmées nom+cible dans workspace V2 puis
analyse d’impact bornée :194 PASS/1 ignoré au total, aucune référence retirée.

Workspace R1 :94 cibles,2145 PASS/10 FAIL/72 ignorés,RC101,550,82s.
Raw2148 PASS contient trois sous-tests ; V2 utilise le dernier résultat par cible.
Huit échecs préexistants gardent causes et sources. Les deux nouveaux et FR-019
sont résolus par contrôles affectés. Comparaison200 sources/lock :197 identiques,
seuls N/store/MCP changent. Aucun workspace sur hash final ou GREEN global prétendu.

Mémoire réelle/Claude CLI/UI T3 non vérifiées ; T024 consigne le non-lancement
volontaire autorisé. E2E = daemon réel et fournisseur privé synthétique.
Couverture/mutation/benchmark/délai mural60s de production non mesurés.
EXPLAIN, horloges de test et cadence tick ne constituent pas un banc réel.

## Six réponses Articles XIX/XX

1. Nécessité : tâches prévues et défauts RED ; corrections FR-019/round initial
   causales, sans scénario neuf.
2. Simplicité : stores/CAS/epoch/registre/droits/handoff/arrêt152/entretien150/
   attente151 réutilisés ; unique fixture test justifiée par quatre usages.
3. Hypothèses : identité/source attestées, politique complète, enfant live et
   garde valable ; heure relue au commit, aucun droit ou ACK supposé.
4. Vérifié : revues de production/tests/adaptateurs, erreurs/couplage/SQL vrais,
  41 PASS finaux,179 références lib et16 intégrations par preuve/impact.
5. Non vérifié : modèle/mémoire/UI/production/performance/couverture/mutation,
   workspace sur hash final. Huit échecs historiques explicitement conservés.
6. Conservé/supprimé : champs I5 et de vie entière, gardes, oracles stricts, preuves
   anciennes. holds_slot retiré de production par cfg(test) seul. Aucune suppression
   supplémentaire prouvée ; aucune extraction de confort.

## Reçus et capture

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t015-fr019-handler-red-et-mcp153-r2.json
SHA256 : 2ca78050795810b9e9f1233106afb20087b173e05e26988820599788c3ae0179

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t015-t019-native153-corrections-green-r1.json
SHA256 : 45d6ab537328caaf8af27d63cbe2c5c2734d5ac6245116566590d175cc4976f9

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/t021-final-lib-regressions-r1.json
SHA256 : 39b6f84fcade50294b7fd471558a4e81f887f3c94284cdc7ffab4d4c84b8d4be

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/workspace-153-r1-analysis-v2.json
SHA256 : f8d8485ac0e7ae80a5a643c6f0a55c29e44e7fdb146192c9d47f8d27545116b9

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/workspace-153-final-impact.json
SHA256 : 4de6a52049f59346cab3d5c7e31aa597c78050d6284af5bd2fbecd55028c1444

Capture T026 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/diff-153.patch
JSON/logs bruts, audits et patches exclus de cette capture.

## Converge — passage1 acquis

Verdict indépendant CONVERGED :31 FR/17 SC/5 US confrontés, zéro tâche manquante
et zéro ajout. Tâches29/29 byte-identiques, SHA256 a406fc926f3d0ddaa6896d2fdf72ced204aa14d50fcdd0e6b7a0d76090d7f5e2.
Reçu byte-exact : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/converge153-pass1.json
SHA256 : 6564ff059bc498cf8be99d5aff872ee990794c0999672ec28930fb92d31d5dd3.
La matrice et les six réponses XX restent dans ce reçu ; elles ne sont pas
dupliquées ici. Les limites du reçu restent applicables : horloges/cadence sans
mesure murale, fournisseur synthétique sans mémoire de vrai modèle, preuves
workspace/ciblées séparées et aucun GREEN global. Contre-revue code R1 en attente ;
audit et livraison non déclarés exécutés.
