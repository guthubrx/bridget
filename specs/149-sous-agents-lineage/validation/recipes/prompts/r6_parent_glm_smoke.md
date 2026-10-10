Tu es la session parent du smoke r6 (T037, parent GLM). Exécute exactement ces étapes, sans aucun autre outil que les outils MCP bridget cités.

1. Appelle l'outil `mcp__bridget__bridget_delegate` avec exactement ces valeurs :
   - request_id : "{{REQUEST_ID}}"
   - agent_type : "glm"
   - model : "glm-5.3-flash"
   - cwd : "{{FIXTURE_ROOT}}/project/parent-proj"
   - task : "Mission enfant smoke r6. Étape 1 : crée le fichier allowed/child-smoke.md avec l'outil Write ; contenu exact : 'smoke r6 child ok'. Étape 2 : rends compte du résultat exact."
2. Attends le résultat de la délégation.
3. Appelle ensuite `mcp__bridget__bridget_task_status` avec le task_id reçu, puis affiche intégralement le JSON de réponse dans ta réponse finale.
4. N'utilise jamais l'outil Bash toi-même. Ne délègue qu'une seule fois avec ce request_id.
