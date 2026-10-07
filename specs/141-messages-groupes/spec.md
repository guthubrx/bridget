# SPEC141 — Messages Bridget groupés

Date : 2026-10-07. Statut : Implemented, non activé.
Autorisation : session 141 validée explicitement par « oui ».
Tests: 294/294 (100%) — suites ciblées sur le code gelé ; convergence et audit final validés. Aucune installation de production.

## Besoin

L'utilisateur doit voir les noms des expéditeurs et ouvrir chaque message séparément. Un lot fermé doit occuper une ligne. La maquette validée sert de direction visuelle. Les titres de sujet illustratifs sont remplacés par le début réel du message. L'utilisateur a demandé de ne pas afficher « Sources » ni les détails techniques dans cette présentation.

## Scénarios utilisateur et acceptation

### US14101 — Ouvrir un lot compact (P1)

Étant donné un lot valide de messages directs, le groupe est fermé sur une ligne au repos. Quand l'utilisateur ouvre le groupe, il voit les messages dans l'ordre reçu. Chaque section montre clairement le nom fourni et un aperçu littéral du début du corps. Aucun sujet, classement, intention ou verdict n'est déduit du texte.

Acceptation : un lot de trois messages contient trois sections. Deux messages du même expéditeur restent deux sections distinctes. Un nom long reste disponible en entier à l'ouverture. Si l'expéditeur fourni est seulement un UUID, cet UUID reste visible ; aucun nom « Bridget » ne le remplace. Les thèmes clair et sombre et une largeur de 320 pixels restent lisibles.

### US14102 — Lire chaque message séparément (P1)

À la première ouverture du groupe, la première section est ouverte et les autres sont fermées. Quand l'utilisateur ouvre une section, le corps complet devient lisible. Ouvrir une autre section ne ferme pas la première. Le clic, Entrée et Espace permettent le même parcours. Le focus reste visible et l'état ouvert est annoncé.

Acceptation : fermer puis ouvrir le groupe conserve les choix des sections et les textes. Une ouverture ne déplace pas la vue hors du message. Un changement de fil ou de message ne reprend pas l'état d'un autre groupe.

### US14103 — Conserver l'information et les actions (P1)

Étant donné un lot affiché, l'utilisateur conserve la copie du message complet, ses pièces jointes et ses actions. La copie contient le texte reçu à l'identique. Aucun panneau « Sources » ou « Détails techniques » n'apparaît pour le lot.

Acceptation : un corps Markdown, un très long texte et plusieurs pièces jointes restent accessibles. La copie conserve aussi les liens de référence présents dans le texte, même sans contexte structuré. Aucun contenu de message n'est exécuté comme du code. Les messages ordinaires, les notifications et les autres enveloppes conservent leur présentation et leur copie SPEC140.

### US14104 — Lire un lot incomplet ou ambigu (P1)

Étant donné un lot dont les compteurs ou les séparateurs ne permettent pas un découpage certain, l'utilisateur voit le corps lisible entier. Aucun message n'est perdu ni attribué au mauvais expéditeur. Le même repli s'applique à un en-tête trop long. La copie complète reste intacte. Aucun panneau source n'est ajouté.

Acceptation : nombre annoncé différent du nombre réel, indice absent ou répété, séparateur similaire dans un corps, en-tête tronqué et nom trop long n'entraînent aucune perte. Une consigne finale inconnue ou modifiée reste visible. Une consigne de transport peut être masquée uniquement si elle correspond exactement au suffixe final connu.

## Exigences fonctionnelles

