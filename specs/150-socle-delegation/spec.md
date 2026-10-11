# Session150 - Socle de la délégation native

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 150-socle-delegation
Titre: Socle de la délégation native (rétention, admission, pause, délai)
Statut: Implémentée (code sur disque, non commité, en validation)
Priorité: P1
Tâches: 39/39 cochées (tasks.md)
Tests: native150 (daemon + transport) ; non-régression 148/149/087 126/0/1 (validation/tests-150.md)

Résumé:
- Contexte: La délégation native (148-149) garde toutes ses tâches sans fin. Après 4096 délégations cumulées, elle est bloquée pour toujours. La pause du référent ne bloque pas les nouvelles délégations. Le délai de réponse est fixe.
- Objectif: Rendre la délégation native durable dans le temps, sans changer son contrat d'appel.
- Dépendances: SPEC-148, SPEC-149, ADR 007, ADR 027

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-150-socle-delegation`
**Created**: 2026-10-11
**Status**: Implémentée (non commitée)
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles
en communication et délégation entre agents ». Cette session est le prérequis
des sessions 151 à 158.

## Besoin et périmètre

La délégation native permet à un agent de confier une tâche à un sous-agent en
un appel. Le moteur tourne dans le daemon Bridget, avec ou sans T3. Quatre
défauts empêchent d'en faire un service durable.

**Défaut 1 - Historique sans fin.** Chaque tâche reste enregistrée pour toujours.
Aucune purge n'existe pour les tâches terminées, leurs reçus d'annulation et
leur trace de lignée.

**Défaut 2 - Blocage définitif à 4096 délégations.** L'admission compte toutes
les tâches enregistrées, terminées comprises
(`crates/bridget-daemon/src/daemon/native_delegation.rs:442`). Dès 4096
délégations cumulées, toute nouvelle délégation est refusée (`task_limit`), et
rien ne libère jamais de place. L'ouverture du magasin refuse aussi plus de
4096 lignes (`crates/bridget-daemon/src/delegation_lineage.rs:80`). Ce second
cas n'est pas atteignable par le chemin normal, car l'admission plafonne avant.
Le magasin réel contient 0 tâche le 2026-10-11 : le risque est latent.

**Défaut 3 - La pause n'arrête pas les nouvelles délégations.** La pause du
référent (ADR 027) n'est pas consultée à l'admission d'une délégation native.
Un agent peut donc lancer de nouveaux sous-agents pendant une pause.

**Défaut 4 - Délai unique.** Le délai de réponse d'une mission vaut toujours
3600 s. Un parent ne peut ni raccourcir le délai d'une tâche courte, ni
l'allonger pour une tâche longue.

Cette session corrige ces quatre défauts. Elle ne change pas le contrat
d'appel existant : un appel sans nouveau paramètre garde exactement le même
comportement qu'en 149.

**Constat transmis à la session152.** Une pause fait aussi échouer pour de bon
les missions natives en cours. Deux chemins y mènent : le refus de remise causé
par l'interruption (`daemon.rs:18080` → `native_delegation.rs:820`) et la
lecture de l'état `interrupted` (`native_delegation.rs:1193-1203`). La reprise
ADR 027 exclut les missions natives (`daemon.rs:4251-4258`). Corriger ce point
demande de reprendre une mission avec son identifiant d'origine et de geler
plusieurs horloges. C'est le mécanisme de la session152 (reprise après panne).
Cette correction y est donc traitée, avec la suspension des lancements et des
remises de mission pendant une pause.

**Hors périmètre** : suspension et reprise des missions pendant une pause
(session152) ; attente synchrone, boîte de fin de tâche, résultat structuré
(session151) ; budgets de tokens ou de coût (session158) ; purge des tables
d'exécution, d'idempotence et de flotte liées aux tâches (dette notée).

## Scénarios utilisateur

### US1 - La délégation reste disponible dans la durée (P1)

Un parent a déjà délégué plus de 4096 tâches au fil des semaines. Il délègue
encore. L'appel réussit. Les limites actives restent seules en vigueur :
128 tâches actives au total, 16 enfants actifs par parent, profondeur 8.

### US2 - Les tâches terminées sont purgées sans perte utile (P1)

Un arbre de tâches terminé depuis plus que la durée de conservation est
purgé : la tâche racine (sans parent), tous ses descendants, leurs reçus
d'annulation et leur trace de lignée. Un arbre n'est jamais purgé tant que :

- une de ses tâches est active ;
- un résultat ou un avis d'échec n'a pas été accusé par le destinataire ;
- un de ses sous-agents tourne encore ou peut encore être relancé.

Deux arbres du même parent sont indépendants : un arbre actif ne bloque pas la
purge d'un autre.

Le rejeu d'une demande dont l'arbre a été purgé rend un refus explicite
(`request_expired`). Il ne crée jamais une nouvelle tâche en silence. La vue
Lineage T3 ne montre pas de tâche fantôme après une purge.

### US3 - La pause refuse les nouvelles délégations (P1)

Pendant la pause, un agent appelle `bridget_delegate` avec une nouvelle clé.
L'appel est refusé avec un refus nommé (`referent_paused`). Aucune tâche n'est
créée et la clé de rejeu n'est pas consommée. Après la levée, le même appel
avec la même clé réussit normalement.

Le rejeu d'une clé déjà admise rend la tâche existante, même pendant la pause.
La lecture de statut, la lecture du catalogue et l'annulation restent
possibles. L'annulation humaine reste possible.

### US4 - Délai de réponse choisi par tâche (P2)

Le parent peut fixer le délai de réponse d'une tâche à la délégation. Sans
valeur, le délai reste 3600 s. Le catalogue annonce le délai par défaut, le
minimum et le maximum. Une valeur hors bornes est refusée avec un refus nommé,
sans tâche créée et sans valeur corrigée en silence.

Le délai fait partie de l'enveloppe de la demande. Rejouer la même clé avec un
autre délai rend `envelope_mismatch`. Un délai explicite d'enfant ne peut pas
dépasser le temps qui reste à sa tâche parente : sinon, refus nommé. Sans délai
explicite, rien ne change par rapport à 149. Le suivi de la
demande de mission (relances) utilise le même délai.

### US5 - Aucun changement pour l'existant (P1)

Les tâches actives au moment de la mise à jour continuent avec leurs règles
d'origine, y compris leur délai de 3600 s. Les contrats MCP et CLI existants
restent compatibles. La vue Lineage T3 garde ses états et ses clés.

## Exigences fonctionnelles

**Rétention et purge**

- **FR-001** : L'unité de purge est l'arbre : une tâche sans parent et tous ses
  descendants.
- **FR-002** : Un arbre est purgeable si toutes ses tâches sont terminales et
  nettoyées, si chaque résultat ou avis d'échec a été remis et accusé par le
  destinataire (preuve durable), et si aucun de ses sous-agents n'est connecté,
  en cours de lancement ou marqué pour tourner dans la flotte. Ce contrôle est
  refait au moment de la suppression. Avant la suppression, chaque sous-agent
  est retiré de la flotte, pour qu'il ne puisse plus être relancé.
- **FR-003** : Un arbre purgeable est purgé quand sa dernière fin date de plus
  de 7 jours.
- **FR-004** : Le nombre total de tâches conservées reste au plus 4096, la
  limite déjà publiée dans les contrats 148 et 149. Quand ce total est atteint,
  les arbres purgeables les plus anciens sont purgés en premier, même avant
  7 jours.
- **FR-005** : Si 4096 tâches sont conservées et qu'aucun arbre n'est purgeable,
  une nouvelle délégation est refusée avec `retention_saturated`. Le daemon
  continue de servir les tâches actives.
- **FR-006** : La purge d'un arbre retire ses tâches, leurs reçus d'annulation
  et leur trace de lignée en une seule opération atomique.
- **FR-007** : Le rejeu d'une clé dont la tâche a été purgée rend
  `request_expired` pendant 30 jours après la purge. Il ne crée aucune tâche.
  Après ce délai, la clé redevient libre au passage d'entretien suivant
  (1 h au plus), comme les envois idempotents.
- **FR-008** : Après une purge validée, les lecteurs Lineage concernés sont
  prévenus et refont une lecture complète, sans tâche fantôme.
- **FR-009** : Chaque passage d'entretien supprime au plus 128 arbres. Il ne
  bloque pas le traitement des tâches actives. Exception assumée : à
  l'admission, la purge de borne retire seulement les arbres nécessaires pour
  repasser sous 4096, sans limite de 128. La base ne dépasse pas 4096 par le
  chemin normal, donc ce cas reste rare et borné par l'excédent.

**Admission et ouverture**

- **FR-010** : L'admission ne refuse plus à cause de l'historique cumulé. Elle
  purge d'abord les arbres purgeables si la borne est atteinte, puis ne refuse
  que pour les limites actives (`task_limit`) ou la saturation
  (`retention_saturated`).
- **FR-011** : L'ouverture du magasin de délégation réussit quel que soit le
  nombre de tâches enregistrées.

**Pause du référent**

- **FR-012** : Pendant la pause, `bridget_delegate` avec une nouvelle clé rend
  `referent_paused`, sans créer de tâche et sans consommer la clé de rejeu.
- **FR-013** : Pendant la pause, le rejeu d'une clé admise, la lecture de
  statut, le catalogue et les annulations (agent et humain) restent
  disponibles.

**Délai de réponse**

- **FR-014** : `bridget_delegate` accepte un délai de réponse optionnel
  (`reply_timeout`, en secondes). Sans valeur, le délai est 3600 s.
- **FR-015** : `bridget_capabilities` annonce le délai par défaut, le minimum
  (60 s) et le maximum (86 400 s).
- **FR-016** : Une valeur hors bornes rend `reply_timeout_out_of_range`, sans
  tâche créée.
- **FR-017** : Le délai fait partie de l'enveloppe idempotente de la demande.
- **FR-018** : Un délai explicite d'enfant ne dépasse pas le temps restant de
  sa tâche parente. Sinon, `reply_timeout_exceeds_parent`. Sans délai
  explicite, le comportement 149 reste inchangé.
- **FR-019** : Le délai choisi s'applique à l'échéance de la mission et au
  suivi de la demande de mission.

**Compatibilité et documentation**

- **FR-020** : Les tâches actives avant la mise à jour gardent leurs règles
  d'origine.
- **FR-021** : Les refus nouveaux sont listés et documentés dans
  `docs/delegation-native.md` et dans la skill Bridget : `referent_paused`,
  `request_expired`, `retention_saturated`, `reply_timeout_out_of_range`,
  `reply_timeout_exceeds_parent`, et `control_state_unavailable` (état de contrôle
  illisible, refus fermé à l'admission).

## Critères de succès

- **SC-001** : Après 5000 délégations terminées et purgeables, une nouvelle
  délégation réussit. Aujourd'hui, elle est refusée dès la 4097e.
- **SC-002** : En fonctionnement continu, le nombre de tâches conservées ne
  dépasse jamais 4096.
- **SC-003** : 0 arbre purgé alors qu'un résultat n'est pas accusé, qu'une
  tâche est active ou qu'un sous-agent tourne, sur l'ensemble des scénarios de
  test.
- **SC-004** : Un arbre actif d'un parent ne bloque pas la purge d'un autre
  arbre terminé du même parent.
- **SC-005** : 100 % des délégations demandées avec une nouvelle clé pendant une
  pause sont refusées avec `referent_paused`, et 0 tâche n'est créée.
- **SC-006** : Le rejeu d'une clé purgée depuis moins de 30 jours rend
  `request_expired` dans 100 % des cas, sans création de tâche.
- **SC-007** : Un appel sans nouveau paramètre produit la même tâche et le même
  résultat qu'en 149 (tests de non-régression 148-149 verts).
- **SC-008** : Avec 10 000 lignes enregistrées, l'ouverture du magasin réussit.

## Entités et hypothèses

**Entités**

- **Tâche native** : demande de délégation d'un parent vers un enfant, avec
  état, délai, lignée, résultat et clé de rejeu.
- **Arbre de tâches** : une tâche sans parent et ses descendants.
- **Reçu d'annulation** : trace idempotente d'une annulation humaine.
- **Pierre tombale** : trace d'une clé de rejeu purgée, gardée 30 jours.
- **Projection de lignée** : vue lue par T3, avec génération et séquence.
- **État de contrôle** : pause ou activité du référent.

**Hypothèses**

- La durée de conservation par défaut est de 7 jours, comme l'historique du
  ledger.
- La borne totale reste 4096 tâches, valeur déjà publiée aux lecteurs T3.
- Le délai de 86 400 s maximum couvre les tâches longues connues.
- Un résultat dont la remise reste indéterminée bloque la purge de son arbre.
  Si ces arbres saturent la borne, la délégation est refusée
  (`retention_saturated`) plutôt que de perdre un résultat.
- Les tests suivent la règle du projet : `umask 077`, environnement privé,
  racine temporaire courte.
- La livraison suit les sessions 147 à 149 : fichiers sur disque, sans relance
  automatique des services.
