# Audit de réutilisation frais —158A

## Decision

Statut: PASS (anti-doublons du plan seulement). Date: 2026-10-12T08:56:12.786977+00:00
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/specs/158-reveils-controle
Base livrée: bd37e927eee0b28f058df9d6a6992e38665a6ad0. Le reçu157 est vérifié par son SHA6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Huit fichiers sources de réemploi sont byte-identiques aux blobs de cette base. Ce PASS ne vaut ni GO, ni baseline158, ni test, ni implémentation.

## Synthese

| Métrique | Valeur |
|---|---:|
| Items extraits et audités |14/14|
| Duplications évidentes non arbitrées |0|
| Tâches158A existantes |22|
| Noms de tests explicites, hors2filtres |44|
| Outils du catalogue157 |30|
| Tests exécutés par cetowner |0|

## Reutilisations correctement identifiees

| Item du plan | Décision/existant | Preuve fraîche | Justification/limite |
|---|---|---|---|
|Protocole contrôle6variantes|Étendre protocol.rs ; aucun schémaDelegate modifié|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-transport/src/protocol.rs|Contrat158 ; fallback/substitution157 préservés.|
|Négociation et autoritéhumaine|Réutiliser ControlStateV1 puis control_client_actor|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/daemon.rs:10713|Gardes15900–15940 ; capacitéaccordée17565–17598 n’estpasautorité.|
|CLI/terminal|Réutiliser require_interactive_terminal/cmd_control|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/cli.rs:5961|Fonctions6096 ; aucun nouveau transport.|
|Vueglobale|Réutiliser tasks ; créer overview_page projectionfermée|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation.rs:758|VueLineageexistante exigeT3(native_lineage.rs56), donc nonéquivalente ; borne4096.|
|Arrêtpersistant|Nouvelleligne native_delegation_halt dans magasinexistant|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/referent_control.rs:43|control_state paused n’estpas halt ; CHECKcontrol_events fermé4genres, pasmigrationdestructrice.|
|Reçus256|Réutiliser le patron commande idempotente ; table spécialisée prévue|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:470|human_cancel reçuroot/request_id estautrecontrat ; nepasl’utiliser comme reçu globalhalt.|
|Admissionpause+halt|Étendre dans referent_control ; NativeDelegation existe|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/referent_control.rs:107|Pure ; même puits ND1060–1090 aprèsrejeu, avantparent_fact.|
|Annulationhumaine|Réutiliser human_cancel et tick|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:470|Le reçu représente cancelling ; pasarrêtphysique confirmé. CycleSTOP/cleanup149–155 conservé.|
|Scope/ToolScope/table|Création mcp_scopes.rs justifiée contrat30outils|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:2247|Aucun équivalent toolscope ; annotations seules sont hints, pasplafond.|
|scope_of par action|Créationdanslemoduleprévu|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:708|Journal avec to envoie ; classifierproviderpermissions155 a uneautreresponsabilité.|
|Plafonddiscovery|Réutiliser for_child/index et champsposture/cleanup|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:126|IndexUNIQUEchild existe ; for_child146 ; O(logn+B). AcquisitionparCallerScopeinterne, identitéinscrite/booléen/erreurfermée ; aucuneDBcliente.|
|Inventaireoutils157|Réutiliser tools()+delegation_mcp::tools|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:2597|30nomsactuels :22+8 ; pas catalogueneuf.|
|Catalogue/refushalt|Étendre snapshotCatalogueexistante|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/daemon/native_delegation.rs:1058|Ne paschangerordre droits/cible/quota/fallback, sélectionProbe épingléeReserve/Commit.|
|Docs/ADR/tests|Réutiliserdocs/fixtures/recettebaselineexistants|/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/specs/158-reveils-controle/tasks.md:17|22tâches/44noms ; zéro servicedépendancebudget/réveil158B ajouté.|

## Existant potentiellement pertinent non mentionne

Index UNIQUE native_delegations_child déjàcréé par migration_lineage (/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:126). L'ancienne hypothèse d'index absent est retirée après cette lecture. Ne créer ni index ni cache avant la mesureT017. Le lecteur for_child bénéficie de cetindex ; identité enfant unique. DelegationStore::open (/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation.rs:296) crée/migre en transaction immédiate : ce constructeur n'estpas un lecteurMCP sans effet. Ce point est traité dans Analyze/prérequisT017.

