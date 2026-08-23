# Plan 017 — Greffière du catalogue

**Branche** : `session-17-greffiere-catalogue` | **Date** : 2026-08-23
**Spec** : `specs/017-greffiere-catalogue/spec.md`

## Résumé

Livrer le noyau mono-projet de Maicie greffière : un journal versionné et
appendable de constats déclarés, une migration conservatrice du catalogue
prose, des transitions automatiques déclenchées exclusivement par des clôtures
d'objectif attestées, et une vue déterministe sans effet. Le travail reste une
délégation durable ; le catalogue ne crée ni plan ni message libre.

## Contexte technique

**Langage** : Rust 2024 du workspace Bridget/Maicie.
**Composant** : `plugins/maicie/`, client public Bridget et SQLite Maicie déjà
présents pour les objectifs, délégations et événements attestés.
**Stockage d'autorité** : un fichier de catalogue versionné par projet hôte,
déclaré explicitement en configuration. Aucune base parallèle.
**Tests** : `cargo test -p maicie`, tests de contrat de format, intégration
fermeture d'objectif et golden de vue ; suite workspace et clippy final.
**Performance** : la vue v1 lit un seul catalogue de projet ; elle est bornée
par l'entrée déclarée et n'effectue aucune requête réseau ni inférence.
**Contrat de sécurité** : parse fermé, append atomique, références par ID,
écriture limitée au fichier de catalogue déclaré.

## Décisions de conception

### D-1701 — Un journal canonique assumé, pas un hybride Markdown invisible

Le catalogue reste un fichier versionné dans le dépôt hôte, déclaré en
configuration. C'est un **journal canonique machine**, non une prose Markdown
à lire entrée par entrée : les lignes JSON v1 fermées sont son autorité, et
`registre list` est l'unique vue humaine officielle. Le fichier peut conserver
un en-tête Markdown stable, mais aucun commentaire ni prose parallèle ne fait
foi sur un constat.

Le journal comporte deux sortes de lignes fermées :

```text
add        = {v, kind:"add", id, date:RFC3339, mission_source, severity, recurrence_of?, text}
transition = {v, kind:"transition", constat_id, from:"open", to:"delivered",
              objective_id, observed_at, trigger:"objective_closed"}
```

`mission_source` est fermé (`mission`, `incident`, `review`, `gate`) et porte
un `id` stable; les sources `gate` portent explicitement leur résultat raté.
`date` est RFC 3339 avec fuseau explicite. `severity` est fermé (`blocker`,
`major`, `minor`, `info`). Tous les identifiants
de constats, missions, objectifs et récurrences sont comparés exactement.

Sous le verrou exclusif du fichier déclaré, la lecture de la clé `(id, bytes
canoniques)` et l'append d'une ligne complète forment une seule opération : un
id identique avec bytes identiques est un no-op rejouable, un id identique avec
bytes divergents est refusé sans append. L'implémentation ouvre un descripteur
régulier non-symlink, écrit en `O_APPEND`, synchronise les données, puis
relâche le verrou. Un fichier temporaire suivi de rename est interdit, car il
ne préserve pas les append concurrents.

La migration ajoute des entrées v1 seulement lorsque les champs requis sont
explicitement fournis par le catalogue ou complétés par un humain. Le texte
historique verbatim et son identifiant de provenance sont conservés ; une ligne
incomplète reste en attente de qualification humaine. Elle ne fabrique ni
gravité, ni récurrence, ni source. Une ambiguïté est exposée au lecteur humain,
jamais résolue automatiquement.

### D-1702 — Le lien constat-remède est un fait de délégation

Le constat `add` ne devine aucun remède. Lorsqu'un humain arbitre qu'un constat
doit être traité, le cas d'usage de délégation durable porte `constat_id` et
journalise, avec l'objectif créé ou désigné, le couple exact
`(constat_id, objective_id)`. Ce fait est l'unique autorité qui rend une
clôture d'objectif pertinente pour un constat ; un titre commun ou une
ressemblance ne crée jamais ce lien.

### D-1703 — État dérivé uniquement d'une clôture attestée

La projection réduit chaque constat à `open` ou `delivered` en pliant les
lignes appendées. Une transition n'est acceptée que pour un `objective_id`
exact, à partir d'un événement de clôture ou d'une réconciliation par lecture
de son état durable attesté. Son identité déterministe
`(constat_id, objective_id, trigger)` rend la reprise idempotente.

