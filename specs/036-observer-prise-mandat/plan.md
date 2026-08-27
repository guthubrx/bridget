# Plan 036 — Observer la prise d'un mandat

## Pourquoi ce dessin

Le défaut se situe après la remise et avant la consommation. Modifier le
transport mélangerait réparation et instrument ; le lot ajoute donc une sonde
en lecture seule à `bridget-idle` et conserve la remise inchangée.

Le fait natif diffère selon le client. Une petite couche de normalisation locale
produit un état commun sans inventer un événement universel dans les journaux
Bridget.

## Algorithme

1. Lire la dernière injection ouverte depuis le journal Bridget existant.
2. Refuser l'observation si l'agent n'est pas sur l'hôte local.
3. Pour un agent tmux local, résoudre le contexte du pane sans écrire dedans.
4. Pour Codex, retrouver la trace active dans les descripteurs ouverts du
   processus et de ses descendants ; pour Claude, sélectionner la session du
   projet local correspondant au répertoire du pane.
5. Fermer la découverte si la première ligne d'un descripteur Codex est
   partielle, puis lire la trace append-only intégralement. Une ligne partielle
   ou invalide invalide la source ; une source vide n'est jamais saine.
6. Corréler les faits d'entrée acceptée postérieurs à l'injection — message
   utilisateur Codex, message utilisateur Claude ou `queue_remove` — par
   égalité avec l'identifiant de l'enveloppe canonique. Une sous-chaîne, un
   préfixe ou une mention dans le corps ne constituent jamais une corrélation.
   Reconstruire ensuite l'état actif au moment de l'injection depuis les bornes
   natives.
7. Calculer `PRISE_ACCEPTEE`, `PRISE_EN_ATTENTE`, `MANDAT_NON_SOUMIS`,
   `REMISE_PENDANT_TOUR_ACTIF` ou `PRISE_INOBSERVABLE` pour chaque injection
   interactive ouverte vue par le daemon, sans dépendre de la copie Maicie
   locale, puis intégrer les états opérateur à la partition.
8. Rendre source, cardinal lu, cardinal corrélé et âge, sans rendre le contenu.

## Découverte et déterminisme du harnais

La production utilise les sources locales réelles. Le harnais injecte une table
explicite agent → trace afin de séparer le parseur de la découverte des
processus. La découverte Codex est également éprouvée sur un faux arbre
`/proc` ; aucun processus réel n'est requis.

Le spécimen rc5 est réduit aux lignes nécessaires, avec les horodatages réels
de la spec. Deux contrôles bloqués conservent une injection et une trace non
vide sans acceptation : l'un porte `[Pasted Content]`, l'autre du texte normal.
Le contrôle sain ajoute la seule entrée corrélée. Deux contrôles adverses
portent un identifiant sur-ensemble et une simple mention dans le corps. Côté
Claude, `enqueue` et `remove` sont exercés séparément. Aucun rendu de pane
n'entre dans le calcul.

## Seuil

Le seuil vaut 60 secondes et ne s'applique qu'à la frontière injection →
acceptation client. Il est supérieur au maximum Claude mesuré de 51,083 s,
onze fois supérieur au maximum des démarrages Codex rapides de 5,421 s et
douze fois inférieur au spécimen bloqué de 720,857 s.

Ce seuil est réservé aux injections effectuées quand la trace native atteste
un client au repos. Une remise pendant un tour actif est nommée séparément et
ne traverse jamais la branche temporelle.

## États et partition

`MANDAT_NON_SOUMIS` et `PRISE_INOBSERVABLE` deviennent deux catégories de la
partition daemon, même si la greffe Maicie locale est vide. `PRISE_ACCEPTEE`
rejoint `OCCUPES` lorsqu'une mission locale existe ; sinon elle reste un fait
d'entrée et n'invente pas une mission. `PRISE_EN_ATTENTE` reste visible comme
indétermination transitoire jusqu'à l'échéance.
`REMISE_PENDANT_TOUR_ACTIF` constitue une catégorie explicite : ni blocage
idle, ni disparition silencieuse du steering.

L'oracle de partition inclut les deux nouvelles catégories. Toute observation
absente est rendue inobservable ; aucune valeur par défaut ne signifie sain.

## Complexité et confidentialité

La découverte Codex parcourt seulement l'arbre de processus du pane et ses
descripteurs. La découverte Claude se borne aux sessions du répertoire projet
correspondant. La lecture est O(n), n étant le nombre de lignes de la trace
active. Aucune dépendance nouvelle, aucun cache durable et aucun corps de
message dans la sortie.

## Validation

- inventaire des contrôles du harnais avant chaque exécution ;
- `python3 -m py_compile scripts/bridget-idle.py` avec cache hors dépôt ;
- `bash -n scripts/test-bridget-idle.sh` ;
- harnais complet `scripts/test-bridget-idle.sh`, base et tête ;
- mutants opposés sur le cas bloqué et le cas accepté, puis mutants ciblés
  rétablissant la sous-chaîne et l'oubli d'une première ligne partielle ;
- tir réel local en lecture seule sur Cartae, avec cardinal rendu ;
- `git diff --check` et relecture du diff complet.

## Limite architecturale

La sonde est locale. Le relais inter-hôtes est volontairement séparé : si
observation et fédération échouaient dans le même lot, le contrôle positif ne
pourrait pas attribuer la panne à l'une des deux couches.
