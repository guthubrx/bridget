# Plan222 — Rôles et spécialistes durables
Statut : corrigé après contre-revue1 ; le contrat closure-after-review.md fait autorité pour les points corrigés. Source de départ : e88d1db46242de93142708fa3396293141ee8e92.
Spec : /home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/spec.md

## Contexte technique et primitives
Rust2024, SQLite/rusqlite, serde, sha2, uuid, protocoles natifs existants ;
Android Kotlin/Compose, client HTTP humain déjà appairé. Aucune dépendance ajoutée.
Les compétences SpecKit sont lues depuis le runtime utilisateur. Les scripts
setup-plan.sh, check-prerequisites.sh et le template projet sont absents dans ce
checkout (test -f retourne1), ainsi que extensions.yml : protocole des compétences
exécuté directement, sans prétendre que les scripts officiels ont été exécutés.
La synchronisation utilisateur a été exécutée dans le worktree isolé.
L'outil Sequential Thinking n'est pas exposé au catalogue : décisions et hypothèses
écrites ici et dans research.md avant code. Pas de contournement par un fournisseur.

## Réutilisation de l'existant
- AgentProfileStore (agent_profile.rs:169/719) conserve identité humaine,
  étiquettes et consignes appliquées. Étendre ce domaine, pas un service concurrent.
- FleetSupervisor (fleet.rs:533/579/891) possède déjà les agents nommés,
  la persistance, les générations, l'idempotence et les demandes de relance.
- lifecycle::submit_spawn (lifecycle.rs:175), ManagedSupervisorCommand et les
  contrôles existants restent les seuls chemins de lancement et d'arrêt.
- AutonomyBudgetPolicy (fleet.rs:78) a déjà des limites Option absentes par
  défaut. Réutiliser sa sémantique, jamais remplacer absence par zéro.
- ExecutionStore et work_submissions/executions/provider_bindings
  (execution_store.rs:241/425/467) restent l'autorité d'exécution et de file.
- NativeDelegation/AutoDelegateV2, send_deliveries et le registre d'idempotence
  gardent leurs canons et reçus. Les métadonnées222 référencent ces reçus.
- MobileOperation/administration, authentification/appairage et scopes humains
  sont conservés ; MCP conserve CallerScope et les attestations d'origine.
- Les journaux et artefacts existants sont les sources des connaissances.
  Ils ne sont ni copiés intégralement ni présentés comme vérité certifiée.

## Architecture retenue
Un domaine de spécialistes dans le daemon. Pas de second orchestrateur,
scheduler périodique, service de mémoire, compte fournisseur ou store de tâches.
Séparer les modèles métier purs et leur persistance des raccords de contrôle
daemon ; nouvelles routes sans RPC réseau sous le mutex métier.

1. Définition versionnée : profil de spécialiste (compétences/procédures incluses) ou
   composition (kinds profile/team). Les consignes et prérequis sont explicitement déclarés.
2. Spécialiste : identité logique durable, version active approuvée, propriétaire,
   périmètre, durable/autoréveil, mandat et budget. Sa présence vient du runtime.
3. Rôles : associations multi-valeurs avec portée. Une déclaration propre
   n'est pas une désignation officielle ; celle-ci exige une action humaine.
4. Connaissances : entrées sourcées, typées et révisables ; rappel lexical
   explicable, préférence du projet à pertinence comparable, provenance affichée.
   Le nombre total de souvenirs n'est pas borné par une fenêtre de transport.
5. Reçus de spécialiste : canon d'une demande et identité du responsable choisi,
   référence vers soumission/exécution/tâche native. Aucun état d'exécution dupliqué.

Les historiques de versions sont append-only ; les pointeurs actifs utilisent
une révision attendue. Une modification concurrente donne un conflit nommé.

## Autorité et réveil
Une définition de rôle n'autorise rien. L'activation humaine prépare un mandat
de lancement lié à l'identité du spécialiste, aux références de projets autorisés,
au profil actif et aux définitions de lancement qualifiées. Il conserve la source
humaine de l'autorisation et une révision révocable. L'entrée native persistante
et la politique résolue restent les autorités effectives de lancement.
Le port agent ne peut pas fabriquer cette autorisation : aucune entrée is_human,
actor ou approved libre ne sert de preuve. Réutiliser le contrôle humain
ControlStateV1/MobileAdministrationV1 et les grants existants.

Pour un titulaire en ligne, la demande devient un envoi natif adressé et corrélé.
Pour un titulaire durable hors ligne, vérifier l'autorisation humaine actuelle,
la pause/révocation, les prérequis et le périmètre ; utiliser FleetSupervisor et
lifecycle pour sa session nommée native avec persistent=false, puis une soumission corrélée à la file existante. La durabilité métier réside dans sa définition et son grant. Aucune relance
si l'ancienne exécution est d'issue inconnue ou si l'arrêt n'est pas attesté.
Le demandeur soumet une demande au spécialiste ; il ne reçoit pas les droits
du spécialiste. Le titulaire exécute avec son propre mandat approuvé.
Un refus de moteur/compte ou de preuve reste nommé, sans changement de droits.
La désactivation ne retire pas les autorisations nécessaires à la fin des missions
déjà admises ; elle ferme les admissions et réveils suivants.

