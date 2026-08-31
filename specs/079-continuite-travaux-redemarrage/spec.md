# Spécification - SPEC-079 Continuité des travaux après redémarrage

## Fiche synthèse

Spec: 079-continuite-travaux-redemarrage
Titre: Continuité durable des travaux et ronde pilotée par projet
Statut: Implémentée - prête pour revue, non livrée
Priorité: P1
Branche: session-079-continuite-travaux-redemarrage
Créée: 2026-08-31
Dépendances: SPEC-063, SPEC-064, SPEC-065, SPEC-066, SPEC-067

## Contexte

Un message humain envoyé depuis l'interface est aujourd'hui remis de manière
idempotente, mais il n'est pas admis comme exécution durable. Si le service
Bridget est redémarré après l'acquittement de la remise et avant la fin du tour,
le processus géré est relancé mais le travail exact n'est pas repris. La carte
générique de reprise consulte Maicie et peut alors prescrire l'attente alors que
le travail venait directement de l'humain.

La ronde de vigilance ne doit pas compenser cette perte. C'est un réveil
périodique, pas une preuve de travail accepté. Elle doit être contrôlable par
projet, conformément aux frontières établies par les SPEC-065 à 067.

## Objectifs

- Admettre tout message humain qui démarre un tour comme travail durable.
- Relier atomiquement la remise idempotente à son exécution Bridget.
- Reprendre le message exact après redémarrage complet du daemon et du provider.
- Ne jamais rejouer un tour encore vivant lors d'une simple reconnexion.
- Publier la reprise comme continuation `reconstructed` de l'exécution interrompue.
- Rendre la ronde activable et désactivable indépendamment pour chaque projet.
- Garantir que la politique de ronde n'influence jamais la reprise d'un travail accepté.

## Hors périmètre

- Reprendre l'état interne exact d'un modèle lorsque le fournisseur ne fournit pas de primitive native.
- Déduire un travail depuis une ronde, un dépôt Git, une conversation ou une mission Maicie.
- Créer une instance Bridget ou Maicie par projet.
- Créer un timer systemd par projet.
- Migrer automatiquement les projets historiques non enregistrés.
- Modifier le cycle métier des objectifs et délégations Maicie.
- Déployer ou redémarrer la production dans le cadre automatique de cette skill.

## User Story 1 - Reprendre un travail humain accepté - P1

Comme utilisateur, je veux qu'un travail lancé depuis l'interface reprenne après
un redémarrage afin de ne pas devoir répéter ma demande ni surveiller les
redémarrages techniques.

### Scénarios d'acceptation

1. Un message humain UI porte `origin=human` et `intent=trigger_turn` avant son admission.
2. La remise, la soumission et l'exécution sont liées par leurs identifiants Bridget avant l'injection fournisseur.
3. Après acquittement de remise puis arrêt du daemon, le prochain wrapper géré reçoit le corps exact du message accepté.
4. L'ancienne exécution devient terminale avec une raison fermée et une nouvelle exécution `reconstructed` conserve le même `submission_id`.
5. Un second redémarrage avant acquittement rejoue la même remise durable sans créer une nouvelle continuation.
6. Une simple reconnexion avec `turn_in_progress=true` ne réinjecte aucun message.
7. Une exécution terminale n'est jamais rouverte.
8. Une enveloppe absente ou corrompue produit un état visible et ne fabrique aucun prompt.

## User Story 2 - Conserver la corrélation projet - P1

Comme opérateur, je veux qu'une reprise conserve la référence de projet afin que
le travail ne puisse pas être déplacé vers une autre racine ou génération.

### Scénarios d'acceptation

1. Une soumission sans projet reste explicitement non rattachée, sans inférence depuis le nom ou le cwd.
2. Une soumission projet conserve exactement `project_id` et `binding_generation` dans tous ses descendants.
3. Une génération de liaison divergente est refusée avant admission.
4. Une reprise ne relit ni racine hôte ni profil de secret pour reconstruire son identité.

## User Story 3 - Piloter la ronde par projet - P1

Comme opérateur, je veux activer ou désactiver la ronde pour chaque projet afin
de contrôler les réveils périodiques sans affecter les autres projets.

### Scénarios d'acceptation

