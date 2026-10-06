# Spécification 135 — Contrôle silencieux des missions

**Branche** : session-135-controle-missions-silencieux
**Date** : 2026-10-05
**Statut** : Implemented
**Priorité** : P1
**Demande** : réduire les messages répétés des boucles et rendre le suivi des
missions plus contraignant.
**Dépendances** : 098 (pont T3), 114 (lots), 133 (relais sous-agents), agent-loop.

## Pourquoi

La boucle Politique répète les mêmes alertes à chaque tranche de temps. Elle
réveille le coordinateur sans nouvelle information. Elle ne distingue pas une
promesse d'action, une prise en charge et un progrès vérifiable. Un agent peut
donc rester silencieux sans conséquence claire.

Le résultat attendu est un contrôleur sobre. Il ne parle que pour une nouvelle
anomalie, un changement d'état ou une escalade. Il conserve une preuve durable
de chaque rappel.

## Scénarios utilisateur et tests

### US1 — Ne pas répéter une alerte inchangée (P1)

Étant donné une anomalie déjà notifiée, quand plusieurs passages ne constatent
aucun changement, aucun nouveau message n'est envoyé.

Test indépendant : exécuter trois heartbeats sur le même état et vérifier un
seul envoi, puis modifier l'état et vérifier un nouvel envoi.

### US2 — Contrôler la prise en charge et le progrès (P1)

Étant donné une mission envoyée, le responsable doit la prendre en charge ou
déclarer un blocage précis sous deux minutes. Un progrès vérifiable doit être
inscrit sous cinq minutes, puis au même délai après le dernier progrès.

Test indépendant : simuler le temps avant et après chaque délai. Vérifier le
rappel du responsable, puis du coordinateur, sans considérer « je vais faire »
comme une preuve.

### US3 — Escalader une inaction persistante (P1)

Étant donné deux rappels sans progrès vérifiable, le passage suivant informe le
rôle d'escalade. Si ce rôle manque, une demande de décision durable est créée.

Test indépendant : laisser une mission inchangée pendant trois échéances et
vérifier la séquence responsable, coordinateur, escalade.

### US4 — Décider la suite de chaque résultat (P1)

Étant donné un résultat terminal créé après l'activation du contrôle, le
coordinateur doit enregistrer une acceptation, une correction, une tâche
suivante ou un blocage avec propriétaire et prochain contrôle.

Test indépendant : vérifier qu'un résultat sans décision est signalé une fois,
puis disparaît après sa décision sans changer artificiellement son verdict.

### US5 — Fermer explicitement une boucle (P2)

Étant donné un chantier ouvert, `ready_tasks: 0` ne prouve pas sa fin. La boucle
ne devient terminée qu'après une clôture explicite et vérifiée.

Test indépendant : refuser la clôture avec une mission active ou un résultat
sans décision. Accepter la clôture lorsque toutes les obligations sont levées.

## Exigences fonctionnelles

- **FR-13501** : chaque mission nouvelle contient un identifiant, un responsable,
  une échéance et un résultat attendu.
- **FR-13502** : le délai de prise en charge vaut 120 secondes par défaut.
- **FR-13503** : le délai de progrès vérifiable vaut 300 secondes par défaut.
- **FR-13504** : une preuve de progrès indique un type d'action, un responsable
  et un élément vérifiable. Une intention future seule est refusée.
- **FR-13505** : la première échéance dépassée vise le responsable. La deuxième
  vise le coordinateur. Après deux rappels sans progrès, la suivante vise le
  rôle d'escalade ou crée une demande de décision.
- **FR-13506** : une anomalie inchangée ne produit pas de message répété.
- **FR-13507** : les notifications sont regroupées par destinataire. Un passage
  produit au maximum un message court par destinataire.
- **FR-13508** : un résultat terminal nouveau exige une décision de suite sans
  modifier son verdict initial.
- **FR-13509** : une pause d'exécution ne bloque pas automatiquement les tâches
  de code, de données ou d'autorisation indépendantes.
- **FR-13510** : `ready_tasks: 0` n'est jamais présenté comme une clôture.
- **FR-13511** : la clôture est explicite et refusée tant qu'une obligation du
  contrôle reste ouverte.
- **FR-13512** : les runs existants sont compatibles. Leur historique n'est ni
  réécrit, ni déclaré accepté. Le nouveau contrôle s'applique à partir d'une
  date d'activation enregistrée.
- **FR-13513** : aucun service résident, dépendance ou session tmux nouvelle
  n'est requis. Les LaunchAgents existants chargent la correction en direct.
- **FR-13514** : aucun contenu de prompt, raisonnement ou secret ne figure dans
  les alertes, états de contrôle ou diagnostics.

## Critères de succès

- **SC-13501** : trois passages sur un état inchangé causent exactement un envoi.
- **SC-13502** : les seuils 120 et 300 secondes sont couverts avant, à et après
  leur limite.
- **SC-13503** : la troisième échéance sans progrès atteint le rôle d'escalade.
- **SC-13504** : chaque destinataire reçoit au plus un digest par passage.
- **SC-13505** : aucun statut `review` n'est converti en `pass` par le contrôleur.
- **SC-13506** : un run ancien est migré sans perte de tâche, résultat ou événement.
- **SC-13507** : tous les tests agent-loop et la validation de skill passent.

## Hors périmètre

- Forcer un agent à produire un raisonnement.
- Évaluer automatiquement la qualité métier d'un résultat.
- Réaffecter automatiquement une mission sans politique explicite.
- Modifier Bridget, T3 ou les fournisseurs de modèles.
