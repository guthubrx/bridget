# Session148 — Identité par session et délégation Bridget fluide

Date : 2026-10-10. Statut : Approved / In Progress.
Accord : « ok on reste sur codex high, go ». Agent principal : Codex, effort high.

## Besoin et périmètre

L'utilisateur veut déléguer une mission à un modèle choisi en une opération,
recevoir son résultat et ne plus rencontrer de refus d'identité lorsque T3 partage
un processus fournisseur entre plusieurs conversations. Les échanges et leur
suivi restent explicites dans Bridget. Bridget fournit lui-même le lancement,
le catalogue, les tâches, le résultat et l'annulation. La présence de T3 n'est
jamais une condition. Le connecteur T3 adapte seulement l'identité et l'interface.
La livraison ne redémarre aucune application ni aucun service existant.

## Scénarios utilisateur

### US1 — Identité indépendante du processus (P1)

Deux conversations Codex utilisent le même processus T3. Chacune consulte son
annuaire et envoie un message sous sa propre identité. La fermeture d'une session
révoque son accès sans retirer celui de l'autre. Une preuve absente, étrangère,
expirée ou révoquée produit un refus explicite, sans emprunter une autre identité.

### US2 — Délégation en un appel (P1)

L'agent demande une mission à GLM depuis Codex. Une opération crée la tâche,
rattache le sous-agent et remet la mission par Bridget. Le résultat précise la
tâche, le fournisseur et le modèle effectivement sélectionnés. Une répétition de
la même opération ne crée ni second enfant ni seconde mission.

### US3 — Catalogue et choix exacts (P1)

L'agent consulte les modèles utilisables et leurs limites. Le choix explicite de
GLM ne devient jamais silencieusement Codex ou Claude. Les permissions du parent
bornent celles de l'enfant. Les désactivations et configurations MCP de l'humain
restent respectées.

### US4 — Résultat et état utiles (P1)

Le parent reçoit une seule remise automatique de résultat. Il peut consulter
l'état sans surveiller en boucle. Une fin de tour avec travail enfant actif ne
clôt pas la mission. Un résultat terminal publié reste stable. Un délai d'attente
écoulé ne constitue ni un échec de mission ni une annulation.

### US5 — Annulation et reprise (P1)

Le parent annule une tâche et ses descendants encore actifs. Il ne peut pas
annuler les tâches d'une autre conversation. Une coupure garde les références
de la tâche et les clés de rejeu. Une issue inconnue est annoncée comme telle.

### US6 — Usage simple et validation (P1)

La skill décrit le parcours direct : mission, cible, résultat. Les tests couvrent
au moins deux conversations partageant un processus, une délégation Codex vers
GLM, le refus étranger, la révocation, le rejeu et l'annulation. Les recettes
synthétiques et celles utilisant un fournisseur réel sont distinguées.

## Exigences fonctionnelles

- FR001 : attester la session appelante sans identité choisie dans les arguments métier.
- FR002 : séparer deux conversations actives dans un même processus fournisseur.
- FR003 : refuser une preuve invalide sans repli vers une identité déduite d'un processus.
- FR004 : vérifier la preuve à chaque appel et conserver la révocation T3.
- FR005 : vérifier le rattachement vivant Bridget avant toute opération sous cette identité.
- FR006 : conserver les identités natives, déléguées et les gardes de projet existantes.
- FR007 : exposer une opération de délégation avec mission et sélection explicite.
- FR008 : remettre la mission par Bridget ; une création T3 seule n'est pas une remise Bridget.
- FR009 : conserver les identifiants de tâche, enfant et message dès leur acceptation.
- FR010 : réutiliser les clés et enveloppes pour tout rejeu après coupure.
- FR011 : consulter le catalogue réel du moteur utilisé et ses raisons de refus.
- FR012 : interdire toute substitution de fournisseur/modèle demandé.
- FR013 : borner les droits de l'enfant par ceux de son parent.
- FR014 : préserver les choix humains de désactivation MCP et les configurations explicites.
- FR015 : remettre automatiquement un résultat corrélé au parent une seule fois.
- FR016 : distinguer travail actif, attente des enfants, résultat, échec et annulation.
- FR017 : ne pas confondre fin de tour et fin de mission.
- FR018 : consulter et annuler uniquement une tâche appartenant au parent.
- FR019 : annuler les descendants actifs avec les gardes de permissions existantes.
- FR020 : transmettre la mission, les références choisies et les faits natifs
  d'identité et de Git utiles à la reprise ; jamais la conversation du parent.
- FR021 : ne pas demander d'UUID, de fichier de suivi ou de commande de lancement à l'humain.
- FR022 : borner réseau, tailles, ressources et attentes ; ne jamais journaliser les jetons.
- FR023 : garder visibles les refus et les étapes acceptées d'une opération partielle.
- FR024 : conserver les processus et configurations de production durant le développement.
- FR025 : proposer les mêmes capacités de délégation avec et sans T3 ; aucun
  appel à delegate_task, au catalogue ou au scheduler T3 dans le moteur Bridget.
- FR026 : limiter les adaptations T3 à son connecteur ; une panne T3 ne doit
  pas interrompre les délégations natives ni masquer le catalogue Bridget.

## Critères de succès

- SC001 : deux conversations simultanées partagent un processus et réussissent leurs appels
  sous deux identités distinctes ; aucun message n'emprunte l'identité voisine.
- SC002 : la mission Codex vers GLM exige un appel de délégation, puis produit un résultat corrélé.
- SC003 : rejouer la même demande dix fois conserve un enfant et une remise de mission.
- SC004 : une session révoquée échoue au prochain appel ; la session voisine reste utilisable.
- SC005 : tous les scénarios négatifs de sélection, permissions et propriété refusent sans mutation.
- SC006 : l'annulation laisse zéro descendant de la tâche en exécution dans la recette isolée.
- SC007 : les tests ciblés et les régressions concernées réussissent avant livraison.

## Entités et hypothèses

Session appelante attestée ; rattachement Bridget ; tâche liée au parent ; reçu de
mission ; résultat final corrélé. Le daemon Bridget et ses fournisseurs déclarés
suffisent. T3 peut être absent.
Le profil GLM existe déjà et son authentification n'est pas copiée ni modifiée.
La mission n'hérite pas implicitement de tout l'historique parent. Aucun calendrier,
recrutement interprojet automatique ou nouveau panneau n'est ajouté.
