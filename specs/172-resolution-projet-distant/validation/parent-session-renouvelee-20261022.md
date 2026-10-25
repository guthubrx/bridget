# Reprise après renouvellement de session fournisseur — 22 octobre 2026

## Résultat livré

Source dbec83e64c8cccfa8b228d2e82ea3156855a0538, build 172-dbec83e6.
Le maître Mac PID 47204 charge le correctif. Binaire signé, SHA-256 :
7bd41314c7b8318ad3155f383f8b6fac48824a4df8a1063844119dc978edca32.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md#extrait-1).

## Défaut et correction

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md#extrait-2).

Pour une preuve provider_turn réattestée, la comparaison neutralise maintenant
provider_session_id, comme run_id et revision. Les deux gardes de propriétaire,
l'instance, le répertoire, les modes et la politique restent vérifiés.
Le chemin native_wrapper garde son égalité stricte. Le snapshot durable et la
preuve du worker ne changent pas. Aucun droit ni mécanisme de relance ajouté.

## Validation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md#extrait-3).

Recette réelle après activation :
Task 492cf951-4ad6-4f1e-97e8-32a2b3cb9706,
enfant 0e3a0548-eb6c-44dc-9d88-8fab529eca4d, Cartae, lecture seule.
Ronde 1 : hostname cartae, version 0.4.1, BRIDGET172_SESSION_R1_OK.
Ronde 2 : REMOTE_RESULT_SLOT_TIMEOUT à 15 s, BRIDGET172_SESSION_R2_OK.
Les deux résultats complets ont été lus puis acquittés. Un rejeu strict de la
ronde 2 rend la même Task, le même enfant, la même ronde et le même résultat.
La recette seule a été clôturée ; ses résultats restent disponibles.

Limite : cette recette ne force pas le renouvellement réel d'un processus Codex.
Cette variante est prouvée par les fixtures ciblées. Les gardes et la remise
distante réelle sont vérifiées séparément. Aucun succès global n'est déduit.

## Admissions métier conservées

28da9844-0695-4f99-9159-b9beeab9bd6d : résultat de ronde 2 conservé,
cleanup_done=true et guard_released=true déjà observés avant activation.
Aucune continuation métier relancée.

aaba664f-b773-44b3-849b-8e651af88c1f : ancien refus de réservation antérieur
au budget de 15 s, tâche annulée et nettoyage confirmé. Aucun remplacement.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md#extrait-4).

## Cas distinct : Codex lancé directement dans un terminal

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md#extrait-5).

Deux processus locaux lancés directement par codex, sans marqueur Bridget,
ont été observés. La commande du terminal de recherche signalé n'a pas été
confirmée. Ce correctif ne prétend pas inscrire automatiquement un Codex brut.

## Preuves et conservation

Preuves privées Mac :
/Users/moi/.cache/bridget-parent-session172
Preuves Cartae :
/home/moi/.cache/bridget-parent-session172
Rollback Mac :
/Users/moi/.cache/bridget-parent-session172/bridget-before

Les preuves comprennent activation.json, live.json, green.json,
review-approved.json et abandoned-admission-preflight.json.
Aucune base, conversation ni preuve supprimée. Le fichier AGENTS.md modifié
par l'utilisateur est préservé.