1. Un projet actif possède une politique de ronde explicite `enabled` ou `disabled`.
2. En l'absence de politique, la ronde est désactivée et cet état est visible.
3. Une activation ou désactivation est corrélée par `command_id`, épinglée à la génération de liaison et rejouable sans second effet.
4. Un tick global sélectionne uniquement les projets actifs et activés.
5. Deux exécutions du même tick produisent la même clé d'idempotence par projet.
6. Désactiver la ronde empêche les futurs réveils de ce projet mais ne stoppe aucun agent et n'annule aucun travail.
7. Réactiver la ronde n'exécute pas les ticks manqués.
8. Un rebind rend l'ancienne politique inapplicable jusqu'à une décision explicite sur la nouvelle génération.

## Exigences fonctionnelles

- FR-07901: `send_ui_message` DOIT attribuer `origin=human` et `intent=trigger_turn`.
- FR-07902: une remise idempotente `trigger_turn` DOIT posséder un `WorkSubmission`, une `Execution` et un `DeliveryExecutionLink` avant injection.
- FR-07903: le lien remise-exécution DOIT être écrit dans la même transaction que la remise aval.
- FR-07904: le wrapper DOIT créer son binding d'exécution avant d'injecter une remise idempotente liée.
- FR-07905: un wrapper DOIT supprimer les doublons par `delivery_id` sans supprimer la corrélation d'exécution.
- FR-07906: la reprise DOIT relire `message_json`; elle NE DOIT PAS reconstruire le prompt depuis un résumé.
- FR-07907: la reprise DOIT conserver le `submission_id` et créer une nouvelle `Execution` liée en mode `reconstructed`.
- FR-07908: le parent DOIT devenir terminal avant que son descendant soit considéré actif.
- FR-07909: une remise de reprise DOIT être durable et idempotente avant écriture vers le wrapper.
- FR-07910: une remise de reprise déjà `dispatching` DOIT être rejouée, pas remplacée par une autre continuation.
- FR-07911: `turn_in_progress=true` DOIT interdire toute reconstruction.
- FR-07912: les états terminaux DOIVENT rester monotones.
- FR-07913: une référence projet absente DOIT rester absente; aucune déduction depuis le cwd, le domaine ou le nom n'est autorisée.
- FR-07914: tout descendant DOIT conserver la référence projet exacte du parent.
- FR-07915: la politique de ronde DOIT être indexée par `project_id` et `binding_generation`.
- FR-07916: l'état implicite d'une politique absente DOIT être `disabled`.
- FR-07917: les mutations de politique DOIVENT être idempotentes par `command_id` et refuser une enveloppe divergente.
- FR-07918: un projet inactif, absent ou d'une génération divergente DOIT être refusé.
- FR-07919: le scheduler global DOIT calculer une occurrence stable et au plus une remise par projet et occurrence.
- FR-07920: la désactivation NE DOIT ni interrompre, ni annuler, ni empêcher la reprise d'une exécution.
- FR-07921: aucune unité systemd par projet NE DOIT être créée.
- FR-07922: les actions de ronde DOIVENT utiliser le transport Bridget et une intention `trigger_turn`, jamais écrire directement dans un provider.
- FR-07923: l'état de continuité et l'état de ronde DOIVENT être observables séparément.
- FR-07924: les chemins Claude, Codex et Cursor ACP DOIVENT partager le même contrat sans branche fournisseur dans la politique de reprise.

## Cas limites

- Crash entre admission d'exécution et création de remise.
- Crash après création de remise mais avant écriture socket.
- Crash après injection fournisseur mais avant événement `running`.
- Reconnexion du même wrapper pendant un tour vivant.
- Redémarrage complet avec le même `instance_id` géré.
- Plusieurs exécutions actives historiques pour le même agent.
- Message historique sans `message_json`.
- Remise liée à une exécution inconnue ou terminale.
- Projet désactivé, rebindé ou supprimé entre deux ticks.
- Double lancement manuel du service de ronde dans la même occurrence.
- Aucun projet enregistré ou aucun projet activé.

## Données et confidentialité

La reprise conserve l'enveloppe Bridget déjà persistée. Elle n'ajoute ni secret,
ni contenu fournisseur, ni racine hôte. La politique de ronde contient seulement
identité opaque, génération, état, révision et horodatages.

## Critères de succès

- SC-07901: le test d'arrêt après acquittement reprend exactement une fois le message humain.
- SC-07902: zéro nouvelle continuation sur reconnexion avec tour vivant.
- SC-07903: cent pour cent des descendants conservent soumission et référence projet.
- SC-07904: un rejeu de remise utilise le même `delivery_id` et la même exécution.
- SC-07905: un double tick produit au plus une admission par projet.
- SC-07906: désactiver un projet ne modifie aucune exécution existante.
- SC-07907: les tests ciblés Rust, shell et Gherkin passent sans régression des SPEC-063 à 067.
