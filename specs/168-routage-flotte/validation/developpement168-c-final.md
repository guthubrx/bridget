# Développement C final — SPEC168

Date : 15 octobre 2026. Sources Rust écrites exclusivement sur Cartae.
Worktree : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
Branche : session-168-routage-flotte.
Mandat ROOT : implémentation intégrale de la chaîne native168, sans Git/service/configuration/production.
Le présent reçu publie du code et des oracles source. Il ne coche aucune tâche et ne remplace aucune preuve d’exécution.

## Interfaces et comportement

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

## Compatibilité arrêtée avec ROOT

Le nouveau nominal CLI/MCP bridget_auto_delegate est V2 stable. Le choix de contrat ne dépend pas de ON/OFF.
L’absence/inactivité de la politique168 donne un refus V2 structuré.
Les anciennes enveloppes AutoDelegateV1 et le parcours explicite Delegate restent traités sans modification de leur canon.
Un ancien maître est refusé avant tout envoi V2. Features stables ajoutées : auto_delegate_v2 et worker_candidate_probe_v2.
Aucun flag caché, aucun cache de pin, nouveau store ou API de sélection de version n’a été ajouté.

L’effort du rôle décideur léger est distinct de l’effort de la tâche.
Seul effort=None a été qualifié pour ce rôle. Une politique qui lui impose un effort produit
lightweight_decision_effort_unavailable dans decision_notes puis le score. Aucun flag arbitraire.
La configuration prévue par ROOT utilise Haiku sans effort imposé.

## Ajustements A précisément autorisés

- registry.rs : correction Borrow du set des connexions, désormais indexé par str.
- delegation_lineage.rs : Some puis flatten pour la projectionv3 ; v3 d’une admission168 reste null.
- registry.rs : seuils CPU et charge bornés à1..100, avec oracle source.
Aucun autre changement de contrat A/E/B/D.

## Contrôles déjà exécutés

1. Premier cargo check daemon --tests : RC101, 26,527s. Six erreurs d’intégration identifiées.
   /home/moi/.local/state/bridget-delivery168/tests/c-check-initial.json
2. Check de contrôle après corrections ciblées : RC0,14,038s.
   /home/moi/.local/state/bridget-delivery168/tests/c-check-controle.json
   SHA log : 1e7298f556fa6a866fb2fc23fc64574cf808c342afb765ba3809a45fed742e50
3. rustfmt --edition2024 --config skip_children=true sur les seuls fichiers autorisés : RC0.
4. git diff --check : RC0.

Le check2 précède les nouveaux oracles et les derniers raccords. Il ne valide donc pas le gel final.
Aucun test daemon, CLI ou E2E exécuté par C. Aucune suite FULL. Permis Cargo rendu explicitement à ROOT.

## Oracles source à exécuter au gel

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

Les scénarios worker et transport appartiennent à leurs propriétaires ; leurs reçus restent distincts.
La qualification réelle du maître/worker/modèles et la livraison appartiennent à ROOT.

## Empreintes

Manifestes des sources à relire avec ce reçu :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/developpement168-c-final.sha256.json

Sources gelées après publication. Aucun succès futur n’est présumé.

## Delta limité après T035 Git local R2

ROOT a compilé le gel précédent avec succès. Son replay Git2.56 a atteint la
précondition Worktree, après succès du SHA exact Shared. Le journal conservé est :
/home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.log

Diagnostic source précis : l’assertion11017 appelle preflight_classified après
workspace=Worktree. La fixture n’écrivait/committait que file. Le produit exige
.worktrees/ ignoré ; le harnais154 existant crée déjà .gitignore avant son commit.

Correction limitée : la fixture écrit puis committe .gitignore avec .worktrees/.
Les trois validations positives rendent maintenant leur erreur réelle via panic.
Le contrôle produit n’a pas été modifié.
L’avertissement private_interfaces nouveau est confirmé dans le journal
/home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.log
remote_candidate_probe_v2 est désormais privée, accessible au module descendant ;
la visibilité V1 héritée reste intacte.

rustfmt ciblé et diffcheck : RC0. Aucun Cargo ni test exécuté par C pour ce delta.
Le replay affecté reste à ROOT. Les deux empreintes du manifeste sont actualisées.
Sources de nouveau gelées.

## Delta limité après T035 Git local R3

Le replay ROOT franchit maintenant le préflight Worktree et atteint la
vérification du résultat. Le produit historique abrège la base à7caractères dans
son résumé ; il garde les SHA complets dans WorkspaceRecord et son JSON.

Seul l’oracle est corrigé : TaskJSON doit garder base_commit et head_commit
complets, ainsi que dirty=true. Le résumé doit afficher le préfixe7caractères,
la branche et le chemin ; son contenu réel apparaît si l’assertion échoue.
La base exacte et l’absence d’intégration implicite restent vérifiées.

Aucun changement produit. rustfmt ciblé et diffcheck : RC0.
Aucun Cargo ni test exécuté par C. Nouvelle empreinte native dans le manifeste.
Sources de nouveau gelées pour le seul replay affecté ROOT.
