# SPEC-162 — Exécution distante explicite et visibilité des tâches

**Statut :** Implemented
**Priorité :** P1
**Périmètre :** Bridget maître, exécuteur distant et Lineage T3
**Branche :** session-162-execution-distante
**Tâches :** 44/44 — développement/préparation ; gates G8/G9 ouverts
**Date :** 2026-10-13

## Problème

Le client Cartae livré en 161 rejoint le maître du Mac. Il ne permet pas de choisir Cartae pour exécuter une mission : le lancement actuel reste sur le maître. L'utilisateur veut choisir une machine de sa flotte, suivre le travail dans T3 et conserver le fonctionnement de Bridget lorsque T3 est fermé. Il faut éviter un lancement doublé après une coupure et conserver les droits du demandeur.

## Solution

Permettre de désigner une machine et un moteur explicitement. La machine doit avoir été provisionnée et autorisée pour le projet. Bridget conserve une seule autorité sur les tâches et leur résultat. L'exécuteur distant applique les permissions, démarre le travail, observe son processus et accepte les annulations corrélées. Il ne remplace pas le maître.

T3 présente les tâches du fil dans son Lineage existant, avec machine, moteur et fraîcheur du contact. Les tâches continuent sans T3. La version de T3 actuellement chargée reste compatible. L'installation des nouveaux fichiers T3 ne déclenche pas sa relance.

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

Comme utilisateur, je veux consulter journal, résultat et état, puis annuler la bonne exécution même si T3 est fermé.

**Vérification indépendante :** lire les données natives depuis une entrée autorisée ; annuler une mission synthétique sans toucher un autre processus.

1. Les journaux restent durables sur le worker et sont lus à la demande via le maître. Une panne du worker rend le journal explicitement indisponible. Le dernier état et le résultat déjà reçus restent au maître.
2. Une annulation vise l'identité complète de l'exécution. Un processus ayant recyclé le même numéro ne reçoit pas le signal.
3. Si le distant est injoignable, l'annulation reste à confirmer ; elle ne devient pas un arrêt réussi par supposition.
4. Sans T3, exécution, suivi et annulation Bridget continuent à fonctionner.

### US5 — Afficher la tâche distante dans T3 (P2)

Comme utilisateur de T3, je veux reconnaître la machine et le moteur d'une tâche du fil et savoir si son état de contact est connu.

**Vérification indépendante :** tests du Lineage et recette avec données synthétiques, sans redémarrer l'application courante.

1. La tâche apparaît dans le Lineage avec machine, fournisseur, modèle, état et contact attesté lors de la lecture ; son journal est accessible lorsque le worker est joignable ; le résultat déjà reçu reste disponible au maître.
2. Maître accessible mais worker perdu : afficher une fraîcheur dégradée distincte de la disponibilité du maître.
3. Fermer puis rouvrir la vue relit la tâche sans relancer un fournisseur ni créer une nouvelle conversation fournisseur.
4. Ancien T3 chargé : réponses historiques compatibles. Nouveau T3 avec ancienne version Bridget : repli de lecture autorisé et métadonnées explicitement indisponibles, jamais machine locale inventée.

### Cas limites

Reconnexion légitime avec ancienne connexion fantôme, credential révoqué, ancienne époque de connexion, mapping changé, racine liée ou déplacée, source de permission devenue illisible, quota temporaire avant capture, résultat divergent, publication concurrente à une relance worker, délai expiré, disque plein, reçu manquant/corrompu, crash entre création et marqueur, après autorisation avant libération, après libération avant réponse, absence de preuve de mort, résultat arrivé après demande d'annulation, projection T3 partielle ou refusée. Chaque cas conserve l'autorité du maître et refuse les inférences dangereuses.

## Exigences fonctionnelles

