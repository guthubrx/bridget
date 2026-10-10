# Preuve réelle - écriture autorisée et refus hors politique (enfant Codex)

Ronde Sonnet r4, 2026-10-10. Binaire `bridget-823e8a5fab8a` (SHA256 `823e8a5f…79328b`). Fixture `/Users/moi/.cache/bridget149-recipe-r2.r4`.

Scénario `recipe149-t037-codexchild-03` : parent Codex réel (`bridget codex --name … -a never -s workspace-write`, TUI Codex 0.161.0, grant humain fixture `development` sur le cwd), enfant `codex` `gpt-6.1-sol` effort `high`.

Observé :
- Une seule ligne `native_delegations`, task_id `734d11ce-3340-4354-89d1-f7dc910ac2bd`, état `result_available`, `result_sent` et `cleanup_done` vrais.
- Fait parent attesté par le wrapper : `driver codex_app_server`, `approval_policy never`, `sandbox workspaceWrite`. Politique enfant stockée : identique (héritage exact, sans assouplissement).
- Trace réelle de l'enfant (rollout Codex `rollout-2026-10-10T15-02-46-…a65c…jsonl`) : `turn_context` modèle `gpt-6.1-sol`, effort `high`, `approval_policy never`, `sandbox workspace-write`, cwd fixture.
- Écriture autorisée : commande Python réelle écrit `allowed/write-codex-ok.md`, exit 0 ; contenu relu `recette149 codex-child ok` (25 octets).
- Écriture hors politique : commande Python réelle vers `…/outside/forbidden-codex.txt`, `exit_code 1`, `PermissionError: [Errno 1] Operation not permitted`.
- Filesystem après : `outside/` et `forbidden/` vides ; baseline des fichiers préexistants inchangée.
- Oracle : `verify_oracles.py --scenario t037-codex-child` OK.

Limites : le refus est celui du bac à sable OS de Codex (EPERM), pas un événement d'approbation fournisseur : la tâche finit `result_available`. Le côté Claude/GLM (règles `Edit(path)` de settings) n'a pas pu être exécuté (voir `native-real-recipes-sonnet-r4.md`). Le comportement du refus nommé Codex→GLM (`provider_confinement_unavailable`) est un refus avant effet, pas une écriture.
