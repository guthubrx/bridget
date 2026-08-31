# Recherche - Pilotage des rondes par projet dans l'interface

## Décision 1 - Étendre l'autorité existante

**Décision**: réutiliser le contrat `ProjectRoundPolicyV1`, la table `project_round_policies` et les mutations `Enable` et `Disable`.

**Raison**: ces éléments portent déjà l'idempotence, la génération, les refus et l'état sûr par défaut de la SPEC-079.

**Alternatives considérées**: préférence locale du navigateur, route écrivant directement SQLite et second service de planification. Rejetées car elles créeraient une seconde autorité ou contourneraient les gardes.

**Impact mainteneur**: un seul contrat explique la CLI et l'interface.

## Décision 2 - Joindre la ronde à la liste des projets

**Décision**: enrichir chaque entrée de `GET /v1/projects` avec sa génération et une projection `round`.

**Raison**: la ligne et le menu dépendent du même fait. Une route de lecture séparée créerait deux chargements, deux états d'erreur et un risque de désynchronisation dans le navigateur.

**Alternatives considérées**: un endpoint par projet et un cache JavaScript secondaire. Rejetés pour éviter N+1 et état dérivé redondant.

**Impact mainteneur**: un rafraîchissement restaure tout l'état confirmé.

## Décision 3 - Persister seulement le dernier résultat utile

**Décision**: ajouter trois champs optionnels à la politique courante : occurrence, résultat fermé et instant d'observation.

**Raison**: `updated_at` décrit la politique, pas le passage du timer. Le ledger contient du contenu et ne constitue pas une projection de santé par projet.

**Alternatives considérées**: analyser les messages de ronde, créer une table d'historique ou ne rien afficher. L'analyse de texte est fragile, l'historique est hors besoin et l'absence d'information maintient l'ambiguïté constatée.

**Impact mainteneur**: une ligne suffit pour répondre à « la ronde est-elle passée récemment ? » sans nouvelle rétention.

## Décision 4 - Résultat fermé à trois états

**Décision**: classer le passage en `deposited`, `refused` ou `indeterminate`.

**Raison**: le socle idempotent peut attester un dépôt, un refus ou une issue dont le résultat final n'est pas prouvé. Transformer l'indétermination en succès serait mensonger.

**Alternatives considérées**: booléen succès/échec et stockage de l'issue complète. Rejetés respectivement pour perte de vérité et fuite de complexité protocolaire dans l'UI.

**Impact mainteneur**: vocabulaire stable et projection sans détail fournisseur.

## Décision 5 - Ne pas calculer une heure exacte

**Décision**: afficher « prochain cycle global, au plus sept minutes » lorsque la politique est active.

**Raison**: le contrat connaît la cadence, mais pas le prochain déclenchement effectif du timer systemd. Une heure calculée dans le navigateur pourrait être fausse après redémarrage ou retard.

**Alternatives considérées**: minute exacte locale et compte à rebours. Rejetés comme fausse précision.

**Impact mainteneur**: aucun timer UI ni synchronisation d'horloge à entretenir.

## Décision 6 - Menu contextuel comme surface canonique

**Décision**: ajouter un `menuitemcheckbox` au menu projet existant et un suffixe discret sur la ligne lorsque la ronde est activée.

**Raison**: la SPEC-080 a déjà rendu ce menu accessible par trois points, clic droit et clavier. La ronde est une propriété du projet, pas un réglage global du serveur.

**Alternatives considérées**: interrupteur permanent sur chaque ligne, écran global ou boîte modale. Rejetés car plus encombrants et moins cohérents.

**Impact mainteneur**: aucune nouvelle navigation ni overlay.
