# Contre-revue adverse RC7 - programme 065-067

**Périmètre relu** : SPEC-065, SPEC-066, SPEC-067 et ADR-016, sans exécution
ni modification de production.

## VERDICT: BLOCKED

Deux garanties fondatrices du programme restent impossibles à démontrer avec
les contrats et tâches actuels : l’unicité de l’identité après canonicalisation
distante, et la frontière inter-projets du runtime ingress. Deux autres
invariants de cycle de vie et d’approbation doivent être rendus causaux avant
implémentation. Aucun finding ne remet en cause la progressivité générale :
065 est autonome en `host`, 066 reste opt-in sans secrets, 067 reste séparée
après attestation du runtime. Les dépendances 063/064 sont correctement des
gates, non des prérequis prétendus déjà prouvés.

## Findings CRITICAL

### C1 - Deux enregistrements concurrents peuvent créer deux identités Maicie pour une seule racine canonique

**Fichiers et sections** :

- `specs/065-registre-identite-projets/data-model.md` -
  `ProjectRegistrationCommand`, lignes 33-42 ; transitions, lignes 59-74.
- `specs/065-registre-identite-projets/contracts/project-registry-v1.md` -
  `ProjectBindOutcome`, lignes 25-69.
- `specs/065-registre-identite-projets/tasks.md` - T010 à T016.

Maicie alloue `project_id` avant l'outbox, alors que Bridget est le seul à
connaître la racine canonique et ne garantit l'unicité qu'après réception. Deux
`command_id` différents visant deux formes lexicales de la même racine peuvent
donc produire deux `ProjectIdentity`. Le second refus `root_already_bound` ne
porte ni identité déjà liée ni transition métier du perdant ; le modèle Maicie
ne connaît que `active` et `disabled`. Le contrat ne peut donc satisfaire
l'exigence d'une identité unique par racine, ni donner une issue durable et
actionnable à cette course.

**Correction minimale** : introduire un état Maicie non actif de
`pending_binding` ou `registration_conflict`; compléter l'issue de collision
par l'identité déjà liée ou par une référence opaque permettant de converger
sans lire SQLite Bridget. Une identité perdante ne devient jamais active ni
référençable par objectif/délégation avant liaison. Ajouter un oracle de deux
commandes concurrentes sur alias/symlink de même racine : une seule identité
active et liaison existent; l'autre commande converge ou termine en conflit
durable explicite, sans double identité active.

### C2 - Le runtime ingress n'est pas une frontière de projet attestée

**Fichiers et sections** :

- `specs/066-environnement-partage-projet/plan.md` - `ProjectRuntimePolicy`,
  lignes 73-87 ; lot 3, lignes 109-118.
- `specs/066-environnement-partage-projet/tasks.md` - T017, T020-T021.
- `specs/066-environnement-partage-projet/data-model.md` -
  `ProjectRuntimePolicy` et `ContainerAgentExecution`.

Le plan annonce un second listener Unix et un répertoire dédié, mais pas un
ingress privé par `project_id`/génération ni une attestation de la première
connexion. Le même point d'entrée peut donc être monté à plusieurs conteneurs;
un processus du projet A peut alors se présenter comme B. Les labels Docker
attestent le conteneur à sa création, pas la provenance d'une connexion Unix.
Cela contredit la séparation entre projets promise par 066 et rend le canal de
pilotage, de remise et de reconnexion trans-projet.

**Correction minimale** : définir un socket ou répertoire runtime non partagé
par projet et génération, monté uniquement dans son conteneur, puis imposer au
premier handshake `project_id`, `binding_generation`, `container_id` et
génération Bridget attendus. Refuser toute divergence avant inscription ou
remise. Ajouter deux oracles : un wrapper A ne peut pas joindre l'ingress B;
une connexion au bon socket avec identité/génération forgée est refusée avant
tout effet. Les preuves de T017 doivent contenir ces deux refus.

## Findings HIGH

### H1 - Stop/remove/switch et admission d'un nouvel agent ne sont pas sérialisés

**Fichiers et sections** :

- `specs/066-environnement-partage-projet/plan.md` - lot 4,
  lignes 120-126 ; cycle de vie, lignes 139-152.
- `specs/066-environnement-partage-projet/data-model.md` - invariants de
  `ContainerAgentExecution`.
- `specs/066-environnement-partage-projet/tasks.md` - T023-T025 et T027-T031.

La règle « refuser si un agent est actif » est un constat, pas une exclusion
atomique. Entre ce constat et `docker stop/remove/switch_backend`, un spawn
peut obtenir son admission ou démarrer `docker exec`. Inversement, un exec peut
être créé après le comptage mais avant le changement de backend. La simple
réutilisation citée de `SpawnLease` ne spécifie ni transaction commune ni
génération d'environnement attendue.

