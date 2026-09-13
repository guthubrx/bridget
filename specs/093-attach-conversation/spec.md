# 093 — Une conversation lisible dans attach

Date : 2026-09-06. Statut : Implémenté et installé. Mandat utilisateur déjà approuvé,
repriorisé sur capture réelle du 6 septembre à 17 h 45. Dépendances : 089, 090,
091 et 092. Ce lot ne prétend pas terminer l'édition avancée, MCP ou SSH de la
suite approuvée : ils restent explicitement à réaliser ensuite.

## Contexte et valeur

L'humain veut lire les réponses, pas le journal technique qui les transporte.
Sa capture montre le Markdown brut, un préfixe UUID qui indente tout le corps,
un raisonnement absent affiché comme contenu et une fin de tour banale. Les
couleurs de saisie/statut et l'historique 092 existent déjà et restent acquis.

## Scénarios utilisateur et acceptation

### US1 — Lire la réponse (P1)

Une réponse avec titre, deux puces, emphase, citation et bloc de code arrive par
fragments : son texte apparaît en flux, avec une hiérarchie visuelle, sans fences
Markdown littérales dans un bloc fermé, sans duplication ni décalage proportionnel
à l'UUID. Le corps reprend la largeur utile sous un en-tête compact séparé.

### US2 — Voir les faits utiles (P1)

Un tour ordinaire ne montre ni raisonnement ni ligne technique de fin réussie.
Une commande, un outil, une permission, un refus, une erreur, une lacune ou une
source indisponible restent visibles. Aucun événement n'est supprimé du journal.

### US3 — Redimensionner sans perdre la saisie (P1)

Le terminal passe de 100 à 45 puis 100 colonnes pendant une réponse et après sa
fin : le bloc encore géré par attach se replie selon la nouvelle largeur, la
saisie et le statut restent présents, aucune lettre saisie n'est perdue. Les
nouvelles réponses prennent la nouvelle largeur. Le vieux scrollback déjà remis
au terminal n'est pas un historique rééditable par Bridget ; sa réorganisation
dépend du terminal et cette limite doit être annoncée.

## Exigences fonctionnelles

- FR-001 : rendu Markdown des titres, listes, emphase, citations, code inline et
  blocs de code ; texte et indentation du code préservés. Les blocs code sont
  distingués visuellement, sans promettre un analyseur lexical de tous langages.
- FR-002 : traitement des fragments sans perte, duplication ou attente de fin
  de tour pour afficher le texte. Une syntaxe incomplète reste lisible puis
  converge vers le même résultat que la réponse reçue d'un coup.
- FR-003 : en-tête et corps séparés ; nom attesté de l'agent sélectionné si
  disponible, sinon UUID compact explicitement cosmétique. UUID d'adressage et
  données sources inchangés. Aucun lookup réseau par événement.
- FR-004 : masquer le raisonnement et les terminaux ordinaires réussis dans la
  vue conversation. Un terminal d'échec/interruption reste visible ; inconnu ne
  vaut jamais réussi. Les événements techniques restent consultables dans la
  sortie diagnostique non-TTY existante, sans nouveau protocole ni stockage.
- FR-005 : conserver les commandes/outils utiles, permissions, refus, erreurs,
  Gap, End et indisponibilités. Ne jamais cacher une demande de décision humaine.
- FR-006 : couleurs purement locales ; contenu externe incapable d'injecter
  contrôles terminal, liens actifs, exécution, HTML ou chargement distant.
  NO_COLOR/TERM=dumb et sortie redirigée restent utilisables sans styles.
- FR-007 : adapter bloc courant et dernière réponse encore active, saisie et
  statut au resize même sans nouvelle frappe/message ; largeur calculée en
  cellules visibles. Repli aux espaces lorsque possible ; rupture d'un mot
  seulement quand il dépasse la largeur utile. Pas de fuite ANSI, coupure
  d'UTF-8 ou caractère perdu. Les espaces et indentations du code restent conservés.
- FR-008 : préserver bornes mémoire, corrélation, snapshot/live, reprise, ordre
  d'abonnement, historique 092, Ctrl-C/D, /model et état des sessions fournisseurs.
- FR-009 : livrer le binaire testé et indiquer précisément le seul processus à
  rouvrir ; aucune obligation de redémarrer daemon ou agents pour changer attach.

## Critères mesurables

- SC-001 : fixture issue de l'exemple utilisateur : titre/listes/code lisibles,
  zéro fence fermée affichée, corps non indenté par le préfixe UUID.
- SC-002 : fixture mixed : aucun raisonnement ni completed banal ; chaque
  erreur/refus/permission/lacune exigé est encore présent.
- SC-003 : même texte en un fragment et en fragments arbitraires, même contenu
  final visible, sans duplication ; séquences ANSI externes neutralisées.
- SC-004 : pseudo-TTY 100→45→100 : bloc actif, dernière réponse gérée et brouillon
  préservés, puis nouveau message à la bonne largeur ; test sans frappe au resize.
- SC-005 : styles absents quand désactivés ; couleur n'affecte pas le calcul de
  largeur ; footer toujours sous la saisie, contrôles restaurés à la sortie.
- SC-006 : tests attach/092 verts, clippy/fmt et consolidation workspace ; preuve
  du binaire installé distincte d'une simple compilation. Observation humaine
  finale explicitement distinguée des injections pseudo-TTY.

## Hypothèses et hors périmètre

Terminal ANSI moderne pour les couleurs ; ancien terminal = repli sans styles.
On ne reconstruit pas le scrollback terminal ni une GUI. Pas de nouvelle identité,
sonde fournisseur, second canal, persistance de vue, écran alternatif ou framework
TUI. Flèches d'édition par mot et fonctions MCP/SSH restent lots suivants.
