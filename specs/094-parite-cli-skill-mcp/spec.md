# 094 — Cohérence binaire, skill et MCP

Date : 2026-09-07. Statut : In Progress. Tests : 0 exécuté pour 094.
Branche : session-094-parite-cli-skill-mcp.

## Intention

L'humain doit pouvoir demander à un agent Bridget de changer son nom ou sa
disponibilité sans se heurter à un outil absent ou à la socket interdite dans
son shell. La demande validée dépasse ces deux actions : chaque capacité du
binaire doit être documentée et avoir une décision d'accès explicite, sans
reconstruire l'administration du produit ni accorder le contrôle d'autrui.

## Scénarios utilisateur et tests

### US1 — Actions propres à l'agent (P1)

L'agent change son nom d'affichage, sa disponibilité et son domaine depuis son
accès de communication autorisé. Son UUID ne change jamais. Les conflits et
les erreurs sont des résultats explicites, pas des succès inventés.

Acceptation : deux agents enregistrés ; A se renomme, B conserve nom et UUID ;
un argument permettant de cibler B est refusé sans mutation. Le nom existant
est refusé sans révision supplémentaire. DND expire ou se lève explicitement ;
les réponses suivies valides restent livrables. Le domaine suit le contrat
existant de reconnexion, sans masquer un échec de sauvegarde.

### US2 — Observation sans shell (P1)

L'agent consulte les états déjà publiés par Bridget sans parser un affichage
humain, lire directement la base ou inventer une deuxième source de vérité.

Acceptation : résultat structuré du daemon ; une panne est indiquée comme
indisponible. Chaque observation administrative reçoit une décision documentée
d'exposition ou de maintien humain selon son autorité et ses effets réels.

### US3 — Mode d'emploi complet et honnête (P1)

L'humain et l'agent trouvent chaque commande du binaire dans un inventaire,
avec son usage, l'acteur autorisé, l'accès MCP ou son équivalent et le motif
d'une éventuelle absence. Les commandes internes sont séparées.

Acceptation : le tableau couvre l'aide ET les commandes du répartiteur ; les
alias reply/requests renvoient aux outils existants ; les artefacts sont
documentés ; les commandes terminal, hooks, migration et administration ne
sont pas présentées comme des outils automatiquement disponibles.

### US4 — Outils réellement utilisables (P1)

Les outils exposés fonctionnent dans les sessions fournisseurs autorisées,
même quand le shell ne peut pas accéder à la socket. L'installation et la
prise en compte par les sessions existantes sont expliquées honnêtement.

Acceptation : le catalogue exact et les permissions fournisseurs concordent ;
le test négatif sans autorisation fournisseur échoue ; aucun bypass global ni
redémarrage silencieux d'un agent vivant. Le binaire testé est celui installé.

## Exigences fonctionnelles

- FR-001 : inventaire exhaustif des commandes publiques et internes, chacune
  classée exposée, équivalente, humaine ou interne avec justification.
- FR-002 : exposer le renommage propre, en conservant UUID, instance et historique.
- FR-003 : exposer DND propre, durée valide et bornée, défaut existant 60 min,
  lever immédiatement ; refuser overflow, zéro, champs inconnus et cible libre.
- FR-004 : exposer le domaine propre avec cohérence reconnexion et erreurs de
  persistance visibles ; aucune nouvelle base ou autorité dupliquée.
- FR-005 : exposer les observations dont les projections existantes sont sans
  mutation et autorisées à l'appelant ; documenter précisément les exclusions.
- FR-006 : toute nouvelle mutation est liée à l'identité/instance attestées à
  l'appel, jamais à un UUID ou nom fourni comme cible par le modèle.
- FR-007 : appliquer un schéma fermé et des résultats corrélés ; une erreur
  technique ou un refus ne vaut jamais mutation confirmée.
- FR-008 : compléter les règles fournisseurs sans autorisation globale ; ne
  pas confondre outil annoncé, autorisé et recette réellement vérifiée.
- FR-009 : compléter skill canonique et documentation FR/EN ; maintenir un
  inventaire vérifiable pour détecter les futurs oublis.
- FR-010 : tests ciblés puis consolidation finale, revue adverse et preuve
  d'installation ; aucune session utilisateur interrompue automatiquement.
- FR-011 : exposer la déclaration propre modèle/effort sans lui attribuer une
  provenance observée ni sélectionner un nouveau modèle chez le fournisseur.

## Cas limites

Identité périmée, seconde instance, nom Unicode/contrôle/conflit, durée immense,
rejeu après renommage, DND et réponse liée, domaine reset/reconnexion, socket
absente, réponse inattendue, politique fournisseur never, ancien MCP encore
vivant après installation. Aucune donnée manquante remplacée par une valeur
plausible. Les scripts d'administration ayant des effets ne deviennent pas
des lectures simplement parce qu'ils s'appellent report.

## Entités et invariants

Agent UUID stable ; nom cosmétique révisé ; instance attestée ; disponibilité
temporaire ; intention de domaine existante ; projections daemon ; décision
d'accès CLI/skill/MCP. Pas de nouvelle table ni de registre de permissions.

## Hypothèses et limites explicites

Le GO valide le tableau précédent, dont la séparation humain/agent. Il ne
demande pas une délégation générale du pilotage : spawn/stop/relaunch et les
actions de référent restent documentés au CLI tant qu'aucune autorité
propriétaire dédiée n'est établie. /model sélectionne réellement un modèle ;
runtime déclare un fait : ces deux responsabilités ne sont jamais fusionnées.
Le présent lot ne répare pas le resize de 093 et ne remplace pas la TUI par MCP.

## Critères de succès

- SC-001 : 100 % des commandes inventoriées ont un accès et un motif explicites.
- SC-002 : renommage/DND/domaine propres prouvés sans mutation d'un autre agent.
- SC-003 : chaque outil neuf a preuve nominale, refus, indisponibilité et test
  d'autorisation pertinent ; les limites de recette sont nommées.
- SC-004 : aucun échec dans les validations exécutées pour la livraison ;
  aucun succès déclaré avant comparaison de l'empreinte du binaire installé.

## Dépendances

001 noms ; 005 domaines/DND ; 039 identité MCP ; 089 client communication ;
090 Codex interactif ; 091 politiques MCP. 092/093 conservées sans modification.
