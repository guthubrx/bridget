# Session200 — Champ interdit en admission automatique

Incident : auto_admission_invalid avec agent_type=codex-sysle2 dans bridget_auto_delegate. Aucun reçu ni tâche enregistrés. Le contrat V2 interdit déjà ce champ mais le catalogue MCP l’expose et le refus du maître ne nomme pas le champ.

Correction : retirer ce champ du schéma automatique, le refuser explicitement avant connexion dans le client, et nommer le champ dans le refus du maître sans changer le code public ni autoriser le champ. engine choisit le moteur ; connection_id facultatif contraint une connexion du catalogue. Les contraintes machine, base, permissions et idempotence restent inchangées. Aucune valeur fournie ne doit être répétée.
