# Session 026 — Opérations fédérées du greffe central

**Branche** : `session-026-operations-greffe-central`
**Base de spécification** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
**Statut** : spécification et témoins TDD ; code partagé bloqué par la session 021
**Priorité** : P1
**Migration réservée** : v19, à composer après v18

## Contexte mesuré

Le guichet Maicie fonctionne déjà pour `delivery_report` : il porte une
enveloppe canonique, l'idempotence par `request_id`, un bail de relève et des
issues terminales durables. Il n'accepte cependant aucune opération permettant
à un agent fédéré de déléguer, de consigner un constat ou de clore un objectif.
Ces agents transmettent donc leur intention à un humain, qui la ressaisit dans
le greffe central.

Exposer directement la SQLite Maicie ou un second écrivain MCP créerait deux
portes de vérité. La session étend la porte existante : le guichet. Un éventuel
outil MCP futur ne pourra être qu'un client mince de ce même contrat.

Une seconde mesure a montré que 121 des 123 objectifs portant `suite` déclarent
`aucune`, y compris quand le mandat cite une relation avec un autre objectif.
La seule obligation de remplir un champ mesure donc la valeur la moins chère,
pas la réalité. La session corrige ce défaut dans le cas d'usage partagé par la
CLI locale et le futur chemin fédéré avant d'ouvrir les trois opérations.

## Propriétés à tenir

### P2601 — Une seule porte d'écriture

`delegate`, `registre_add` et `objective_close` sont déposées dans le guichet
Bridget, relevées côté maître et appliquées par Maicie au store et au catalogue
centraux. Aucun appelant distant n'ouvre une base Maicie, ne choisit un chemin
de catalogue et ne bénéficie d'un repli local.

Le dépôt n'est pas l'application. `queued`, `in_flight` et `outcome_unknown`
attestent au plus que le transport a peut-être accepté une enveloppe. Seul un
reçu terminal durable, relu par `request_id`, atteste l'effet métier.

### P2602 — Matrice fermée et approbations impossibles

La matrice publique ajoute exactement :

- `delegate` ;
- `registre_add` ;
- `objective_close`.

Les opérations historiques de la session 015 restent compatibles.
`profile_approve` et `routine_approve` ne sont jamais des variantes autorisées
de `ServiceRequestOperation`. Une enveloppe attribuable qui les tente produit
un refus terminal durable portant exactement le nom demandé, sans créer ni
consommer d'approbation, routine, objectif, délégation ou ordre de lancement.

La prélecture peut conserver un nom d'opération borné uniquement pour auditer
le refus ; seule sa conversion réussie vers l'enum fermé autorise un cas
d'usage. Un JSON sans identité exploitable reste une erreur de protocole et ne
doit jamais être présenté comme un refus métier persisté.

### P2603 — Ne pas demander une promesse, confronter des faits

La validation de la relation `suite` vit dans le cas d'usage `delegate`, avant
toute création. La CLI locale et le traitement du guichet appellent exactement
ce même point. Aucun parseur CLI ou adaptateur réseau ne porte une discipline
parallèle.

Le validateur rend une décision fermée : `coherent`, `refused` ou `signaled`.
Un refus et un signal laissent chacun une ligne append-only et idempotente avec
le canal, la version de règle, la clé de requête, les identifiants confrontés et
la décision. Le but libre n'est pas recopié dans cet audit.

### P2604 — Aucun champ de conformité libre obligatoire

Aucune des trois opérations n'exige un motif, une raison ou une corrélation en
texte libre. Les identifiants, relations, durées, sévérités et issues sont des
types fermés ou des références vérifiables.

Le `goal` d'une délégation et le `text` d'un constat restent du contenu métier :
ils ne prétendent pas prouver une conformité. Ils sont validés et bornés, mais
ne remplacent jamais une relation structurée ni une raison typée.

### P2605 — Tout refus est durable et exact

Tout refus produit après identification canonique de la requête est persisté
avant sa réponse. Il est consultable après reconnexion et rejoué à l'identique.
Les oracles comparent enums et chaînes exactes ; aucune assertion par
sous-chaîne n'est admise.

Les vocabulaires d'opérations et de motifs ont une source Rust unique qui
engendre `ALL`, la projection SQL et le parseur. Les colonnes SQL qui stockent
ces noms n'en recopient pas la liste dans un `CHECK`. Une valeur injectée hors
code peut exister physiquement, mais sa lecture échoue explicitement comme
corruption. Les `CHECK` de machine d'état restent en SQL.

## Contradiction attestable

Une contradiction est une opposition vérifiable entre deux pièces de la même
requête ; ce n'est pas une interprétation du but.

| Pièces observées | Décision | Exemple |
|---|---|---|
| `suite=aucune` et `depends_on` non vide | refus `no_suite_with_dependency` | le mandat déclare `--depends-on <UUID>` tout en déclarant aucune suite |
| `suite=aucune` et le but associe le même UUID à un marqueur causal fermé, tandis que la relation le classe `reference` | refus `declared_relation_conflicts_with_body` | `depends_on=<UUID>` dans le corps mais `<UUID>` dans `references` |
| UUID connu cité mais absent de `depends_on` et `references` | refus F37 existant | le corps cite `<UUID>` sans classement |
| `suite=aucune` et UUID classé uniquement `reference`, sans marqueur causal fermé | signal `no_suite_with_reference` ; pas de blocage | « relire le verdict de `<UUID>` » |
| formulation causale naturelle, négative, conditionnelle ou citée sans marqueur machine non ambigu | signal `ambiguous_relation` ; pas de blocage | « vérifier si ce lot dépend de `<UUID>` » |
| aucun UUID connu, ou `suite=<UUID>` cohérente | passage | mandat autonome, ou suite explicitement nommée |

