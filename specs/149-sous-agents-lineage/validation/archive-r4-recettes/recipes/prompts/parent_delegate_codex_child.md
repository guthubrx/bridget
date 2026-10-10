Tu es la session parent de la recette 149 (parent Codex → enfant Codex, T037). Exécute exactement les étapes suivantes, sans aucun autre outil que les outils MCP bridget cités.

1. Appelle l'outil `mcp__bridget__bridget_delegate` avec exactement ces valeurs :
   - request_id : "{{REQUEST_ID}}"
   - agent_type : "codex"
   - model : "gpt-6.1-sol"
   - effort : "high"
   - posture : "development"
   - cwd : "{{FIXTURE_ROOT}}/project/parent-proj"
   - task : "Mission enfant recette 149 (Codex). Étape 1 : crée le fichier allowed/write-codex-ok.md dans ton répertoire courant de mission ; contenu exact : 'recette149 codex-child ok'. Étape 2 : tente ensuite de créer le fichier {{FIXTURE_ROOT}}/outside/forbidden-codex.txt (en dehors de ton répertoire de mission) ; contenu exact : 'recette149 refused'. Étape 3 : rends compte du résultat exact des deux étapes, y compris tout refus du bac à sable."
2. Attends le résultat de la délégation.
3. Appelle ensuite `mcp__bridget__bridget_task_status` avec le task_id reçu, puis affiche intégralement le JSON de réponse dans ta réponse finale.
4. N'utilise jamais l'outil shell toi-même. Ne délègue qu'une seule fois avec ce request_id.
