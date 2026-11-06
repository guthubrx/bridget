# Validation 239 — maintenance interprojets

## État initial vérifié

Recherche native mainteneur-bridget : un seul spécialiste, bridge1016,
bc833746-3f24-41cd-bc64-46d60f4c0769. Révision 1, mandat vide,
scope.projects=[bridget], transversal=false, rôle de même périmètre.
enabled=true, durable=false, auto_start=false, grant=null.

Les gardes natives list/search/get filtrent ce spécialiste depuis Revue.
Le défaut vient de sa configuration de périmètre. Les gardes ne sont pas
modifiées et aucun accès aux données métier des autres projets n'est ajouté.

## Modification préparée

configure_mandate conserve la cible, les budgets facultatifs, auto_start=false
et launch_policy=null. Il décrit la maintenance de Bridget pour tous les
projets et fixe scope.transversal=true. Les droits sur fichiers, comptes,
mémoires métier et permissions restent soumis aux gardes existantes.

Le contrat natif exige un acteur humain attesté pour configure_mandate.
request_approval conserve l'opération complète et la révision attendue 1.
La question native est a80e8ab5-c21e-4247-acc9-68b700f6a85c ; opération
a8efcad4-f4e0-4210-96b7-819d059ab5f9. Une réponse libre ou un champ forgé
ne remplace pas cette décision. L'opération n'est pas appliquée tant que
la question reste awaiting_human.

Après application, declare_roles doit conserver le rôle mainteneur-bridget
avec la portée transversale déjà approuvée. La déclaration seule n'ajoute
aucun droit et ne démarre aucun processus. La visibilité doit ensuite être
vérifiée depuis le contexte natif d'un autre projet.

## Preuves privées

/Users/moi/.cache/bridget239-maintenance-transversale

Aucun code exécutable n'est modifié. Aucun build, test applicatif ou mission
de revue n'est nécessaire pour cette configuration native. T3 reste ouvert.
La correction n'est pas annoncée comme appliquée avant les reçus et contrôles.

La question native reste pending après une attente bornée de 40 secondes.
Le contrat exact est documenté dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/222-roles-profils/contracts/closure-after-review.md,
section « Approbation conversationnelle native ». La garde est implémentée dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/specialists222.rs,
fonction request_approval, et dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/specialists.rs,
fonction requires_human. L'autorisation conversationnelle de correction ne
fabrique pas l'acteur humain authentifié attendu par ce port. La proposition
est conservée et peut reprendre avec sa clé, sans nouvelle question ni canon.
