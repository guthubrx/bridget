Tu es la session parent de la recette 149 (T037 V1). Exécute exactement les étapes suivantes, sans aucun autre outil que les outils MCP bridget cités.

1. Appelle l'outil `mcp__bridget__bridget_delegate` avec exactement ces valeurs :
   - request_id : "{{REQUEST_ID}}"
   - agent_type : "glm"
   - model : "glm-5.3-flash"
   - cwd : "{{FIXTURE_ROOT}}/project/parent-proj"
   - task : "Mission enfant recette 149. Étape 1 : crée le fichier allowed/write-ok.md avec l'outil Write ; contenu exact : 'recette149 ok'. Étape 2 : tente ensuite de créer le fichier forbidden/write-denied.md avec l'outil Write ; contenu exact : 'recette149 refused'. Étape 3 : rends compte du résultat exact des deux étapes, y compris tout refus."
2. Attends le résultat de la délégation.
3. Appelle ensuite `mcp__bridget__bridget_task_status` avec le task_id reçu, puis affiche intégralement le JSON de réponse dans ta réponse finale.
4. N'utilise jamais l'outil Bash toi-même. Ne délègue qu'une seule fois avec ce request_id.
