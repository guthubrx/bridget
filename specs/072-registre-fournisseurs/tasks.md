# Tâches - SPEC-072 Registre de fournisseurs

## Préparation et sécurité

- [x] T001 Créer les tests de régression du registre pour claude_config_dir, validation de chemin, absence de repli et stabilité du digest dans crates/bridget-daemon/src/registry.rs, crates/bridget-daemon/src/lifecycle.rs et crates/bridget-transport/src/protocol.rs. Preuve : les tests échouent avant le contrat puis passent après.
- [x] T002 Étendre AgentDefinition et ResolvedAgentDefinition avec claude_config_dir, mettre à jour la définition résolue, le digest, les constructeurs et build_environment. Preuve : un profil absolu est transmis, un chemin relatif ou interdit est refusé et aucune valeur secrète n est sérialisée.
- [x] T003 Ajouter les tests de provenance Claude Code dans crates/bridget-transport/src/claude_stream_json.rs et crates/bridget-daemon/src/wrapper.rs. Preuve : claude, anthropic, glm et deepseek produisent chacun leur provider_kind déclaré.
- [x] T004 Transmettre le type d agent à ClaudeStreamJsonTransport et remplacer la provenance claude codée en dur. Preuve : les contextes, bindings et événements gardent le type exact sans régression du type historique claude.

## Profils fournisseur

- [x] T005 Documenter le contrat opérateur non secret et déclarer les types anthropic, glm et deepseek dans le registre privé du serveur, sans modifier claude existant. Preuve : validation JSON, droits 0600, registre chargé et aucune différence secrète dans Git.
- [x] T006 Créer les profils Claude Code privés GLM et DeepSeek à partir des fonctions locales, avec répertoires 0700 et settings.json 0600. Preuve : les profils sont lisibles seulement par moi, leur contenu ne traverse aucun terminal ni journal.
- [x] T007 Activer un agent Cursor géré depuis la définition ACP existante. Preuve : Cursor authentifié répond à une demande sans écriture, son contexte est cursor et son transport acp.
- [x] T008 Prouver GLM et DeepSeek en tours réels sans écriture : texte, lecture d un fichier, résultat, modèle réellement observé, erreur ou refus si applicable. Preuve : journaux et exécutions associés aux fournisseurs glm et deepseek.
- [x] T009 Prouver l isolement Anthropic pendant un tour GLM, puis contrôler interruption, remise et un échec de profil compréhensible. Preuve : Anthropic ne change pas d upstream et les refus ne déclenchent aucun repli silencieux.

## Qualité et clôture

- [x] T010 Exécuter formatage, suites ciblées, suite workspace et build release ; mettre à jour tous les artefacts, le registre de handoff et le statut de SPEC. Preuve : commandes, résultats, secrets absents et procédure opérateur vérifiée.
- [x] T011 Rejouer la contre-revue adverse si un canal de réponse sûr est disponible, puis exécuter converge manuel et audit. Preuve : adversarial-review, convergence et audit consignés ; toute tâche ajoutée est exécutée avant clôture.
