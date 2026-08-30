# Feature Specification: Registre et identité des projets

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 065-registre-identite-projets
Titre: Registre et identité des projets
Statut: Draft
Priorité: P1
Tâches: 0/30 (0%)
Tests: 0/14 (0%)

Résumé:
- Contexte: Bridget déduit aujourd'hui un domaine depuis le dépôt courant et Maicie ne connaît qu'un projet de revue optionnel, sans identité commune durable pour tous les usages.
- Objectif: enregistrer un projet une seule fois, lui attribuer une identité stable et relier sans confusion les vérités projet de Maicie et de Bridget.
- Exécution: introduire un registre additif et idempotent, conserver le backend hôte actuel et propager une référence projet optionnelle dans les contrats publics.
- Risque principal: créer deux registres concurrents ou casser les projets historiques non enregistrés.
- Mitigation: autorité explicite par champ, saga de liaison rejouable et compatibilité intégrale des commandes historiques.
- Validation: enregistrement, rejeu, déplacement explicite, désactivation et redémarrage sont prouvés sans modification ni suppression du dépôt.
- Dépendances: SPEC-046, SPEC-048, SPEC-052, SPEC-063, SPEC-064, SPEC-068

Fichiers:
- spec.md: ✓ (specs/065-registre-identite-projets/spec.md)
- tasks.md: ✓ (specs/065-registre-identite-projets/tasks.md)
- plan.md: ✓ (specs/065-registre-identite-projets/plan.md)
- implementation.md: ✓ (specs/065-registre-identite-projets/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-065-programme-environnements-projet`
**Created**: 2026-08-29
**Status**: Draft
**Priority**: P1
**Dependencies**: SPEC-046, SPEC-048, SPEC-052, SPEC-063, SPEC-064, SPEC-068

## Contexte et problème

Bridget sait lancer un agent depuis un répertoire absolu et lui associe un
`domain` dérivé du nom de la racine Git. Ce domaine sert au classement et ne
constitue ni une identité stable, ni une frontière de sécurité. Deux dépôts de
même nom peuvent collisionner, un déplacement change la déduction et aucun
enregistrement ne relie durablement ce chemin à une mission Maicie.

Maicie possède les objectifs, délégations, décisions et profils. Elle possède
aussi une configuration `review_project`, mais celle-ci décrit un seul dépôt
de revue et ne constitue pas un registre général. Les prochaines étapes
d'environnement par projet ne peuvent pas s'appuyer sur ces deux approximations.

Cette spec crée uniquement le socle d'identité et de liaison. Elle ne change
pas la manière d'exécuter les agents: `backend=host` reste la seule exécution
active et tous les usages historiques continuent à fonctionner.

## Principes directeurs

- **Autorité métier Maicie**: Maicie possède l'identité stable et le cycle de
  vie du projet.
- **Autorité technique Bridget**: Bridget possède la liaison à la racine hôte,
  au backend d'exécution et aux faits runtime.
- **Une action utilisateur**: l'enregistrement déclenche une saga rejouable au
  lieu d'exiger deux commandes manuelles.
- **Compatibilité additive**: l'absence de `project_id` conserve le
  fonctionnement historique, explicitement marqué `unregistered`.
- **Aucune opération destructive**: désactiver ou retirer une liaison ne
  supprime jamais un dépôt, un worktree, un secret ou une donnée Maicie.

## User Scenarios & Testing

### User Story 1 - Enregistrer un projet existant ou nouveau (Priority: P1)

En tant qu'opérateur, je veux enregistrer une racine de projet en une seule
action afin d'obtenir une identité stable utilisable par Maicie, Bridget et les
agents, sans déplacer le code.

**Why this priority**: toutes les isolations et politiques futures dépendent
d'une identité projet non ambiguë.

**Independent Test**: enregistrer une racine Git située sous une racine
autorisée, rejouer exactement la demande après redémarrage, puis vérifier le
même `project_id` et la même liaison `backend=host` des deux côtés.

**Acceptance Scenarios**:

1. **Given** une racine existante non enregistrée, **When** l'opérateur lance
   l'enregistrement, **Then** un identifiant stable est rendu et la liaison
   technique devient `active` sans modifier le dépôt.
2. **Given** la même demande et la même racine canonique, **When** elle est
   rejouée, **Then** le résultat antérieur est rendu sans seconde identité ni
   seconde liaison.
3. **Given** une racine hors des préfixes autorisés, **When** elle est proposée,
   **Then** l'enregistrement est refusé avant toute écriture durable.
4. **Given** une panne après l'écriture Maicie mais avant la liaison Bridget,
   **When** la commande est rejouée, **Then** la saga reprend et atteint une
   issue unique et explicite.
5. **Given** deux commandes distinctes visant deux alias ou liens symboliques
   de la même racine canonique, **When** elles sont traitées en concurrence,
   **Then** une seule identité devient active et l'autre commande converge vers
   cette identité ou termine en conflit durable explicite.

### User Story 2 - Administrer la liaison sans risque pour le code (Priority: P1)

En tant qu'opérateur, je veux lister, diagnostiquer, déplacer explicitement ou
désactiver un projet afin de maîtriser sa liaison sans risque de perte de code.

**Why this priority**: un registre sans diagnostic ni retrait sûr devient une
nouvelle dépendance opaque.

**Independent Test**: déplacer un dépôt de test, constater `path_missing`,
effectuer un `rebind` explicite, puis désactiver le projet et vérifier que tous
les fichiers du dépôt sont inchangés.

**Acceptance Scenarios**:

1. **Given** une racine déplacée hors du contrôle de Bridget, **When** le statut
   est consulté, **Then** le projet reste identifiable et sa liaison indique
   `path_missing` sans correction automatique.
2. **Given** un nouveau chemin valide, **When** l'opérateur confirme le
   déplacement, **Then** la même identité est reliée au nouveau chemin et
   l'ancien chemin reste dans l'historique d'audit.
3. **Given** un projet désactivé, **When** un lancement lui est attribué,
   **Then** il est refusé avec une raison structurée sans affecter les
   lancements historiques sans projet.
4. **Given** une désactivation ou un retrait de liaison, **When** l'opération
   termine, **Then** aucun fichier du dépôt et aucun worktree n'est supprimé.

### User Story 3 - Corréler agents, travaux et missions au projet (Priority: P2)

En tant que responsable de mission, je veux voir à quel projet appartient une
exécution afin de filtrer la flotte et les preuves sans transformer ce lien en
autorité métier ou en mécanisme de sécurité.

**Why this priority**: la corrélation doit précéder le backend Docker, mais elle
peut être livrée après l'enregistrement et l'administration de base.

**Independent Test**: lancer deux agents dans deux projets enregistrés et un
agent historique sans projet, puis vérifier que les trois états restent
distincts dans les contrats et projections.

**Acceptance Scenarios**:

1. **Given** un lancement portant un `project_id`, **When** Bridget l'admet,
   **Then** l'identité projet est conservée dans la génération et les faits
   d'exécution sans être redéduite du nom du dossier.
2. **Given** un agent historique sans `project_id`, **When** il se connecte,
   **Then** il continue de fonctionner et apparaît `unregistered`.
3. **Given** une délégation Maicie liée au projet A, **When** une exécution du
   projet B est proposée, **Then** la divergence est refusée sans réécrire la
   délégation.
4. **Given** un filtre par projet, **When** l'opérateur consulte la flotte,
   **Then** seuls les agents explicitement liés sont sélectionnés et le domaine
   historique reste une simple métadonnée de classement.

## Edge Cases

- Deux chemins lexicaux différents qui se canonisent vers la même racine.
- Deux racines distinctes ayant le même nom de dernier répertoire.
- Racine devenue inaccessible, renommée ou remplacée par un lien symbolique.
- Rejeu après expiration d'une clé de commande ou avec un corps divergent.
- Maicie disponible mais Bridget indisponible, puis situation inverse.
- Projet désactivé avec des agents encore actifs.
- Ancienne configuration `review_project` visant une racine enregistrée.
- Tentative de lier un worktree comme nouveau projet alors qu'il appartient à
  un projet déjà enregistré.

## Requirements

### Functional Requirements

- **FR-001**: chaque projet enregistré DOIT posséder un `project_id` opaque,
  stable et indépendant du nom ou du chemin courant.
- **FR-002**: une seule action utilisateur DOIT créer une intention Maicie,
  demander la liaison Bridget puis activer l'identité gagnante par une saga
  idempotente.
- **FR-003**: Maicie DOIT rester l'autorité du nom métier, du statut métier et
  des relations aux objectifs et délégations.
- **FR-004**: Bridget DOIT rester l'autorité du chemin hôte canonique, du
  backend d'exécution et de la santé de la liaison.
- **FR-005**: le registre NE DOIT PAS partager une base privée entre Maicie et
  Bridget.
- **FR-006**: chaque étape de la saga DOIT être rejouable avec le même
  `proposed_project_id`, les mêmes octets et une issue durable.
- **FR-007**: l'enregistrement DOIT canonicaliser une racine existante avant
  écriture et refuser les chemins relatifs, disparus ou hors racines autorisées.
- **FR-008**: une même racine canonique active NE DOIT PAS recevoir deux
  identités projet actives; une identité en attente ou perdante NE DOIT PAS
  être référençable par un objectif, une délégation ou une exécution.
- **FR-009**: un `project_id` actif NE DOIT PAS changer de racine sans une
  opération explicite de `rebind`.
- **FR-010**: le `rebind` DOIT conserver l'ancienne liaison dans une trace
  d'audit et ne jamais déplacer le dépôt.
- **FR-011**: la désactivation DOIT empêcher les nouveaux lancements attribués
  au projet sans arrêter implicitement les agents déjà actifs.
- **FR-012**: aucune opération de registre NE DOIT supprimer ou modifier le
  contenu d'une racine, d'un worktree ou d'un dépôt.
- **FR-013**: le statut DOIT distinguer au minimum `pending_binding`, `active`,
  `disabled`, `path_missing`, `binding_failed` et `unregistered`.
- **FR-014**: les erreurs de liaison DOIVENT inclure une raison structurée, la
  dernière tentative et la prochaine action autorisée.
- **FR-015**: les projets historiques sans enregistrement DOIVENT conserver
  les commandes de lancement et de communication actuelles.
- **FR-016**: le champ `project_id` ajouté aux contrats runtime DOIT être
  optionnel pendant la migration.
- **FR-017**: un lancement attribué à un projet DOIT vérifier que son `cwd`
  appartient à la racine canonique ou à un worktree Git rattaché à cette
  racine.
- **FR-018**: le système NE DOIT PAS confondre le `domain` actuel avec
  `project_id` ni présenter le domaine comme une frontière de sécurité.
- **FR-019**: une délégation et une exécution explicitement liées à des projets
  différents DOIVENT être refusées avant effet runtime.
- **FR-020**: les projections DOIVENT afficher séparément l'identité projet,
  le domaine historique et la fraîcheur de la liaison.
- **FR-021**: la configuration `review_project` existante DOIT être migrée ou
  rapprochée explicitement, jamais copiée silencieusement en second projet.
- **FR-022**: toute lecture croisée Maicie-Bridget DOIT passer par un contrat
  public versionné et tolérer l'indisponibilité de l'autre composant.
- **FR-023**: l'enregistrement initial DOIT fixer `backend=host`; aucun backend
  Docker ne peut être activé par cette spec.
- **FR-024**: les opérations de registre DOIVENT produire un événement d'audit
  sans contenu de dépôt ni secret.
- **FR-025**: lors d'une collision après canonicalisation, Bridget DOIT rendre
  l'identité opaque déjà liée et sa génération afin que Maicie converge ou
  termine en `registration_conflict` sans lire le store Bridget.
- **FR-026**: un rebind avec agent actif NE DOIT PAS arrêter cette exécution;
  elle termine sur son ancienne génération, tandis que les admissions qui
  exigent la nouvelle liaison suivent les gates des runtimes et profils aval.
- **FR-027**: Bridget DOIT charger les racines projet autorisées depuis une
  politique hôte v1 fournie par un chemin absolu explicite au démarrage; une
  politique absente, vide, trop permissive, invalide ou modifiable par un autre
  compte DOIT faire échouer toute mutation de registre sans affecter les
  lancements host historiques non enregistrés.
- **FR-028**: les mutations de registre Maicie vers Bridget DOIVENT utiliser
  des variantes requête/résultat dédiées, versionnées et négociées sur la
  connexion locale; elles NE DOIVENT PAS détourner le flux `ServiceRequest`
  orienté Bridget vers Maicie et DOIVENT refuser un pair dont le rôle, la
  capability ou l'UID local ne correspond pas au daemon.
- **FR-029**: ProjectReference DOIT être conservée dans toutes les surfaces
  durables de SPEC-064: ordre et lease de spawn, état désiré, snapshot
  d'exécution, référence et projection Maicie, reprise par curseur et
  génération d'agent, ainsi que dans les liens parent-enfant, événements de
  lien et incidents runtime délégués livrés par SPEC-068; aucune reprise ne
  peut la redéduire depuis `cwd`, `domain`, une instance ou un execution_id.
- **FR-030**: le rapprochement de `review_project` DOIT être une commande
  locale explicite avec prévisualisation, résultat audité et issue
  idempotente; le démarrage, une migration SQLite ou un enregistrement voisin
  NE DOIVENT créer aucune identité à partir de cette configuration.

### Non-Functional Requirements

- **NFR-001 - Durabilité**: un crash à chaque frontière de la saga ne crée ni
  double identité ni double liaison.
- **NFR-002 - Compatibilité**: 100 % des tests de lancement historiques sans
  `project_id` restent valides.
- **NFR-003 - Sécurité**: les chemins sont validés après canonicalisation et
  les fichiers de registre privés conservent des permissions au plus `0600`.
- **NFR-004 - Performance**: l'accès par `project_id` et par racine canonique
  utilise un index et reste logarithmique ou constant dans le nombre de projets.
- **NFR-005 - Observabilité**: logs et métriques exposent issue, latence et
  composant de la saga avec une cardinalité bornée.
- **NFR-006 - Maintenabilité**: aucun nouveau daemon, crate ou moteur de
  workflow générique n'est introduit.
- **NFR-007 - Réversibilité**: ignorer les champs projet rétablit le
  comportement historique sans migration destructive.

### Key Entities

- **ProjectIdentity**: identité métier durable possédée par Maicie.
- **ProjectBinding**: liaison technique possédée par Bridget entre projet,
  racine canonique et backend.
- **ProjectRegistrationCommand**: commande idempotente et son corps canonique.
- **ProjectBindingAttempt**: tentative rejouable de liaison avec issue et date.
- **ProjectReference**: référence optionnelle portée par délégation, soumission,
  exécution et génération.
- **ProjectAuditEvent**: trace bornée d'enregistrement, rebind ou désactivation.

## Hors périmètre

- Créer, démarrer ou administrer un conteneur.
- Déplacer automatiquement un dépôt ou un worktree.
- Migrer automatiquement tous les projets historiques.
- Stocker des secrets, plugins, skills ou mémoire partagée.
- Transformer `domain` en contrôle d'accès.
- Introduire Kubernetes, Vault, gVisor ou un service de registre séparé.

## Success Criteria

### Measurable Outcomes

- **SC-001**: un projet valide est enregistré et visible des deux composants en
  moins de 30 secondes par une seule action utilisateur.
- **SC-002**: 100 rejouements identiques, dont un après redémarrage à chaque
  frontière, produisent un seul `project_id` et une seule liaison active.
- **SC-003**: 100 % des chemins relatifs, inexistants ou hors racines autorisées
  sont refusés avant écriture durable.
- **SC-004**: un opérateur identifie en moins de 20 secondes l'état de la liaison,
  sa fraîcheur et la prochaine action autorisée.
- **SC-005**: un `rebind` conserve le même `project_id`, laisse l'ancien dépôt
  intact et produit exactement un événement d'audit.
- **SC-006**: une désactivation ne modifie aucun octet sous la racine projet et
  ne tue aucun agent déjà actif.
- **SC-007**: la suite de compatibilité historique sans `project_id` passe à
  100 % avec `backend=host`.
- **SC-008**: 100 % des agents attribués à un projet affichent le même
  `project_id` dans génération, exécution et projection.
- **SC-009**: aucune observation Bridget ne ferme, rouvre ou réécrit un objectif
  Maicie.
- **SC-010**: aucun nouveau composant externe ou dépendance Rust n'est ajouté.
- **SC-011**: après redémarrage, reprise par curseur et reconstruction des
  projections, 100 % des exécutions liées conservent le même `project_id` et
  la même `binding_generation` que leur admission initiale.

## Assumptions et dépendances

- SPEC-063 doit être prouvée et les contrats d'exécution de SPEC-064 doivent
  être stabilisés avant de propager `project_id` dans les exécutions.
- Le système reste local et coopératif sous le même compte Unix.
- Les racines autorisées sont fournies par la politique fermée décrite dans
  `contracts/project-root-policy-v1.md`; aucun défaut implicite n'est admis.
- Le registre peut contenir plusieurs projets, mais une racine canonique active
  n'appartient qu'à un seul projet.
- Les worktrees sont reconnus par leur rattachement Git, pas par leur nom.
