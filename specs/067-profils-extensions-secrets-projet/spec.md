# Feature Specification: Profils, extensions et secrets bornés par projet

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 067-profils-extensions-secrets-projet
Titre: Profils, extensions et secrets bornés par projet
Statut: Draft
Priorité: P1
Tâches: 5/38 (13%)
Tests: 2/19 (11%)

Résumé:
- Contexte: un environnement Docker sans credentials, profils ni extensions ne peut pas servir durablement Codex, Claude ou Cursor, mais monter le home ou tous les secrets de l'hôte annulerait la frontière projet.
- Objectif: activer par projet un profil approuvé qui sélectionne agents, skills/plugins et références de secrets sans stocker les valeurs dans Maicie, Bridget ou le dépôt.
- Exécution: réutiliser les profils et approbations existants, résoudre les références sur l'hôte, monter seulement les ressources approuvées et recréer l'environnement lors d'un changement.
- Risque principal: faire croire à une isolation entre agents alors qu'ils partagent le même conteneur et les mêmes secrets projet.
- Mitigation: projet comme frontière de confiance explicite, approbation locale, aucun secret individuel ni broker dynamique dans cette version.
- Validation: trois fournisseurs utilisent un profil projet, aucune ressource hors allowlist n'est visible et une rotation retire l'ancienne ressource après recréation.
- Dépendances: SPEC-011, SPEC-015, SPEC-026, SPEC-064, SPEC-065, SPEC-066, SPEC-068

