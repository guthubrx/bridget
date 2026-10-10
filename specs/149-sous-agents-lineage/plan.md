# Plan d'implémentation — session149

**Spec**: 149-sous-agents-lineage  
**Date**: 2026-10-10  
**Statut du plan**: Livré sur disque, non activé ; 45/45 tâches validées au reçu final (validation/final.md). SOURCE_ONLY_APPROVE Sonnet r2 et recettes réseau r4 / Codex vivant r7 conservées. Aucun restart ni activation différée.
**Accord utilisateur**: session149 acquis ; aucune nouvelle permission de session requise  
**Dépendances**: sessions147 et148 livrées ; Lineage T3 existant

## Résumé

Bridget crée, exécute et termine les vrais enfants. T3 présente ces tâches dans son Lineage existant. Le journal de chaque enfant reste natif. Le connecteur n'appelle jamais une commande T3 qui lance un fournisseur. Les droits effectifs du parent sont attestés puis figés avant admission. Un parent autorisé à écrire peut déléguer à Codex ou GLM sans nouveau grant humain Bridget.

Les recherches sont dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/research-native.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/research-t3.md`. Les contrats définitifs149 priment sur les exemples proposés dans ces recherches. La stratégie de tests est `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/test-strategy.md` ; ses oracles doivent être alignés sur les contrats149 avant la gate.

## Contexte technique

| Élément | Choix |
|---|---|
| Moteur | Rust synchrone, socket Unix, SQLite ; flotte, wrapper et saga native148 existants |
| Connecteur | T3 TypeScript/Effect, MCP HTTP par session ; binaire Bridget pour lecture humaine locale |
| Données UI | Événements et projections T3 existants ; fils enfants virtuels, sous-agents et Lineage existants |
| Dépendances ajoutées | Aucune prévue ; aucun framework d'orchestration, sandbox OS ou journal métier parallèle |
| Sources natives | Tâche, définition fournisseur, résultat corrélé et journal durables Bridget |
| Tests et relectures | Claude : Sonnet 5.5 high (permissions, reprise, tests complexes), Haiku 5.5 medium (docs, tests simples), Haiku 5.5 high (relectures ciblées) ; Sol ne crée ni n'exécute ces tests et ne fait aucune revue |
| Production | Aucune base réelle ouverte en écriture ; aucun modèle de production, restart ou activation différée |

Worktree Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage` ; base148 `347d788510b4`.

