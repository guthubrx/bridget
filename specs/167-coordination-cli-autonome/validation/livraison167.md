# Reçu de livraison — SPEC-167

Date : 2026-10-15
Statut : Implemented
Tâches : 28/28
Pipeline : Livré

## Résultat

Un agent Bridget authentifié peut maintenant créer un agent enfant et lui confier une mission avec `bridget delegate <MISSION>`. Le groupe `bridget task` couvre la liste, l’état, l’attente, la continuation, la réponse, l’annulation et l’accusé de lecture.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/167-coordination-cli-autonome/validation/livraison167.md#extrait-1).

Un terminal humain sans identité d’agent reçoit `identity_not_found` avant tout effet métier. Aucune identité humaine synthétique n’est créée.

## Git et publication

- Source des binaires : `24a6507a99520bf2245caf67fa69cbac40bd300f`.
- Préparation de release : `f97453641b939023cfad6d572ca2bfc810f6a472`.
- Les commits sont intégrés et poussés sur `github/main`.
- Tag annoté : `v0.3.0`, cible exacte de la source des binaires.
- Release : https://github.com/guthubrx/bridget/releases/tag/v0.3.0
- État GitHub : publiée, non brouillon, non préversion et marquée `Latest`.

## Actifs

- Client macOS signé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
- SHA-256 macOS : `4783e8cb4e3389d71c40d6ecb5e0d89bb98f2d7af14e2275c7e48f6b92ed98cd`.
- Le lien actif `/Users/moi/.local/bin/bridget` vise ce binaire et annonce `bridget 0.3.0`.
- Archive Linux x86_64 : `/Users/moi/.cache/bridget-release-0.3.0/bridget-0.3.0-linux-x86_64.tar.gz`.
- SHA-256 de l’archive Linux : `3f4455dc380c855002178df49a4f0b6447bd7323d00f2acc090ca83eeb0296d8`.
- SHA-256 du binaire Linux : `989c27777d34c57fab9658f9aee2a05409be243bd567daec61597def305a8089`.
- Le digest publié par GitHub correspond à l’archive locale.

## Validation

- Tests ciblés : 87 PASS et 2 ignored.
- Recette E2E native par le parcours natif Bridget : création, rejeu, réponse corrélée, annulation et reprise PASS.
- Client installé : `task list --json` rend un objet machine valide.
- Refus sans identité : code retour 1, `identity_not_found`, aucune tâche créée.
- Build release macOS et Linux verrouillés et hors ligne : conformes.
- Signature macOS stricte : conforme.
- Audit v14 : A, aucun finding, validateur 0 erreur et 0 avertissement.
- `git diff --check` : conforme après retrait des espaces finaux des artefacts SpecKit.

La suite globale initiale a rendu 1 949 PASS, 91 FAIL et 16 ignored. Un échec appartenait au diff et a été corrigé. Son oracle ciblé passe. Les 90 autres échecs restent hors diff ou liés à l’environnement. La suite complète n’a pas été rejouée. Aucun succès global n’est revendiqué.

## État actif

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/167-coordination-cli-autonome/validation/livraison167.md#extrait-2).

## Retour arrière

Sauvegarde avant construction et remplacement :

`/Users/moi/.cache/bridget-install167.eT471KDV`

Binaire précédent :

`/Users/moi/.cache/bridget-install167.eT471KDV/bridget.previous`

Les deux plists avant livraison sont conservés dans le même dossier. Aucune base n’a été modifiée ou supprimée.