## Duplications evidentes

Aucune. LedgerScope/CommunicationDirectoryScope sont des portées de données, pas des classes d'effets des outils. Le classifier155 décide du relais d'une demande fournisseur depuis un snapshot ; il ne classe pas30outilsMCP. Les reçushuman_cancel et control_events ne remplacent pas le reçu decommandehalt avec réponse historique.

## Memoires et regles applicables

AGENTS et ponts constitution/standards lus. ConstitutionXVIII : coûtstore et vuebornée ; XIX : réemployeravantcréer, passervice/Saga/index/cachegratuit ; XX : nepasprésenter hints/CI commeconfinementOS oureçuprecommit comme arrêtphysique. Recetteprivée, ROOT seulCargo/Git/install. Mémoirecanonique /Users/moi/.Codex/projects/bridget/memory/MEMORY.md absente ; aucuncontenureconstruit.

## Specs livrees applicables

087/ADR027 : plan decontrôle et gardeautonome ;149 : acteurattesté, root_owner/human_cancel etSTOP ;150 : rejeuavantadmission, rétention4096 ;151 : wait/ACK ;152/153 : enfantgardé/rondes ;154 : worktreeProbe/Reserve/Commit ;155 : fermetureavantcleanup et Respond ;156 : queue/guidage ;157 : cataloguequota/fallback/substitution. Cesinvariants restentauxpuits existants. Aucunréveil/tour/token158B misdansles22tâches.

## Journal de recherche

Recherches nom ETresponsabilité via rg sur8sourcesprincipales ; lecture ciblée des fonctions dont les plages sont dans readmanifest. Inventaire outils extrait des déclarations mcp2247–2650 et delegation_mcp67–80, sans compilation. Comparaison des8blobs à Gitbd37e927. Noms nouveaux scope_of/overview_page/native_delegation_halt absents des sources lues ; motsScope présents ont des responsabilitésdistinctes.

## Arbitrages

ROOT a retenu cancelled=nombre d'arbres actifs visés par la demande, figéavantCOMMIT. Patchdocseulementcache ; champwireconservé, aucunSaga/champdurable/ID/scénario neuf. Réponse replay conserve halted historique et ne modifie pasl'étatcourant. Pertepartiellecrash toujoursassumée.

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée.
- [x] Chaque item extrait a sa ligne de réemploi/justification.
- [x] Règles pertinentes lues.
- [x] Fonctions reprises de la base157 livrée vérifiées.
- [x] Créations nécessaires justifiées dans le périmètre22tâches.

## Limites

Préflightcanonique check-prerequisites.sh absent, RC127 ; FEATURE_DIR explicitement donné parROOT, artefacts présents. Aucunscriptde remplacement créé. Analyze et baselineT001 restent des gatesdistincts. Lecture168Bbyteidentique confirmée, pas évaluation des futures fonctionnalitésB. Potentielminimalisme : zéro ligne d'implémentation158 à supprimer à ce stade (aucun code158écrit) ; pas d'estimationdevolumenonmesurée.

## Décisions finalesROOT après relecture

CallerScope ajoute une seule opération dans l’enveloppe native déjà existante, pas un outil MCP ni un service. Contrainte de sécurité écrite : dispatcher30outils garde un plafond validé par le daemon de l’identité inscrite, sansDBcliente ni repli surpanne. Delegate/Continue composentpause+halt ; matrice4combinaisonsdéjàprévue,22tâches/44nomsconservés. T012réemploiesnapshotroot/index/annulation : FnOnce premièreapplication, countviséprecommit, verroujusqu’àhuman_cancel, receipt historique aurejeu. MetadataOverviewOption etrefuseClientRejectedréemploientlesframesprévuessansnouvellevariante. Aucunanti-doublonouarbitrage restant.

BaselineROOT253P/0F/2I,217sourcesstables,351,95sRC0 vérifiéeviareçus, aucuntestexécutéparcetowner. NouvelleopérationjustifiéeparFR024 ; pasnouveauDTO/table/framework/helpergénérique.
