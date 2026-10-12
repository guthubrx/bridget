# Porte160 T001 — primitives présentes et baseline RC0

Les primitives prescrites dans le contrat sont présentes sur la base courante. Aucun manque de primitive livré n’est identifié. Cette porte ne déclare ni l’implémentation160 ni un succès workspace global.

Baseline exacte : **276 PASS, 0 FAIL, 2 ignored**, RC0,208,39s. 84cibles Cargo,5avec tests sélectionnés,0résultat enfant. Les223sources sont identiques pendant le run. Aucun rerun.

Baseline : /Users/moi/.cache/bridget-tests160-preparation-20261012/baseline160.json
SHA85b9e9490bc7617bc4be2f141f1f71b8b834c55482d5f532b464075b0891f6ea

Baseline lisible : /Users/moi/.cache/bridget-tests160-preparation-20261012/baseline160.md
SHA7899ac265cbd498536f1e4b072b7b2344bef1f8957e48529583127386c4645e8

| Primitive | Source actuelle | Ligne | Vérification |
|---|---|---:|---|
| latest_in_chain | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/execution_store.rs | 1024 | Feuille générationmax ; employée par rejected/restart/tick152. |
| NativeMissionDispatch | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/referent_control.rs | 117 | Effetautonome livré ; autorité pause au dispatchmission ; aucune nouvelle variante requise. |
| rejected() pause + sortie DeliveryRejected | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 2839 | Booltrue conserve mission si tourannulé etpausecouvre ; false après savefailed. Pas encore enum160. |
| Gestionnaire COMPLET DeliveryRejected | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon.rs | 22374 | Ok(true) coupe ; Ok(false) continue get_request/pending_retain et message direct. Constatsbase à préserverpour nonoptin, défaut160connu. |
| pause_overlap | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/referent_control.rs | 354 | Somme journal durablecontrol_events ; unionquota futureT012 non encorelivrée. |
| effective_deadline | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 220 | Échéanceeffective152 avecgelpause ; frozen_overlap160 futur. |
| frozen_secs du suivi | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 234 | Fonctiongel suivi et restaurationdéjàlivrée ; pas prétendre unionquota160. |
| restart_resumes | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/delegation.rs | 247 | Compteurrestart152 durable Task ; quota rounds futures distinctes. |
| prepare_restart | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 3130 | Préparation152 reconstruct/compteuravantretour possible ; ciblefutureT021, pas manque primitive. |
| native_child_env | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 2502 | Environnement/identité enfantépinglés avantspawn. |
| current_command | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs | 270 | Lecturecommandeexistante de la flotte dans restart_identity ; pas API neuveinventée. |
| provider_limits / Quota | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/provider_limits.rs | 60 | Quota agrégée157 etfenêtres attestees ; shared provider mémoire. |
| Seen.observed_at | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/provider_limits.rs | 10 | Observation par fenêtre existe déjà privée ; attested_exhausted_since160 futurT013. |

Lectures composites utilisées avec attribution conservée :

- /Users/moi/.cache/bridget-design160-native-20261012/brief.md et /Users/moi/.cache/bridget-design160-native-20261012/read-manifest.json — audit_existant151.
- /Users/moi/.cache/bridget-forensic160-20261012/forensic160-readonly.json — prelecture154 ; le refus de quota réel reste «non vu en réel».
- /Users/moi/.cache/bridget-plan160-preparation-20261012/readiness160-after159B-candidate.json — préparation historique, pas une preuve Analyze/GO.

Toutes les sources de ces lectures composites existent et correspondent aux SHA déclarés. Les plages et dates originales sont conservées dans le JSON. Ce préparateur a relu les artefacts et vérifié les ancres et hashes actuels ; il ne s’attribue pas les lectures des autres agents.

Le gestionnaire DeliveryRejected continue après Ok(false), retire pending et envoie le message direct : c’est le comportement livré servant de témoin rouge160. La reprise ordinaire ignore failed ; une réservation gouvernée ne constitue pas sa reconstruction. Les interfaces nouvelles160 ne sont pas exigées dans cette porte de primitives existantes.

Limites : aucune observation de quota réel corrélée ; capture T036 avant suppression des bindings terminaux nécessaire ; traitement Past160 à coordonner avec le filtre périmé157 ; les positions a–h restent toutes prévues. Pas de FULL ni RED déclarés à partir de ces lectures. Aucun cochage.

Carte et preuves complètes : /Users/moi/.cache/bridget-tests160-preparation-20261012/porte160.json
