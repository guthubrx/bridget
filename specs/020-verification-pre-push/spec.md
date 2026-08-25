# Spec 020 — Vérification transactionnelle avant envoi

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 020-verification-pre-push
Titre: Vérification transactionnelle avant envoi
Statut: In Progress
Priorité: P1
Tâches: 2/8 (25%)
Tests: 0/7 (0%)

Résumé:
- Contexte: un contrôle de message au moment de l'intégration ne voit pas les ancêtres importés par une branche.
- Objectif: refuser une transaction d'envoi dès qu'un commit nouvellement atteignable porte une trace d'autorat tiers.
- Dépendances: SPEC-018 pour l'activation après admission ; aucune pour le contrôle et ses témoins.

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-20-verification-pre-push`
**Created**: 2026-08-25
**Status**: In Progress
**Priority**: P1
**Dependencies**: SPEC-018 uniquement pour l'activation du hook après jury

## Intention produit

Un responsable peut envoyer plusieurs références en une seule opération. La
protection doit raisonner sur la transaction complète et sur tous les commits
qu'elle rend nouvellement atteignables, pas seulement sur une tête ou sur le
message du commit d'intégration.

La propriété validée est :

> Avant toute transaction de push, le hook calcule l'union de tous les commits
> que l'ensemble des mises à jour de références rend nouvellement atteignables
> par rapport à l'état distant observé. Si l'un de leurs messages porte une
> trace d'autorat tiers interdite, l'envoi entier est refusé. Si l'état distant,
> un objet nécessaire ou la plage ne peut pas être établi, l'envoi entier est
> refusé.

## Scénarios utilisateur et validation

### US1 — Refuser toute introduction interdite (P1)

Étant donné un dépôt distant observé, quand une branche neuve, un envoi forcé
ou une transaction multi-références introduit au moins un commit portant une
ligne de co-autorat, alors aucune référence de la transaction n'est envoyée.

Critères d'acceptation :

1. Une branche neuve sans amont est contrôlée.
2. Un envoi forcé est contrôlé sur sa nouvelle histoire.
3. Une transaction de plusieurs références est atomiquement refusée si une
   seule référence introduit un commit interdit.
4. Un nom de co-auteur inventé est détecté sans dépendre d'une liste d'outils.

### US2 — Préserver les envois légitimes (P1)

Étant donné un historique interdit déjà présent et explicitement accepté sur
le distant, quand l'envoi n'ajoute que des commits propres, alors il reste
autorisé. Une transaction composée de plusieurs références propres reste
également autorisée.

### US3 — Échouer sans ambiguïté (P1)

Quand le distant est inconnu, qu'un objet annoncé manque localement, que l'état
observé diverge de l'état annoncé ou que le calcul de portée échoue, alors
l'envoi entier est refusé avec un diagnostic actionnable.

### US4 — Prouver le filtre lui-même (P1)

Le banc appelle le filtre réellement utilisé par le hook avec une ligne de
co-autorat inconnue et démontre qu'une mutation syntaxique historique du motif
fait rougir ce témoin.

## Cas limites

- Une suppression de référence n'introduit aucun commit et ne fournit donc
  aucune tête positive à inspecter.
- Une référence peut désigner un tag annoté ; le commit épluché est inspecté.
- Une référence ne menant à aucun commit n'ajoute aucun commit à l'union.
- Un distant vide implique que toute l'histoire des nouvelles têtes est
  nouvellement atteignable.
- Une variation de casse ou des espaces avant le trailer ne doit pas contourner
  la règle.
- Les anciens commits interdits déjà atteignables depuis le distant ne doivent
  pas être rescannés comme nouveaux.

## Exigences fonctionnelles

- **FR-0201** — Le contrôle DOIT lire toutes les mises à jour reçues sur son
  entrée standard avant de prendre une décision.
- **FR-0202** — Le contrôle DOIT observer les références du distant nommé par
  l'opération, sans utiliser un amont de branche implicite.
- **FR-0203** — Le contrôle DOIT refuser si l'état distant observé contredit
  l'ancien identifiant annoncé pour une référence mise à jour.
- **FR-0204** — Le contrôle DOIT vérifier la présence locale de chaque objet
  nécessaire au calcul et refuser si l'un manque.
- **FR-0205** — Le contrôle DOIT calculer une seule union des commits
  atteignables depuis toutes les nouvelles têtes et non atteignables depuis
  l'état distant observé.
- **FR-0206** — Le contrôle DOIT inspecter le message complet de chaque commit
  de cette union.
- **FR-0207** — Toute ligne de co-autorat, insensible à la casse et tolérant
  les espaces de présentation, DOIT être interdite sans énumérer de noms.
- **FR-0208** — Toute erreur d'observation, de lecture ou de calcul DOIT
  produire un refus non nul et un diagnostic sur la sortie d'erreur.
- **FR-0209** — Un seul commit interdit DOIT refuser toute la transaction,
  y compris ses autres références propres.
- **FR-0210** — Le hook ne DOIT PAS être activé depuis un espace de travail
  avant admission par jury et intégration dans la branche de référence.

## Critères mesurables

- **SC-0201** — Les sept témoins validés passent sans réseau externe.
- **SC-0202** — Une transaction contenant 100 % de commits propres est
  acceptée ; une transaction contenant au moins un commit interdit est refusée.
- **SC-0203** — Chaque mode d'indétermination testé produit un refus et un
  message expliquant l'action corrective.
- **SC-0204** — Le calcul visite au plus une fois l'union des commits
  introduits ; sa complexité est linéaire dans le nombre de mises à jour,
  références distantes et commits introduits.

## Hors périmètre

- Réécrire ou nettoyer l'historique existant.
- Modifier les protections ou l'offre du forgeur distant.
- Activer le hook avant jury et intégration.
- Remplacer la barrière côté réception, qui reste une couche distincte.
- Déduire l'identité d'un outil à partir du nom du co-auteur.

## Hypothèses

- Le client dispose de Bash et de Git.
- Le distant est interrogeable au moment de l'envoi.
- Les objets annoncés par le distant ont été récupérés localement ; sinon le
  comportement attendu est un refus invitant à actualiser le dépôt.
