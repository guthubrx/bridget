# Réemploi157 après livraison155/156

Les objections M1, M2 et M3 restent valables pour le raccord futur157. Elles ne constituent pas un défaut confirmé des versions155/156. Dix sources ont été relues par plages. Elles sont identiques au commit demandé cf498755dc871b51eb8c0add5f8a882d67651d2a et au commit de livraison8b5b313694cf05eabfe6a86a970b96f03b015f49. Elles sont restées stables pendant cette lecture.

Aucun test, Cargo, build, fournisseur, service, Git mutateur, code ou document du dépôt modifié. Aucun Analyze157 frais ni code157 validé. Le gate11/11 ancien du reuse-audit reste historique. ROOT confirme avoir appliqué le patch candidat au worktree157 après livraison156 ; ses documents actualisés ne sont pas relus ici.

## M1 : conserver le candidat dans les trois passes

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:655,668,681 transmet la requête originale à Probe, Reserve et Commit. À817–826, Probe ne porte que le cwd. Commit recalcule la politique dans le vrai dossier enfant1069–1075. La sélection unique et la définition épinglée doivent suivre les trois passes, comme le candidatT018 le prévoit déjà. Commit refuse toute invalidation sans nouvelle sélection. Les structures WorkspaceAdmission et Task sont réutilisables. Aucun nouveau scénario autonome n’est nécessaire.

## M2 : préserver les gardes155 et154

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:948–956 impose Codex pour l’enfant en mode forward. À959–970, le parent doit aussi être Codex, attesté, avec reviewer user. Le refus de ce parent reste hors fallback pour tous les candidats. Un refus de protocole propre à un candidat listé peut figurer dans tried. À1038–1044, le worktree exige Development et ferme Claude vers Claude. Commit conserve les contrôles de chemin, limite et politique1061–1075. Les options child_requests, workspace, brief, keep_alive et les délais restent immuables. Le booléen de catalogue1312 ne prouve que le protocole enfant, pas les droits du parent.

## M3 : préserver le détail sans nouveau porteur

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:74–82 ne décode que les codes nus et les préfixes brief/workspace. /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation_mcp.rs:259–263 ne conserve que le code du résultat refusé. Le choix code:JSON dans Err(String) reste minimal : étendre le seul failure, préserver les anciennes formes et comparer les listes sur le code canonique. /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:687–689 requalifie déjà les erreurs de Commit en workspace_create_failed ou workspace_path_unsafe, avec chemin et branche. Ce diagnostic154 doit survivre au raccord. /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation.rs:644–652 exclut déjà quota et fallback de sa liste durable.

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/native_permissions.rs:478/504 distingue l’absence de launch_context au point de contrôle. Les erreurs de CLI, configuration ou environnement peuvent donner le même permission_source_unavailable via319–351. La relecture d’un contexte existant414–429 produit settings_revision_changed : conserver ce code. Aucun incident réel n’a été observé ; les messages doivent venir du point de diagnostic, sans recalcul de droits.

## Gate de réemploi ciblé

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/lifecycle.rs:209–227 et341–405 contient les contrôles à extraire. build_environment reste ignoré par if let Ok à220 avant réservation, puis propagé à387. launch_preflight n’existe pas encore. Réemployer ces contrôles dansT005, sans sonde de fournisseur ou d’authentification. registry.rs314–343 conserve modèle et effort ; l’ordre157 avant motifs remplaçables reste à implémenter.

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:861 et7080–7135 utilise Presence.rate_limits par fenêtre. Le fait ne conserve ni source ni observed_at ; il disparaît avec la présence. /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:3735–3752 et5070–5077 offre les trois sources fermées et RateLimitFact réutilisables. Aucun provider_limits trouvé dans les sources daemon/transport ni dans lib.rs. La mémoire volatile prévue se justifie ; ses quatre usages restent planifiés. service_limits gère RLIMIT_NOFILE, pas un quota de fournisseur.

## Références à corriger par ROOT

- Remplacer les affirmations courantes «155/156 futures» par la base livrée8b5b3136 ; conserver c24f6e3a et les anciennes traces comme historiques. Le vrai dossier est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas.
- Corriger les chemins malformés native_/Users… dans /Users/moi/.cache/bridget-plan157-20261012-r1/reponse-review.md:7,11,15 lors de leur import. Le bon fichier est /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs.
- Relocaliser child_policy à456–526, record_refusal à636–663, le pontMCP à259–273, RateLimitFact à5070–5077 et handle_rate_limit à7080–7135. Les chemins complets et SHA figurent ci-dessous.
- Dans le futur reuse-audit157, qualifier le PASS11/11 ancien ; remplacer la mention failure_with_detail par le failure/code:JSON unique déjà décidé.
- Préciser que child_requests_reviewer_unsupported est un refus du parent, jamais contourné en parcourant les candidats.
- Ne pas interpréter cwd_root=null du catalogue1319 comme une admission sans cwd. authorize_cwd425–454 ou les contrôles972–996 restent requis.

## Plages et SHA relus

/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs : 60–106, 153–180, 284–331, 425–459, 522–590, 590–646, 646–718, 817–859, 859–1088, 1080–1143, 1140–1180, 1180–1249, 1250–1325 ; SHA256 51905c9443f3a47ede4799b176827039192ab20c583f6cf2b2fb9d1e40ccf99b
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/lifecycle.rs : 145–259, 330–484, 530–623 ; SHA256 a6a2c2383689a29b398c1833eb5d3bfdd5f32bd48ee16500157fe5efd0f0f985
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation.rs : 636–677 ; SHA256 01336553b359e82c6231d8ff8aa18575a9f818bd4043f15a040b89384012660a
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation_mcp.rs : 250–277 ; SHA256 4bd59788f4742ab3737d9092bbb996660e198ab36309a8c4ba7b1fb1c4164192
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs : 840–870, 7080–7160, 19230–19250, 19555–19610 ; SHA256 a11310b737d1ac186309a57f306babe3146ec8af7f57a921bc6adf6b398c3c3c
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/native_permissions.rs : 254–272, 319–351, 409–440, 456–545 ; SHA256 96b6c021d1f7ead6d7f6ba698f17975d09a8ecdb6ca3ad6bbba70d7908171cd3
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs : 2095–2155, 3710–3755, 5065–5087 ; SHA256 b360cf27dd54fbce93e08de4075e2925e4524ae111f16b39edbc7a352b838bc1
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs : 314–374 ; SHA256 ab898f159b54b3001fd95904e23570aae472d8e867c46fe9f1fc4124c9520d91
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/lib.rs : 1–66 ; SHA256 674cb14f46c88c91102aab6a6e74f9b41916c5ae22438286fb248bf6f5886aad
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/service_limits.rs : 1–49 ; SHA256 d62aee2c357bacf6e4d6bf10b4c75e5ca8b33d9a408833ac3f38c02b9e86f394

Limites : dix sources ciblées, pas une revue complète du dépôt. Aucun runtime fournisseur/UI, incidentR1-7, mesure de latence ou baselineT001157 validé. Les26 tâches et leurs scénarios planifiés restent inchangés. Les archives candidates sont préservées.

/Users/moi/.cache/bridget-plan157-20261012-r1/readiness157-after156.json ; SHA256 9de506e51b9d3b00a7e2b7cf3ab76c35453b29181e091f7e24560e5ea5fb7654
