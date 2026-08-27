# Spécification 048 — Identité d’instance du daemon

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 048-identite-instance-daemon
Titre: Attester l’instance du daemon sur chaque connexion
Statut: Implemented
Priorité: P0
Tâches: 4/4 (100%)
Tests: 4/4 (100%)

Résumé:
- Contexte: La garde Maicie 037 atteste une connexion préliminaire ; un daemon peut être remplacé avant les connexions d’effet sans que BUILD_ID ne le révèle.
- Objectif: Faire circuler une identité nouvelle à chaque démarrage du daemon et l’attester sur toute connexion Client ou Service.
- Exécution: Générer l’identité à la création de l’état du daemon, l’ajouter au rapport d’identité et prouver le remplacement local-vers-local sur socket Unix.
- Risque principal: Réutiliser BUILD_ID ou générer la valeur par réponse rendrait la provenance inutilisable.
- Mitigation: Comparer deux états successifs sur le même socket, hôte, binaire et chemin de base ; exiger égalité dans un état et inégalité après redémarrage.
- Validation: Quatre oracles, dont le redémarrage local-vers-local et son mutant.
- Dépendances: SPEC-037-refus-ecriture-federee, SPEC-012-contrat-client-idempotent

Fichiers:
- spec.md: ✓ (specs/048-identite-instance-daemon/spec.md)
- tasks.md: ✓ (specs/048-identite-instance-daemon/tasks.md)
- plan.md: ✓ (specs/048-identite-instance-daemon/plan.md)
- implementation.md: ✓ (specs/048-identite-instance-daemon/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-048-identite-instance-daemon`
**Created**: 2026-08-27
**Status**: Implemented
**Priority**: P0
**Dependencies**: SPEC-037-refus-ecriture-federee, SPEC-012-contrat-client-idempotent
**Base mesurée**: `b00d417182622152bd9cc191ff98a7e1dca45ce4`

## Contexte et problème

Le rapport d’identité actuel atteste seulement `host` et `db_path`. Le seul
autre identifiant disponible, `BUILD_ID`, est le commit intégré à la
compilation : deux daemons successifs du même binaire sur la même machine et
la même base portent donc exactement les mêmes valeurs.

Cette absence bloque la levée de M1 de la session 037. Un socket Unix peut être
remplacé après l’attestation préliminaire, puis servir les connexions d’effet
depuis un daemon différent. Même un remplacement local-vers-local reste
indiscernable aujourd’hui.

## Principes directeurs

- **Identité de démarrage, pas identité de compilation** : elle naît une fois
  avec l’état du daemon et change au prochain démarrage.
- **Attestation par connexion** : Client et Service peuvent obtenir le même
  rapport de l’instance actuellement jointe ; aucune valeur ne vient du client.
- **Aucune persistance** : réemployer l’identité après redémarrage annulerait
  la propriété recherchée.
- **Portée minimale** : ce lot produit le contrat et le daemon. Les APIs
  Maicie qui rendront cette provenance obligatoire relèvent du lot de levée M1.

## Scénarios et tests

### Scénario 1 — Attester une connexion d’effet (P0)

Un composant qui vient de négocier une connexion Client ou Service demande
l’identité du daemon réellement joint. Il reçoit l’hôte, le chemin de base et
l’identifiant d’instance de ce même daemon.

**Test indépendant** : les deux rôles demandent le rapport et obtiennent les
trois valeurs conservées dans l’état du daemon.

1. **Étant donné** un daemon en cours, **quand** une connexion Client demande
   son identité, **alors** le rapport contient un identifiant non vide.
2. **Étant donné** ce même daemon, **quand** une connexion Service le demande,
   **alors** elle reçoit le même identifiant.

### Scénario 2 — Détecter le redémarrage local-vers-local (P0)

Un opérateur conserve le même hôte, le même binaire, le même chemin de base et
le même chemin de socket, mais le daemon est redémarré entre deux connexions.

**Test indépendant** : sur une vraie socket Unix, le premier listener est
fermé, le chemin est réutilisé par un nouvel état du daemon, puis les deux
rapports sont comparés.

1. **Étant donné** deux démarrages successifs avec la même configuration,
   **quand** chacun répond à la sonde sur le même chemin de socket,
   **alors** hôte, chemin de base et BUILD_ID restent égaux mais les identités
   d’instance sont différentes.
2. **Étant donné** un seul démarrage, **quand** deux connexions demandent le
   rapport, **alors** les identités d’instance sont égales.

## Cas limites

- Un daemon antérieur qui ne fournit pas le nouveau champ n’est pas une
  instance attestée ; les consommateurs futurs doivent refuser cette absence.
- Un redémarrage très rapide ne réutilise jamais volontairement une identité,
  même si le PID, le binaire et la configuration sont identiques.
- Une requête hors rôle conserve les refus actuels ; le nouveau champ ne crée
  aucun droit ni nouvelle négociation.

## Exigences

- **FR-001** : chaque création de l’état du daemon DOIT produire une identité
  d’instance non vide et la conserver jusqu’à sa destruction.
- **FR-002** : `DaemonIdentityReport` DOIT transporter cette identité en plus
  de l’hôte et du chemin de base.
- **FR-003** : les rôles Client et Service DOIVENT pouvoir obtenir le même
  rapport sur chaque connexion autorisée, sans négociation supplémentaire ni
  effet d’écriture.
- **FR-004** : deux rapports d’un même état DOIVENT porter la même identité ;
  deux états créés successivement avec la même configuration DOIVENT porter
  des identités différentes.
- **FR-005** : `BUILD_ID` demeure l’identifiant de compilation et NE DOIT PAS
  être utilisé comme identité d’instance.

## Entités

- **Identité d’instance** : valeur opaque, non vide et créée au démarrage ;
  elle distingue deux états successifs du daemon même quand toutes leurs
  métadonnées persistantes sont égales.
- **Rapport d’identité du daemon** : attestation filaire de l’hôte, du chemin
  de base et de l’identité d’instance du daemon qui répond sur la connexion.

## Critères de succès

- **SC-001** : le tour de sérialisation du rapport conserve les trois valeurs.
- **SC-002** : 2/2 rôles autorisés (Client, Service) reçoivent l’identité de
  l’état qui les sert.
- **SC-003** : l’oracle sur socket Unix constate exactement trois égalités
  (hôte, chemin de base, BUILD_ID) et une inégalité (identité d’instance) entre
  deux démarrages successifs.
- **SC-004** : le mutant qui réutilise une valeur stable au redémarrage fait
  échouer l’oracle local-vers-local, puis le code restauré le repasse.

## Hors périmètre

- rendre les clients Maicie incapables de créer une connexion non attestée ;
- modifier `ClientWelcome`, les capacités, les autorisations ou le schéma ;
- persister l’identité dans SQLite, la déduire du chemin de socket ou du PID ;
- corriger les six ouvertures d’effet de M1 : elles consommeront ce contrat
  dans le lot suivant.

## Résultats mesurés

- L’oracle réseau était rouge sur la base :
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 565 filtered out`.
  Il échouait exclusivement sur l’absence de `instance_id`.
- Après correctif, les deux oracles de rôle et de redémarrage ferment à
  `2 passed; 0 failed`; le tour filaire ferme à `1 passed; 0 failed`.
- Le mutant qui remplace l’UUID de démarrage par `BUILD_ID` échoue sur
  l’inégalité attendue après redémarrage :
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 565 filtered out`.
  Le code UUID restauré repasse l’oracle à `1 passed; 0 failed`.
- Le transport complet ferme à `181 passed; 1 ignored; 0 failed`. Le contrat
  Maicie ciblé ferme à `13 passed; 0 failed`; son univers complet compilé
  contient 405 tests.
- La campagne daemon complète (univers 566) est **incomplète** : deux rouges
  hors 048 (`enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant`
  et `sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique`) ont
  été suivis d’un blocage supérieur à 115 secondes. Seuls le parent cargo et
  son enfant de cette campagne ont reçu SIGTERM. Aucun verdict global n’est
  revendiqué pour cette campagne.
