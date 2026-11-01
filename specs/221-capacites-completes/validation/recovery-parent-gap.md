# Qualification du refus17 — reprise avant retour du parent

Demande : comparer les droits figés de
opus2d-r55-reseau-integre-revue-20261101-17 au contexte courant du parent.
Investigation en lecture seule. Aucun reçu, droit, compte ou mission modifié.

## Preuves corrélées

Registre maître auto_admission_receipts :
owner87515984-f549-49ae-ab9c-a6c47dada0e5, metadataV2, coordinator_epoch101,
created_at1793529557, updated_at1793529593, state=refused,
refusal_code=permission_not_inherited, task_id=null.
Choix conservé : c001/claude-ksstore/sonnet. Sourcecanon :
/Users/moi/Nextcloud/10.Scripts/69.opus2D/.worktrees/158-reprise-rapide-r55,
Shared, expected_base_commit d773f70e3e60a23730f6a13aef601d99f1526ac6.

Chronologie UTC du 1 novembre2026 :
- 10:39:17 : reçu créé et choix figé.
- 10:39:26 : recontrôle du candidat élu, WorkerUnavailable surKS.
- 10:39:38 : arrêt du maître précédent dans la maintenance222.
- 10:39:40 : nouveau maître PID1256995 démarré, image222-specialists-r2.
- 10:39:42 : pont du fil relance monde interrompu, reconnexion annoncée.
- 10:39:53 : reçu déjà refusé durablement permission_not_inherited.
- 10:40:32 : fil relance monde réexposé sous son identité native.
- 10:41:57 : le rejeu strict du propriétaire relit le refus durable.

La réattestation et le rejeu n’ont donc pas provoqué la transition àrefused.
Le reçu avait été fermé avant le retour du lien principal.

Lecture officielle T3 du fil e4999bf4-8f05-401b-bdc2-a9bb32b0b9f7 :
run actif ordinal1225 commencé10:29:15, full-access/default, modèle gpt-6.1-sol,
même fil/projet. Le parent rapporte never/révision30/cwd exact ; la lecture
officielle confirme le run et les deux modes, pas un snapshot complet des
anciennes politiques fournisseur.

## Cause de classification dans le code chargé

Le fichier du WT222 chargé conserve auto_recovery_context et
recover_routing_admission/recover_auto_admissions. Le contexte de récupération
rend permission_not_inherited si la route du parent manque, si son identité
n’est pas vivante, ou si ses faits/root_permission ne sont pas disponibles.
L’appelant transforme ce code en refus durable du reçu d’époque précédente.

Cette branche précède la comparaison d’une politique fournisseur. Dans
l’intervalle de reconnexion observé, le code confond ainsi preuve d’origine
indisponible et divergence de droits constatée. Le message reçu « les droits
actuels ne correspondent pas aux droits figés » ne constitue pas une preuve
de changement de compte, droits ou posture.

Le canon du reçu fige l’enveloppe et ses candidats ; aucun snapshot complet
de ProviderPermissions du parent n’y est présent. Les permission_digest
d’observation sont les projections des candidats, pas une ancienne politique
parent permettant de prouver une différence avec le contexte courant.
Aucune politique ancienne n’est reconstruite ni déclaration de révocation inventée.

## Transmission au responsable et état du correctif

Défaut transmis au responsable222 f09593b3-433e-4075-8781-e3093cdc2225,
propriétaire du code actuellement chargé et de la maintenance en cours.
Message bridget221-recovery17-owner222-fix-20261101-01,
deliveryb9d3c907-1481-4bf2-b999-893c9623e091, in_flight àl’envoi.
Diagnostic demandé transmis au parent par
bridget221-recovery17-qualified-20261101-01,
deliveryfd483bc6-3212-45e3-ada2-f3df6ae6aaf8, in_flight àl’envoi.
Aucune lecture ou acceptation de ces messages n’est présumée.

Correctif demandé : distinguer preuve transitoirement indisponible et refus de
droits établi, préserver la fermeture de toute admission sans preuve, réutiliser
les reçus/canons/choix existants. Examiner maintien pending/chosen pendant retour
du parent/réattestation ou rejeu propriétaire, avec bornes explicites ; aucune
résurrection du reçu17 déjà refusé. Test ciblé : reprise d’un reçu chosen avant
retour du parent puis retour avec droits valides ; zéro tâche avant preuve.
Cas révocation réelle séparé. Aucun second ordonnanceur ou grant ajouté.

Le correctif n’est PAS déclaré livré par ce diagnostic. Sa réalisation suit
le propriétaire222 pour éviter une édition concurrente et une activation d’un
ancien221 incompatible avec le nouveau floor specialists.requests.v1.
Le refus17 et ses IDs restent intacts. Aucune mission de remplacement ou revue
métier lancée ; les préparations froides indépendantes continuent.
