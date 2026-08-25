# Spécification 025 — Carte de criticité auto-élue et régime de revue

**Branche** : `session-025-carte-criticite-regime`
**Créée** : 2026-08-25
**Statut** : prête pour implémentation par phases
**Base de conception** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
**Migration réservée** : v20
**Dépendances d’intégration** : session 021 (verdict lié au SHA, v17),
session 026 (vocabulaire fédéré du guichet, v18), session 027 (horodatages de
délégation, v19).
**Dépendance de la suite** : session 023 doit être absorbée avant le lot
distinct qui élira et déléguera les relecteurs.

## Intention

Le régime de revue d’un lot ne doit plus dépendre d’une liste de fichiers
choisie à la main. Maicie calcule une carte de criticité propre à chaque
projet depuis des faits déjà disponibles, propose un régime sur le diff exact
`base → tête`, puis exige que le référent retienne explicitement un régime.

Le référent conserve le droit de choisir un régime plus fort ou plus léger.
Il perd trois possibilités : ne pas décider, modifier silencieusement les
entrées de la carte, et produire un écart sans trace. Aucun motif libre n’est
demandé : l’écart entre le régime proposé et le régime retenu est la donnée.

La session forme un incrément cohérent : carte, soumission, proposition,
décision et comptage durable. Elle ne choisit pas les relecteurs. Ce second
incrément dépend d’un capteur de disponibilité fiable ; avant son absorption,
aucun chemin ne consulte `bridget-idle` ni ne transforme son résultat en
mandat.

## Décisions de conception

### D-2501 — Soumission liée à un dépôt déclaré, jamais à un chemin fourni par l’auteur

La configuration d’un projet de revue porte un identifiant stable, la racine
absolue de son dépôt et l’identité du référent. Elle ne porte aucun chemin
critique. Une soumission contient seulement l’identifiant de projet, une
référence Git complète, la tête et la base sous forme de SHA complets.
L’auteur est l’émetteur attesté par l’enveloppe du guichet ; il n’existe pas de
champ déclaratif séparé permettant de le remplacer.

Maicie vérifie localement que :

1. la racine configurée est un dépôt lisible ;
2. la référence complète existe et pointe exactement sur la tête soumise ;
3. la base et la tête sont des commits complets de quarante hexadécimaux
   minuscules ;
4. la base est ancêtre de la tête ;
5. le diff est calculé entre ces deux objets immuables, sans checkout et sans
   lire le worktree courant.

Une référence courte telle que `main` ou `fix/x` est refusée. Les formes
admises sont explicites, par exemple `refs/heads/fix/x` ou
`refs/remotes/origin/fix/x`. Cette contrainte ferme la même ambiguïté que les
noms de fichiers nus.

### D-2502 — Une carte composée de règles fixes et de zones élues

La carte contient toujours quatre règles de germe, identiques pour tout
projet :

1. schéma de persistance et migration ;
2. format de protocole ou de message ;
3. authentification, permissions, capacités et approbations ;
4. écriture durable dérivée d’une entrée externe.

Le quatrième germe est retenu. Une fuite de confidentialité récente a montré
qu’une écriture durable de trames externes est une porte sans retour au même
titre qu’une migration. La règle est volontairement conservatrice : elle
élargit le jury quand une unité de code contient à la fois une frontière
d’entrée externe et un puits durable. Le coût est un faux positif possible ;
le bénéfice est de ne pas rendre permanent un angle mort de confidentialité.
La carte n’enregistre jamais le contenu lu, seulement l’identifiant de règle
et le chemin résolu.

À ces règles s’ajoutent trois voies d’élection :

- **contenu du diff** : marqueurs intrinsèques tels que `SCHEMA_VERSION`,
  `ALTER TABLE` ou un chemin de migration ;
- **registre du dû** : une zone citée par un Bloquant ouvert, ou par au moins
  deux Majors ouverts distincts ;
- **contrats** : un chemin complet cité sous `specs/*/contracts/`.

Les zones élues sont monotones dans cette session : une nouvelle preuve peut
ajouter ou enrichir une zone, jamais la retirer. La proposition de sortie
après une période calme et son approbation locale selon ADR 011 restent un lot
distinct. Aucune commande d’édition ou de suppression de zone n’est exposée.

### D-2503 — Résoudre des chemins, pas lire des noms

La clé d’une zone est toujours un chemin complet relatif à la racine du dépôt,
normalisé avec `/`. Un index est construit depuis l’arbre du commit de tête.

Pour le registre historique :

- un chemin exact est accepté ;
- un suffixe ou nom nu n’est accepté que s’il désigne un seul fichier suivi ;
- un suffixe `:ligne` peut départager des homonymes si un seul candidat
  contient cette ligne au commit mesuré ;
- si plusieurs candidats subsistent, aucun n’est élu et l’ambiguïté est
  rendue avec le constat et les candidats complets.

