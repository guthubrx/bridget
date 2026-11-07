# Session 241 — Confier un travail sans administrer Bridget

Date : 7 novembre 2026. Statut : In Progress.
Branche : session-241-fiabilite-usage.

## Demande initiale et autorisation

François constate des refus répétés, des plafonds arbitraires, des reprises
manuelles et des demandes d'autorisation déjà satisfaites. Il veut que Bridget
travaille pour lui. Il a invoqué my-specify-all puis demandé l'exécution autonome.
Le chantier couvre spécification, réalisation, vérification, livraison Git,
activation des composants autorisés et nettoyage. Ne pas redémarrer T3.
Ne pas rejouer, remplacer ou modifier les missions métier existantes pour tester.
Préserver notamment 7024, 77ba, 702d, les revues métier et les nouvelles tâches
Revue 218369de, 5564cdf5 et bb7d6237.

## Demande reformulée

Rendre les demandes courantes fiables sans intervention du mainteneur :
confier un travail une fois, retrouver son état après une coupure et obtenir
son résultat sans doublon. Réparer les défauts de raccordement et de durée de
vie de l'autorisation ; supprimer les contraintes sans justification réelle ;
choisir une machine qui dispose des outils nécessaires. Vérifier ces parcours
ensemble dans les conditions des incidents récents, sans ajouter de droits,
installer des outils ou engager de nouvelles dépenses implicitement.

Apport du cadrage : le résultat attendu porte sur le parcours complet et la
charge humaine évitée. Un composant installé ou un test local vert ne suffit
pas à déclarer la demande aboutie. Une admission durable engage Bridget sur
le suivi de cette demande. Une reprise idempotente retrouve la même demande.

## Scénarios utilisateurs et vérification

### US1 — Confier et retrouver une demande (P1)

Un appelant donne une mission autorisée. Une réponse perdue ou un raccordement
transitoire ne lui impose pas une nouvelle clé, un changement de droits ou
une intervention humaine. L'état retrouvé appartient à la même demande.

Acceptation : coupure avant envoi, réponse perdue après enregistrement,
raccordement lent puis disponible, appels concurrents de même clé et service
réellement indisponible. Aucun doublon ; une indisponibilité persistante rend
un état précis et conserve la possibilité de retrouver la demande.

### US2 — Poursuivre une demande acceptée (P1)

Le tour de conversation qui a créé une demande peut se terminer pendant sa
préparation. La demande déjà autorisée reste prise en charge. Un changement
réel d'autorité et une annulation restent applicables au travail concerné.

Acceptation : fin du tour sans changement de droits, nouveau tour de mêmes
droits, autorité de mission révoquée, propriétaire différent, annulation et redémarrage
du coordinateur. Aucune ancienne tâche en échec n'est relancée pour tester.

### US3 — Utiliser des dossiers et capacités ordinaires (P1)

L'appelant n'a pas à connaître les registres internes ni à augmenter lui-même
un plafond pour un travail réalisable avec les ressources et accès disponibles.
Une préparation hors Git déjà autorisée dispose d'un parcours explicite.

Acceptation : gros dépôt, inventaire volumineux, paramètres de taille utiles,
fichiers hors Git, correspondance déjà connue et réellement nouvelle portée.
Les contrôles d'intégrité restent internes ; un désaccord réel identifie la
version ou le périmètre à résoudre. Aucune autorisation nouvelle n'est inventée.

### US4 — Choisir une machine capable de faire le travail (P1)

Une mission qui exige Docker ou un autre outil de compilation identifié
n'est pas confiée à une cible dont cette dépendance manque sans l'annoncer.
La ressource disponible ne suffit pas à prouver la capacité fonctionnelle.

Acceptation : Docker requis et absent, présent mais inutilisable, présent
et utilisable ; outil inconnu ; compilation qui ne requiert pas Docker ;
machine imposée. Aucun outil installé et aucun modèle ou hôte imposé remplacé
silencieusement. Une contrainte libre permet de sélectionner une autre cible
qualifiée avant admission.

### US5 — Continuer à utiliser le service pendant les évolutions (P1)

Une réponse ancienne reste limitée à sa demande. Les fonctions de consultation,
récupération et annulation restent vérifiables en situation dégradée. Les
clients déjà ouverts reçoivent un traitement compatible ou un diagnostic ciblé.

Acceptation : réponse tardive pendant une autre mission, ancienne façade,
mise à jour des composants, lecture et annulation pendant une autre préparation.
T3 reste ouvert. Si un composant ne peut être rechargé dans cette exclusion,
son état est explicitement déclaré incomplet, sans prétendre une activation.

