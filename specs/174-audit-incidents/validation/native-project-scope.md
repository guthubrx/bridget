# Remise native et emplacements de projet

Le contrôle de communication comparait le projet du parent sur Mac à celui de
son enfant sur Cartae comme deux projets distincts. La tâche était admise, puis
la remise interne échouait avec `cross_project_reason_required`. Le statut
lisait cet échec enregistré ; sa propre lecture n'était pas refusée.

La remise de la mission exacte réutilise maintenant la relation parent/enfant
admise. La connexion interne doit être éphémère, sans socket externe, dans le
scope de la tâche. Le contenu, la ronde courante, les deux identités vivantes,
les révocations et l'absence de clôture sont recoupés. Les contrôles de lancement,
de permissions et de placement du tick restent en amont. Le résultat canonique
utilise son autorité privée existante, déjà contrôlée. Aucun motif volontaire
interprojet n'est inventé. Les envois directs restent soumis à ce motif.

Le statut explique aussi que ce code décrit l'échec de remise et non celui de
la lecture. Il conserve le code initial. Il ne relance ni ne modifie la tâche.

## Référence T3

`apps/server/src/orchestration/decider.ts` rattache un fil à `projectId`,
indépendamment de son `worktreePath`. La comparaison de projet lors d'une
reprise de plan porte sur cette identité logique. Bridget réutilise ce principe
pour sa relation native déjà admise. T3 ne fournit pas la preuve de placement
multi-hôte de Bridget : celle-ci et les droits hérités restent obligatoires.

## Validation ciblée

Le test de reproduction utilise admission, tick, enregistrement de l'enfant et
sockets Unix de contrôle, sans fournisseur ni mission réelle. Avant correction,
il échoue avec le refus exact de l'incident. Après correction, les cinq tests
`native174_projet` passent : remise entre deux emplacements, refus du même message
par un client ordinaire, contenu et instances altérés, retour du résultat
canonique et diagnostic de statut sans écriture. Les projets des fixtures sont
injectés : ce n'est pas une recette complète avec un worker distant réel.

Le lot consolidé comprend aussi les contrôles de résultat retenu, de projet à
l’admission et de délai natif : 26 réussis, aucun échec ou test ignoré. Aucun
succès de suite complète n’est déduit de ce lot.

Aucune tâche métier existante n'est rejouée ou modifiée. Les journaux de tests
restent hors Git. La livraison doit conserver le candidat mobile actif et ne
pas le remplacer par une compilation de l'ancien main.

## État de livraison du 24 octobre 2026

Source `d5af84f8` intégrée et poussée sur main, checkout Mac et Cartae alignés.
Le worktree de correction et sa branche sont retirés après contrôle : intégrés,
propres y compris fichiers ignorés, aucune tâche liée, observation des fichiers
ouverts complète et sans usage. Le cache partagé de validation reste dans le
budget ; aucun nouveau cache ni copie de binaire n'a été créé.

L'activation reste ouverte. Le maître actif est `177-mobile-candidate-r6-opt`
sur Cartae, issu du chantier mobile en cours. Le correctif est transmis à son
responsable pour intégration à ce candidat ; aucune ancienne version de main
n'a été installée à sa place. Cette note ne prouve pas une livraison binaire.
Le demandeur connaît cette limite. La tâche de l'incident et ses fichiers sont
conservés ; son empreinte durable est identique avant et après l'intervention.