Pour un contrat, seule une citation complète relative au dépôt constitue un
ancrage. Un `store.rs` ou `daemon.rs:1203` nu est une erreur de contrat
visible, jamais une élection implicite. Cette règle doit reproduire l’ancrage
actuel unique de `crates/bridget-transport/src/protocol.rs`. Une référence
croisée vers un autre artefact sous `specs/*/contracts/` n’élit pas ce second
contrat : les contrats sont les sources de l’élection, jamais leurs propres
zones, ce qui interdit une cascade récursive.

Une ambiguïté n’élit jamais tous les homonymes et ne durcit pas le lot par
défaut. Elle reste comptée comme dette de résolution afin que le faux positif
ne soit pas remplacé par un faux négatif invisible.

### D-2504 — Proposition mécanique, décision explicite, aucun motif libre

Les régimes forment un ordre fermé :

1. `revue_simple` : un relecteur distinct de l’auteur ;
2. `jury_1_plus_1` : une lentille constructive et une lentille à charge ;
3. `jury_2x2` : deux relecteurs par polarité pour les décisions fondatrices.

La carte propose `jury_1_plus_1` si le diff touche une zone élue ou déclenche
une règle de germe ; sinon elle propose `revue_simple`. Le référent doit
ensuite sélectionner un régime fermé. Une sélection identique est enregistrée
comme décision sans écart ; une sélection différente ouvre un écart. Aucun
champ `motif`, `raison`, `justification` ou texte libre n’existe dans le
contrat.

La soumission reste `en_attente_decision` tant que cette sélection n’existe
pas. Elle ne peut pas atteindre l’étape d’élection des relecteurs par silence,
échéance ou valeur par défaut.

### D-2505 — Le mécanisme ne choisit pas son propre régime

Le régime de toute modification du noyau de criticité est fixé à
`jury_2x2`, conformément à la doctrine utilisateur déjà inscrite pour les
mécanismes fondateurs. Cette valeur n’est ni calculée par la carte ni
remplaçable par une sélection du référent. Les chemins du noyau et le contrat
025 sont bornés dans le code ; leur suppression ou renommage apparaît elle-même
dans le diff et conserve donc la garde.

### D-2506 — Chaque refus et chaque décision laissent une ligne durable

Après ouverture réussie du store, toute issue métier du mécanisme produit une
ligne idempotente : soumission acceptée, décision, écart ou refus fermé. Les
erreurs techniques qui empêchent l’ouverture du store restent des erreurs de
démarrage explicites ; elles ne sont pas maquillées en reçus métier.

Les refus portent uniquement une énumération produite par le système et les
identifiants canoniques de la demande. Les octets du diff, le contenu des
fichiers, le texte complet des constats et les trames externes ne sont jamais
copiés dans ce journal. Les métriques exposent au minimum : soumissions,
décisions, écarts par sens, écarts ouverts/clos, refus par condition et
citations ambiguës.

### D-2507 — Un écart se ferme seulement sur un fait lié au même lot

Un durcissement se ferme lorsqu’un résultat de ronde atteste tous les verdicts
attendus sur la tête exacte :

- zéro constat de revue → `non_confirme` ;
- au moins un constat de revue → `confirme`.

Ce fait consomme le verdict lié au SHA de la session 021 et, lorsque plusieurs
relecteurs sont requis, la complétude de ronde portée par le futur lot
d’élection. Un verdict isolé ne prétend jamais représenter tout un jury.

Un allègement devient `refute` seulement si un constat ultérieur cite à la fois
la tête ou l’identifiant de soumission exact et un chemin modifié par ce lot.
Un chemin seul, une date seule ou le silence ne suffisent pas. En l’absence de
ce fait, l’écart reste ouvert ; il n’est jamais confirmé par absence
d’incident.

La session 025 persiste les écarts, leurs compteurs et la forme fermée des
faits de clôture. Le branchement sur les verdicts attend l’absorption de la
session 021 ; la complétude multi-relecteurs attend le lot suivant.

### D-2508 — Migration v20 séquentielle et vérifiée par le DDL

La migration v20 n’est ajoutée qu’après absorption du code des migrations v17,
v18 et v19. Son oracle part d’une base v19 réelle et vérifie les objets attendus
des trois versions précédentes avant toute écriture v20. Une ligne
`schema_migrations` sans la forme de schéma correspondante provoque un refus
explicite et laisse `user_version`, `schema_migrations` et `sqlite_master`
inchangés.

La branche ne doit donc jamais marquer 17, 18 ou 19 par une simple boucle alors
que leur DDL est absent.

### D-2509 — L’élection des relecteurs reste fermée tant que son capteur ne l’est pas

La session ne consomme pas `bridget-idle`. Après décision, elle rend un état
visible `decision_enregistree_election_non_disponible`. Le lot suivant devra
garantir auteur ≠ relecteur, polarités croisées, file visible si personne
n’est libre et refus typé si le capteur est périmé. Aucune option de forçage du
régime ou du relecteur n’est préparée ici.

## Scénarios et tests d’acceptation

### US1 — Soumettre un lot sur des objets Git exacts (P1)

Un auteur soumet projet, référence complète, tête et base. Maicie mesure le
diff des commits, refuse une référence déplacée ou une base non ancêtre, puis
persiste une proposition unique.

**Test indépendant** : créer un dépôt avec deux commits, soumettre le couple,
modifier ensuite le worktree et vérifier que la proposition ne change pas.

