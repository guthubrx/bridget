# Spécification 051 — Borner la sonde d’identité du daemon

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 051-borner-sonde-identite-daemon
Titre: Distinguer le silence du daemon de son absence
Statut: En cours
Priorité: P0
Tâches: 1/4 (25%)
Tests: 1/4 (25%)

Résumé:
- Contexte: La sonde d’identité attend sans limite après qu’un pair Unix a accepté la connexion.
- Objectif: Rendre cette lecture bornée et distinguer un daemon absent, une identité non attestée et une identité momentanément indisponible.
- Risque principal: Convertir une expiration en absence rassurante, puis poursuivre avec une flotte vide ou une seconde connexion.
- Mitigation: Faire remonter l’indisponibilité comme erreur explicite et éprouver le vrai binaire contre un pair qui accepte puis se tait.
- Validation: Oracle réseau rouge sur la base, contrôles des états voisins et mutant retirant la borne.
- Dépendances: SPEC-048-identite-instance-daemon, SPEC-039-validation-identite-mcp

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
- implementation.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-051-borner-sonde-identite-daemon`
**Created**: 2026-08-27
**Status**: En cours
**Priority**: P0
**Dependencies**: SPEC-048-identite-instance-daemon, SPEC-039-validation-identite-mcp
**Base gelée**: `f893b1e4a010a333dc8ba2348df9b8ee8e45663c`

## Contexte et problème

La lecture de l’identité du daemon n’a aujourd’hui aucune échéance. Un pair
qui accepte la socket Unix puis ne répond ni ne ferme suspend donc le client
indéfiniment. L’existence du chemin de socket ne prouve ni que le daemon
répond, ni que son identité est absente.

Trois observations appellent trois décisions différentes :

1. aucune connexion n’est possible : aucun daemon n’est observable ;
2. un daemon répond au protocole connu, mais ne fournit pas le rapport récent :
   il est présent et son identité n’est pas attestée ;
3. le pair accepte, puis la lecture expire : la source d’identité est
   indisponible et aucun verdict d’absence ne peut être rendu.

## Principes directeurs

- **L’incertitude reste visible** : une expiration n’est jamais transformée en
  daemon absent, en identité absente ou en annuaire vide.
- **Échec avant la suite** : après expiration de la sonde d’identité, aucune
  seconde connexion de statut n’est ouverte.
- **Compatibilité déclarée** : un daemon ancien qui répond sans attestation
  conserve son état « présent, non attesté ».
- **Borne locale** : le correctif ne modifie ni le protocole ni les délais des
  autres commandes.

## Scénarios et tests

### US1 — Refuser le silence sans bloquer (P0)

Un opérateur interroge un chemin de socket où un pair accepte la connexion,
lit la requête puis garde la connexion ouverte sans répondre.

**Test indépendant** : le vrai binaire est lancé contre une socket jetable ;
le pair capture la trame reçue puis se tait. Le processus doit terminer dans
la borne, rendre un code non nul et nommer l’identité indisponible ainsi que
le délai dépassé.

### US2 — Conserver les deux états voisins (P0)

L’absence réelle de daemon reste distincte d’un daemon ancien qui répond sans
attestation complète.

**Test indépendant** : une socket absente rend le statut hors ligne ; un pair
compatible qui répond sans le champ d’identité récent reste reconnu comme
daemon présent mais non attesté.

### US3 — Ne pas inventer une flotte vide (P0)

Les consommateurs du statut reçoivent l’indisponibilité de la sonde au lieu
d’une structure vide. Une carte de reprise ou un inventaire de nettoyage ne
peut donc pas conclure « aucun agent » à partir d’un pair silencieux.

**Test indépendant** : l’erreur de sonde traverse la frontière du statut et
est rendue par la commande avant toute collecte ultérieure.

## Cas limites

- Une fermeture avant la négociation complète est une source indisponible,
  pas la preuve qu’aucun daemon n’existe.
- Une réponse ancienne mais décodable prouve la présence du daemon tout en
  laissant l’identité non attestée.
- Une réponse au rapport ancien dont seul le champ récent manque conserve la
  compatibilité ; une lecture qui expire reste une erreur.
- La socket peut disparaître entre le test de présence et la connexion : une
  absence ou un refus de connexion conserve le verdict hors ligne.

## Exigences fonctionnelles

- **FR-051-01** : toute lecture de la négociation d’identité DOIT porter une
  échéance finie avant le premier appel bloquant.
- **FR-051-02** : l’expiration DOIT produire une erreur qui nomme la sonde
  d’identité et le délai dépassé.
- **FR-051-03** : après expiration, le client NE DOIT PAS ouvrir la connexion
  suivante de collecte du statut.
- **FR-051-04** : l’absence de socket ou le refus de connexion DOIT rester un
  état hors ligne, distinct d’une lecture expirée.
- **FR-051-05** : un daemon qui répond mais ne porte pas l’attestation récente
  DOIT rester présent avec une identité non attestée.
- **FR-051-06** : les commandes et outils qui consomment le statut NE DOIVENT
  PAS transformer l’indisponibilité en liste vide ou en succès silencieux.
- **FR-051-07** : le correctif NE DOIT ajouter ni message filaire, ni option de
  ligne de commande, ni dépendance.

## Entités

- **Daemon absent** : aucune connexion au chemin de socket n’aboutit.
- **Identité non attestée** : le daemon répond, mais son protocole ne fournit
  pas les attributs récents permettant l’attestation.
- **Identité indisponible** : le pair a accepté la connexion, mais la sonde ne
  peut obtenir une réponse dans la borne déclarée.

## Critères de succès

- **SC-051-01** : le pair muet reçoit réellement la trame de négociation et le
  vrai client termine avec un code non nul dans la borne du banc.
- **SC-051-02** : le diagnostic du pair muet contient à la fois « identité du
  daemon indisponible » et « délai de lecture ».
- **SC-051-03** : exactement une connexion est acceptée dans le scénario muet ;
  aucune collecte ultérieure n’est amorcée.
- **SC-051-04** : les contrôles « socket absente » et « daemon ancien qui
  répond » conservent leurs verdicts respectifs.
- **SC-051-05** : retirer la borne fait échouer ou dépasser l’échéance de
  l’oracle muet, puis la restauration le repasse.

## Hors périmètre

- borner toutes les autres lectures de la CLI ;
- modifier le protocole ou la génération de l’identité d’instance ;
- changer la politique de redémarrage du service ;
- corriger l’arrêt coopératif du daemon ou nettoyer ses processus orphelins ;
- introduire une relance automatique après expiration.

## Mesure initiale

- Univers ciblé listé avant le tir : 1 test, 0 benchmark.
- Le vrai binaire envoie bien `RoleHandshake(Client)` au pair muet.
- La base dépasse l’échéance externe et rend :
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s`.
- Après SIGTERM de l’enfant possédé par le banc, aucun processus ni répertoire
  `bg51-*` ne subsiste.
