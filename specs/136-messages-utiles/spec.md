# Spécification 136 — Messages utiles, historique conservé

Statut: Implemented | Priorité: P1 | Date: 2026-10-06
Branche: session-136-messages-utiles | Tests: 10/10 ciblés (100%) ; workspace PASS

## Pourquoi

64 messages directs du coordinateur ont atteint un même agent pendant une nuit.
Les lots de huit ont gardé les preuves mais aussi les ordres remplacés ensuite.
Il faut séparer le journal des actions actuelles, sans deviner le sens du texte.

## Scénarios utilisateur et tests

### US1 — Preuve conservée sans sollicitation (P1)

Publier un compte rendu comme historique ne réveille personne. Une lecture de
travail indique où le relire sans recopier son corps. L'historique le garde exact.
Test indépendant: 64 comptes rendus, zéro sollicitation, 64 corps récupérables.

### US2 — Consignes actuelles (P1)

L'auteur publie une courte action, puis une correction qui la remplace explicitement.
La lecture montre le corps courant et une référence vers l'ancien ; les deux
restent dans l'histoire. Un blocage indépendant reste visible.
Test: dix remplacements, une décision nouvelle, un blocage indépendant et texte
« annule tout » sans métadonnée (aucun remplacement implicite).

### US3 — Reprise et compatibilité (P1)

Après redémarrage, preuves et remplacements subsistent. Un autre auteur ne peut
pas remplacer un ordre. Les anciens messages non classés restent inchangés.
Test: ancienne base, mauvais auteur/audience, conflit, lecture rejouée à
l'identique après publication concurrente, confirmation puis page suivante.

## Exigences fonctionnelles

- FR-13601: distinguer explicitement historique, action, blocage et décision.
- FR-13602: refuser une sollicitation attachée à un historique, sans la masquer.
- FR-13603: conserver les corps exacts et leur provenance dans l'histoire.
- FR-13604: limiter actions/blocages/décisions à 2 Kio UTF-8 ; citer les preuves longues.
- FR-13605: remplacement explicite d'une seule entrée antérieure du même fil,
  du même auteur et pour les mêmes destinataires effectifs.
- FR-13606: refuser référence absente, historique remplacé, branche concurrente
  et remplacement depuis un historique, sans écriture partielle.
- FR-13607: lecture de travail avec corps actuels et références compactes pour
  comptes rendus et ordres remplacés. Aucun résumé automatique.
- FR-13608: page non confirmée rejouée à l'identique ; publications suivantes
  visibles après confirmation. Le reçu confirme la projection, pas la mission.
- FR-13609: anciens dépôts, messages directs et réponses attendues inchangés.
  Aucun tri rétroactif par texte ; les 64 messages déjà remis restent conservés.
- FR-13610: dépôts idempotents et regroupement par fil/destinataire conservés :
  une rafale avant remise crée une seule sollicitation en attente.
- FR-13611: mêmes champs/refus CLI/MCP et règle lisible pour les coordinateurs.
  Un ancien serveur refuse les nouveaux champs plutôt que les ignorer.
- FR-13612: accès, quotas, pagination et reprise conservés. Aucun service,
  modèle ou minuteur supplémentaire.

## Entités

Entrée: auteur, ordre, corps exact, classe et remplacement explicites optionnels.
Lecture: projection au moment d'une lecture, références, reçu.
Sollicitation: invitation à consulter, distincte de la prise en charge d'une mission.

## Critères de succès

- SC-13601: 64 historiques, zéro réveil, 64 corps exacts récupérables.
- SC-13602: dix remplacements avant lecture, un seul corps courant transmis,
  anciens corps récupérables et blocages indépendants visibles.
- SC-13603: zéro perte, modification de verdict ou remplacement non autorisé
  dans les essais de conflit, pagination et redémarrage.
- SC-13604: tests ciblés, suite existante, lint et compilation passent.

## Limites et dépendances

Réutiliser fils 102, recherche sourcée 104 et remise 012/114. Mission 135 distincte.
L'auteur classe et référence ses entrées. Le pont ne comprend pas le texte libre.
Il ne rappelle pas un ordre déjà lu et ne retire rien du contexte fournisseur.
Les coordinateurs doivent utiliser le canal structuré ; la livraison transmet
cette règle. Les anciennes files ne sont pas converties automatiquement.
Conservation selon quotas existants, sans purge nouvelle. Pas de refonte d'interface.
