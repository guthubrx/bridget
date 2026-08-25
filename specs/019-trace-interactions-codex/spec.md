# Spec 019 — Trace durable des interactions bloquantes Codex

**Branche** : `session-19-trace-interactions-codex` | **Créée** : 2026-08-25
**Status** : Complète
**Tests** : 5/5 (100 %)
**Dependencies** : SPEC-007, SPEC-008

## Contexte

Deux tours Codex (`relec8` et `jury1`) ont accepté une délégation puis sont
restés silencieux jusqu'à leur arrêt. Leur journal ne contient que les
événements Bridget et leur sortie d'erreur est vide : aucune pièce
contemporaine ne permet de connaître l'interaction fournisseur qui précédait
la panne. Le libellé « tour terminé : inconnu » est seulement le rendu d'une
clôture au payload vide et n'est pas une cause établie.

Le transport ne sait actuellement ni répondre aux demandes d'approbation du
pilote, ni fixer une politique d'approbation. Ce risque est établi par le code,
mais aucune trace ne prouve qu'une telle demande a causé les deux pannes. Cette
session rend le prochain cas falsifiable sans appliquer de correctif causal.

## Propriété

Toute interaction fournisseur susceptible de suspendre un tour doit produire,
avant l'attente, un événement durable corrélé au message Bridget et au tour
fournisseur, contenant au minimum sa méthode et son identifiant, sans donnée
sensible. Toute fin par échéance ou fermeture du transport doit permettre de
déterminer si une telle interaction restait pendante.

## Scénarios utilisateur et tests

### US-1901 — Expliquer un prochain tour silencieux (P1)

Un opérateur consulte l'historique d'un agent après une échéance ou une
fermeture du transport. Il voit l'éventuelle requête fournisseur restée en
attente, son identifiant et son tour, puis l'erreur corrélée au même message.

**Test indépendant** : un faux pilote accepte un tour, émet une requête
serveur→client puis reste silencieux. Le journal porte la requête avant
l'erreur finale et la vue la rend lisiblement.

1. **Étant donné** une requête fournisseur avec méthode, identifiant et tour,
   **quand** elle arrive pendant un message Bridget actif, **alors** un fait
   durable est écrit immédiatement avec ces seuls identifiants utiles.
2. **Étant donné** que le pilote ne répond plus après cette requête, **quand**
   le tour expire ou que le transport ferme, **alors** l'erreur désigne la
   dernière requête encore pendante et reste corrélée au message.
3. **Étant donné** que la requête contient un corps de commande ou du texte
   utilisateur, **quand** elle est journalisée, **alors** aucun de ces contenus
   n'apparaît dans le journal.

## Exigences

- **FR-1901 — Trace avant attente** : toute requête fournisseur reçue pendant
  un tour actif produit un événement durable avant que le système poursuive
  son attente.
- **FR-1902 — Corrélation minimale** : l'événement conserve le fournisseur, la
  méthode, l'identifiant de requête, l'identifiant de tour et l'identifiant du
  message Bridget quand ils sont attestés ; une absence reste absente.
- **FR-1903 — Expurgation par liste blanche** : aucun paramètre, prompt,
  commande, chemin ou résultat fournisseur n'est persisté dans cet événement.
- **FR-1904 — Sort terminal explicable** : une échéance ou fermeture survenue
  avec une requête pendante porte la même corrélation dans son fait d'erreur.
- **FR-1905 — Lecture humaine** : la vue d'historique distingue une interaction
  pendante d'une notification ordinaire et affiche méthode, requête et tour.
- **FR-1906 — Compatibilité additive** : les journaux antérieurs et les autres
  transports gardent leur sens ; un lecteur ancien peut ignorer le nouvel
  événement.

## Hors périmètre

- Répondre à une requête d'approbation ou choisir une décision à la place de
  l'utilisateur.
- Fixer ou modifier une politique d'approbation du pilote.
- Attribuer rétroactivement les pannes de `relec8` et `jury1` à une requête non
  observée.
- Assimiler ce mécanisme au prompt de confiance d'un répertoire inconnu.
- Modifier le rendu générique des clôtures « inconnu ».

## Critères mesurables

- **SC-1901** : le banc déterministe écrit, dans cet ordre, le fait de requête
  pendante puis le fait d'erreur, avec le même message et le même tour.
- **SC-1902** : une sentinelle sensible injectée dans les paramètres du faux
  pilote a zéro occurrence dans le journal durable.
- **SC-1903** : la vue rend la requête pendante et l'erreur sans JSON brut.
- **SC-1904** : les suites de compilation et de tests existantes conservent
  leurs résultats, hors rouges préexistants explicitement imputés.

## Hypothèses

- Un agent géré ne traite qu'un message actif à la fois ; cette unicité permet
  de corréler une requête reçue au message actif sans inventer de concurrence.
- Une trame serveur→client qui porte une méthode et un identifiant est une
  requête en attente de réponse ; une notification sans identifiant ne l'est
  pas.

## Résultat

Le journal porte désormais un fait `provider_request` construit par liste
blanche. Lorsqu'un tour finit par échéance ou fermeture de stdout, son fait
`error` reprend la dernière requête encore pendante. `attach` rend les deux
faits sans JSON brut. Aucun choix d'approbation ni réglage du pilote n'a été
ajouté : la session rend une cause future observable, elle ne conclut rien sur
les pannes historiques.
