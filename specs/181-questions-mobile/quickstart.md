# Recette181

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/quickstart.md#extrait-1).


## Terminal et indépendance

L’agent pose une question avec l’outil MCP `bridget_ask_user`, son UUID stable et ses questions/options. `create` attend par défaut ; `wait_timeout=0` publie sans attendre. `wait` reprend le même UUID. La réponse structurée reste `selected` + `text`. Le reçu MCP est confirmé après écriture réussie au producteur.

Le commandant humain peut utiliser `bridget questions list`, `show UUID`, puis `answer UUID --revision SHA256 --command-id UUID --answers-file /chemin/absolu.json` depuis un vrai terminal. Le même commandId reprend une réponse incertaine ; il ne faut pas inventer un nouvel identifiant pour réessayer.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/quickstart.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/quickstart.md#extrait-3).