- **FR-001** : accepter un placement explicite vers une machine provisionnée et un moteur explicitement désigné ; sans placement nouveau, préserver la voie locale historique.
- **FR-002** : désactiver par défaut les machines distantes non configurées ; machine inconnue, désactivée ou capacité absente interdit la route sans repli local.
- **FR-003** : authentifier le worker et lier sa connexion à une machine et une révision de provisionnement ; refuser un hôte simplement déclaré ; une reconnexion autorisée remplace atomiquement l’ancienne connexion et invalide ses trames.
- **FR-004** : conserver un seul maître pour tâches, réservations, droits, reprises et résultats ; aucune seconde base métier distante.
- **FR-005** : exiger un mapping approuvé entre projet et racines source/distante, vérifier identité du projet et appartenance canonique du répertoire aux racines. Le commit de base est contrôlé à la préparation ; les commits et modifications autorisés de la tâche ne sont pas traités ensuite comme une divergence à effacer.
- **FR-006** : utiliser les outils et secrets déjà configurés sur la machine d'exécution, sans transport de l'environnement ni des secrets du maître.
- **FR-007** : conserver le plafond des droits du parent, appliquer les restrictions locales et refuser une traduction de permission non démontrable.
- **FR-008** : recontrôler les sources de droits et le mapping au moment de l'autorisation finale ; le worker vérifie encore ses sources locales et l'expiration juste avant libération.
- **FR-009** : réserver la commande au maître et conserver un reçu durable sur le worker avant création du bootstrap, puis le marqueur avant sa libération.
- **FR-010** : rejouer une commande identique sans duplication et refuser le même identifiant avec un contenu différent.
- **FR-011** : exiger une autorisation finale corrélée et fraîche du maître après préparation du processus ; une connexion perdue ou un permis invalide empêche sa libération.
- **FR-012** : après une panne ambiguë, réconcilier le processus attesté ou exposer l'inconnu ; ne jamais interpréter l'absence de reçu complet comme une autorisation de recréer.
- **FR-013** : borner reçus, messages, connexions et rétention ; saturation ou corruption ferme les nouveaux lancements sans effacer les preuves actives.
- **FR-014** : distinguer réception, processus préparé, libération, connexion du wrapper et résultat métier ; chaque observation porte son identité et sa génération.
- **FR-015** : annuler uniquement le processus attesté par machine, instance, génération et naissance ; conserver l'annulation en attente quand l'arrêt ne peut être prouvé.
- **FR-016** : conserver le placement fixé pendant les reprises 152/160 ; une nouvelle génération n'est permise qu'après réconciliation terminale de la précédente selon les règles existantes.
- **FR-017** : conserver les échéances du maître et les seuls gels autorisés ; une déconnexion ne prolonge pas une mission.
- **FR-018** : réemployer les journaux, résultats, permissions enfant et statuts natifs. Conserver le journal durable sur le worker, avec lecture autorisée à la demande via le maître. Signaler son indisponibilité lorsque le worker est perdu, sans miroir permanent. Conserver au maître le dernier état et le résultat déjà reçus.
- **FR-019** : poursuivre les tâches lorsque T3 est fermé et relire leur état à la réouverture sans déclencher une nouvelle exécution.
- **FR-020** : présenter les tâches distantes du fil dans le Lineage existant ; aucun nouveau fil fournisseur ni entrée de conversation par tâche distante.
- **FR-021** : exposer machine, fournisseur, modèle, état, contact observé lors de la lecture de la page et fraîcheur du worker depuis des données Bridget attestées, sans inférence depuis le chemin ou l'hôte T3.
- **FR-022** : garder les réponses historiques Lineage strictement compatibles par défaut ; enrichir seulement à la demande d'une version supportée.
- **FR-023** : préserver les gardes parent/projet, pagination, annulation native et données historiques en cas de panne ; ne pas mélanger pages ou versions d'un snapshot.
- **FR-024** : garder les machines/projets et credentials dans une configuration privée et bornée ; toute lecture invalide ou révocation ferme la route concernée.
- **FR-025** : provisionner le worker sans afficher ses secrets, négocier la version avant mutation et protéger le maître contre tout message worker non autorisé.
- **FR-026** : livrer les sources intégrées et les fichiers requis dans Bridget et le fork T3 autorisé, vérifier les pushes, distinguer installation sur disque et runtime chargé.
- **FR-027** : préserver l'interdiction de relancer T3 ; fournir les preuves du nouveau rendu en environnement isolé et annoncer que l'application chargée garde sa version actuelle jusqu'à relance autorisée.
- **FR-028** : archiver les preuves, conserver un rollback utile et nettoyer les ressources propres au lot selon leurs gardes, sans supprimer les bases ni le travail étranger.