### US2 — Élire sans confondre les homonymes (P1)

Deux fichiers `main.rs` et deux fichiers `store.rs` coexistent. Une citation
complète élit un seul chemin ; une citation nue ambiguë n’en élit aucun ; un
numéro de ligne discriminant résout un seul candidat.

**Test indépendant** : muter le résolveur pour élire tous les homonymes et
faire échouer l’oracle sur la liste exacte des chemins.

### US3 — Obtenir un germe non vide au démarrage (P1)

Un projet sans registre contient néanmoins les quatre règles génériques. Une
modification de schéma, de protocole, d’autorisation ou d’écriture externe
durable propose un jury.

**Test indépendant** : retirer chaque règle séparément et vérifier que son
scénario passe de jury à simple, donc rougit.

### US4 — Décider sans justification décorative (P1)

Le référent sélectionne un régime fermé. Il peut confirmer, durcir ou alléger,
mais ne peut omettre la décision ni fournir de texte libre. Chaque choix est
persisté et les écarts sont comptés.

**Test indépendant** : tenter d’avancer sans sélection, avec un champ inconnu
et avec un autre émetteur ; vérifier trois refus durables et zéro progression.

### US5 — Lire ce que la machine a fait (P2)

L’utilisateur consulte les zones, leurs preuves structurées, les ambiguïtés,
les propositions, les décisions, les écarts et les refus. Aucun contenu source
ou trame externe n’apparaît dans cette vue.

**Test indépendant** : injecter un secret synthétique dans un diff et un
constat, puis vérifier son absence octet par octet dans la base et la sortie.

## Exigences fonctionnelles

- **FR-2501** : aucune configuration ou commande ne permet de fournir une
  liste de chemins critiques.
- **FR-2502** : la soumission exige projet, référence Git complète, tête et
  base ; l’auteur vient de l’enveloppe attestée.
- **FR-2503** : le calcul lit exclusivement les objets des commits mesurés et
  le catalogue déclaré, jamais le worktree courant.
- **FR-2504** : la carte contient les quatre règles de germe et les trois voies
  F38 ; leur résultat porte des chemins complets relatifs au dépôt.
- **FR-2505** : Bloquant vaut une preuve ; Major exige deux constats ouverts
  distincts ; une occurrence répétée dans le même constat ne double pas le
  compte.
- **FR-2506** : une citation ambiguë n’élit jamais plusieurs chemins ; elle est
  exposée et comptée.
- **FR-2507** : un contrat n’ancre qu’un chemin complet existant au commit.
- **FR-2508** : toute soumission reçoit une proposition déterministe parmi les
  trois régimes fermés.
- **FR-2509** : toute soumission ordinaire exige une sélection explicite du
  référent avant progression ; aucun texte libre n’est accepté.
- **FR-2510** : le noyau de criticité impose `jury_2x2` sans option de
  contournement.
- **FR-2511** : toute issue métier, y compris tout refus, est persistée
  idempotemment et apparaît dans les compteurs.
- **FR-2512** : les journaux de revue ne stockent ni diff, ni contenu de
  fichier, ni texte complet de constat, ni trame externe.
- **FR-2513** : les écarts se ferment seulement sur les faits exacts définis
  par D-2507 ; le silence ne ferme rien.
- **FR-2514** : aucune zone ne sort de la carte dans cette session et aucune
  approbation de profil ou de routine n’entre au guichet.
- **FR-2515** : la migration v20 vérifie les prérequis structurels v17–v19 et
  refuse sans écriture toute chaîne incomplète.
- **FR-2516** : aucune élection ou délégation de relecteur n’est effectuée
  tant que le capteur fiable n’est pas absorbé.

## Critères de succès

- **SC-2501** : les trois voies reproduisent un univers non vide et rendent
  uniquement des chemins complets ; le cas `main.rs`/`store.rs` n’élit jamais
  deux homonymes.
- **SC-2502** : quatre mutants, un par règle de germe, meurent dans
  l’assertion de régime.
- **SC-2503** : modifier le worktree, la configuration Git de diff ou une
  variable d’environnement ne change pas le résultat sur les mêmes SHA.
- **SC-2504** : 100 % des décisions et refus métier produits par le corpus
  possèdent une ligne durable et un compteur exact.
- **SC-2505** : aucune chaîne synthétique présente seulement dans un diff,
  une trame ou un texte de constat ne se retrouve dans la base ou la sortie.
- **SC-2506** : une migration v20 sur une fausse v19 sans DDL échoue et laisse
  l’empreinte bit à bit de la base inchangée.
- **SC-2507** : aucun appel à `bridget-idle`, aucune élection de relecteur et
  aucune approbation distante n’existent dans le diff de la session.

## Hors périmètre

- élection, disponibilité, polarité et délégation des relecteurs ;
- proposition puis approbation humaine des sorties de carte ;
- interprétation sémantique libre des constats ou des contrats ;
- garantie cryptographique de l’identité locale du dépôt ;
- authenticité du contenu métier d’un constat ;
- installation de v20 avant absorption et rebase des migrations v17–v19.
