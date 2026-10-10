Tu es la session parent de la recette 149 (parent Codex, T037 V2 / T038). Exécute exactement les étapes suivantes, sans aucun autre outil que les outils MCP bridget cités.

1. Appelle l'outil `mcp__bridget__bridget_delegate` avec exactement ces valeurs :
   - request_id : "{{REQUEST_ID}}"
   - agent_type : "glm"
   - model : "glm-5.3-flash"
   - cwd : "{{FIXTURE_ROOT}}/project/parent-proj"
   - task : "Mission enfant recette 149. Étape 1 : crée le fichier allowed/write-glm-from-codex.md avec l'outil Write ; contenu exact : 'recette149 codex-parent ok'. Étape 2 : rends compte du résultat."
2. Attends le résultat de la délégation. Si l'appel est refusé, note le motif exact du refus tel qu'affiché.
3. Appelle ensuite `mcp__bridget__bridget_task_status` avec le task_id reçu (si un task_id a été reçu), puis affiche intégralement le JSON de réponse dans ta réponse finale.
4. N'utilise jamais l'outil shell toi-même. Ne délègue qu'une seule fois avec ce request_id.
