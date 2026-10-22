# Reprise après renouvellement de session fournisseur — 22 octobre 2026

## Résultat livré

Source dbec83e64c8cccfa8b228d2e82ea3156855a0538, build 172-dbec83e6.
Le maître Mac PID 47204 charge le correctif. Binaire signé, SHA-256 :
7bd41314c7b8318ad3155f383f8b6fac48824a4df8a1063844119dc978edca32.

Le worker Cartae PID 2219043 reste sur 172-76b682d6. Aucun redémarrage du worker
ni de T3. Pont 898, application 50681 et serveur 50799 préservés.
La mission becec88c-197d-4653-8452-0b936d04945e a gardé son wrapper
2479540, sa naissance 219447002 et les processus fournisseur mesurés pendant
l'activation. Son état après activation est working, sans erreur.

## Défaut et correction

La continuation d'un même propriétaire T3 était refusée quand Codex renouvelait
son identifiant de session. Le fil, l'instance fournisseur et la politique
restaient identiques. Le comparateur traitait pourtant cet identifiant technique
comme un changement de droits.

Pour une preuve provider_turn réattestée, la comparaison neutralise maintenant
provider_session_id, comme run_id et revision. Les deux gardes de propriétaire,
l'instance, le répertoire, les modes et la politique restent vérifiés.
Le chemin native_wrapper garde son égalité stricte. Le snapshot durable et la
preuve du worker ne changent pas. Aucun droit ni mécanisme de relance ajouté.

Référence T3 lue :
/Users/moi/11.Repositories/t3code/apps/server/src/orchestration/Layers/ProviderRuntimeIngestion.ts,
source ef86c2bbc3388be8fc593db23a323e3d80b58509, lignes 1483–1503, 1530–1532,
1581–1585 et 1630–1643. T3 conserve thread.id lors d'un nouveau session.started.
Bridget doit conserver l'autorité de ce même fil, sans transposer ce principe
aux identités de wrappers natifs.

## Validation

Neuf tests ciblés PASS, sans suite complète. RED puis GREEN sur la comparaison.
Les tests couvrent le renouvellement, les changements de politique refusés, les
deux gardes, la même ronde rejouée et la liaison T3 opaque.
La revue 9e5b4b1c-cfbb-449a-9f8b-5809f2b69f7f rend APPROVE. Résultat lu,
acquitté et garde clôturée.

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

L'admission monuments-notre-dame-qualification-intrants-reels-20261022-01
était restée chosen, sans Task, depuis 13:52 UTC. Le propriétaire a confirmé son
abandon dans t3-8f8ffab4-341b-4441-9851-571267a62568. Sa récupération native au
redémarrage l'a refusée avec permission_not_inherited ; TaskID toujours nul.
Aucun rejeu manuel, changement de clé ni mutation SQL opérateur.
La qualification sur une nouvelle base reste au propriétaire.

## Cas distinct : Codex lancé directement dans un terminal

identity_not_found signifie qu'aucune identité Bridget vérifiable n'est liée
à cet appel. La seule présence du connecteur MCP ne fait pas cette inscription.
Le parcours interactif actuel est bridget codex, ou bridget codex resume pour
reprendre une conversation existante. Il ne dépend pas de T3.

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