Les marqueurs lexicaux bloquants sont volontairement une grammaire machine
minimale (`depends_on=<UUID>`, `depends-on:<UUID>` et
`prerequisite_objective_id=<UUID>`), pas une liste de mots interprétés. Une
phrase française ou anglaise est signalée quand elle ne peut pas être réduite
sans ambiguïté. Aucun LLM, score ou heuristique probabiliste n'intervient.

## Scénarios utilisateurs

### US2601 — Refuser une déclaration contradictoire sur les deux chemins (P1)

Un appel local ou fédéré qui déclare `suite=aucune` avec une dépendance
structurée reçoit le même motif exact. Aucune délégation n'est créée et une
ligne durable permet de compter le refus. Une simple référence de revue ne
bloque pas ; elle produit un signal durable.

**Test indépendant** : jouer le même corpus contre la CLI locale et un claim de
guichet, comparer les décisions exactes et les lignes d'audit, puis muter le
validateur dans un seul chemin ; le témoin de parité doit rougir.

### US2602 — Déléguer depuis un agent fédéré (P1)

Un agent dépose un but borné, une cible ou des tags, une durée fermée et ses
relations classées. Le maître résout la configuration et l'annuaire, crée une
seule fois l'objectif, la délégation et l'outbox, puis rend leurs identifiants
dans un reçu terminal. Le `request_id` est l'unique clé d'idempotence externe.

**Test indépendant** : déposer, perdre l'accusé, rejouer les mêmes octets et
relire l'issue ; un seul couple objectif/délégation existe. Un rejeu divergent
est refusé et persisté.

### US2603 — Consigner au registre central (P1)

`registre_add` transporte une entrée `add` fermée du catalogue : version,
identifiant, date, source de mission structurée, sévérité, récurrence éventuelle
et texte du constat. Le chemin de journal vient de la configuration centrale.
Un rejeu exact produit `idempotent_noop`; un même identifiant divergent produit
un refus durable.

**Test indépendant** : placer un faux catalogue dans la configuration locale de
l'appelant, déposer deux fois la même entrée, puis vérifier une seule ligne dans
le catalogue central et aucune écriture dans le faux chemin.

### US2604 — Clore seulement sur une preuve centrale (P1)

`objective_close` ne demande aucun motif libre. Il porte `objective_id` et
`delegation_id`; Maicie vérifie la relation, l'identité du participant et une
livraison terminale acceptée, puis dérive la base fermée
`delivery_attested`. Sans cette preuve, la demande est refusée avec
`closure_not_attested_intervention_required`.

Une demande exacte rejouée après clôture retrouve son reçu original. Une autre
demande visant un objectif déjà clos reçoit
`objective_already_closed_intervention_required`; elle ne s'approprie pas la
clôture existante.

### US2605 — Connaître l'issue après une coupure (P1)

Le client peut relire l'issue et la charge terminales par `request_id`. Si le
tunnel est absent avant dépôt, il échoue explicitement sans base locale. Si la
réponse est perdue après dépôt, il rejoue strictement le même `request_id`, le
même `issued_at` et les mêmes octets, puis consulte l'issue. Il ne génère jamais
une seconde intention.

## Exigences fonctionnelles

- **FR-2601** : les trois mutations passent exclusivement par le guichet et le
  greffe centraux.
- **FR-2602** : les opérations et charges utiles sont versionnées, fermées et
  `deny_unknown_fields` à chaque niveau.
- **FR-2603** : les approbations de profil et de routine sont absentes du
  vocabulaire autorisé et refusées par des témoins distincts réellement joués.
- **FR-2604** : chaque refus attribuable et chaque signal de contradiction est
  persisté avant retour, avec comparaison exacte des valeurs.
- **FR-2605** : le validateur partagé précède les mutations locales et
  fédérées ; aucune validation équivalente ne subsiste dans `main.rs`.
- **FR-2606** : aucun champ libre obligatoire ne sert de motif, de base ou de
  corrélation ; le contenu métier libre reste distinct des gardes.
- **FR-2607** : les horodatages, horizons, chemins, configuration, candidats et
  identité de l'émetteur sont établis ou vérifiés côté maître.
- **FR-2608** : la consultation rend l'issue et la charge terminales durables,
  pas seulement l'état de dépôt.
- **FR-2609** : `queued`, `in_flight` et `outcome_unknown` ne valent jamais
  succès applicatif.
- **FR-2610** : aucun repli n'ouvre la SQLite ou le catalogue local de
  l'appelant.
- **FR-2611** : v19 retire les listes d'opérations/motifs recopiées dans les
  `CHECK`, conserve les gardes d'état et échoue à la lecture d'une valeur Rust
  inconnue.
- **FR-2612** : toute variante Rust autorisée possède un cas de corpus exact ;
  ajouter une variante sans fixture fait rougir le gate.

## Hors périmètre

- Exposer directement Maicie ou sa SQLite par MCP.
- Ajouter un daemon résident Maicie ou promettre une latence de traitement.
- Approuver, proposer ou consommer un profil ou une routine à distance.
- Interpréter un but par LLM ou classifier probabilistiquement une relation.
- Modifier la politique de sélection d'agent ou les règles d'approbation ADR 011.

## Dépendances de composition

- La session 021 doit être admise avant tout code partagé ; elle refond les
  motifs de refus dans les mêmes fichiers Maicie et Bridget.
- La migration v18 doit précéder v19 dans l'historique admis. Une tête 026 qui
  sauterait v18 ne peut pas être livrée.
- La session ne requiert aucune autre migration que v19.
