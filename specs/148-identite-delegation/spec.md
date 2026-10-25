# Session148 — Identité par session et délégation Bridget fluide

Date : 2026-10-10. Statut : Implemented / livré sur disque, sans redémarrage.
Accord : « ok on reste sur codex high, go ». Agent principal : Codex, effort high.

## Besoin et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/spec.md#extrait-1).

## Scénarios utilisateur

### US1 — Identité indépendante du processus (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/spec.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/spec.md#extrait-3).

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
suffisent.
Le profil GLM existe déjà. Son authentification n'est ni écrite dans un nouveau
fichier ni modifiée. La recette privée réutilise l'accès existant en mémoire.
La mission n'hérite pas implicitement de tout l'historique parent. Aucun calendrier,
recrutement interprojet automatique ou nouveau panneau n'est ajouté.