La conversation du spécialiste est un point d'entrée stable. Les liaisons
provider/session restent distinctes et leur état n'est pas reconstruit depuis
un nom. Une conversion conserve l'ancienne liaison et les travaux. Une nouvelle
session ne s'approprie pas silencieusement une ancienne exécution active.

## Sélection, parallélisme et compositions
La résolution filtre par rôle, périmètre, prérequis, activation et présence
observée ; un tri stable explicite favorise adéquation au projet et expérience
pertinente. Elle rend les raisons, pas un score d'expertise présenté comme preuve.
L'attribution est réservée avec le canon avant tout effet. Même clé/enveloppe :
même responsable et mêmes références. Divergence : envelope_mismatch.
Un agent occupé n'est pas caché. L'entrée en file est visible et une nouvelle
demande ne force pas l'arrêt. Les travaux indépendants réutilisent la délégation
native et ses gardes de ressources ; pas de broadcast d'une même mission.
Les compositions sont des profils de coordination avec membres/livrables et
dépendances validés. Chaque admission utilise une clé dérivée stable et le reçu
existant ; une issue partielle est conservée. Aucun redémarrage d'un membre inconnu.

## Mémoire et synthèse
Une connaissance comprend source, projet, portée, type, révision et statut.
La source est vérifiée sur la même identité et son périmètre. Une référence
existante n'atteste pas la justesse du texte : les connaissances déclarées et les
résultats rapportés restent étiquetés. Seuls le résultat Task corrélé et ObservedActivity attesté sont capturés
automatiquement comme résumé rapporté et sourcé ; les synthèses et corrections explicites
enrichissent les faits/décisions/hypothèses sans changer les consignes permanentes.
La reprise d'historique est une opération humaine ciblée, paginée et reprenable.
L'oubli retire l'entrée du rappel et empêche sa réintroduction automatique à partir
de la même source ; les produits et journaux originaux ne sont pas détruits.
Le rappel recherche les projets autorisés, classe d'abord la pertinence puis
favorise le projet courant à égalité, puis date/identifiant. La réponse porte
les sources et une limite de fenêtre explicite. Les preuves de livraison ne
sont pas déduites de la seule présence d'un commit ou d'un souvenir.
Pas d'embeddings, de service externe ni de distillation payante ajoutés.
Les procédures/skills de profil sont du contenu versionné. Un outil externe
exigé mais non attestable produit requirement_unverified ; aucune installation
depuis un simple nom de rôle. La préparation autorisée reste une mission explicite.

## Surfaces et fichiers
- Domaine/persistance : étendre AgentProfileStore ; sous-module ciblé
  crates/bridget-daemon/src/agent_profile/specialists.rs si la responsabilité
  et les tests ne tiennent pas lisiblement dans le fichier existant.
- Frontière : contrat fermé version1 dans bridget-transport/protocol.rs,
  négociation SpecialistsV1, refus explicite par ancien pair.
- Raccord daemon : daemon/specialists222.rs, dispatcher et hooks de cycle de vie.
- MCP/CLI : mcp_scopes.rs, mcp.rs, communication/client.rs et cli.rs ;
  actions distinctes lecture, déclaration propre, demande et administration humaine.
- Administration : MobileOperation, mobile/http.rs et daemon/mobile_operations.rs ;
  écran Kotlin SpecialistsScreen, raccord navigation/MainActivity et BridgetClient.
- Documentation FR/EN, ADR222, preuves-corrections et artefacts222.

## Découpage
A. Contrats, données, profils, déclarations et mémoire, avec tests de propriété.
B. Attribution, envois, autorisations humaines et réveil via le cycle existant.
C. Capture d'expérience, rappel, conversion et compositions natives.
D. MCP/CLI et catalogue Android complet ; refus et diagnostics cohérents.
E. Converge, contre-revue, audit, build unique de livraison, activation et nettoyage.
Aucun lot ne réduit la spec. Les tests ciblent la responsabilité modifiée ;
pas de répétition des campagnes221 ni de recette métier d'un autre projet.

## Constitution, migration et retour
Isolation du worktree, un responsable ; aucune nouvelle dépendance. Complexités
froides bornées par la fenêtre/liste de rôles, requêtes indexées et paramètres SQL.
Les schémas existants ne changent pas de sens. Aucune conversion globale des
étiquettes, des anciens profils ou des journaux. Tables additionnelles sous
migrations idempotentes et anciens parcours invariants.
Retour : image du même lecteur compatible avec les nouveaux enregistrements,
qualifiée avant activation. Un ancien lecteur qui ignore les spécialistes ne
reçoit aucune nouvelle opération ; pas de réécriture de tâches pour tester.
Contrôle des processus chargés, pointeurs rollback natifs et lien après passage.
Le programme221 n'est pas déclaré clos par cette session.