Fichiers:
- spec.md: ✓ (specs/067-profils-extensions-secrets-projet/spec.md)
- tasks.md: ✓ (specs/067-profils-extensions-secrets-projet/tasks.md)
- plan.md: ✓ (specs/067-profils-extensions-secrets-projet/plan.md)
- implementation.md: ✓ (specs/067-profils-extensions-secrets-projet/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-065-programme-environnements-projet`
**Created**: 2026-08-29
**Status**: Draft
**Priority**: P1
**Dependencies**: SPEC-011, SPEC-015, SPEC-026, SPEC-064, SPEC-065, SPEC-066, SPEC-068

## Contexte et problème

SPEC-066 prouve volontairement un environnement sans secret. Pour y faire
tourner des agents réels, il faut fournir des configurations fournisseur, des
skills, des plugins et parfois des credentials. La solution la plus simple en
apparence, monter `$HOME`, exposer les variables du daemon ou partager tous les
catalogues, détruirait la séparation entre projets et rendrait les accès
impossibles à expliquer.

Maicie possède déjà des profils déclaratifs, leur approbation humaine et leur
liaison à un `SpawnOrder`. Bridget possède déjà le registre des adaptateurs, les
capacités, `forbidden_env`, `pass_env` et la définition résolue épinglée. Cette
spec étend ces mécanismes au niveau projet. Elle n'introduit ni plugin manager
agentique, ni service de secrets, ni mémoire globale.

La frontière v1 est le projet. Tous les agents d'un conteneur projet sont
considérés coopératifs et peuvent techniquement lire les secrets montés au
projet. Un besoin de secret individuel ou de code non fiable impose une
isolation distincte qui reste hors périmètre.

## Principes directeurs

- **Valeurs hors registres**: Maicie et Bridget conservent des références et
  versions, jamais les valeurs secrètes.
- **Approbation locale**: l'activation d'un profil ou d'un nouveau secret exige
  une frappe humaine locale et ne passe pas par UI, MCP ou agent.
- **Extensions immuables**: skills et plugins sont en lecture seule, issus de
  racines autorisées et épinglés par version et digest.
- **Projet comme frontière**: aucune promesse d'isolation secrète entre agents
  du même projet.
- **Recréation contrôlée**: montages et credentials changent seulement après
  arrêt des agents et recréation de l'environnement.
- **Générique fournisseurs**: le contrat décrit fichiers, répertoires, variables
  et capabilities sans branche de sécurité codée par nom Codex, Claude ou Cursor.

## User Scenarios & Testing

### User Story 1 - Définir et approuver un profil de projet (Priority: P1)

En tant qu'opérateur, je veux voir exactement quels agents, outils, extensions
et références secrètes seront exposés avant d'activer un projet.

**Why this priority**: l'approbation doit précéder tout montage ou injection.

**Independent Test**: proposer un profil comprenant trois agents, deux
extensions et deux références secrètes, comparer la vue d'approbation à la
définition Bridget résolue, puis approuver localement et vérifier le même digest
des deux côtés.

**Acceptance Scenarios**:

1. **Given** un profil valide, **When** il est proposé, **Then** la vue locale
   affiche projet, image, agents, modèles, efforts, outils, extensions, secrets
   référencés, réseau et limites sans afficher de valeur secrète.
2. **Given** une définition Bridget différente du digest proposé, **When**
   l'approbation est tentée, **Then** elle est refusée sans montage ni spawn.
3. **Given** une demande provenant d'un agent, de l'UI ou de MCP, **When** elle
   tente d'approuver, **Then** aucune surface d'approbation n'est disponible.
4. **Given** un profil déjà approuvé avec les mêmes octets, **When** il est
   rejoué, **Then** l'approbation et son résultat sont rendus sans duplication.

### User Story 2 - Exposer seulement les extensions approuvées (Priority: P1)

En tant qu'agent du projet, je veux retrouver les skills et plugins approuvés en
lecture seule afin de travailler avec un environnement reproductible.

**Why this priority**: les extensions exécutent du code ou des instructions et
doivent être aussi traçables que l'image.

**Independent Test**: activer deux extensions épinglées, démarrer Codex, Claude
et Cursor, vérifier leur visibilité et tenter d'écrire ou d'accéder à un autre
catalogue.

**Acceptance Scenarios**:

1. **Given** une extension sous une racine autorisée et au digest approuvé,
   **When** l'environnement est créé, **Then** elle est montée en lecture seule
   à une destination déclarée.
2. **Given** un digest, chemin, propriétaire ou permission divergent, **When**
   l'environnement est préparé, **Then** l'activation est refusée avant spawn.
3. **Given** deux projets aux profils différents, **When** leurs agents listent
   les extensions, **Then** aucun catalogue non approuvé pour leur projet n'est
   visible.
4. **Given** une extension modifiée après approbation, **When** un nouveau spawn
   est demandé, **Then** l'environnement passe `recreate_required`.

### User Story 3 - Utiliser et révoquer des secrets de projet (Priority: P1)

En tant qu'opérateur, je veux référencer, monter, faire utiliser puis révoquer
des credentials par projet sans les copier dans le dépôt ou les journaux.

**Why this priority**: les agents réels ne peuvent être activés qu'après preuve
de cette frontière sensible.

**Independent Test**: activer une fixture de secret fichier et une fixture de
secret process-env, vérifier leur utilisation par un agent test, faire tourner
leur génération puis recréer le conteneur et prouver l'absence des anciennes
valeurs.

**Acceptance Scenarios**:

1. **Given** une SecretRef approuvée, **When** Bridget la résout, **Then** la
   source reste sur l'hôte sous une racine privée et seule la destination
   déclarée est exposée au projet.
2. **Given** un secret process-env, **When** le fournisseur démarre, **Then** la
   valeur n'apparaît ni dans les arguments Docker, ni dans les labels, ni dans
   les stores ou logs.
3. **Given** plusieurs agents du même projet, **When** un secret projet est
   monté, **Then** le statut avertit explicitement qu'il est accessible à tous
   les processus du conteneur.
4. **Given** une rotation ou révocation, **When** les agents sont arrêtés et
   l'environnement recréé, **Then** l'ancienne génération n'est plus accessible
   et tout spawn sur l'ancien profil est refusé.
5. **Given** un profil approuvé, **When** le projet est rebindé vers une nouvelle
   racine, **Then** le profil devient `stale` et tout spawn est refusé avant
   lecture ou montage jusqu'à nouvelle approbation et recréation.
6. **Given** un fournisseur qui réémet tardivement une sentinelle process-env,
   **When** la sortie traverse le wrapper, **Then** aucun JournalWriter, log ou
   crash report ne reçoit les octets bruts.

### User Story 4 - Exploiter plusieurs fournisseurs avec le même contrat (Priority: P2)

En tant qu'exploitant, je veux appliquer le même profil de projet à Codex,
Claude et Cursor afin d'éviter une sécurité différente et implicite par outil.

**Why this priority**: la portabilité est utile après preuve des approbations,
extensions et secrets.

**Independent Test**: lancer les trois fournisseurs avec des profils fixtures
et vérifier que les capabilities, montages, refus et redactions suivent le même
contrat public.

**Acceptance Scenarios**:

1. **Given** un AgentProfile approuvé, **When** son adaptateur est résolu,
   **Then** modèle, effort, outils et besoins de secret sont comparés à la
   définition Bridget sans heuristique par nom.
2. **Given** une capability absente, **When** le profil l'exige, **Then** le
   spawn est refusé avant exposition des secrets.
3. **Given** Cursor via ACP, **When** il utilise le profil, **Then** aucune
   branche Cursor séparée du transport ACP n'est créée.
4. **Given** un backend host, **When** un profil exige des montages secrets
   Docker, **Then** il est refusé explicitement et l'ancien comportement host
   reste disponible sans ce profil.

## Edge Cases

- Secret source absent, trop permissif, symlink, fichier spécial ou sous le
  dépôt projet.
- Secret tourné sans incrément de génération déclaré.
- Extension valide à l'approbation puis modifiée avant create.
- Plugin qui tente d'écrire dans son propre montage read-only.
- Deux extensions demandant la même destination.
- Deux secrets demandant le même nom de variable ou destination.
- Profil modifié alors que des agents sont actifs.
- Projet docker revenu sur host avec profil docker encore actif.
- Credential fournisseur qui veut persister un token rafraîchi.
- Valeur secrète accidentellement incluse dans stderr ou sortie provider.
- Suppression d'une extension source pendant l'exécution.
- Tentative de partager une mémoire globale writable entre projets.

## Requirements

### Functional Requirements

- **FR-001**: Maicie DOIT posséder ProjectProfile, son statut et son approbation
  métier; Bridget DOIT posséder la définition runtime résolue et son attestation.
- **FR-002**: un ProjectProfile DOIT référencer une ProjectIdentity 065 active
  et une ProjectRuntimePolicy 066 compatible.
- **FR-003**: l'activation DOIT réutiliser l'approbation locale et le digest de
  définition déjà établis pour les profils Maicie.
- **FR-004**: la vue d'approbation DOIT afficher tous les agents, modèles,
  efforts, outils, extensions, SecretRefs, image, réseau, limites et destinations.
- **FR-005**: aucune valeur secrète NE DOIT apparaître dans la vue d'approbation.
- **FR-006**: UI, MCP, Bridget distante et agents NE DOIVENT exposer aucune
  opération d'approbation ou de rotation.
- **FR-007**: une définition, capability, image ou politique divergente DOIT
  invalider le digest et refuser l'activation.
- **FR-008**: chaque ExtensionRef DOIT déclarer type, source, destination,
  version et digest.
- **FR-009**: seules les extensions sous des racines hôte autorisées et privées
  DOIVENT être admises.
- **FR-010**: les extensions DOIVENT être montées en lecture seule et ne peuvent
  pas demander une destination absolue hors de l'ensemble fermé.
- **FR-011**: deux extensions ou secrets NE DOIVENT pas partager une destination
  ou une variable sans refus explicite.
- **FR-012**: une modification d'extension après approbation DOIT placer
  l'environnement en `recreate_required` avant tout nouveau spawn.
- **FR-013**: Maicie et Bridget DOIVENT stocker uniquement SecretRef, type,
  destination, usage, génération et état, jamais la valeur.
- **FR-014**: chaque source secrète DOIT être résolue sur l'hôte sous une racine
  autorisée distincte des dépôts et vérifier type, propriétaire et permissions.
- **FR-015**: un fichier secret DOIT être privé au plus `0600`; un répertoire
  secret DOIT être privé au plus `0700` et ne contenir aucun lien symbolique
  admis implicitement.
- **FR-016**: les secrets de type fichier ou répertoire DOIVENT être montés en
  lecture seule dans un chemin `/run` réservé.
- **FR-017**: un secret process-env DOIT être lu par le wrapper dans le
  conteneur et injecté au seul processus fournisseur, sans passer dans les
  arguments Docker, labels ou environnement global du conteneur.
- **FR-018**: les noms de variables process-env DOIVENT être allowlistés par la
  définition résolue et ne peuvent contourner `forbidden_env`.
- **FR-019**: tout secret de projet DOIT être considéré accessible à tous les
  agents du conteneur; le produit DOIT afficher cette limite avant approbation.
- **FR-020**: aucun secret individuel par agent NE DOIT être annoncé ou simulé
  dans cette version.
- **FR-021**: une rotation DOIT créer une nouvelle génération de SecretRef et
  exiger l'arrêt des agents puis la recréation de l'environnement.
- **FR-022**: l'ancienne génération DOIT être refusée pour tout nouveau spawn
  dès la rotation approuvée.
- **FR-023**: la révocation DOIT retirer la référence du profil et ne supprimer
  la source hôte que par une opération séparée hors Bridget/Maicie.
- **FR-024**: aucun log, métrique, événement, store, crash report ou projection
  NE DOIT contenir une valeur secrète; les incidents runtime délégués SPEC-068
  ne transportent que leur code fermé, une référence pseudonymisée et la
  ProjectReference, jamais les octets fournisseur.
- **FR-025**: les redactions DOIVENT couvrir les sorties provider qui répètent
  une valeur connue sans persister cette valeur comme règle de log.
- **FR-026**: les profils agents existants DOIVENT être référencés ou étendus,
  pas dupliqués dans un second registre.
- **FR-027**: AgentRegistry DOIT rester l'autorité des commandes, protocoles,
  capabilities, forbidden_env et pass_env.
- **FR-028**: l'admission DOIT vérifier toutes les capabilities avant de monter
  ou lire un secret.
- **FR-029**: Codex app-server, Claude stream-json et Cursor ACP DOIVENT suivre
  le même contrat de profil et de refus.
- **FR-030**: un profil exigeant un montage Docker DOIT être refusé sur backend
  host sans fallback ou exposition différente silencieuse.
- **FR-031**: un profil sans extension ni secret PEUT fonctionner sur host en
  conservant le comportement historique.
- **FR-032**: les catalogues globaux PEUVENT être montés en lecture seule par
  référence approuvée; aucune mémoire globale writable n'est autorisée.
- **FR-033**: les données de projet writable, caches et mémoire éventuelle
  DOIVENT rester dans le state root propre au projet.
- **FR-034**: le statut DOIT afficher profil actif, digest, générations,
  ressources exposées, état de recréation et prochaine action sans valeurs.
- **FR-035**: toute activation, rotation, révocation et recréation DOIT produire
  une trace d'audit avec principal humain local et raisons structurées.
- **FR-036**: ResolvedProjectProfile, ProjectProfileApproval et chaque
  réservation de spawn DOIVENT épingler `binding_generation` et
  `policy_digest`.
- **FR-037**: tout rebind 065 DOIT rendre le profil `stale`, interdire lecture
  ou montage de secret et exiger nouvelle approbation puis recréation avant
  spawn; les exécutions déjà actives terminent sur leur ancienne génération
  sans arrêt implicite.
- **FR-038**: le wrapper DOIT être l'unique frontière de redaction pour toute
  sortie d'un fournisseur recevant process-env; aucun sink durable ne peut
  recevoir les octets bruts avant comparaison binaire à état conservé entre
  fragments, par canal et jusqu'à fermeture complète.
- **FR-039**: Bridget DOIT résoudre chaque `source_ref` depuis un catalogue
  hôte v1 fermé, fourni par chemin absolu explicite au démarrage, qui associe
  référence opaque, type, chemin canonique, révision, UID/GID attendus et liste
  fermée de projets autorisés; aucun chemin fourni par Maicie ou un agent ne
  peut remplacer cette autorité.
- **FR-040**: chaque source secrète approuvée DOIT posséder une
  `SecretSourceStamp` calculée sans contenu à partir du type, device, inode,
  taille, mtime/ctime nanoseconde, mode, UID/GID et, pour un répertoire, du
  manifeste récursif trié de ces métadonnées; toute divergence avant create ou
  spawn DOIT produire `secret_generation_stale` avant lecture ou montage.
- **FR-041**: ProjectProfileProposal, ResolvedProjectProfile,
  ProjectProfileApproval et chaque réservation de spawn DOIVENT porter le même
  `runtime_policy_version` en plus de `policy_digest`; une divergence rend le
  profil `stale`.
- **FR-042**: un changement de backend, runtime policy, image, UID/GID ou
  `runtime_policy_changed` émis par SPEC-066 DOIT rendre le profil `stale`,
  interdire toute nouvelle résolution de ressource et exiger une nouvelle
  approbation puis recréation; les exécutions actives ne sont pas arrêtées.
- **FR-043**: l'UID/GID runtime de SPEC-066 DOIT correspondre aux droits
  attestés des sources secrètes; la fixture DOIT prouver la lecture d'un
  fichier `0600` et le refus avec propriétaire incompatible avant activation.

### Non-Functional Requirements

- **NFR-001 - Confidentialité**: zéro valeur secrète dans stores, arguments,
  labels, logs, métriques, UI ou artefacts de preuve.
- **NFR-002 - Moindre privilège**: seules les ressources explicitement
  approuvées pour le projet sont exposées.
- **NFR-003 - Reproductibilité**: image, extensions, profil et générations sont
  épinglés et vérifiables avant spawn.
- **NFR-004 - Réversibilité**: désactiver le profil et recréer retire montages et
  injections sans modifier le dépôt.
- **NFR-005 - Bornage**: nombre, taille de métadonnées et destinations des
  extensions et secrets sont bornés.
- **NFR-006 - Observabilité**: toute divergence produit une raison exploitable
  sans révéler la ressource.
- **NFR-007 - Maintenabilité**: aucun broker, Vault, mémoire globale, plugin
  manager ou dépendance externe n'est introduit.
- **NFR-008 - Portabilité**: le contrat de profil est fournisseur-neutre et
  validé sur les trois transports supportés.

### Key Entities

- **ProjectProfile**: composition approuvable d'agents, runtime, extensions et
  références secrètes.
- **ResolvedProjectProfile**: définition Bridget exacte et digestée.
- **ExtensionRef**: source, destination, version, digest et mode read-only.
- **SecretRef**: référence opaque, type, usage, génération et destination.
- **SecretBindingAttestation**: preuve de résolution sans valeur.
- **ProjectProfileApproval**: décision humaine locale et digest épinglé.

## Hors périmètre

- Secret individuel par agent dans un conteneur partagé.
- Broker de secrets dynamique, Vault, cloud secret manager ou rotation live.
- Conteneur exceptionnel par agent.
- Mémoire globale writable, RAG global ou synchronisation de connaissances.
- Téléchargement automatique de plugin ou skill depuis Internet.
- Hot reload des extensions ou secrets avec agents actifs.
- Analyse de confiance automatique du code d'un plugin.
- Support d'un backend autre que host/docker local.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100 % des champs du profil résolu apparaissent dans la vue
  d'approbation, hors valeurs secrètes qui n'y apparaissent jamais.
- **SC-002**: une modification d'un seul octet d'image, extension ou définition
  fait diverger le digest et refuse l'activation avant spawn.
- **SC-003**: aucun des scanners stores/logs/inspect/process args/projections ne
  retrouve les valeurs fixtures après 100 cycles.
- **SC-004**: deux projets aux profils différents n'exposent aucune extension ou
  SecretRef non commune explicitement approuvée.
- **SC-005**: Codex, Claude et Cursor utilisent le même contrat de profil et
  passent 100 % des oracles fournisseur-neutres.
- **SC-006**: une capability manquante refuse le spawn avant toute lecture ou
  montage secret dans 100 % des cas.
- **SC-007**: une rotation suivie d'une recréation rend l'ancienne génération
  inaccessible dans 100 % des probes.
- **SC-008**: un opérateur identifie en moins de 30 secondes le profil actif,
  les ressources exposées et la prochaine action.
- **SC-009**: toute tentative d'approbation UI/MCP/agent est impossible par
  absence de route ou commande.
- **SC-010**: un backend host refuse explicitement un profil Docker sans
  modifier son comportement historique.
- **SC-011**: aucune mémoire writable ou catalogue non approuvé n'est partagé
  entre projets.
- **SC-012**: aucune nouvelle dépendance externe ou service résident n'est
  ajouté.
- **SC-013**: toute mutation hors rotation d'une source secrète fixture est
  détectée avant lecture, montage ou spawn, tandis qu'aucune valeur ni digest
  de contenu secret n'est stocké dans les contrats, stores ou preuves.

## Assumptions et dépendances

- SPEC-066 est prouvée avec runtime ingress, recreate et rollback.
- Les agents d'un même projet sont coopératifs et partagent la confiance.
- Les secrets hôte sont préparés par l'opérateur sous une racine privée.
- Un credential qui doit être rafraîchi est copié vers un état privé projet ou
  traité par une future spec; un montage global writable est interdit.
- Le besoin de mémoire globale est différé jusqu'à des usages et règles de
  confidentialité mesurés.
