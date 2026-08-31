# Etat implementation - SPEC-076

**Date**: 2026-08-31
**Statut**: Implementation fonctionnelle complete. Cloture de validation bloquee par quatre taches de preuve, sans deploiement.
**Branche**: session-076-interface-projets-coordinateur.
**Base**: 4ad487e, tete main au demarrage de la tranche.

## Fonctionnalites realisees

- Reglages locaux des racines autorisees : schema ferme, generation optimiste, ecriture atomique, verrou inter-processus et refus fail-closed.
- Creation et import bornes : previsualisation pure, Git explicite, reprise apres panne, intention Maicie typee et absence de shell libre.
- Retrait non destructif, rebind explicite et reactivation typee avec conservation de identite, liaison et audit Bridget.
- Projection UI sans store metier : identite, liaison, dernier audit non sensible, coordinateur et agents lies par ProjectReference.
- Configuration durable du coordinateur : outil, upstream, modele, effort, permissions et digest. Aucun remplacement automatique.
- Decouverte lecture seule : 10, 30, 60 ou 120 minutes, confirmation de poursuite, empreintes non sensibles avant et apres, rapport dans la conversation.
- Barre projets repliable : Toute la flotte, ouverture du coordinateur, creation, import, retrait, reactivation et reconnexion locale.

## Frontieres preservees

- Maicie reste autorite de identite. Bridget reste autorite de liaison et audit. Aucun store UI metier ni acces direct a la base Maicie.
- Aucune route UI pour profil, extension, secret, Docker ou commande libre.
- Aucune racine reelle de production ajoutee. En absence de politique de racines, les mutations projet sont refusees.
- Aucun deploiement ou redemarrage effectue.

## Validation executee

| Commande | Resultat |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo clippy --workspace --all-targets -- -D warnings | PASS |
| cargo test -p bridget-daemon --lib | PASS - 743 passes, 7 ignores |
| cargo test -p bridget-daemon spec_076 --lib | PASS - 20 passes |
| cargo test -p bridget-daemon spec_065 --lib | PASS - 11 passes |
| cargo test -p bridget-daemon spec_075 --lib | PASS - 2 passes |
| cargo test -p maicie project --lib | PASS - 12 passes |
| cargo test -p bridget-transport --test project_runtime_contract_test | PASS - 2 passes |
| node --test crates/bridget-daemon/assets/ui/app.js | PASS - 93 passes |
| cargo test --workspace | ECHEC - managed_parity_test : 6 passes, 4 echecs |

Les quatre echecs reproductibles de managed_parity_test sont : matrice FR-008
corpus et frames, deux assertions de reprise Codex MCP, puis nettoyage de six
groupes managed-wrapper. Les sources de ce test, wrapper.rs, managed_process.rs
et managed_session.rs ne font pas partie du diff SPEC-076. Les echecs avaient
deja ete observes avant les derniers changements 076.

## Taches restant a prouver

- T062 : la commande workspace ne passe pas a cause des quatre echecs de parite ci-dessus.
- T063 : les fixtures automatiques sont couvertes, mais la validation manuelle utilisateur distincte reste a recueillir.
- T066 : aucune contre-revue adverse n a pu etre recueillie depuis ce canal. Le fait est consigne sans le presenter comme une relecture reussie.
- T067 : Converge reste impossible tant que T062, T063 et T066 ne sont pas closes.

Aucun commit, merge, push ou deploiement ne doit etre fait avant une cloture
honnete de ces points ou une decision explicite de les accepter.
