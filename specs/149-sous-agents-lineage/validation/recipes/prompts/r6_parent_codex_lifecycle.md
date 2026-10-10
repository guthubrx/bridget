Tu es la session parent du smoke r6 (T038, cycle de vie, parent Codex). Exécute exactement ces étapes dans l'ordre.

1. Appelle `mcp__bridget__bridget_delegate` avec exactement ces valeurs :
   - request_id : "{{REQUEST_ID}}"
   - agent_type : "codex"
   - model : "gpt-6.1-sol"
   - effort : "high"
   - cwd : "{{FIXTURE_ROOT}}/project/parent-proj"
   - task : "Mission longue smoke r6. Exécute une seule commande shell : python3 -c \"import time; [(open('{{FIXTURE_ROOT}}/project/parent-proj/allowed/heartbeat.txt','a').write(str(i)+chr(10)), time.sleep(3)) for i in range(1,121)]\". Puis rends compte."
2. Note le task_id reçu.
3. Répète au plus 40 fois : exécute la commande shell `sleep 10`, puis appelle `mcp__bridget__bridget_task_status` avec ce task_id. Arrête dès que l'état est terminal (result_available, failed, cancelled, completed ou expired). Si l'appel échoue, note l'erreur exacte et continue la boucle.
4. N'appelle JAMAIS `mcp__bridget__bridget_task_cancel`. Ne délègue qu'une seule fois avec ce request_id. N'utilise aucune autre commande shell que `sleep 10`.
5. Affiche dans ta réponse finale le dernier JSON de `bridget_task_status` ou la dernière erreur exacte reçue.
