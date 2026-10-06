# Plan 135 — Contrôle silencieux des missions

Statut : Implemented. Branche : session-135-controle-missions-silencieux.
Spec : specs/135-controle-missions-silencieux/spec.md.

## Contexte technique

La boucle est un script Python autonome. Un LaunchAgent l'appelle à intervalle
fixe. Le script lit des fichiers JSON durables et utilise Bridget uniquement si
une action exige un message. Le wrapper Politique charge le script officiel par
`runpy`. Une correction du script est donc prise en compte sans réinstaller le
LaunchAgent.

Le défaut principal est dans `notify_signature()`. La signature contient une
tranche d'âge. Elle change à chaque délai et réveille le coordinateur avec le
même problème. Le second défaut est l'absence d'état séparé pour la prise en
charge, le progrès et l'escalade.

## Architecture retenue

1. Étendre les politiques du run avec les délais de prise en charge, de progrès
   et le nombre de rappels avant escalade.
2. Compléter chaque nouvelle tâche avec un responsable, une échéance et un
   résultat attendu, sans casser les anciens fichiers.
3. Ajouter `progress` pour enregistrer une action vérifiable avec un type fermé.
4. Remplacer la signature par un registre durable d'anomalies. Une anomalie ne
   redevient visible que si son étape change ou si la preuve observée change.
5. Construire un plan de notifications groupé. Le premier rappel vise le worker,
   le second le coordinateur, puis le rôle d'escalade.
6. Exiger une décision séparée pour les nouveaux résultats terminaux.
7. Ajouter `close-run`. Cette commande vérifie les obligations avant de fermer.
8. Activer le nouveau contrat sur Politique après sauvegarde et validation.

## Réutilisation de l'existant

- Conserver `events.jsonl`, les tâches JSON et `heartbeat-state.json`.
- Conserver `ack`, `resolve-task`, `relaunch` et les sessions Bridget.
- Étendre les reçus de continuation au lieu de créer un second journal.
- Conserver le LaunchAgent. Remplacer les deux surcharges Politique par l'appel
  direct au script officiel, qui porte désormais leurs garanties utiles.
- Ne pas modifier Bridget, T3, tmux ou les fichiers métier Politique.

## Fichiers

- `/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py`
- `/Users/moi/.codex/skills/agent-loop/tests/test_agent_loop.py`
- `/Users/moi/.codex/skills/agent-loop/SKILL.md`
- `specs/135-controle-missions-silencieux/`
- `docs/decisions/045-controle-suivi-missions.md`

## Tests

Les tests utilisent uniquement des répertoires temporaires et des transports
simulés. Ils couvrent les seuils, la déduplication, le groupement, la séquence
d'escalade, le destinataire injoignable, le progrès, la décision terminale, la
clôture et la migration. Un test de non-régression vérifie qu'une pause de type
`execution_authorization` ne bloque pas une tâche de type `code`.

La validation finale exécute tous les tests agent-loop, la compilation Python,
la validation de skill et un heartbeat isolé. La boucle Politique est sauvegardée
puis migrée. Son LaunchAgent est relancé seulement après ces contrôles.

## Déploiement et retour arrière

Le script officiel est chargé en direct. Le déploiement consiste à modifier la
skill, sauvegarder le run Politique, activer les nouvelles politiques et lancer
un heartbeat. Le retour arrière restaure les trois fichiers de skill et la copie
du run comme archive. Après la reprise des workers, ne jamais remplacer leur
registre par cette ancienne copie : restaurer uniquement le lancement et les
politiques nécessaires sur la version courante. Aucune base de données ni
binaire Bridget n'est remplacé.

## Constitution

- PASS : solution simple, locale et sans nouvelle dépendance.
- PASS : état durable et écriture atomique déjà présents.
- PASS : aucun verdict métier n'est fabriqué.
- PASS : aucune donnée libre ou confidentielle dans les diagnostics.
- PASS : parcours indexés et tris O(n log n). Trois vagues de notification au
  maximum. Les lectures des fichiers JSON indépendants restent nécessaires
  pour conserver le format historique ; aucun nouveau stockage central.
- PASS : comportement legacy explicite et testable.
