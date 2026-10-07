# SPEC140 — Bulles Bridget compactes dans T3

Date : 2026-10-07. Statut : Implemented, non déployé.
Autorisation : maquette validée puis « my-specify-all go ». Le titre réel du fil est inclus par autorisation complémentaire.

## Besoin

L'utilisateur doit comprendre qui lui est présenté et pourquoi un message arrive, sans lire les consignes techniques destinées à l'agent. L'agent conserve toutes ses informations. Aucun contrôle de mission n'est affaibli.

## Scénarios utilisateur

### US1 — Lire une bulle courte (P1)

Les messages Bridget ont une bulle à droite, un logo Bridget et un fond bleu-gris discret. Une seule ligne est visible par défaut. Un clic, Entrée ou Espace ouvre le contenu lisible. Le chevron reflète l'état. Le contenu technique intégral reste disponible dans « Détails techniques ».

Acceptation : les cinq familles sont couvertes : message direct, sollicitation de fil, observation, lot de messages directs et lot d'observations. Les messages ordinaires restent inchangés.

### US2 — Comprendre sans information inventée (P1)

Les noms sont ceux de l'enveloppe reçue. Un message direct affiche « Message » ou « Réponse attendue » selon son indicateur. Une sollicitation affiche « Nouveautés ». Une observation affiche « Notification ». Un lot affiche son nombre d'éléments. Le contenu libre n'est jamais analysé pour inventer une demande, un blocage ou une décision.

Acceptation : les identifiants techniques ne prennent pas la place principale. Un nom absent ne devient pas un faux nom. Aucun destinataire n'est inventé.

### US3 — Voir le vrai titre du fil quand il est disponible (P1)

Bridget fournit le titre du fil seulement lors d'une remise à un membre autorisé. T3 l'affiche sans consulter d'autres données. Une ancienne sollicitation sans titre affiche « Fil partagé ». Son identifiant reste accessible dans les détails.

Acceptation : absence de titre, ancien format et destinataire non membre n'exposent aucun titre indu. Un titre hostile ne casse ni l'enveloppe ni le rendu.

### US4 — Garder le comportement existant (P1)

Le texte fournisseur, la copie, les pièces jointes, les actions de message et les verdicts sont conservés. Les anciennes conversations bénéficient du rendu sans migration. Les agents reçoivent le texte complet, y compris les nouvelles métadonnées de présentation. Les relances, alertes et échanges ne changent pas de destinataire.

Acceptation : changer de fil ou recycler une ligne de la liste ne réutilise pas l'état ouvert d'un autre message. L'ouverture conserve un défilement stable.

## Exigences fonctionnelles

- FR14001 : une bulle compacte, alignée à droite et repliée sur une ligne par défaut, pour les cinq familles d'enveloppes complètes.
- FR14002 : logo réel local, fond discret, thèmes clair et sombre, largeur de 320 pixels utilisable.
- FR14003 : commande accessible au clavier, état exposé aux technologies d'assistance et focus visible.
- FR14004 : contenu lisible puis détails bruts intégraux ; aucune perte du texte original ni changement du mécanisme de copie.
- FR14005 : libellés tirés des faits présents dans l'enveloppe, jamais d'une interprétation du corps. Les types de publication action/blocker/decision/history ne sont pas déduits d'une alerte.
- FR14006 : titre du fil facultatif, résolu à la remise pour le destinataire autorisé ; ancien format ou titre absent implique « Fil partagé ».
- FR14007 : le titre est échappé, normalisé et borné ; le corps, le canon idempotent, le journal persistant et le contrat historique de sollicitation restent inchangés.
- FR14008 : reconnaissance ancrée et bornée de l'en-tête ; enveloppe incomplète ou message ordinaire conserve son rendu historique. Cette reconnaissance n'atteste pas une identité.
- FR14009 : état local de présentation isolé par fil et message ; ouverture compatible avec la liste virtualisée et son ancrage.
- FR14010 : aucun nouveau service, API, stockage, migration, recherche de nom dans le navigateur à l'exécution ou dépendance externe. La résolution du titre par le daemon à la remise est explicitement permise par FR14006.
- FR14011 : essais avec données synthétiques et transports isolés ; aucun redémarrage de production ni écriture dans les conversations actives.

## Critères de succès

Les tests couvrent formats positifs et négatifs, libellés, titre compatible et hostile, rejet d'exposition hors membre, conservation du canon, copie brute, clavier, longs textes, largeur étroite et isolation d'état. Les contrôles ciblés de format, lint, types et build web passent. La recette du composant réel est consignée sans présenter une maquette comme une preuve de production.

## Dépendances et limites

SPEC134 : noms d'affichage. SPEC136 : publications typées et historique silencieux. SPEC138 : priorité au projet et exceptions volontaires. SPEC139 : reconnaissance des cinq familles et styles discrets. Cette session ne modifie ni leur autorité ni leurs missions. Compilation de paquet, installation, commit et déploiement restent des actions distinctes, non automatiques.
