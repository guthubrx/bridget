# Preuve réelle - parents externes sans T3 (ronde r5)

Ronde Sonnet r5, 2026-10-10. Binaire RELEASE `bridget-abfb346e23cc`. Fixture privée `/Users/moi/.cache/bridget149-recipe-r2.r5`. Variables `BRIDGET_T3_*` absentes ; aucun processus T3 dans la fixture ; voie `NativeDelegation` seule.

- Premier parent GLM : `bridget gclaude` (lanceur `dd8dee56…872a8e`, profil `/Users/moi/.claude-glm`, CLI 2.1.296) dans un PTY. A délégué, suivi (`bridget_task_status`) et reçu l'échec corrélé. Sources de permission révisionnées : 17 (user, managed, project, local, plugins), CLI résolu `c9b53416…`.
- Premier parent Codex : `bridget codex … -a never -s <sandbox>` (TUI Codex 0.161.0, app-server propriétaire). Fait publié avant la TUI (mode réel `full-access` ou `workspaceWrite`).
- Lancement unique et résultat corrélé : 1 ligne par `request_id` (4 délégations attendues, 4 lignes ; 1 ligne accidentelle `neg-daemon-r5-01`, voir rapport).
- Annulation (`recipe149-t038-cancel-r5-01`) : enfant Codex déjà au travail (`compte.txt` à 6 lignes quand le parent annule), état `cancelled` en 20 s, fichier inchangé ensuite (7 → 7), aucun processus restant.
- Branches négatives : `permission_source_unavailable` (`--settings` utilisateur explicite), `permission_attestation_unavailable` (daemon arrêté). 0 ligne chacune.
- Daemon arrêté pendant qu'un enfant GLM travaille : l'enfant s'arrête, aucun orphelin ; au redémarrage la tâche passe `failed` + `unreachable`.
- Nettoyage : tous les processus de la fixture arrêtés par SIGTERM individuel ; aucun artefact observer restant ; `native_delegation_grants` = 0.

Détail, limites et incidents de harnais : `native-real-recipes-sonnet-r5.md`.