Worktree T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` ; base148 `33f6d04e1164` de `local/main-20261009`.

Le principal conserve Git, l'intégration, les builds de livraison et les installations. Les sources natives et T3 restent séparées jusqu'à validation des contrats.

## Contrôle de constitution

Les constitutions globale et locale ainsi que les standards ont été chargés. Les worktrees sont dédiés. Les trois owners Sol ont des fichiers ou sections réservés. Aucun owner ne revert le travail d'un autre. Les modifications de tests dans des modules Rust existants attendent la libération du fichier par l'owner natif.

Les baselines et références officielles utilisées sont consignées dans les recherches. Une décision transversale est consignée par l'owner natif dans l'ADR149. Aucune nouvelle architecture générique n'est admise sans règle concrète d'identité, de cycle de vie ou de rejeu.

Le plan suit la spec actuelle : 19 exigences (FR001–FR019). FR018 préserve les permissions effectives du parent et retire seulement la garde du grant humain supplémentaire dans l'héritage autorisé, sans élargissement au-delà du parent. FR019 fige les droits, le modèle et la définition avant premier effet. US5/FR011 visent les désactivations réelles, pas un nouveau réglage de projection. Aucun owner de ce plan ne modifie la spec.

Toutes les tâches restent non cochées jusqu'à leur preuve de test et à l'accord du principal sur cette preuve. Deux gates séparées s'appliquent : G-L pour Lineage, lectures natives, journal et filtre du pont ; G-P pour permissions, attestation et héritage. APPROVE de relecture sur G-L puis GO principal permettent le code de présentation indépendant pendant la fermeture de G-P. Aucun code de permission ou héritage n'est autorisé avant G-P. Un contrôle ciblé adapté au dépôt remplace les commandes Cartae inapplicables. Aucun contrôle global T3 n'est prévu sans demande explicite.

## Invariants

1. Une demande native possède une tâche, un enfant et une remise stables. Une consultation ou projection ne lance jamais d'exécution.
2. La saga, les descendants, les délais, le résultat et l'annulation appartiennent à Bridget. T3 est une présentation optionnelle.
3. Aucun paramètre MCP de l'agent ne fait autorité pour le parent, les droits ou une preuve. Une preuve privée absente, étrangère, périmée ou révoquée ferme l'admission.
4. Les paramètres réellement appliqués au fournisseur font autorité. Le prompt et le seul mode affiché dans T3 ne prouvent aucun droit.
5. La politique admise est figée avant effet et n'est jamais automatiquement élargie par retry, reprise ou changement ultérieur des droits du parent. Une preuve fraîche d'identité est requise pour une nouvelle admission ou un rebinding ; elle ne remplace pas la politique d'une tâche existante.
6. La mission admise et ses descendants poursuivent leur cycle natif si T3 tombe. Le moteur ne réatteste pas auprès de T3 à chaque étape autonome.
7. Sans posture explicite, l'enfant hérite. `discovery` réduit volontairement les droits. `development` conserve uniquement une écriture attestée. Aucun grant humain supplémentaire n'est demandé pour le chemin d'héritage normal.
8. Les désactivations MCP, définitions utilisateur et révocations existantes restent prioritaires. Aucun nouvel interrupteur de fonctionnalité n'est ajouté.
9. Un fil virtuel149 est marqué `bridgetTaskRef` et porte la relation `subagent`. Il n'est ni un agent remonté par le pont, ni un fournisseur T3 lançable, ni une conversation de premier niveau.
10. Les snapshots sont publiés atomiquement. Une panne masque l'état courant non vérifié sans effacer l'historique vérifié. Les flux sont bornés et fermés quand le contexte affiché cesse de les utiliser.
11. Aucun credential n'entre dans la tâche, le prompt, l'enfant, les journaux ou une sortie de diagnostic. Aucun chemin libre de journal n'est accepté depuis un client.
12. Les anciens agents ordinaires restent inchangés. Seules les tâches natives explicitement prouvées entrent dans la projection149.

## Architecture et contrats

### Permissions de session et héritage

Étendre l'outil vide `bridget_session` en version2. Il conserve l'identité148 et ajoute un fait de politique effective version1 lié au credential courant, au fil, au tour, à la session réelle du fournisseur, au fournisseur, au dossier et au projet. Le contrat public est unique dans `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/bridgetPermissions.ts`.

Le registre privé existant dans `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/provider-core/src/server/mcpSession.ts` reçoit les faits publiés aux frontières réelles des adaptateurs. Codex publie les paramètres validés de `turn/start` avant l'appel et les retire si cet appel échoue ou si le tour finit. Claude/GLM publie après openQuery et après établissement de activeTurn, avant query.offer. Le contexte de query garde une copie sanitaire des options finales pour les réutilisations. Les messages init/status actualisent seulement le tour propriétaire. Une configuration ou règle brute avec secrets n'est jamais exposée.

La réponse externe conserve les champs d'identité camelCase148. Le fait de permissions est en snake_case et porte une révision monotone. Le contrat conserve l'objet Codex sandboxPolicy exact, avec ses champs camelCase et ses éventuelles approbations granulaires. Claude expose mode/tools/allow/disallow/répertoires supplémentaires/flag de bypass exacts et la médiation T3 effective. Un champ settings_sources=provider_default décrit le chargement réellement délégué au fournisseur. Ce champ ne prouve pas une fusion de règles user/managed que le SDK n'expose pas.

Pour une famille Claude identique, l'héritage peut réutiliser l'entrée CLI, le profil, le cwd, les sources de settings réellement sélectionnées et leurs surcharges vérifiées. Il n'invente aucune API donnant toutes les règles fusionnées. La preuve lie ces entrées à la session propriétaire et le fournisseur conserve l'application de ses règles. Une copie de profil global ou une règle d'agent autodéclarée ne remplace jamais cette preuve. Une traduction inter-fournisseurs doit porter les contraintes connues et son équivalence prouvée ; full-access attesté doit permettre l'écriture GLM demandée. Le plan n'autorise ni un refus global de tous les parents Claude, ni une annonce d'isolation OS fondée sur la seule liste d'outils.

Le contexte de lancement fige le launcher choisi et le CLI réellement exécuté. L'owner résout PATH à frais avant la publication du snapshot figé, avant le premier spawn et à chaque reprise. Les symlinks sont résolus avant hachage. Le contexte porte `cli_path`/`cli_revision` du launcher et, en plus, `resolved_cli_path`/`resolved_cli_revision` du binaire exec, avec leur sha256. Les digests sont recontrôlés avant premier spawn et à chaque reprise. Une mutation, une suppression ou un ajout de source, un retarget de symlink ou un changement de priorité PATH produit `settings_revision_changed` : aucune re-résolution, aucune adoption des nouvelles permissions, aucun nouveau lancement. Une source nécessaire opaque produit `permission_source_unavailable`, seul nom de ce cas dans l'inventaire du contrat, sans équivalence inventée et sans refus global des parents Claude de même famille aux sources valides.

Preuve d'environnement consignée (2026-10-10) : le service launchctl actif (PID58394) tourne avec `BRIDGET_HOME=/Users/moi/.cache/bridget-core`. Le registre vivant est `/Users/moi/.cache/bridget-core/agents.json` ; son entrée `glm` porte `gclaude`, le profil `/Users/moi/.claude-glm` et le modèle `glm-5.3` ; l'entrée `claude` y figure aussi. Le fichier `/Users/moi/.config/bridget/agents.json` (`cursor`, `claude`) n'est pas sélectionné par ce runtime. La recette utilise un clone privé contrôlé de ce registre avec le modèle exact `glm-5.3-flash` ; la différence entre le compte privé de recette et le modèle de production n'autorise aucun repli hors Flash. Aucune entrée de production n'est modifiée. Les lectures de preuve restent des lectures JSON saines, en lecture seule, sans impression d'env ni de jeton.

Le daemon Bridget revalide la preuve privée via l'adaptateur T3 hors verrou. Il recontrôle ensuite le binding vivant avant admission. Le jeton est éphémère, masqué et jamais durable. Les descendants utilisent le snapshot natif prouvé de leur parent tâche. Le premier parent hors T3 utilise une source effective propriétaire documentée dans le contrat149. Une source inconnue n'est pas convertie en full-access, ni masquée par discovery.

La traduction inter-fournisseurs décrit séparément les outils, racines et réseau. Les mêmes droits doivent réussir pour les cas supportés Codex et GLM, surtout full-access avec écriture. Une exigence de confinement sans équivalent disponible produit un refus explicite. Des permissions d'outils Claude ne sont jamais présentées comme une sandbox OS Codex.

Le wrapper applique la politique figée à la seule définition enfant. Il ne modifie pas le profil global GLM et ne transforme pas l'ancien `permissions=allow` en bypass universel. Claude conserve son mode et ses règles ; les permissions interactives non couvertes sont refusées sous leur corrélation, sans dialogue Bridget supplémentaire ni faux succès.

### État natif et lectures humaines

La tâche conserve `parent_task_id` et le root owner stable. Ces valeurs viennent de l'ownership prouvé, pas du contenu de la mission. SQLite conserve les index root/parent et la séquence de mutation dans la même transaction que l'état. Les reprises gardent la tâche, le modèle, la politique et la clé de remise.

Commandes natives proposées, sous les gardes humaines147 :

- `bridget lineage inspect --t3-thread ID --project-root ROOT --action list [--limit N] [--cursor opaque] --json` ;
- `bridget lineage inspect --t3-thread ID --project-root ROOT --action show --task UUID [--offset N] [--limit N] --json` ;
- `bridget lineage inspect --t3-thread ID --project-root ROOT --action journal --task UUID [--after-seq N] [--limit N] [--follow] --json` ;
- `bridget lineage watch --t3-thread ID --project-root ROOT --json` ;
- `bridget lineage cancel --t3-thread ID --project-root ROOT --task UUID --request-id UUID --json`.

La liste est limitée à100 tâches et128KiB par page. Le curseur lie le binding et le snapshot. Une mutation entre pages donne `snapshot_changed` ; T3 reprend le staging. Les détails et résultats restent séparés de la liste. Le résultat natif148 reste limité à256KiB ; show le rend par fenêtres UTF8 de16KiB avec offset validé et curseur de fin. Le journal est limité à16KiB et100 événements par page. Le mode follow réutilise AttachRelay et ses frontières snapshot/caught-up/live/gap. Bridget choisit le journal réel de l'enfant et conserve son accès après nettoyage. La fin de tour ne vaut jamais résultat final de mission.

Le suivi invalide des lectures. Il n'envoie aucun corps de mission, résultat ou journal. `ready` précède les autres événements. Génération et séquence portent une continuité vérifiable. Seules les mutations committées signalent un changement. Les lectures, rejets, ACK et rejeux identiques restent muets. Le contrat149 fixe un domaine unique de séquence : la mutation du magasin de projection des tâches natives, partagée par le flux watch et le snapshot.

Bornes conservées148 : 16 tâches actives par parent,128 globales,4096 enregistrements et profondeur8. Une page utilise un index root/parent : objectif O(log N + P), P≤100. La réconciliation T3 indexe les tâches une fois : O(n). Aucun appel par ligne de Lineage.

### Projection T3 et journal

Un seul service de domaine `BridgetLineage` réutilise la résolution du binaire, ProcessRunner, les gardes et les motifs de lecture147. Il possède les lectures natives, la souscription affichée, la réconciliation et l'annulation native. Il n'introduit pas de moteur, de scheduler ni de journal durable concurrent de Bridget.

Une commande interne de synchronisation T3 écrit les fils virtuels et les sous-agents via EventSink, avec zéro effet fournisseur. Les IDs sont déterministes par taskId. Les enfants imbriqués pointent leur parent tâche et leur racine T3. Le marqueur `bridgetTaskRef` est facultatif uniquement pour préserver les données historiques ; il est obligatoire pour un fil virtuel149.

Les états et horaires des résumés virtuels viennent de Bridget, car ces fils ne possèdent aucun tour T3. L'origine `bridget_native` ne déclenche ni les reprises `app_owned`, ni leurs résultats ou réveils automatiques. La remise native148 au parent reste unique.

Lineage utilise ses lignes et sa navigation actuelles. La colonne de gauche masque déjà `subagent`. Le clic ouvre une vue en lecture du journal natif dans le fil enfant. Les contrôles de message, fournisseur, fork, reprise et rollback sont indisponibles et refusés côté serveur pour ce fil. L'arrêt de l'enfant appelle la saga native. Si l'arrêt parent existant annule ses enfants, les tâches natives héritent du même comportement sans créer un nouveau mode d'arrêt.

Le contrat de résumé T3 lu par le pont Rust reconnaît `bridgetTaskRef` et exclut ces fils avant montage de wrapper ou présence. Le préfixe d'ID ne constitue jamais la seule garde. Aucun ancien fil normal n'est converti pour satisfaire cette règle.

## Répartition exclusive

| Owner | Fichiers ou responsabilités |
|---|---|
| Sol natif149 | Code Bridget Rust : protocol, delegation, admission, registry, wrapper, transports fournisseur, lecture/suivi/journal/annulation149, attestation T3 native, contrats Rust du résumé et filtre du pont |
| Sol permissions T3 | Nouveau `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/bridgetPermissions.ts` ; `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/provider-core/src/server/mcpSession.ts` ; adaptateurs Codex/Claude ; section `sessionIdentity` seule de OrchestratorMcpService et `BridgetSessionTool` seul de tools.ts |
| Sol Lineage T3 | Contrats Lineage/RPC, export du contrat permission, BridgetLineage, commande interne/projections, états client communs, navigation/journal/arrêt web et garde mobile ; aucune édition des fichiers des adaptateurs |
| Claude (Sonnet 5.5 high, Haiku 5.5 medium et high) | Stratégie, tests, fixtures, tests négatifs, relectures du plan et du code, régressions ciblées et preuves ; recettes réelles conduites par Sonnet 5.5 high, GLM 5.3 Flash restant le fournisseur produit testé ; aucun Sol ne remplace ce rôle |
| Principal | Spec et suivi global, arbitrages, Git, livraison, builds finaux, signatures, installation de fichiers et reçu final |

Les owners libèrent chaque fichier avant sa relecture ou l'écriture de tests embarqués Claude. Les sections de OrchestratorMcpService et tools.ts restent séparées ; aucun travail concurrent dans le même fichier n'est autorisé sans passage de relais explicite.

## Phases et gates

| Phase | Objectif | Gate et preuve |
|---|---|---|
| 0 — Contrat | Corriger la spec et lire la stratégie de tests ; figer Lineage et fermer les sources de droits séparément | G-L : APPROVE de relecture Claude sur contrat Lineage, plan et tâches de présentation puis GO principal. G-P : APPROVE distinct sur permissions/sources/mappage puis GO principal. Toute ambiguïté bloque seulement son volet |
| 1 — Autorité | Attestation T3 réelle et admission native héritée, définition figée | Claude prouve cas positifs/négatifs, modes mutables,2 sessions partagées, full parent→GLM write |
| 2 — État natif | Parent/root durables, pages atomiques, watch, journal, cancel | Claude prouve pureté des lectures, perte de continuité, limites, nested/cancel et reprise sans T3 |
| 3 — Projection | Fils virtuels, zéro effet de lancement, journal et Lineage | Claude prouve unicité, sidebar, bridgefilter, statuts et refus des actions fournisseur |
| 4 — Recettes | Flux complet réel T3 et standalone, six scénarios dégradés | Claude distingue réel/synthétique, fournit journaux et compteurs d'exécution observés |
| 5 — Livraison | Arbres validés, builds et installation sans relance | Commits sources, push vers forks seulement, signatures et empreintes, processus inchangés, aucun job d'activation |

Les phases2 et3 peuvent avancer après G-L, sans modifier les droits148 ni prétendre valider l'héritage149. Leurs lectures restent pures et leur projection ne lance rien. La phase1 permissions T3 et natif attend G-P puis peut avancer en parallèle. Les tests Claude ont une dépendance sur les fichiers libérés. La phase3 dépend du protocole natif phase2 mais peut préparer ses contrats après G-L. Les recettes complètes phase4 attendent les deux gates et l'intégration des trois owners.

## Validation prévue

Claude crée les tests avant validation de chaque incrément. Il teste le comportement réel des services avec DB et contrôles isolés. Les faux fournisseurs restent limités aux frontières utiles pour les refus, crash et timeout. Aucune fixture décorative ni test qui répète seulement les champs du code n'est une preuve.

Les recettes fournisseur incluent une vraie écriture GLM Flash, une vraie écriture Codex, un refus hors droits, le moteur standalone avec T3 absent, un résultat corrélé unique et un compteur observé de lancement. La recette UI web tourne sur un serveur T3 fixture (port éphémère, base privée, même gabarit que l'interop R4) avec un daemon Bridget fixture. Elle ouvre Lineage et le journal, puis vérifie l'absence de conversation de premier niveau. Elle ne valide pas la coque desktop native. Le serveur de recette, les providers et les bases sont isolés. Aucune seconde application T3 n'est lancée. Aucun redémarrage des processus existants n'est permis.

Les contrôles Rust, TypeScript et Effect sont ciblés sur les composants modifiés. Claude consigne les commandes exactes et leurs sorties pass/fail. Le principal effectue les builds finaux depuis les arbres sources intégrés. Les propriétaires Sol ne déclarent aucun succès de test sans preuve de test Claude.

Les scénarios S149-01 àS149-33 de la stratégie de tests servent de point de départ, dont S149-32 (union d'introspection v1/v2, oracle G-P-07) et S149-33 (observer propriétaire du chemin PTY, oracle G-P-08, amendement G-P r3). Claude les adapte à l'héritage149 : aucune dépendance à un nouveau grant humain, aucun confinement artificiel de full-access et aucune exigence « zéro ligne thread T3 ». L'oracle correct est zéro conversation de premier niveau, zéro tour/provider session/outbox de lancement et un fil virtuel par tâche. Une fixture qui refuse elle-même une écriture ne prouve pas un droit réellement imposé au fournisseur. Les recettes réelles Codex et GLM ainsi que la recette UI web du journal (S149-28) complètent cet inventaire. La profondeur8 doit être testée avant le correctif si la garde annoncée148 n'est pas appliquée.

## Risques et décisions avant code

| Risque | Décision ou garde |
|---|---|
| Droits Claude changés pendant une requête | Attester le mode courant réel et les règles finales ; tests init/status, réutilisation de query et arguments prioritaires |
| Premier parent Claude PTY standalone sans source effective | Contrat doit identifier une source propriétaire prouvée avant admission ; gate bloquante si le cas reste non couvert |
| Snapshot changé entre pages | Curseur lié à snapshot, staging complet puis publication atomique ; aucun mélange de versions |
| Rejeu ou reprise élargissant les droits | Définition et politique figées, identité fraîche contrôlée séparément |
| Fil virtuel lancé par un chemin T3 alternatif | Marqueur et gardes communes serveur ; web/mobile/MCP refusent les mêmes opérations |
| Boucle T3→Bridget | Filtre du contrat de résumé avant montage ; tests inventaire sans nouvelle présence |
| Parent arrêté avec enfant résiduel | Reprendre le contrat d'arrêt existant et diriger les descendants natifs vers leur saga |
| Journal indisponible après cleanup | Référence durable liée à tâche, lecture sous garde native, recette après fin et reprise |
| Opt-out inventé | Réutiliser enabled=false/définitions utilisateur et désactivations Claude existants ; aucun nouvel interrupteur |

Volet Lineage figé pour G-L : contrat CLI149 publié, snapshot de métadonnées paginé, détail/résultat UTF8 borné, journal natif AttachRelay, invalidation scoped, cancel saga, fils virtuels déterministes et filtre de pont. Son implémentation réutilise les droits existants tant que G-P n'est pas validée.

Volet permissions : le contrat `contracts/permissions.md` ferme la source du premier parent Claude PTY standalone, les entrées CLI/profil/cwd/settings réellement sélectionnées, le mappage inter-fournisseurs et le périmètre des règles attestées. La revue G-P r1 (`validation/plan-permissions-r1.md`) est levée par la revue r2 (`validation/plan-permissions-r2.md`, APPROVE) : G-P-01–G-P-06 restent fermés. La revue r3 (`validation/permissions-contract-deltas-r3.md`) est REQUEST_CHANGES ciblé sur les deux deltas : G-P-07 (phrase « n'a jamais existé pour ce credential », oracle de l'union v1/v2) et G-P-08 (oracle de l'observer PTY) sont corrigés dans le contrat, la stratégie (S149-32, S149-33), les tâches (T015, T038) et la fiche spec (0/33) ; la re-revue G-P reste en attente puis le GO principal. Aucun oracle n'est abaissé pour s'adapter au code ; la preuve d'entrée du Rust autonome reste la recette S149-14(a)/T038 (T3 absent), jamais un descendant managed substitué. L'opt-out reprend les chemins existants ; aucune nouvelle fonction de configuration n'est ouverte par149. Cette réconciliation ne justifie aucune nouvelle demande d'autorisation de session.

## Livraison et estimation

Effort total estimé : 18 à28 heures de travail réparties entre owners, validation comprise. La durée réelle dépend surtout de la preuve standalone Claude et du mappage Codex→GLM. Les owners peuvent paralléliser des fichiers indépendants ; une gate échouée suspend le code concerné.

La livraison reprend le processus148 : principal committe, intègre et pousse Bridget sur son dépôt et T3 sur le fork autorisé, jamais upstream. Il construit depuis les arbres propres validés. Il vérifie signatures et empreintes, puis remplace uniquement les fichiers de livraison autorisés. Les sauvegardes et preuves antérieures sont conservées.

L'installation et l'activation sont distinctes. Les processus Bridget et T3 existants gardent leur code chargé. Aucun restart réel, aucune activation différée, aucune configuration ou base de production n'est modifiée pour la recette. Le reçu final indique les commits, preuves de test, chemins absolus, empreintes, limites de recette et absence de relance.