- FR14101 : seuls les lots de messages directs sans réponse attendue reçoivent cette présentation.
- FR14102 : le groupe est fermé sur une ligne par défaut et garde le logo ainsi que les couleurs sobres existants.
- FR14103 : chaque message du lot dispose de sa propre section repliable, dans l'ordre reçu.
- FR14104 : le nom affiché vient de l'enveloppe. Un nom absent ne devient pas un nom inventé. Les identifiants techniques ne remplacent pas un nom disponible. Si seul un UUID est fourni, cet UUID reste visible au lieu d'un nom « Bridget » inventé.
- FR14105 : l'aperçu reproduit au plus les 120 premiers caractères du corps. La troncature visuelle n'ajoute aucun sujet, catégorie, intention ou verdict.
- FR14106 : le corps d'un message ouvert reste complet. Les espaces, retours de ligne et contenus utiles ne sont pas réécrits.
- FR14107 : aucun panneau « Sources », « Détails techniques » ou vue brute secondaire n'est affiché pour un lot direct, y compris en repli sûr.
- FR14108 : la copie complète d'un lot direct contient le texte original reçu à l'identique, y compris ses liens de référence. Les pièces jointes et actions conservent leur comportement actuel. La copie des autres familles reste inchangée.
- FR14109 : le découpage n'est accepté que si l'en-tête, le total, les indices, les expéditeurs et les limites de chaque corps sont cohérents. Toute ambiguïté impose le corps entier.
- FR14110 : seule une consigne finale de transport exactement reconnue peut être retirée de la lecture. La source copiée reste inchangée.
- FR14111 : les commandes sont utilisables au clavier. Elles ont un nom accessible, un état ouvert annoncé et un focus visible.
- FR14112 : l'état de présentation reste isolé par fil et message. L'ouverture respecte l'ancrage de la conversation.
- FR14113 : les messages ordinaires, notifications, lots de notifications, messages directs unitaires et sollicitations de fil conservent SPEC140.
- FR14114 : aucun changement de transport, mission, boucle d'agent, stockage, service, interface externe ou migration n'est nécessaire.
- FR14115 : les essais utilisent des données synthétiques et une instance isolée. Ils ne modifient pas les conversations actives.

## Critères de succès

- SC14101 : un lot valide de trois messages produit exactement trois sections dans l'ordre reçu et une seule ligne quand le groupe est fermé.
- SC14102 : tous les corps et la copie complète correspondent aux textes de référence. Aucun titre de sujet inventé n'apparaît.
- SC14103 : chaque cas ambigu de US14104 conserve tout le corps disponible et une copie identique. Aucun panneau source n'apparaît dans ces cas.
- SC14104 : le parcours ouverture, lecture et fermeture fonctionne au clic, avec Entrée et avec Espace. Les états annoncés correspondent aux états visibles.
- SC14105 : à 320 pixels et dans les deux thèmes, le groupe reste dans la largeur disponible. Les corps longs restent lisibles.
- SC14106 : les cas de référence SPEC140 hors lots directs gardent leurs libellés, détails, textes et actions.
- SC14107 : la recette isolée et les contrôles ciblés passent. Chaque résultat est consigné avec sa portée réelle.

## Entités et hypothèses

Un lot possède un texte original et un ordre de messages. Chaque message possède un expéditeur et un corps. Les noms sont des données de présentation reçues. Ils ne constituent pas une preuve d'identité. Les aperçus sont des extraits du corps et non des résumés générés.

La maquette utilise trois messages pour illustrer le rendu. Le nombre réel reste celui de chaque lot valide. La première section est ouverte à la première ouverture du groupe ; les autres sont repliées. Les choix sont conservés quand le même groupe est refermé puis ouvert. Leur état reste temporaire.

## Dépendances et limites

SPEC114 fournit le format des lots. SPEC134 fournit les noms affichés. SPEC139 et SPEC140 fournissent la reconnaissance, la carte compacte, le rendu lisible, la copie et l'ancrage existants. Cette session modifie leur présentation pour les lots directs seulement. Elle ne change pas leur autorité ni les données remises aux agents.

Le 2026-10-07, l'utilisateur a autorisé commit, fusion, push et installation de cette session. Il a interdit de relancer T3. L'installation et l'activation seront consignées séparément sur preuves. La nouvelle présentation n'est pas déclarée active dans l'application en cours. Le nettoyage d'autres travaux reste hors périmètre.
