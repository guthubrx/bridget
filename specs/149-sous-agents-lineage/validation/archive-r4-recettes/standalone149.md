# Preuve réelle - parent externe hors T3 (Codex)

Ronde Sonnet r4, 2026-10-10. Aucun `BRIDGET_T3_MCP_*` dans l'environnement, aucun T3 lancé, voie `NativeDelegation` seule. Daemon et registre fixture privés (`BRIDGET_HOME` fixture), jamais la prod.

- Premier parent réel Codex : `bridget codex --name recipe149-parent-codex-… -a never -s workspace-write` (TUI humaine interactive, app-server propriétaire de Bridget). Le fait de permissions du parent (publié avant le démarrage de la TUI) a été accepté par le daemon : la délégation a franchi la politique (`permission_snapshot.parent`, `source native_wrapper`, `driver codex_app_server`, `runtime_mode auto`).
- `bridget_delegate` : une seule ligne par `request_id`, résultat corrélé (`task_id 734d11ce-…` → `result_available`), `bridget_task_status` lu par le parent.
- Annulation : `recipe149-t038-cancel-01`, `bridget_task_cancel` après `bridget_delegate` → `cancelled`, plus aucun processus enfant. Limite : annulé 2 s après le démarrage, avant la commande longue.
- Refus nommé : Codex `workspace-write` → GLM : `provider_confinement_unavailable`, 0 ligne créée.
- Non couvert : parent GLM/Claude hors T3 (bloqué par la latence de l'observer, voir `native-real-recipes-sonnet-r4.md`).
