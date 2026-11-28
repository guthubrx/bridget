# Spec 318 : Observabilité globale des coûts par version et améliorations Console UI

## Objectif
1. Étendre le suivi comparatif par version de Bridget (`bridget_build_id`) à toutes les missions natives (graphes et délégations unitaires), journaliser automatiquement l'historique des builds déployés (`bridget_build_history`), ajouter un estimateur indicatif de coût équivalent API (`estimated_api_equiv_usd`) et une commande CLI `bridget metrics [--window JOURS] [--compare BUILD_A..BUILD_B] [--json]`.
2. Remettre les logos vectoriels des moteurs (Antigravity, Codex/OpenAI, Claude, Cursor, Gemini, GLM) dans la fenêtre Quotas (filtres, en-têtes de section, tableau Machines) ainsi que dans Comptes et Machines & CLI.
3. Détecter automatiquement dans l'onglet Comptes les CLI officiels installés sur chaque machine (y compris ceux absents de `agents.json`, ex. `cursor-agent` et `gemini` sur `sysle2`) afin qu'ils apparaissent immédiatement en état « À connecter » ou « Connecté » avec connexion interactive opérationnelle.
4. Dans « Machines & CLI », interroger (avec cache borné 15 min) la dernière version publiée en amont pour distinguer clairement `À jour` vs `MàJ disponible` (`vX -> vY`) vs `source amont non vérifiable`, et proposer un bouton « Tout mettre à jour » (global et par machine).
