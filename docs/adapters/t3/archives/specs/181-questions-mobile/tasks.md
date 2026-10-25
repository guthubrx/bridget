# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/tasks.md](../../../../../../specs/181-questions-mobile/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T003 Écrire les tests de validation, propriété, révision, course et rejeu du contrat dans crates/bridget-daemon/src/user_input.rs et les lancer avant la réalisation complète ; états et refus observables.
- [x] T004 Implémenter le magasin natif et ses transitions bornées dans crates/bridget-daemon/src/user_input.rs, store.rs, protocol.rs et daemon.rs ; les tests T003 passent sans affaiblir087/155.
- [x] T005 Exposer l’outil MCP natif et la commande humaine terminal dans mcp.rs, mcp_scopes.rs, cli.rs et communication/client.rs ; test d’identité négatif et réponse reçue par le parcours natif Bridget.
- [x] T006 Réemployer le callback Codex natif pour les agents principaux dans wrapper.rs/codex_app_server.rs et daemon/native_child_requests.rs ; le routage155 reste couvert et inchangé.
- [x] T007 Traiter AskUserQuestion dans claude_stream_json.rs via le même contrat ; conserver le refus des autres permissions et vérifier corrélation/réponse native dans les tests du transport.
- [x] T008 Lire les demandes T3 dans t3code_contract_v2.rs et conserver la forme/questions originales ; tests de demande résolue/non reprenable/étrangère et borne.
- [x] T009 Brancher publication, réponse et réconciliation de l’adaptateur dans t3code.rs/t3code_contract.rs ; commandId stable, clôture externe et aucune réinjection de prompt testés.
- [x] T010 Étendre les opérations Questions/HTTP dans daemon/mobile_questions.rs et mobile/http.rs ; tests Rust autorisé/refusé, entrée périmée et origine appareil.

<a id="extrait-2"></a>

## Extrait 2

- [x] T017 Ajouter et lancer les tests instrumentés Android du formulaire, reprise, agrandissement, résultat vocal contrôlé et notification ; rapports conservés hors Git.
- [x] T018 Recette réelle sur émulateur avec producteur/commandant terminal natif, par le parcours natif Bridget : choix/libre/complément, fermeture et réponse reçue une seule fois ; reçus181 privés.
- [x] T019 Recette réelle notification en arrière-plan et dictée ; qualifier le moteur vocal présent et les limites éventuelles, sans présenter une injection de résultat comme reconnaissance réelle.
- [x] T020 Recette contrôlée de l’adaptateur T3 et modes terminal ; préserver les vrais fils, aucun redémarrage fournisseur/T3 pour répondre ; modes indisponibles affichés avec guidage.
- [x] T021 Lancer les tests ciblés Rust/JVM, lint et builds concernés ; fault injection réseau/reprise, révocation et course entre clients ; aucune régression nouvelle bloquante.
- [x] T022 Produire et vérifier le diaporama des circuits réels dans preuves-corrections/2026-10-25-questions-mobile/preuves.html ; rapports, manifestes, captures et reçus privés protégés.

<a id="extrait-3"></a>

## Extrait 3

- [x] T023 Préserver les reçus existants à capacité32 dans l’adaptateur T3 ; test avant/après et fermeture explicite.
