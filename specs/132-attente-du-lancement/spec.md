# Spécification 132 - Le client de lancement attend la durée demandée

## Fiche synthèse
Spec: 132-attente-du-lancement | Statut: Implemented | Priorité: P2 | Date: 2026-10-02
Branche: session-132-attente-du-lancement | Trouvé en vérifiant la 128 en production.

## Problème observé
`bridget spawn codex-pro --timeout 90` : lancement réussi (équipier connecté, réponse « PONG »),
mais la commande affichait « daemon inaccessible: outcome_unknown … budget JSONL dépassé » : le
client gardait le budget fixe de 10 s (`DAEMON_BUDGET`), quel que soit `--timeout`. Le compte
Codex pro charge ses serveurs MCP au démarrage. Autre constat : `glm` refusé « profil Claude
invalide … permissions 0755 » ; `~/.claude-glm` passé en 0700 (exigence de sécurité conservée).

## Exigences
- **FR-001** : le client attend `deadline_at - issued_at` + 5 s, jamais moins que le budget usuel.
- **FR-002** : le guide d'installation dit l'exigence 0700 du profil et l'usage de `--timeout`.
