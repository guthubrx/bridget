# Recette 090 — Codex interactif sans tmux

Après build validé et adoption explicite, depuis un terminal ordinaire :

```sh
bridget codex
# Choix explicite, sinon réglages natifs de Codex :
bridget codex -m gpt-5.6-terra --no-alt-screen
```

Le bandeau Bridget et `bridget who` donnent l'UUID de l'agent. Depuis un autre
agent : `bridget send --to UUID --reply "une demande"` ; répondre par outil
Bridget avec `in_reply_to` égal à l'id du message. `bridget ledger` et
`bridget attach UUID` lisent les mêmes faits que pour une session gérée.
Une réponse finale à l'écran n'est PAS envoyée automatiquement à l'autre agent.

Codex 0.153.4 est la version réellement vérifiée. Le serveur et le fil sont
privés à ce lancement. Le répertoire est celui du shell (`cd` avant lancement).
La TUI conserve l'autorité des permissions ; aucun réglage global n'est écrit.
Cette version lie UNE conversation Bridget, sans limiter les sous-agents internes
de Codex. Leur création/reprise et les notifications d'autres fils ne ferment pas
la session. Leur gestion reste native, sans inscription dans l'annuaire Bridget.
L'adresse Bridget cible toujours le fil initial : elle ne suit pas implicitement
une navigation `/new` ou `/resume` de la TUI. Relancer `bridget codex` pour rendre
une autre conversation joignable. Les autres agents Bridget restent indépendants.

Cette commande est interactive : quitter termine la session. Pour travailler
après fermeture du terminal, conserver `bridget spawn ... --persistent` et attach.

Sonde sans authentification ni production :

```sh
python3 specs/090-codex-interactif/probe_shared_session.py
python3 specs/090-codex-interactif/probe_shared_session.py --approval
```

La sonde utilise le vrai CLI installé et une réponse synthétique locale, pas le
compte fournisseur. Elle ne vaut pas la recette du raccord Bridget complet.

Recettes complètes opt-in (vrais daemon, app-server, TUI et outils MCP, HOME isolé) :

```sh
BRIDGET_CODEX_090_BIN=/opt/homebrew/bin/codex cargo test -p bridget-daemon --test codex_interactive_090_test -- --include-ignored --test-threads=1 --nocapture
```

Le fournisseur HTTP local ne remplace que les réponses modèle. Chaque processus
de test a un budget global de 80 s et un nettoyage borné ; il ne touche ni le
daemon installé ni les sessions personnelles. La variante `--subscription` du
harnais Python exige `BRIDGET_CODEX_090_AUTH` désignant explicitement un compte
ChatGPT ; copie privée temporaire effacée en sortie, aucun fallback API.
