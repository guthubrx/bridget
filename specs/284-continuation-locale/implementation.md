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

Qualification : 31 tests ciblés, Clippy strict workspace/all-targets et release réussis sur Mac et Debian 12. Activation vérifiée le 12 novembre 2026 avec la version 0.8.48.

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

## Activation et publication

Version `0.8.48`, build `284-sondes-continuation-r1`, code
`1e917ec4ae279ee90d32f80d02ae8de11ded1c44`.
Publication : https://github.com/guthubrx/bridget/releases/tag/v0.8.48.
Les six fichiers publics ont été téléchargés sans authentification et leurs
empreintes vérifiées. L’APK 8048 garde l’identité de signature existante.

Les images réellement chargées sont vérifiées sur le coordinateur et mobile
Cartae, les workers Cartae, SYSLE2, KS, Mac et M1, ainsi que le connecteur et
l’outil de mise à jour Mac. Empreintes des exécutables :

- Mac : `53b667ff794fd57150b7a8c075d6545def6db1af13189f77bb06d6f9e78f90b6`.
- Linux : `fc1366610871ec8d0738b65e0b4006b0822f90daca766ce4d3df2c08d0b66df2`.

La version de retour est 282, avec empreinte et compatibilité de formats
vérifiées. Les pointeurs privés des outils de mise à jour sont alignés ;
M1 n'a pas cet outil, son image de retour est conservée et vérifiée.
La coupure du coordinateur a duré 3,79 secondes. Les lectures avant et après
n'ont relevé aucune admission courante ni démarrage en transition ; ce contrôle
ponctuel n'est pas un verrou contre une admission concurrente. Les processus
de mission contrôlés sont conservés. T3 garde le PID 77762 et sa naissance du
11 novembre à 14:10:41. Le lien natif et le service mobile répondent.
Les six propriétaires concernés ont reçu un avis `information`, conservé sans
réveil ; ce reçu n'atteste pas la lecture.

Preuves d’activation, de publication, de qualification et reçus :
/Users/moi/.cache/bridget283-284.

## Ménage et conservations

Le nettoyeur natif a retiré 1,15 Gio d’intermédiaires sur le Mac et 0,70 Gio
sur SYSLE2, en tailles logiques arrondies. Les exports, la copie de recette et
deux anciennes images non utilisées retirés sur Mac, SYS et M1 représentent
359 126 016 octets supplémentaires. Aucun worktree ni branche de travail n’a
été créé ; le rapport gardé du dépôt ne trouve aucun retrait éligible.

Le profil release SYS reste conservé : le lecteur de processus ne peut pas
qualifier le descripteur 7 du PID 4157668. La première relecture, justifiée
par sa disparition constatée, a permis le retrait du debug ; le refus est
revenu pour release, sans nouvelle boucle. Des copies Linux et les exports
Cartae/KS restent également conservés faute d’observation complète. Les
anciennes images Mac réellement utilisées par des sessions restent en place.
Les versions active et de retour, les vues de revue, les preuves et les
worktrees métier sont préservés. Aucun autre processus n’a été arrêté pour
nettoyer. Le ménage est donc partiel, avec conservations explicites.

Espace disponible mesuré sur `/` après ce passage, sans attribuer ses variations
au seul nettoyage : Mac 72 931 622 912 octets ; Cartae 105 235 292 160 ;
SYSLE2 39 130 480 640 ; KS 59 794 128 896.