**Correction minimale** : rendre la transition vers `stopping`/`switching`
transactionnelle dans l'autorité Bridget, avec un epoch d'environnement porté
par chaque réservation de spawn et vérifié juste avant `docker exec`. Une
opération lifecycle bloque les nouvelles réservations; une réservation acquise
fait refuser le lifecycle. Ajouter un test d'interleaving déterministe entre
admission et exec : soit le spawn gagne et stop/remove refuse, soit le
lifecycle gagne et aucun exec n'est lancé.

### H2 - Un rebind 065 peut réutiliser un profil approuvé et ses secrets sur une nouvelle racine

**Fichiers et sections** :

- `specs/065-registre-identite-projets/data-model.md` -
  `ProjectBinding.generation`, lignes 20-31 et transition de rebind.
- `specs/067-profils-extensions-secrets-projet/data-model.md` -
  `ProjectProfile`, lignes 3-22 ; `ProjectProfileApproval`, lignes 55-86.
- `specs/067-profils-extensions-secrets-projet/tasks.md` - T012, T024-T030.

Le rebind conserve volontairement `project_id` mais incrémente la génération
de liaison. Cette génération ne figure ni dans le profil, ni dans l'approbation
ni dans les invariants d'activation de 067. Un profil approuvé, avec extensions
et SecretRefs projet, peut donc rester actif après que la même identité a été
reliée explicitement à une autre racine. Le digest de politique peut varier,
mais aucun contrat n'oblige ce changement à invalider l'approbation.

**Correction minimale** : épingler `binding_generation` et `policy_digest`
dans `ResolvedProjectProfile`, l'approbation et la réservation de spawn. Un
rebind rend le profil `stale`, interdit tout montage/lecture de secret et exige
une nouvelle approbation puis une recréation. Ajouter un scénario
approbation → rebind → tentative de spawn : refus avant lecture ou montage;
ré-approbation sur la nouvelle génération seulement.

## Findings MEDIUM

### M1 - La redaction de `process_env` n'a pas de durée de vie ni de frontière de persistance définie

**Fichiers et sections** :

- `specs/067-profils-extensions-secrets-projet/contracts/project-profile-v1.md`
  - section `Secret process-env`, lignes 74-80.
- `specs/067-profils-extensions-secrets-projet/plan.md` - lot 3.
- `specs/067-profils-extensions-secrets-projet/tasks.md` - T023 et T028-T029.

Le wrapper doit lire la valeur, l'injecter au provider, effacer son buffer et
redacter ensuite stdout/stderr avant journalisation. La spécification ne dit
pas quel composant conserve une valeur éphémère pour redacter une sortie qui
arrive après le spawn, ni quelle frontière interdit d'écrire l'octet brut avant
la redaction. La promesse « zéro valeur dans crash report ou journal » est donc
testée par scan final sans mécanisme causal défini.

**Correction minimale** : désigner le wrapper comme unique frontière de
redaction pour toute sortie fournisseur contenant un process-env, définir la
durée de vie bornée du redactor et interdire l'écriture brute avant comparaison.
Ajouter un oracle qui fait émettre la sentinelle après le démarrage du provider
et vérifie que le JournalWriter ne reçoit jamais l'octet brut.

## Findings LOW

Aucun finding LOW distinct : les réserves restantes sont déjà couvertes par les
limites explicitement déclarées (Docker non hostile, egress non filtré,
confiance partagée à l'intérieur d'un projet) ou par les findings ci-dessus.

## Vérifications sans finding bloquant

- **Progressivité et valeur autonome** : ordre 063/064 → 065 → 066 → 067,
  activation explicite et rollback host sont cohérents; 065 ne crée aucun
  conteneur, 066 aucun secret et 067 ne redéfinit pas le runtime.
- **Autorités et doublons** : Maicie conserve identité/profil/approbation;
  Bridget conserve chemin, binding, runtime et AgentRegistry. Aucun second
  store partagé, daemon, SDK Docker, broker ou registre fournisseur n'est
  introduit.
- **Host/Docker et worktrees** : `host` demeure explicite, sans fallback
  silencieux; les worktrees hors racine sont refusés si leur montage attesté ne
  peut rester minimal. Ces choix sont proportionnés.
- **Fournisseurs** : Codex app-server, Claude stream-json et Cursor ACP sont
  traités par le même contrat, Cursor restant sous ACP; aucun chemin de
  sécurité par nom de fournisseur n'est exigé.
- **Secrets/extensions** : les limites intra-projet, les montages read-only,
  l'absence de Docker socket/home global et le refus host d'un profil Docker
  sont explicitement annoncés. Ils ne suffisent toutefois pas à lever C2, H2
  et M1.
- **Tâches** : le découpage est majoritairement test-first et les gates de
  SPEC-063/064 sont déclarés. Les quatre corrections ci-dessus doivent être
  ajoutées comme tâches et oracles avant passage hors Draft.

**Aucun finding supplémentaire bloquant n'a été trouvé.**
