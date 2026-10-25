# Plan d'implémentation — session149

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-1).

## Résumé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-3).

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-4).

Worktree Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage` ; base148 `347d788510b4`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-6).

## Contrôle de constitution

Les constitutions globale et locale ainsi que les standards ont été chargés. Les worktrees sont dédiés. Les trois owners Sol ont des fichiers ou sections réservés. Aucun owner ne revert le travail d'un autre. Les modifications de tests dans des modules Rust existants attendent la libération du fichier par l'owner natif.

Les baselines et références officielles utilisées sont consignées dans les recherches. Une décision transversale est consignée par l'owner natif dans l'ADR149. Aucune nouvelle architecture générique n'est admise sans règle concrète d'identité, de cycle de vie ou de rejeu.

Le plan suit la spec actuelle : 19 exigences (FR001–FR019). FR018 préserve les permissions effectives du parent et retire seulement la garde du grant humain supplémentaire dans l'héritage autorisé, sans élargissement au-delà du parent. FR019 fige les droits, le modèle et la définition avant premier effet. US5/FR011 visent les désactivations réelles, pas un nouveau réglage de projection. Aucun owner de ce plan ne modifie la spec.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-7).

## Invariants

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-8).

## Architecture et contrats

### Permissions de session et héritage

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-10).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-11).

Pour une famille Claude identique, l'héritage peut réutiliser l'entrée CLI, le profil, le cwd, les sources de settings réellement sélectionnées et leurs surcharges vérifiées. Il n'invente aucune API donnant toutes les règles fusionnées. La preuve lie ces entrées à la session propriétaire et le fournisseur conserve l'application de ses règles. Une copie de profil global ou une règle d'agent autodéclarée ne remplace jamais cette preuve. Une traduction inter-fournisseurs doit porter les contraintes connues et son équivalence prouvée ; full-access attesté doit permettre l'écriture GLM demandée. Le plan n'autorise ni un refus global de tous les parents Claude, ni une annonce d'isolation OS fondée sur la seule liste d'outils.

Le contexte de lancement fige le launcher choisi et le CLI réellement exécuté. L'owner résout PATH à frais avant la publication du snapshot figé, avant le premier spawn et à chaque reprise. Les symlinks sont résolus avant hachage. Le contexte porte `cli_path`/`cli_revision` du launcher et, en plus, `resolved_cli_path`/`resolved_cli_revision` du binaire exec, avec leur sha256. Les digests sont recontrôlés avant premier spawn et à chaque reprise. Une mutation, une suppression ou un ajout de source, un retarget de symlink ou un changement de priorité PATH produit `settings_revision_changed` : aucune re-résolution, aucune adoption des nouvelles permissions, aucun nouveau lancement. Une source nécessaire opaque produit `permission_source_unavailable`, seul nom de ce cas dans l'inventaire du contrat, sans équivalence inventée et sans refus global des parents Claude de même famille aux sources valides.

Preuve d'environnement consignée (2026-10-10) : le service launchctl actif (PID58394) tourne avec `BRIDGET_HOME=/Users/moi/.cache/bridget-core`. Le registre vivant est `/Users/moi/.cache/bridget-core/agents.json` ; son entrée `glm` porte `gclaude`, le profil `/Users/moi/.claude-glm` et le modèle `glm-5.3` ; l'entrée `claude` y figure aussi. Le fichier `/Users/moi/.config/bridget/agents.json` (`cursor`, `claude`) n'est pas sélectionné par ce runtime. La recette utilise un clone privé contrôlé de ce registre avec le modèle exact `glm-5.3-flash` ; la différence entre le compte privé de recette et le modèle de production n'autorise aucun repli hors Flash. Aucune entrée de production n'est modifiée. Les lectures de preuve restent des lectures JSON saines, en lecture seule, sans impression d'env ni de jeton.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-12).

La traduction inter-fournisseurs décrit séparément les outils, racines et réseau. Les mêmes droits doivent réussir pour les cas supportés Codex et GLM, surtout full-access avec écriture. Une exigence de confinement sans équivalent disponible produit un refus explicite. Des permissions d'outils Claude ne sont jamais présentées comme une sandbox OS Codex.

Le wrapper applique la politique figée à la seule définition enfant. Il ne modifie pas le profil global GLM et ne transforme pas l'ancien `permissions=allow` en bypass universel. Claude conserve son mode et ses règles ; les permissions interactives non couvertes sont refusées sous leur corrélation, sans dialogue Bridget supplémentaire ni faux succès.

### État natif et lectures humaines

La tâche conserve `parent_task_id` et le root owner stable. Ces valeurs viennent de l'ownership prouvé, pas du contenu de la mission. SQLite conserve les index root/parent et la séquence de mutation dans la même transaction que l'état. Les reprises gardent la tâche, le modèle, la politique et la clé de remise.

Commandes natives proposées, sous les gardes humaines147 :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-13).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-14).

Le suivi invalide des lectures. Il n'envoie aucun corps de mission, résultat ou journal. `ready` précède les autres événements. Génération et séquence portent une continuité vérifiable. Seules les mutations committées signalent un changement. Les lectures, rejets, ACK et rejeux identiques restent muets. Le contrat149 fixe un domaine unique de séquence : la mutation du magasin de projection des tâches natives, partagée par le flux watch et le snapshot.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-15).

### Projection adaptateur de bureau et journal

Un seul service de domaine `BridgetLineage` réutilise la résolution du binaire, ProcessRunner, les gardes et les motifs de lecture147. Il possède les lectures natives, la souscription affichée, la réconciliation et l'annulation native. Il n'introduit pas de moteur, de scheduler ni de journal durable concurrent de Bridget.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-16).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-17).

Lineage utilise ses lignes et sa navigation actuelles. La colonne de gauche masque déjà `subagent`. Le clic ouvre une vue en lecture du journal natif dans le fil enfant. Les contrôles de message, fournisseur, fork, reprise et rollback sont indisponibles et refusés côté serveur pour ce fil. L'arrêt de l'enfant appelle la saga native. Si l'arrêt parent existant annule ses enfants, les tâches natives héritent du même comportement sans créer un nouveau mode d'arrêt.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-18).

## Répartition exclusive

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-19).

Les owners libèrent chaque fichier avant sa relecture ou l'écriture de tests embarqués Claude. Les sections de OrchestratorMcpService et tools.ts restent séparées ; aucun travail concurrent dans le même fichier n'est autorisé sans passage de relais explicite.

## Phases et gates

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-20).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-21).

## Validation prévue

Claude crée les tests avant validation de chaque incrément. Il teste le comportement réel des services avec DB et contrôles isolés. Les faux fournisseurs restent limités aux frontières utiles pour les refus, crash et timeout. Aucune fixture décorative ni test qui répète seulement les champs du code n'est une preuve.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-22).

Les contrôles Rust, TypeScript et Effect sont ciblés sur les composants modifiés. Claude consigne les commandes exactes et leurs sorties pass/fail. Le principal effectue les builds finaux depuis les arbres sources intégrés. Les propriétaires Sol ne déclarent aucun succès de test sans preuve de test Claude.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-23).

## Risques et décisions avant code

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-24).

Volet Lineage figé pour G-L : contrat CLI149 publié, snapshot de métadonnées paginé, détail/résultat UTF8 borné, journal natif AttachRelay, invalidation scoped, cancel saga, fils virtuels déterministes et filtre de pont. Son implémentation réutilise les droits existants tant que G-P n'est pas validée.

Volet permissions : le contrat `contracts/permissions.md` ferme la source du premier parent Claude PTY standalone, les entrées CLI/profil/cwd/settings réellement sélectionnées, le mappage inter-fournisseurs et le périmètre des règles attestées. La revue G-P r1 (`validation/plan-permissions-r1.md`) est levée par la revue r2 (`validation/plan-permissions-r2.md`, APPROVE) : G-P-01–G-P-06 restent fermés. La revue r3 (`validation/permissions-contract-deltas-r3.md`) est REQUEST_CHANGES ciblé sur les deux deltas : G-P-07 (phrase « n'a jamais existé pour ce credential », oracle de l'union v1/v2) et G-P-08 (oracle de l'observer PTY) sont corrigés dans le contrat, la stratégie (S149-32, S149-33), les tâches (T015, T038) et la fiche spec (0/33) ; la re-revue G-P reste en attente puis le GO principal. Aucun oracle n'est abaissé pour s'adapter au code ; la preuve d'entrée du Rust autonome reste la recette S149-14(a)/T038 (dans un environnement natif isolé), jamais un descendant managed substitué. L'opt-out reprend les chemins existants ; aucune nouvelle fonction de configuration n'est ouverte par149. Cette réconciliation ne justifie aucune nouvelle demande d'autorisation de session.

## Livraison et estimation

Effort total estimé : 18 à28 heures de travail réparties entre owners, validation comprise. La durée réelle dépend surtout de la preuve standalone Claude et du mappage Codex→GLM. Les owners peuvent paralléliser des fichiers indépendants ; une gate échouée suspend le code concerné.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-25).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/plan.md#extrait-26).
