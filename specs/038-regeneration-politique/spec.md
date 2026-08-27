# Feature Specification: Régénération explicite de la politique du greffe

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 038-regeneration-politique
Titre: Régénération explicite de la politique du greffe
Statut: Ready for Review
Priorité: P0
Tâches: 9/10 (90%)
Tests: 17/17 (100%)

Résumé:
- Contexte: la politique lie chaque droit à une instance qui change à chaque redémarrage d'agent ; elle devient donc silencieusement périmée au fil de la journée.
- Objectif: renouveler explicitement les instances vivantes sans autoriser de nouveau principal ni révoquer un agent momentanément arrêté.
- Exécution: inventorier les marqueurs sur chaque hôte, vérifier la complétude des sources nommées par la politique, puis remplacer atomiquement les seules instances vivantes approuvées.
- Risque principal: une collecte locale ou vide pourrait être prise pour l'inventaire complet et révoquer les agents distants.
- Mitigation: chaque principal approuvé nomme sa source de marqueurs ; toute source absente, incomplète ou sans marqueur vivant interdit la réécriture.
- Validation: ancienne instance retirée, principal absent conservé et signalé, génération croissante, fichier 0600, puis mutation réellement autorisée avec la nouvelle instance.
- Dépendances: SPEC-026

Fichiers:
- spec.md: ✓ (specs/038-regeneration-politique/spec.md)
- tasks.md: ✓ (specs/038-regeneration-politique/tasks.md)
- plan.md: ✓ (specs/038-regeneration-politique/plan.md)
- implementation.md: ✓ (specs/038-regeneration-politique/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-038-regeneration-politique`
**Created**: 2026-08-27
**Status**: Ready for Review
**Priority**: P0
**Dependencies**: SPEC-026

---

## Contexte et Problème

La garde livrée par la session 026 refuse par défaut toute mutation durable du
greffe. Un droit est accordé au couple `(principal, instance_id)`, et
`instance_id` est généré à la naissance du processus agent. Un redémarrage
normal change donc l'instance sans modifier le principal : la politique reste
syntaxiquement valide mais le droit devient inutilisable.

La mise en service du 27 août 2026 a exigé onze principaux, dont dix distants.
Une politique exacte le matin devient partiellement fausse le soir, agent par
agent. Le refus public uniforme est volontaire ; sans outil explicite,
l'exploitant ne peut cependant pas distinguer une instance périmée d'un droit
absent sans ouvrir le journal privé.

Deux raccourcis sont exclus : renouveler automatiquement au `Register`
auto-enrôlerait une identité encore déclarative, tandis qu'un joker d'instance
annulerait la protection contre le remplacement d'une instance. Le remède est
un acte d'exploitation explicite, contrôlable et atomique.

## Principes Directeurs

- **Renouveler n'est pas autoriser** : seuls les principaux et actions déjà
  présents dans la politique peuvent survivre à la régénération.
- **Absence n'est pas révocation** : un principal approuvé mais momentanément
  absent des marqueurs est conservé octet-logiquement et signalé comme mort.
- **Présence remplace l'ancienne instance** : lorsqu'un principal approuvé est
  vivant, sa liste d'instances devient exactement sa nouvelle instance ; les
  anciennes disparaissent.
- **Complétude déclarée dans la donnée autoritative** : chaque principal nomme
  la source qui doit l'observer. Une invocation ne peut pas réduire cette liste.
- **Échec fermé avant écriture** : zéro marqueur vivant, source manquante,
  inventaire incomplet, ambiguïté ou politique invalide laissent le fichier
  original inchangé.
- **Aucun secret implicite** : le chemin de politique est toujours explicite ;
  l'outil n'ouvre jamais par défaut la politique réelle en service.

## User Scenarios & Testing

### User Story 1 — Renouveler un agent redémarré (Priority: P0)

L'exploitant possède une politique privée approuvant un agent. Après le
redémarrage de celui-ci, il collecte ses marqueurs vivants et applique une
régénération explicite. Le même principal conserve les mêmes actions, la même
expiration et le même état de révocation, mais seule la nouvelle instance peut
désormais muter le greffe.

**Why this priority**: sans cette opération, la garde en service se désarme à
chaque redémarrage normal.

**Independent Test**: régénérer une fixture privée portant `instance-ancienne`
depuis un inventaire vivant portant `instance-nouvelle`, puis vérifier qu'une
mutation durable passe avec la nouvelle instance et échoue avec l'ancienne.

**Acceptance Scenarios**:

1. **Given** un principal approuvé avec une instance ancienne et une source
   complète qui observe sa nouvelle instance, **When** l'exploitant applique la
   régénération, **Then** la génération croît strictement et l'ancienne instance
   n'existe plus dans le fichier 0600.
2. **Given** la politique régénérée, **When** la nouvelle instance réalise une
   mutation autorisée, **Then** l'effet durable a lieu et l'ancienne instance
   reste refusée avant effet.

### User Story 2 — Composer les inventaires locaux et distants (Priority: P0)

L'exploitant collecte un inventaire sur chaque machine qui héberge des agents,
y compris par SSH pour les machines distantes, puis présente l'ensemble à
l'outil central. L'inventaire est vérifié contre les sources exigées par la
politique avant toute réécriture.

**Why this priority**: dix des onze principaux mesurés sont distants ; une
collecte locale seule donnerait un faux succès et laisserait leurs droits morts.

**Independent Test**: fournir deux sources attendues, omettre d'abord la source
distante et vérifier l'absence totale d'écriture, puis fournir les deux
inventaires complets et vérifier la mise à jour.

**Acceptance Scenarios**:

1. **Given** une politique qui référence deux sources, **When** une seule est
   fournie, **Then** l'outil refuse avant réécriture et nomme la source absente.
2. **Given** une source vide ou dont un marqueur vivant est illisible, **When**
   l'inventaire est demandé, **Then** aucun inventaire déclaré complet n'est
   produit et la politique reste inchangée.
3. **Given** un inventaire produit sur une machine distante puis rapatrié par
   un canal d'exploitation, **When** sa source correspond à celle déclarée par
   la politique, **Then** il participe à la même vérification que l'inventaire
   local sans comparer ses PID depuis la machine du daemon.

### User Story 3 — Ne pas révoquer un agent arrêté (Priority: P1)

Un principal approuvé n'apparaît dans aucun marqueur vivant de sa source, par
exemple parce que son agent est momentanément arrêté. La régénération des autres
principaux ne doit ni le retirer ni masquer son absence.

**Why this priority**: un arrêt momentané n'est pas une décision de révocation ;
les confondre transformerait la maintenance en retrait silencieux de droits.

**Independent Test**: régénérer un principal vivant en présence d'un second
principal approuvé absent ; vérifier que le second est restitué à l'identique et
figure dans la liste des entrées mortes.

**Acceptance Scenarios**:

1. **Given** un principal approuvé absent des marqueurs vivants d'une source
   par ailleurs complète, **When** un autre principal est renouvelé, **Then** le
   principal absent est conservé et signalé comme mort.
2. **Given** un marqueur vivant pour un principal absent de la politique,
   **When** la régénération est calculée, **Then** ce principal est signalé mais
   n'est jamais ajouté à la politique.

### Edge Cases

- Deux marqueurs vivants portent le même principal : refus pour ambiguïté.
- Un principal apparaît sur une source différente de celle qu'il déclare :
  refus, même s'il apparaît aussi sur la bonne source.
- Une politique historique ne porte pas encore `marker_source` : la garde
  continue à la lire, mais l'outil refuse de la régénérer avant annotation.
- Un inventaire est trop ancien ou daté dans le futur : refus avant écriture.
- Un inventaire n'est pas un fichier privé appartenant à l'utilisateur effectif
  : refus sur le descripteur ouvert avant toute réécriture.
- La politique régénérable ou son verrou possède plusieurs liens physiques :
  refus avant plan, car un verrou attaché à un nom ne protège pas ses alias.
- Plusieurs instances existantes portent des expirations ou révocations
  différentes : refus ; l'outil ne choisit jamais quel droit conserver.
- La génération vaut la valeur maximale : refus plutôt que retour à zéro.
- L'outil est relancé sans changement d'instance : aucune réécriture et aucune
  génération artificielle.
- Une erreur survient avant le renommage : l'original reste octet-identique.
  Une erreur survient après : l'outil relit et valide le chemin final, puis rend
  une issue indéterminée distincte au lieu d'affirmer un échec ou un succès.

## Requirements

### Functional Requirements

- **FR-001**: L'outil DOIT exiger un chemin explicite vers une politique privée
  existante et NE DOIT PAS utiliser implicitement la politique en service.
- **FR-002**: Chaque principal régénérable DOIT nommer exactement une source de
  marqueurs ; la garde de mutation DOIT rester compatible avec une politique v1
  historique qui ne porte pas encore cette annotation.
- **FR-003**: Le scanner DOIT vérifier localement chaque marqueur contre le PID,
  la naissance du processus et le fichier de nom avant de le déclarer vivant.
- **FR-004**: Un inventaire DOIT identifier sa source, son instant d'observation,
  ses marqueurs vivants et ses marqueurs périmés ; il ne peut être complet si
  une entrée n'a pas été lue ou validée. Son fichier DOIT être régulier,
  appartenir à l'utilisateur effectif et interdire tout accès au groupe et aux
  autres ; ces attributs sont vérifiés sur le descripteur déjà ouvert.
- **FR-005**: Zéro marqueur vivant dans une source DOIT produire une erreur et
  NE DOIT jamais produire une politique vide ou une réécriture partielle.
- **FR-006**: La régénération DOIT exiger exactement toutes les sources nommées
  par les principaux approuvés et refuser toute source manquante, dupliquée,
  inattendue, incomplète ou trop ancienne.
- **FR-007**: Pour un principal approuvé observé vivant sur sa source, l'outil
  DOIT remplacer toutes ses anciennes instances par exactement la nouvelle,
  sans changer ses actions, son expiration ni son état de révocation.
- **FR-008**: Pour un principal approuvé absent d'une source complète, l'outil
  DOIT conserver son entrée et la signaler comme morte ; il NE DOIT PAS la
  retirer.
- **FR-009**: Un principal observé mais non approuvé DOIT être signalé et NE
  DOIT jamais être ajouté, même si son marqueur est valide.
- **FR-010**: La génération DOIT croître strictement à chaque réécriture ; une
  exécution sans changement NE DOIT PAS réécrire le fichier.
- **FR-011**: La réécriture DOIT être sérialisée entre processus, atomique,
  synchronisée sur disque et produire un fichier régulier en mode `0600`. La
  politique régénérable et son verrou DOIVENT chacun posséder exactement une
  entrée de répertoire afin qu'un même inode ne puisse recevoir deux verrous
  indépendants par deux noms absolus.
- **FR-012**: Toute erreur antérieure au renommage DOIT laisser la politique
  originale inchangée. Toute erreur postérieure DOIT provoquer une relecture et
  une validation du chemin final, puis rendre une issue explicitement
  indéterminée et distincte d'un échec propre ; aucun retour arrière fictif ne
  doit être annoncé.
- **FR-013**: Le mode par défaut DOIT être une prévisualisation ; une option
  explicite est nécessaire pour appliquer la réécriture.
- **FR-014**: Le rapport DOIT nommer les principaux renouvelés, inchangés,
  morts et non approuvés, ainsi que les générations avant/après, sans exposer la
  clé d'attestation.
- **FR-015**: Le même scanner DOIT pouvoir être exécuté sur un hôte distant et
  son inventaire rapatrié ; aucun PID distant ne doit être validé localement.

### Non-Functional Requirements

- **NFR-001**: Complexité O(n log n), où `n` est le nombre total de principaux
  et marqueurs, sans recherche linéaire répétée dans les boucles.
- **NFR-002**: Aucun nouveau secret, jeton ou droit ne doit être généré par
  l'outil ; la clé existante est seulement conservée dans le fichier privé.
- **NFR-003**: Les erreurs doivent être explicites et stables pour un opérateur,
  sans rendre le contenu secret de la politique.
- **NFR-004**: Linux et macOS doivent employer leur source existante de naissance
  de processus ; aucun comportement silencieux n'est admis sur une autre
  plateforme.

### Key Entities

- **Source de marqueurs** : identifiant stable d'une machine ou d'un domaine de
  collecte, associé à un ou plusieurs principaux approuvés.
- **Inventaire** : observation bornée dans le temps, produite sur la machine qui
  héberge les PID, contenant marqueurs vivants et périmés.
- **Plan de régénération** : différence calculée entre politique approuvée et
  inventaires complets, avant toute écriture.
- **Rapport de régénération** : preuve non secrète des remplacements, absences,
  refus d'enrôlement et générations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100 % des anciennes instances d'un principal vivant renouvelé
  disparaissent de la politique appliquée, et exactement une nouvelle demeure.
- **SC-002**: 100 % des sources absentes, incomplètes ou vides empêchent toute
  modification octet du fichier original.
- **SC-003**: 100 % des principaux approuvés mais absents sont conservés et
  listés dans le rapport ; 0 principal inconnu est ajouté.
- **SC-004**: Après régénération, au moins une mutation autorisée produit son
  effet durable avec la nouvelle instance, tandis que l'ancienne est refusée.
- **SC-005**: Toute réécriture réussie produit une génération supérieure de un,
  un fichier `0600` et aucun état intermédiaire observable au chemin final.

## Assumptions

- La déclaration `Register` reste forgeable par un client local du même compte ;
  la session 038 ne prétend pas établir l'identité et conserve la borne de 026.
- Un principal possède normalement une seule sémantique de grant. Si plusieurs
  instances existantes divergent sur expiration ou révocation, une décision
  humaine est nécessaire et l'outil refuse.
- Le transport de l'inventaire distant passe par le canal SSH d'exploitation ;
  la session n'ajoute ni découverte réseau ni exécution distante automatique.
- Les horloges des hôtes sont suffisamment synchronisées pour refuser un
  inventaire vieux de plus de cinq minutes.
