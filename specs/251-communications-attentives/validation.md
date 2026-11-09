# Validation251 — In Progress
Départ09novembre08:28CET, ETAinitiale110–200min, aprèsTasks90–160min restants vers08:45CET. Code0.8.17 candidat, aucun service activé à ce stade. T3 reste ouvert.
## Réalisé
Cadrage montré/repris ; recherche primaire T3/Claude/OpenAI/LangGraph ; reuse-audit PASS ; Analyze manuel deux passes (scripts/runtime absents vérifiés). 6critères de qualité des exigences PASS, distincts de la réalisation.
Nouvelle intention StoreOnly, capacité négociée aussi serveur, transaction ledger+reçu, aucune remise/suivi/exécution. Garde structurelle de création et sérialisation des remises, refus des corrélations avant capture, dormant/DND inchangés. Docs/skill et consignes réutilisées.
## Tests Mac
Commande : /Users/moi/.cache/bridget251-attention/run-tests.sh /Users/moi/.cache/bridget251-attention/mac-targeted.log -p bridget-daemon --lib -- spec251 guidance156_cli_intent_valide guidance156_mcp_ send_idempotent_rejoue_sans_rerouter_et_isole_les_scopes send_idempotent_conserve_les_invariants_de_reponse_et_dnd remote162_resultat_canonique_notifie_parent_sans_recapture_legacy spec250_ spec135_ --test-threads=1
Résultat :20tests PASS, zéro ignoré,1,19s après compilation1m41. Six nouveaux tests251 et contrôles existants CLI/MCP/guidage/résultat/observation ; répétitions ultérieures ne seront pas additionnées.
SC001 : dix corps conservés, zéro send_deliveries/tracked_requests/work_submissions/executions/execution_queue, zéro contrôle ; vraie demande témoin avec suivi. Rejeu/mismatch, panne SQL puis réouverture, collision sans écrasement, garde négative de capacité, dépôt dormant/DND et one-shot actif observés dans les tests.
Premières tentatives : erreur de compilation du test (comparaison enum sans PartialEq) corrigée dans l'assertion ; première exécution1PASS/5FAIL (namespace de test trop long, scopes courts et identifiant de reçu simulé divergent), seconde4PASS/2FAIL (operation_id du banc de sommeil invalide). Données d'erreur relues avant correction. Aucun défaut runtime déduit de ces échecs de banc. Dernière exécution20/20 avec namespace privé court et UUID corrects. Traces privées conservées ; aucun échec masqué.
## Revue plan
Claude Haiku SYS task e5382c43, APPROVE_WITH_CHANGES lu/ACK ; dispositions détaillées dans adversarial-review-plan.md. Exploration distincte c4225813 lue/ACK. Pas de test/revue runtime prétendu par ces agents.
## À vérifier
Linux/builds, lecteur retour, revue code et V15, activation sept composants/quatre hôtes, dépôt réel sans modèle, Git/publication/nettoyage. Anciennes façades ouvertes exclues de l'activation globale tant que T3 reste ouvert ; CLI installée et nouveaux processus seront vérifiés.
