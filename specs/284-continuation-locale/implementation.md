# Présence locale avant proposition de continuation

Défaut confirmé : une tâche locale gardée pouvait afficher `continuable=true`
alors que son enfant n'était pas présent. Au même instant, `Continue` refusait
la demande et fermait sa garde. La projection consulte désormais la même
preuve de présence pour les tâches locales et distantes.

Une présence locale non confirmée donne `continuable=false`,
`status=unconfirmed`, `code=local_child_unconfirmed`. Elle ne prouve pas
l'arrêt physique. La lecture ne ferme pas la garde et n'appelle aucun fournisseur.
Le guidage demande de conserver la tâche et de relire son état, sans utiliser
`Continue` comme test de présence.

`master_presence_confirmed` vaut désormais `true` ou `false` aussi pour un
enfant local éligible. `null` reste une absence de contrôle exploitable, pas
une preuve d'absence. Les anciennes projections locales à `null` n'attestaient
pas la présence. `requires_recheck=true` demeure : pause, droits et état peuvent
encore changer avant une reprise.

Revue indépendante : tâche 2ed70cd0-bd67-4e93-918d-c7fff75e4ccd.
Le test négatif affirme des valeurs littérales sur une fixture sans enfant.
Le test positif existant `native153_ronde_meme_enfant_rejeu_et_plafond`
vérifie la reprise d'un enfant local réellement enregistré.

Le double chargement préexistant de l'état désiré dans le contrôle de présence
reste hors de ce correctif. Aucun gain de performance n'est revendiqué.
Une optimisation de cette lecture demande une mesure et la préservation du
contrôle de génération ; elle n'est pas remplacée par un cache implicite.

Qualification : 31 tests ciblés, Clippy strict workspace/all-targets et release réussis sur Mac et Debian 12. Activation encore en cours.

## Preuves de qualification ciblée

Le test négatif local échoue avec le code antérieur (101, `true` au lieu de
`false`), puis passe avec la correction. Sur chaque plateforme, 31 tests
réussissent : un test de phases, un de projection, un de reprise locale,
13 de sonde et 15 de continuation. Aucun fournisseur métier n'a été lancé.

La première version du test de phases attendait un refus de base inconnue.
La fixture possède un origin fictif non local : le repli Git produit en réalité
`WorkspaceUnavailable`. L'attendu a été corrigé après lecture de ce parcours ;
aucun comportement de production n'a été changé pour cette fixture.

Le runtime est reconstruit depuis le lot 282. Les fonctions remplacées ont
été comparées avant modification après normalisation de format : elles étaient
identiques entre main et runtime. Les six fichiers modifiés par la recette
reproduisent exactement ceux qualifiés. Le programme réservé 260 reste exclu.