## Exigences fonctionnelles

- FR-001 : identifier la phase et la cause mesurée du délai JSONL récurrent ;
  conserver les preuves et corriger la cause confirmée, pas seulement le message.
- FR-002 : reprendre automatiquement les raccordements transitoires avant
  envoi dans le budget de l'opération, sans rejeu de mutation inconnue.
- FR-003 : après enregistrement, retrouver le reçu sous la même identité et
  la même clé ; toute incertitude conserve ses identifiants et son contenu.
- FR-004 : une admission autorisée porte une autorité liée à cette mission et
  à sa durée ; la fin ordinaire du tour n'est pas une révocation de mission.
- FR-005 : maintenir le propriétaire, les permissions effectives, les sources
  autorisées, les annulations et les révocations réelles aux étapes critiques.
- FR-006 : résoudre les correspondances et variantes de source déjà autorisées
  sans faire éditer les registres par l'appelant ; les fichiers hors Git restent
  utilisables dans leur périmètre attesté sans faux dépôt ou déplacement destructif.
- FR-007 : inventorier les plafonds de nombre, volume et durée sur les parcours
  exposés ; justifier chacun par un coût ou un invariant réel, supprimer ou
  remplacer les contraintes arbitraires par traitement progressif ou capacité réelle.
- FR-008 : le dépassement d'une page ou d'une capacité instantanée ne rend pas
  inaccessible l'état durable de toutes les missions ; la portée du refus reste locale.
- FR-009 : qualifier les prérequis d'outillage identifiés avant le choix final
  d'une cible de compilation ; distinguer présence, utilisabilité et inconnu.
- FR-010 : demander une action humaine uniquement pour une décision nouvelle,
  avec la décision exacte ; ne pas redemander une autorisation déjà appliquée.
- FR-011 : une réponse tardive ne crée pas de droit, ne réactive pas une tâche
  et ne ferme pas le canal de travaux indépendants lorsqu'elle est reconnue.
- FR-012 : garder les scénarios réels comme preuves reproductibles de bout en
  bout ; mesurer premier appel, récupération, délai et interventions humaines.
- FR-013 : livrer les changements intégrés et actifs sur les cibles prévues,
  avec retour compatible ; préserver T3 ouvert et les travaux étrangers.

## Critères de réussite

- SC-001 : zéro travail dupliqué dans les scénarios de perte de réponse et de
  concurrence de même clé ; les états retrouvés portent la même demande.
- SC-002 : les scénarios de fin de tour et de tour suivant à autorité identique
  aboutissent sans intervention du mainteneur ; les révocations natives de
  l'autorité de mission et annulations restent effectives. Modifier le mode
  des futurs tours ne révoque pas implicitement une mission déjà admise.
- SC-003 : les scénarios de raccordement transitoire, état, récupération,
  préparation et résultat ne demandent aucune confirmation humaine technique.
- SC-004 : une dépendance connue manquante empêche son placement incompatible
  avant admission ; aucune installation ni nouvelle dépense automatique.
- SC-005 : chaque plafond exposé inventorié reçoit une décision et une preuve ;
  aucun plafond présenté comme corrigé ne reste inchangé sans justification.
- SC-006 : les mesures séparent réussite au premier appel, après récupération
  et échec ; les inconnues et limites de l'échantillon sont publiées.
- SC-007 : sources intégrées et poussées, composants autorisés chargés et
  vérifiés, T3 non redémarré ; toute exclusion d'activation reste explicite.

## Entités et dépendances

Réutiliser les demandes, reçus durables, missions, instantanés de permissions,
placements et capacités existants. Dépendances : 168/171 (admission/source),
177/206 (permissions), 221 (préparation/routage), 229 (projets préparés),
232/234 (capacités/inventaires), 235/237/240 (canaux/réponses), 239 (mandat).

## Hypothèses et exclusions

Les refus prévus sont examinés comme décisions produit ; leur conformité au
code ne suffit pas à les valider. Une indisponibilité réelle peut empêcher
l'aboutissement ; elle ne doit pas produire une fausse réussite ou un doublon.
Pas de nouveau moteur d'orchestration, framework, dépendance externe ou
nouvelle infrastructure de métriques sans besoin prouvé. Les mesures réutilisent
les faits et journaux autorisés, sans données métier ni secret dans les preuves.
Le rechargement complet de T3 reste exclu ; aucun travail métier existant n'est
utilisé comme banc destructif. Les besoins nouveaux hors de l'autorité attestée
ne sont pas résolus par un élargissement implicite des droits.
