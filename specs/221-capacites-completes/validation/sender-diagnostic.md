# Correction221 — diagnostic expéditeur interne et préparation

État : correctif de diagnostic intégré, poussé et actif sur les sept services.
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

Deux tests spec221_sender PASS0,13s après le delta final, via scripts/build.py cargo test filtré.
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


## Livraison active

Source fonctionnelle e948eef59de471f41897e03d647bbb090014a4ff intégrée à main
et poussée. Manifeste357sources identiques au commit, un seul fichier
exécutable changé. Assemblages finaux Mac1min10s et Linux1min57s.
La nuance historique de revue a été prise en compte avant toute activation ;
les assemblages antérieurs n’ont pas été chargés.

Build221-sender-guide-r2 :
- Linux SHA14f656bf55f4c3a229d93fd07c068d45f24b66c9a2bd249da5d195124dc1e669.
- Mac SHA3aa93e29716226471c86faaaab39af5a1aa7266823f2049562ee5564464f3571.
- Cartae maître960490/mobile960607/worker961752.
- SYS worker1284999 ; KS worker1650281 ; Mac worker56371/pont56518.
Sept images réellement chargées hachées. Coupure maître2,53s.
Wrappers Cartae2500061/naissance270585460 et SYS1278163/naissance69239797
conservés. Le résultat de56a0a893 reste disponible ; aucun service métier,
ancienne mission, compte, moteur ou droit relancé/modifié.

Retour vers221-program-compat-r2 contrôlé dans les deux sens pour les formats
et par clients check/validate-report. Pointeurs rollback natifs Cartae/Mac
écrits atomiquement sous verrou, sauvegardes et actifs conservés, updaters repris.
Après exposition du fil dans le log du pont : une bridget_session révision33
puis capabilities PASS45options/gel=false. Aucun cycle de reprise d’application,
aucun nouvel outil MCP. Préavis puis retour transmis aux propriétaires ;
les remises in_flight ne sont pas présentées comme des lectures attestées.

## Fin Git et entretien

Main Mac/Cartae alignées au distant ; main des magasins mirror.git/repository
SYS/KS mise à jour par bundle, HEAD détachés et origins conservés.
T187 clôturée, programme57/187InProgress. Les catégories restantes ne sont
pas déclarées terminées par ce correctif de diagnostic.

Trois copies de transfert retirées :125078904octets. Actif, retour et preuves
conservés. installed appelé sur les deux stagings : Mac refuse la propriété
non enregistrée, SYS garde les caches car l’observation des processus est
incomplète. Worktree221 toujours responsable de la suite ; worktree de revue
conservé faute de contrôle d’usage complet. Aucun travail non intégré supprimé.

Limite de diagnostic : l’origine exacte de l’ancien refus d’attestation17b
et de la capacitéb27 n’est pas reconstruite. Le changement livre un guidage
honnête et précis, pas une requalification rétroactive de leurs missions.
