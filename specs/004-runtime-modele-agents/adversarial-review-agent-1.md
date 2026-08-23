# Contre-revue adverse — plan (Phase 2)

**Date** : 2026-08-17
**Agent interrogé** : `agent-1`
**Croisement** : relecture par un moteur distinct de celui de l'auteur
**Question posée** : ce plan a-t-il un défaut de conception qui le rendra faux ou
fragile en production, sur la sonde Codex, le hook Claude, l'invariant `null` et
l'absence de disjoncteur sur le message `Runtime` ?
**Verdict reçu** : `BLOCKED`, puis `APPROVE` après corrections.

## Traitement des objections

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| P0-1 | L'invariant « `null` n'écrase jamais » fige un effort périmé : Opus/`high` → Haiku (sans effort) afficherait `haiku` + `high` | Recoupée avec ma propre preuve de research.md D-001, où Haiku ne porte aucun champ `effort` | **Oui** | Défaut réel et démontrable. Corrigé, mais **par simplification et non par le tri-état proposé** : une observation est atomique, elle remplace les deux champs d'un coup. Les deux parseurs lisent le modèle et l'effort au même endroit du même fichier ; le cas « je connais l'un sans l'autre » n'existe dans aucune source. L'invariant 1 disparaît au lieu de se complexifier |
| P0-1 bis | Conserver un curseur/horodatage d'observation pour qu'une sonde lente n'écrase pas une déclaration manuelle plus récente | Examen du chemin réel : la sonde n'émet que sur changement, et ce qu'elle lit est l'état effectif de l'agent | **Non** | Si la sonde contredit une déclaration manuelle, c'est la sonde qui a raison : elle observe, l'humain déclarait. Ajouter un curseur d'ordonnancement pour protéger une valeur moins fiable est une abstraction sans usage (Article XIX) |
| P0-2 | La fenêtre de lecture doit jeter la première ligne tronquée et la dernière ligne incomplète, scanner à rebours les lignes complètes, agrandir avec recouvrement | Raisonnement direct : une fenêtre de 256 Kio prise depuis la fin d'un fichier de 945 Mo commence presque toujours au milieu d'une ligne | **Oui** | Défaut réel. Ma formulation initiale de T006 mentionnait le cas sans imposer le comportement. Le contrat est maintenant explicite et testé aux deux bornes |
| P0-2 bis | Identifier le rollout actif par identifiant de session + offset plutôt que par `mtime`, et ne publier que si plus récent | Observation `lsof` du PID 25987 : deux rollouts ouverts, celui de la conversation reprise et celui du jour. Le fichier écrit est nécessairement celui dont le `mtime` avance | **Non**, sauf la fixture | Le tri par `mtime` ne peut pas régresser : on relit le dernier `turn_context` du fichier actif, jamais un état antérieur. Le curseur proposé protège d'un scénario que je n'ai pas pu produire. La fixture « deux rollouts, `mtime` inversés » est ajoutée, elle, car elle coûte zéro et verrouille le tri |
| P0-3 | Prendre la dernière ligne assistant **portant réellement `message.model`**, pas simplement la dernière ligne assistant hors sous-agent | Inspection des transcrits : des lignes `assistant` synthétiques existent | **Oui** | Correction gratuite et strictement plus sûre. Appliquée à T006 |
| P0-3 bis | Le hook doit rejeter l'ambiguïté d'une session Claude enfant héritant de `BRIDGET_AGENT_NAME`, sinon le modèle d'un sous-agent est attribué au parent ; « contredit SC-001 » | Recherche d'un discriminant : `session_id`, `cwd`, chaîne de PID, `instance_id` — un processus enfant hérite de tout l'environnement du parent et partage ses ancêtres | **Non** | Objection réelle sur le fond, mais aucune correction à la fois simple et correcte n'existe. Fixer le `session_id` au premier hook casserait le cas nominal du `/clear`, qui change d'identifiant. Le risque reste accepté et documenté (research.md D-002), le `session_id` est journalisé en `debug` pour rendre le cas diagnosticable. Le lien avec SC-001 est inexact : ce cas produit une valeur potentiellement fausse, pas une valeur inconnue |
| P1-a | `source` doit être une énumération fermée, pas une chaîne libre | Lecture du contrat : trois valeurs, aucune extensibilité prévue | **Oui** | Gratuit, plus sûr. Appliqué à T001 |
| P1-b | Limite de débit par connexion, déduplication par `observation_id`, métrique de rejet | Analyse des producteurs : la sonde n'émet que sur changement, le hook une fois par tour, la déclaration est manuelle | **Non** | Les trois producteurs sont notre propre code. Un flot anormal serait un bug chez nous, pas un abus externe : le rate-limit masquerait le symptôme au lieu de le révéler. La journalisation `debug` de chaque mise à jour avec sa source suffit au diagnostic (Article XIX) |

## Corrections appliquées aux artefacts

1. `data-model.md` — invariant 1 remplacé par le remplacement atomique ; table
   des transitions réécrite.
2. `contracts/protocol.md` — règle de traitement 2 réécrite ; `source` en
   énumération fermée ; contrat de lecture par fenêtre explicité.
3. `tasks.md` — T001 (`source` énumérée), T003 (transitions), T006 (scan à
   rebours, ligne portant `message.model`, quatre fixtures supplémentaires).
4. `research.md` — D-002 complété sur le risque de session enfant.

## Objections rejetées, assumées

Deux propositions ont été écartées avec raison écrite : le curseur
d'ordonnancement des observations et le garde-fou de débit sur `Runtime`. Si
l'un des deux scénarios se produit en usage réel, la journalisation `debug`
prévue permet de le constater, et la décision pourra être rouverte sur données.