Une échéance, un silence, une observation périmée ou la ressemblance d'un titre
ne peut jamais produire `delivered`. La réconciliation est idempotente et ne
sert qu'à rattraper une livraison d'événement perdue. Aucun état `planned` n'est créé : la
proposition de travail reste humaine et les routines futures sont hors v1.

### D-1704 — Vue pure, expliquée par des faits déclarés

`registre list` reconstruit une vue sans écriture. La clé de tri est figée :
constat ouvert d'abord, sévérité déclarée dans son ordre fermé, présence de
`recurrence_of`, source `gate` ratée, date puis `id`. Elle ne lit ni texte libre
pour classer, ni données d'autres projets, ni état de planification.

Le pied de page est un calcul du même journal :
`N constats OUVERTS dont M récurrents, K liés à un gate raté, P en attente de qualification`.
Les trois filets
de découvrabilité sont : prescription `registre list` par la skill en ouverture
de session et avant toute suite, pied de page aux jalons, consultation de la
vue triée au début du rituel de clôture.

### D-1705 — Les actions de travail passent par les délégations

Le catalogue peut présenter un constat et recevoir une décision humaine, mais
ne connaît aucune primitive d'envoi libre. Une décision qui lance un remède
appelle le cas d'usage de délégation durable existant, avec son reçu et son
identifiant. Cette frontière interdit la réapparition des jonctions non
corrélées découvertes hors greffe.

## Constitution check

| Gate | État | Preuve de conception |
|---|---|---|
| Français, artefacts SpecKit | Conforme | Spec et plan sont en français. |
| Article XIX, noyau minimal | Conforme | Deux entrées de journal et une projection pure ; pas d'adaptateur hôte. |
| FR-022, absence de jugement | Conforme | Tri exclusif de champs déclarés ; aucune déduplication ni score. |
| Source unique/versionnée | Conforme | Un catalogue Markdown déclaré par projet, sans base parallèle. |
| Sécurité et intégrité | Conforme | Parse fermé, références exactes, append atomique, migrations explicites. |
| Travail durable | Conforme | Une action déclenchante passe obligatoirement par délégation. |

## Structure cible

```text
plugins/maicie/
├── src/
│   ├── catalogue.rs             # grammaire, lecture, append et migration v1
│   ├── app.rs                   # cas d'usage constat/transition/délégation
│   ├── config.rs                # chemin explicite du catalogue par projet
│   └── main.rs                  # `registre add` et `registre list`
├── tests/
│   ├── contract/catalogue.rs    # corpus fermé et migration
│   └── integration/catalogue.rs # clôture attestée, reprise et absence d'I/O hôte
└── README.md                    # découvrabilité et limites de la greffière
specs/017-greffiere-catalogue/
├── spec.md
├── plan.md
├── data-model.md
├── contracts/
├── quickstart.md
└── tasks.md
```

## Phasage d'implémentation

1. Figer le corpus v1 et les fixtures de migration avant tout code de lecture.
   Les mutations de type/champ/sévérité/référence doivent refuser sans append.
2. Implémenter le parse et l'append atomique du seul catalogue déclaré, puis
   prouver la migration conservatrice, l'idempotence de `add`, deux writers
   concurrents et l'absence d'écriture dans les plans hôtes.
3. Construire la projection pure `registre list` et son footer sur corpus
   permuté ; prouver l'absence de déduplication et de classement textuel.
4. Raccorder la délégation d'arbitrage (`constat_id`, `objective_id`) aux
   clôtures attestées, puis tester transition idempotente, reprise/crash et
   réconciliation après perte d'événement.
5. Exposer seulement la décision humaine qui délègue ; tester que la voie
   message libre est absente et qu'une délégation porte sa corrélation.
6. Mettre à jour skill, rituel de clôture, documentation et quickstart ; lancer
   la suite workspace complète avant le gate final.

## Risques et garde-fous

| Risque | Garde-fou |
|---|---|
| La prose historique induit une interprétation | Migration verbatim et état inconnu explicite ; intervention humaine requise. |
| Homonymie d'objectif | UUID/identifiant exact obligatoire, aucun sélecteur textuel. |
| Dérive vers un outil de gestion de projet | Aucune écriture dans plans/issues/tasks ; pas d'état planifié. |
| Tri déguisé en jugement | Clé fermée, golden de la vue et absence de score composite. |
| Jonction de travail non traçable | Délégation obligatoire, aucune primitive d'envoi libre. |
