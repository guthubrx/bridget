# Correction221 — diagnostic expéditeur interne et préparation

État : deux tests ciblés PASS ; revue traitée, activation à consigner.
Base b721e29f9c32467dbe2b8f08dc98c255bcfbbd4d ; lot isolé de la session222.

## Incidents rapprochés du registre courant, sans mission rejouée

- bridget222-plan-contre-revue-20261101-02 : reçu admitted, tâche
  b47a61ea-1b0a-4383-ac1a-7ed0e9dda17e. Le demandeur doit relire cette tâche.
- b27b9714-e163-4f15-bbdb-cb82e81f5bbc : cancelled, cleanup_done=true,
  erreur remote_workspace_prepare:ResourceLimit. Aucun reçu de commande
  correspondant dans le store worker lu ; cela ne nomme pas la sous-cause.
- opus201-patrimoine-cultes-est8-20261101-01 : aucune ligne trouvée dans
  auto_admission_receipts, native_delegations ou native_delegation_refusals.
  Cette lecture du registre courant ne transforme pas guichet/idempotency_expired
  en reçu d’admission natif et ne déclenche aucun rejeu.
- 17b1a14f-47ff-474c-8aab-2256a5dc661e : cancelled, cleanup_done=true ;
  worker terminal/native_ready=true/exit0. Le refus a eu lieu lors de la remise
  interne. Le log du pont montre plusieurs refus agent_id déjà connecté vers
  07:50–07:51UTC, puis monuments exposé à07:51:32. Ces faits ne prouvent pas
  la sous-cause du refus initial, qui n’a pas été conservée.
- c2240df5 : ancien WorkerUnavailable, mis à jour à1793512076 ; f7ab1d73 :
  ancien terminal Claude inconnu de la ronde3, mis à jour à1793485610.
  Leurs notifications tardives ne prouvent pas un nouvel incident après les
  correctifs déjà chargés. Aucun état historique réécrit.
- dd2f2ed3 : result_available/error=null, résultat conservé. Les refus de
  variables/heredocs/hooks rapportés restent informatifs sans preuve d’un défaut
  Bridget dans les seules données reçues ; aucun droit accru. Chrome SIGTRAP
  reste distinct.

## Défaut confirmé et changement

send_as_with_result_authority crée déjà son rattachement auxiliaire interne
avant d’appeler le garde d’expéditeur. Le texte demandant de créer ce même
rattachement est donc trompeur sur ce chemin. La correction remappe seulement
ce Nack exact, pour un envoi sans autorité de résultat conservé et dont from
est le propriétaire interne. Sous le même verrou : route absente, instance
actuelle différente ou identité vivante non confirmée deviennent trois codes
fermés. Le refus reste un refus ; aucune réattribution, aucun grant, Start,
rejeu, reprise ou publication n’est ajouté.

task_view expose la prochaine action, la portée this_task, les prérequis et
la décision responsable. L’ancien texte donne cause=unobserved ; aucune
cause historique n’est reconstruite depuis l’état courant du parent.
Le ResourceLimit de préparation garde cause=unobserved et nomme les lectures
manquantes : file, bootstrap retenu, préparations et store. Il ne promet pas
qu’un drainage résout le problème et ne conseille pas d’arrêter autrui.

## Contrôles et revue

Deux tests spec221_sender PASS0,15s, via scripts/build.py cargo test filtré.
L’un modifie réellement les tables de connexion du fixture : instance changée
puis connexion absente ; Nack, aucune commande de contrôle, aucune demande
publiée et tâche inchangée octet pour octet. L’autre contrôle codes nouveaux,
historique inconnu, ResourceLimit inconnu, portée et recours humain sans mutation.
Une première compilation des tests a révélé leur module de fixture mal qualifié ;
qualification corrigée, code produit inchangé. Aucun troisième essai empilé.

Minimalisme : un seul fichier exécutable ; aucun store, service, protocole ou
capacité nouveau. Autorité de résultat conservé et chemins externes inchangés.
Le petit helper porte la cohérence du guidage, pas une couche de délégation.
Responsabilité future : ne pas traiter les codes de diagnostic comme un droit
de relance. Pas de nouvelle interface graphique : preuve technique suffisante.


Revue indépendante f9ecb01d : aucunP1, unP2 sur le message historique commun.
Corrigé : unobserved ne parle plus de l’identité actuelle et nomme directement
la preuve absente. Test enrichi sur cette phrase. RemarqueP3 de précédence
alignée sur live_connection_identity : absence de connexion avant différence
d’instance. Le test couvre aussi une route supprimée. Le chemin avec autorité
de résultat conservé est exclu explicitement de la remap ; cette revue n’a pas
rejoué un fixture d’autorité conservée, limite déclarée. ACK/clôture une fois.
Les Nack restent des chaînes dans le contrat existant ; task.projected_error
les conserve et task_view les projette en Value, sans enum réseau nouveau.
Delta final auto-audité, aucune nouvelle campagne de revue générale.
