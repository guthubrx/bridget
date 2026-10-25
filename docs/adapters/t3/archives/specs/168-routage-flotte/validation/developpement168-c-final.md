# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/developpement168-c-final.md](../../../../../../../specs/168-routage-flotte/validation/developpement168-c-final.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- AutoDelegateV2 est raccordé au vrai puits natif. Les deux refus provisoires ont été retirés.
- Les cinq axes sont filtrés par le registre. Un effort imposé ferme supported_efforts dans le snapshot de demande ; le catalogue public garde la liste complète.
- Le catalogue utilise AgentRegistry::routing_catalogue(), avec version1, mêmeUID Unix, rôle Client, CLIENT_CONTRACT_VERSION et RoutingCatalogueV1 négociée. Aucune session d’agent ni autorité T3.
- Les probes distants V2 restent groupés par machine. Une ancienne capacité reçoit un refus précis. Aucun Prepare pendant le classement.
- HostResourceCollector est enregistré et réemployé au tick local existant. Pas de thread de collecte ajouté.
- Les ressources contrôlent bornes, source_observed_at, received_at et époque. Les inconnus et les mesures désactivées restent neutres.
- ProviderLimits lie les faits à connexion/instance/définition/époque ; la lecture exige l’instance vivante et des fenêtres fraîches. Aucune somme entre comptes.
- Le score calcule une fois les termes machine/connexion, puis trie une fois. Continuité<=20, CPU/RAM/charge<=20 chacun, quota<=5, reset<=3, compilation déclarée<=10 uniquement pour workload=compilation.
- La saturation fraîche est retirée avant les décideurs si une autre machine admissible est attestée disponible ; machine imposée saturée refuse.
- Une seule chaîne : filtres/score → service générique → Claude léger isolé → repli déterministe figé. Candidat unique/axes fixés évitent les décideurs.
- Le service reçoit la liste entière ; sa capacité insuffisante est visible, sans top8 ni tournoi.
- Le thread du léger possède RoutingDecisionPermit jusqu’au drain physique. Il ne reçoit pas le store et ne peut pas écrire un retour tardif.
- La tâche utilisateur entière, ou l’objectif/résumé du Handoff, est une donnée inerte. Aucun texte n’est coupé pour faire tenir un niveau. Une charge trop grande rend les décideurs indisponibles puis laisse le score.
- Les CAS E portent époque, révision et niveau. Snapshot/repli avant consultation ; décision complète écrite atomiquement avec Chosen ; Task ensuite.
- RoutingAdmissionSnapshot conserve policy/candidates/decision_level/decision, coordinateur, observations, faits agrégés, scores/fallback et notes structurées. decision reste null avant Chosen puis contient le LineageDecisionV4 fermé.
- La reprise d’une ancienne époque utilise le repli ou le choix figé ; elle re-sonde strictement le seul élu sans nouvelle consultation.
- Les retours tardifs vérifient le CAS avant un re-probe et rendent admission_pending.
- Le lancement local revalide le digest du lanceur. Le lancement distant utilise PrepareV2 avec connexion/moteur/workload et digests issus du probe frais. Le WorkerPrepare porte le SHA exact.
- Le SHA source est lu par la CLI avant le canon. Les admissions Git Shared/Worktree exigent ce SHA ; base_ref de Worktree doit résoudre le même SHA. Shared général horsGit exige une preuve filesystem fermée d’absence de métadonnées Git. Une E/S ou des métadonnées Git cassées ne deviennent pas horsGit.
- Le rapport Git existant reste utilisé ; aucune intégration/fetch/push/merge ajoutée au daemon.

<a id="extrait-2"></a>

## Extrait 2

- Bibliothèque daemon : filtre spec168, notamment daemon::native_delegation::tests::spec168_.
  13 scénarios natifs : catalogue12/128 et gardes ; axes/effort ; unique/OFF/rejeu ;
  score/saturation/TTL ; objectif/contraintes/charge sans coupe ; Gitexact/worktree/horsGit ;
  POST/concurrence/retour tardif/reprise/réouverture ; servicevalide ; léger12/deuxauth ;
  rollbackSQL ; crashPending/Started ; UNION4096 ; inaccessible/effort de rôle.
- delegation_mcp::tests::spec168_ : nouveau nominalV2, wire163 exact et ancienmaître.
- provider_limits::tests::spec168_ : connexion/instance/définition/époque/TTL, crédits contre100%.
- registry::tests::spec168_telemetrie : seuils fermés.
- Binaire intégration cli_arguments_integration_test : filtre spec168,3 scénarios.
- Binaire intégration native_delegation_e2e : filtre spec168, vrai daemon isolé/catalogue12/128,
  zéroTask/receipt/fournisseur, lecture sansagent/T3.
