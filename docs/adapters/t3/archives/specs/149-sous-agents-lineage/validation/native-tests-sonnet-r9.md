# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-tests-sonnet-r9.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-tests-sonnet-r9.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Recette `--new-thread` (et toute recette qui attend la TUI) : TIMEOUT 80 s.** Même timeout avec l'ancienne fixture et le binaire r8 (`native-r9-alias-recipe-control-r8-binary.log`). Cause : la recette attend le mot « fixture » dans la TUI et date de Codex 0.153.4 ; Codex 0.161.0 s'affiche autrement. Sans lien avec l'alias, dont le contrat est validé avant ce point. À traiter hors de cette ronde.
- Échecs transitoires sous charge, rejoués isolés en PASS : `t3code::tests::spec105_question_suivie_relayee_une_fois_puis_arret` (EAGAIN WouldBlock), `wrapper::reconnect_tests::lignes_illisibles_et_trop_grandes_signalent_puis_laissent_progresser` (borne de test), `wrapper::reconnect_tests::spec_024_enregistrement_tmux_ecrit_protocole_et_canal_separes` (moins de 3 s).
- `CodexInteractiveEndpoint` du wrapper est privé : couvert seulement par les recettes réelles. Redémarrage du parent vivant avec TUI : non prouvé ici, revient à la tâche réseau séparée.
- Chemin `native_execution_changed` (CAS refusé en cours d'appel) : écrivain concurrent requis ; contrat du store testé en r8.
- Ordre de `run()` (état durable lu avant réconciliation, grâce de 8 s) : relu dans la source, non exécuté.
- Avertissement de compilation préexistant : variable `message_id` inutilisée dans `crates/bridget-transport/src/managed_session.rs:357`.
