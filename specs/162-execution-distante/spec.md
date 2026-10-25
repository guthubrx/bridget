# SPEC-162 — Exécution distante explicite et visibilité des tâches

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-1).

## Problème

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-2).

## Solution

Permettre de désigner une machine et un moteur explicitement. La machine doit avoir été provisionnée et autorisée pour le projet. Bridget conserve une seule autorité sur les tâches et leur résultat. L'exécuteur distant applique les permissions, démarre le travail, observe son processus et accepte les annulations corrélées. Il ne remplace pas le maître.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-3).

## Scénarios utilisateur et tests

### US1 — Choisir la machine et le moteur (P1)

Comme utilisateur, je veux demander une tâche sur Cartae avec un moteur explicite, dans un projet autorisé.

**Vérification indépendante :** une tâche synthétique identifie son lieu réel d'exécution et produit un résultat corrélé au maître.

1. Machine activée, projet mappé et droits compatibles : la tâche s'exécute sur cette machine, une seule fois.
2. Machine inconnue, désactivée ou non joignable : refus explicite ; aucun lancement local de remplacement.
3. Projet absent ou répertoire extérieur à sa racine autorisée : refus avant lancement.
4. Demande locale historique sans placement : même comportement et mêmes droits qu'avant 162.

### US2 — Garder les permissions et les données sur le bon hôte (P1)

Comme propriétaire du projet, je veux que changer de machine n'élargisse pas les droits de l'agent et ne copie pas mes secrets.

**Vérification indépendante :** scénario de lecture limitée, accès interdit et révocation avant libération du processus.

1. Une permission de lecture seule reste de lecture seule sur le distant ; une racine non mappée reste interdite.
2. Une règle dont la traduction n'est pas démontrable provoque un refus, pas une permission plus large.
3. Un changement de politique ou de mapping après préparation invalide l'autorisation de lancement avant sa délivrance.
4. Les identifiants fournisseur et l'environnement du distant sont locaux au distant ; aucun environnement ou secret du Mac n'est copié pour lancer l'agent.

### US3 — Surmonter les coupures sans doublon (P1)

Comme utilisateur, je veux connaître l'état réel d'une tâche malgré un crash ou une déconnexion.

**Vérification indépendante :** coupures injectées aux frontières du lancement, puis réconciliation.

1. Une commande rejouée à l'identique restitue son état ; elle ne crée pas un autre processus.
2. Après un crash à un instant où le lancement peut avoir eu lieu, l'état devient à réconcilier ou inconnu ; aucun nouveau lancement automatique ailleurs.
3. Le contact perdu ne signifie ni succès ni échec métier. Une fin constatée et le résultat de mission restent deux preuves distinctes.
4. Les reprises autorisées conservent machine, moteur, projet et échéance. Une reprise quota n'est pas transformée en nouveau lancement distant.

### US4 — Suivre et annuler depuis Bridget (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-4).

**Vérification indépendante :** lire les données natives depuis une entrée autorisée ; annuler une mission synthétique sans toucher un autre processus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-5).

### US5 — Afficher la tâche distante dans adaptateur de bureau (P2)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-6).

**Vérification indépendante :** tests du Lineage et recette avec données synthétiques, sans redémarrer l'application courante.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-7).

### Cas limites

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-8).

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-9).

- **FR-029** : réemployer la complétion, l’attente et les notifications parent des specs 149–153. Persister atomiquement l’enveloppe finale dans un blob immutable privé avant publication, distinct des métadonnées écrites par le worker. La réémettre, même après mort du wrapper, jusqu’à disposition durable du maître. Distinguer résultat accepté, rejet définitif prouvé, conflit et report temporaire sans confondre un report quota avec une fin et rendre le résultat accessible au parent absent lorsqu’il revient. Ne pas confondre ce résultat avec une intégration Git.
- **FR-030** : pour une tâche de code, figer le dépôt et le commit de base, puis réemployer l’isolation 154 sur la machine choisie. Autoriser les modifications de la branche de tâche. Documenter dépôt, branche, commit et tests dans le résultat. Les pushes suivent la mission ; un intégrateur unique conserve la responsabilité de la branche principale. Aucun nettoyage d’un travail non récupéré.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-10).

## Dépendances, hypothèses et limites

Specs 095/096/161 pour le canal et client Linux ; 149 pour identité/Lineage ; 152 pour reprise ; 154 pour isolation ; 155 pour demandes enfant ; 157 pour catalogue/quota ; 160 pour reprise sur limite. L'authentification effective des fournisseurs et les mappings de production ne sont pas déduits de leur installation. Les paramètres à figer sont listés dans le plan ; leur absence interdit l'activation réelle concernée, pas les tests isolés.

Les processus exécutés sous le même UID partagent leur domaine de confiance : 0600/0700 ne confine pas un fournisseur full-access. Le credential prouve sa possession, pas l’intégrité du worker. Les chemins explicitement accordés ne doivent jamais chevaucher le credential, son répertoire privé ou l’état worker ; ce contrôle ne remplace pas une sandbox OS.

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/spec.md#extrait-11).