- **FR-029** : réemployer la complétion, l’attente et les notifications parent des specs 149–153. Persister atomiquement l’enveloppe finale dans un blob immutable privé avant publication, distinct des métadonnées écrites par le worker. La réémettre, même après mort du wrapper, jusqu’à disposition durable du maître. Distinguer résultat accepté, rejet définitif prouvé, conflit et report temporaire sans confondre un report quota avec une fin et rendre le résultat accessible au parent absent lorsqu’il revient. Ne pas confondre ce résultat avec une intégration Git.
- **FR-030** : pour une tâche de code, figer le dépôt et le commit de base, puis réemployer l’isolation 154 sur la machine choisie. Autoriser les modifications de la branche de tâche. Documenter dépôt, branche, commit et tests dans le résultat. Les pushes suivent la mission ; un intégrateur unique conserve la responsabilité de la branche principale. Aucun nettoyage d’un travail non récupéré.

## Critères de succès

- **SC-001** : une tâche synthétique sur chacun des placements autorisés s'exécute au lieu demandé et produit un seul résultat corrélé.
- **SC-002** : les cas machine/projet/capacité non autorisés sont refusés avant lancement ; aucun lancement local de remplacement.
- **SC-003** : tous les tests de permission non traduisible, restriction locale, révocation et source modifiée refusent sans gain de droits ni fuite d'environnement.
- **SC-004** : chaque frontière de panne définie dans le contrat possède un test ; aucun doublon ni libération sans permis dans ces cas.
- **SC-005** : annulation d'une génération attestée réussit ; PID recyclé et preuve insuffisante ne signalent pas le mauvais processus.
- **SC-006** : reprises 152/160 conservent placement et échéance ; perte du distant ne déclenche pas un autre hôte.
- **SC-007** : les lectures natives et les demandes enfant fonctionnent sans T3 ; le journal distant indisponible est signalé. Une coupure après production du résultat et avant publication permet sa récupération sans double complétion. Un parent revenu retrouve le résultat durable.
- **SC-008** : tests du Lineage montrent machine et contact worker distinct du maître, historique préservé, lecture de reprise sans lancement fournisseur.
- **SC-009** : ancien décodeur T3 accepte les sorties par défaut du nouveau Bridget ; les champs distants n'apparaissent que dans la réponse enrichie négociée.
- **SC-010** : limites de messages/reçus/connexions sont vérifiées aux bornes et une saturation ne supprime pas de preuve active.
- **SC-011** : Git et fichiers installés des deux dépôts ont une provenance vérifiée ; aucune relance T3 n'est déclenchée et cette limite est nommée dans le reçu.
- **SC-012** : chaque ressource créée pour 162 possède une preuve de retrait ou de conservation justifiée ; aucun travail étranger perdu.

## Dépendances, hypothèses et limites

Specs 095/096/161 pour le canal et client Linux ; 149 pour identité/Lineage ; 152 pour reprise ; 154 pour isolation ; 155 pour demandes enfant ; 157 pour catalogue/quota ; 160 pour reprise sur limite. L'authentification effective des fournisseurs et les mappings de production ne sont pas déduits de leur installation. Les paramètres à figer sont listés dans le plan ; leur absence interdit l'activation réelle concernée, pas les tests isolés.

Les processus exécutés sous le même UID partagent leur domaine de confiance : 0600/0700 ne confine pas un fournisseur full-access. Le credential prouve sa possession, pas l’intégrité du worker. Les chemins explicitement accordés ne doivent jamais chevaucher le credential, son répertoire privé ou l’état worker ; ce contrôle ne remplace pas une sandbox OS.

## Hors périmètre

Choix automatique machine/modèle et serveur de décision 163 ; vue globale de flotte dans la colonne gauche T3 ; remplacement de l'ordonnanceur T3 ; création de conversations fournisseur supplémentaires ; copie de secrets ; réplication du maître ; isolation par nouveau compte Unix ou sandbox OS ; réactivation des moteurs historiques 065/066 ; migration automatique d'une tâche déjà lancée vers un autre hôte.
