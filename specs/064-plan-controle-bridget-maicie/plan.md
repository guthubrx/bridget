# Plan d'implémentation : Plan de contrôle Bridget et Maicie

**Branche** : `session-064-plan-controle-bridget-maicie`
**Spec** : `specs/064-plan-controle-bridget-maicie/spec.md`
**Date** : 2026-08-29
**Statut du plan** : proposé, non implémenté

## Résumé

Le programme transforme Bridget d'un transport durable avec gestion de parc en
plan de contrôle d'exécution explicite. Maicie reste le plan de mission et ne
devient ni superviseur de processus ni moteur de workflow. Les adaptateurs
fournisseurs traduisent des commandes et événements neutres tout en conservant
leurs preuves brutes et leurs capacités propres.

La livraison est incrémentale. La session 063 est d'abord finalisée. Les
contrats, identifiants, files et états d'exécution forment ensuite le socle
commun. Le graphe d'agents, le découplage Maicie et les capacités fournisseurs
peuvent alors progresser indépendamment ; la reprise dépend des capacités
fournisseurs et les budgets d'autonomie restent le dernier incrément.

## Contexte technique

| Élément | État et choix |
|---|---|
| Langage | Rust dans le workspace existant |
| Composants | `bridget-core`, `bridget-transport`, `bridget-daemon`, `plugins/maicie` |
| Persistance Bridget | SQLite existante, migrations progressives et idempotentes |
| Persistance Maicie | SQLite privée existante, jamais partagée avec Bridget |
| Protocoles fournisseurs | Codex app-server JSONL, Claude stream-json, Cursor via ACP maintenu par Cursor, autres ACP et tmux selon capacité |
| Source de preuve | événements fournisseur bruts, journal Bridget, états durables corrélés |
| Compatibilité | champs optionnels et versions de contrat pendant la migration |
| Tests | tests Rust existants, scénarios d'acceptation SPEC-064, contrats sur vrais schémas fournisseurs |
| Observabilité | journal existant enrichi, métriques et corrélation structurée sans contenu sensible par défaut |
| Dépendances externes | aucune nouvelle dépendance par défaut ; toute addition exige un arbitrage documenté |

## Invariants non négociables

1. Une acceptation RPC n'est pas une preuve de consommation.
2. Une disparition ou une observation périmée ne devient jamais une décision
   métier Maicie.
3. Une livraison déjà persistée est rejouée avec la même identité et les mêmes
   octets, jamais recréée implicitement.
4. Les identifiants Bridget et fournisseur restent distincts même lorsque leurs
   valeurs coïncident dans un test.
5. Les sorties tardives sont corrélées au thread et au tour qui les ont
   produites.
6. Maicie ne possède aucun processus fournisseur et Bridget ne clôt aucun
   objectif métier.
7. Une capacité fournisseur est observée et versionnée avant d'être utilisée.
8. Chaque attente possède une borne et une issue structurée.
9. Chaque incrément reste réversible et ne dépend pas d'un worktree actif.
10. Le système gouverne les engagements observables, pas le raisonnement interne
    de l'agent.

## Architecture cible

```text
Maicie - plan de mission
  objectifs, délégations, décisions, preuves, budgets métier
                     |
            WorkSubmissionRequest
            ExecutionReference
            ExecutionProjection
                     |
Bridget - plan de contrôle
  file logique, livraisons, exécutions, agent graph, politiques
                     |
            ProviderCommand
            ProviderEvent
            ProviderCapabilities
                     |
Adaptateurs fournisseurs
  Codex app-server, Claude stream-json, Cursor via ACP, autres ACP, tmux
```

### Autorités

| Fait | Autorité |
|---|---|
| Objectif, délégation, décision et clôture | Maicie |
| Soumission, livraison, file et exécution | Bridget |
| Instance, génération, parent et enfant | Bridget |
| Session, thread, tour et item fournisseur | Adaptateur, persisté par Bridget |
| Capacité réellement disponible | Adaptateur, attestée pour le binaire observé |
| Projection affichée | Vue dérivée avec fraîcheur, jamais autorité |

## Réutilisation de l'existant prévue

Le plan n'introduit pas immédiatement de nouveaux crates. Il étend d'abord les
seams déjà présentes :

| Besoin | Existant à étendre |
|---|---|
| Message, origine et corrélation | `crates/bridget-core/src/message.rs` |
| Session fournisseur neutre | `crates/bridget-transport/src/managed_session.rs` |
| Adapter Codex | `crates/bridget-transport/src/codex_app_server.rs` |
| Cursor et autres fournisseurs ACP | `crates/bridget-transport/src/acp.rs` et définition `cursor` de `crates/bridget-daemon/src/registry.rs` |
| Livraison et rejeu durable | `crates/bridget-daemon/src/idempotency.rs` |
| Routage wrapper | `crates/bridget-daemon/src/wrapper.rs` |
| Parc, génération et saga de création | `crates/bridget-daemon/src/fleet.rs` et `desired_state.rs` |
| Protocole public Bridget | `crates/bridget-transport/src/protocol.rs` |
| Domaine Maicie | `plugins/maicie/src/domain.rs` |
| Persistance Maicie | `plugins/maicie/src/store.rs` |
| Client Bridget de Maicie | `plugins/maicie/src/bridget_client.rs` |
| Projection UI | `crates/bridget-daemon/src/ui.rs` |

Un nouveau module n'est créé que lorsqu'il porte une machine d'état ou un
contrat réutilisé par au moins trois consommateurs. La création d'un nouveau
crate reste un arbitrage ultérieur fondé sur le graphe réel de dépendances.

## Modèle de commandes et événements

### Intentions de soumission

- `QueueOnly` : conserver sans réveiller un tour.
- `TriggerTurn` : démarrer le travail dès que l'agent peut l'admettre.
- `SteerCurrent` : injecter dans le tour courant uniquement.
- `InterruptAndStart` : interrompre proprement puis démarrer un nouveau tour.
- `ControlOnly` : opération de contrôle qui ne devient pas un prompt métier.

L'intention est fournie par le demandeur autorisé. L'adaptateur ne la déduit
pas à partir du seul nom de l'expéditeur. Une politique peut refuser une
intention incompatible et annoncer son repli, mais ne peut pas la transformer
silencieusement.

### États de livraison

```text
prepared
received_by_wrapper
provider_accepted
model_visible
acked | rejected | indeterminate | orphaned | expired
```

### États d'exécution

```text
queued
starting
running
waiting_approval
waiting_user_input
interrupting
interrupted | completed | failed | unreachable
```

Les deux machines sont séparées. Une livraison peut être acquittée sans que le
travail soit terminé. Une exécution peut échouer après consommation sans rendre
la livraison indéterminée.

## Identifiants et corrélation

Le modèle complet est détaillé dans `data-model.md`. Le contrat conserve au
minimum :

- `objective_id` et `delegation_id`, émis par Maicie ;
- `submission_id`, `delivery_id` et `execution_id`, émis par Bridget ;
- `agent_instance_id` et `generation`, émis par Bridget ;
- `provider_session_id`, `provider_thread_id`, `provider_turn_id` et
  `provider_item_id`, observés par l'adaptateur ;
- `client_message_id`, émis par Bridget et propagé au fournisseur quand le
  contrat le permet.

Chaque mapping est durable et horodaté. Aucun champ fournisseur ne remplace
un identifiant Bridget.

## Découpage technique

Les numéros de lots identifient les incréments et leurs gates, pas une chaîne
totalement séquentielle. Les lots 0 à 3 constituent le chemin commun. Après ce
socle, les lots 4 et 6 peuvent avancer indépendamment, le lot 5 attend la
matrice de capacités du lot 1 et les mappings provider stabilisés, et le lot 7
attend les faits d'activité, de flotte, de frontière et d'usage nécessaires.

### Lot 0 - Finaliser la session 063

Objectif : prouver le chemin nominal de steering avant de construire dessus.

- Corréler la consommation Codex sur `item/started.userMessage.clientId`.
- Séparer dans le faux fournisseur `item.id` et `clientId`.
- Ajouter les cas négatifs de mauvaise corrélation et d'événement tardif.
- Produire une preuve réelle avec le binaire configuré et sa version observée.
- Ne modifier ni Maicie ni le modèle général dans ce lot.

Gate de sortie : un message humain consommé est acquitté exactement une fois,
et un message non consommé atteint le repli borné sans rester en distribution.

### Lot 1 - Stabiliser les contrats et capacités

Objectif : disposer d'un vocabulaire neutre avant d'ajouter la persistance.

- Ajouter les intentions, origines, références et raisons structurées au
  contrat public existant avec compatibilité descendante.
- Étendre `ManagedSession` avec les identifiants, états et capacités réellement
  disponibles.
- Enregistrer chemin réel, version, empreinte et contrat supporté du fournisseur.
- Générer ou figer les schémas de versions fournisseurs supportées dans les
  fixtures de contrat, sans les charger comme dépendance runtime.
- Introduire un client Bridget public partagé seulement après preuve de trois
  consommateurs réels.

Gate de sortie : un test de contrat distingue explicitement les versions Codex
présentes sur le serveur, exerce Cursor sur l'ACP commun et refuse une opération
absente sans déduire les capacités du nom du fournisseur.

### Lot 2 - Introduire la soumission et l'exécution durables

Objectif : séparer le travail logique de la tentative de transport.

- Ajouter la soumission durable et sa politique d'admission.
- Conserver les livraisons existantes comme tentatives idempotentes.
- Ajouter l'exécution et ses transitions conditionnelles.
- Rendre les transitions atomiques avec la réinsertion de file, les issues
  d'autorisation et le réveil du worker.
- Migrer progressivement les appels existants en conservant un mode de
  compatibilité explicite.

Gate de sortie : les crashes injectés à chaque frontière ne perdent ni ne
dupliquent une soumission et aucune phase ne reste ouverte sans borne.

### Lot 3 - Exposer l'activité et l'observabilité

Objectif : rendre l'état exploitable sans lecture forensique du JSONL.

- Remplacer la projection booléenne `busy` par les états détaillés tout en
  conservant une projection de compatibilité temporaire.
- Propager les identifiants de corrélation dans journaux, métriques et traces.
- Instrumenter profondeur et âge des files, latence, erreurs, saturation,
  steering, interruption et autorisations.
- Afficher la preuve la plus récente, sa fraîcheur et la prochaine action
  autorisée dans l'UI.
- Définir des alertes sur les messages vieillissants et les tours sans progrès.

Gate de sortie : un opérateur répond aux questions de SC-004 et SC-009 depuis
les projections sans ouvrir les journaux bruts.

### Lot 4 - Ajouter le graphe d'agents et la propriété

Objectif : représenter la flotte de travail, pas seulement le parc de processus.

- Étendre la saga de création avec parent, mandat, rôle et lien durable.
- Appliquer réservations, limites de profondeur et quotas avant création.
- Ajouter les opérations distinctes d'envoi passif, relance active,
  interruption et attente événementielle.
- Définir la règle de disparition du parent et de remise des résultats tardifs.
- Agréger état et coût sur les descendants sans modifier les objectifs Maicie.

Gate de sortie : chaque agent enfant et résultat possèdent un propriétaire ou
un motif d'absence explicite, y compris après redémarrage.

### Lot 5 - Ajouter reprise et bifurcation

Objectif : exploiter les capacités natives sans verrouiller Bridget sur Codex.

- Persister les références de session et thread lorsqu'elles existent.
- Ajouter les commandes neutres de reprise et bifurcation avec capacités.
- Implémenter Codex selon son contrat natif et Cursor dans l'ACP commun, puis
  les autres fournisseurs selon leurs contrats, sans adaptateur Cursor séparé.
- Conserver la carte textuelle comme repli déclaré.
- Empêcher une ancienne génération ou un contexte incompatible de reprendre
  silencieusement une exécution.

Gate de sortie : chaque reprise indique son mode réel et conserve ascendance,
mandat, référence et preuve de compatibilité.

### Lot 6 - Découpler Bridget de l'état privé Maicie

Objectif : rendre conforme la frontière annoncée par l'ADR 003.

- Définir une projection de mission publique, versionnée, atomique et en lecture
  seule pour les vues Bridget qui en ont besoin.
- Faire produire cette projection par Maicie lors de ses mutations explicites,
  sans boucle résidente cachée.
- Retirer les imports de domaine Maicie depuis le daemon après migration des
  consommateurs.
- Ajouter `ExecutionReference` et `ExecutionProjection` au domaine Maicie.
- Consommer les événements Bridget avec curseur, fraîcheur et détection de gap.

Gate de sortie : Bridget compile et fonctionne sans dépendre du crate Maicie,
et Maicie n'accède qu'aux contrats publics Bridget.

### Lot 7 - Ajouter budgets et continuation gouvernée

Objectif : reprendre l'autonomie utile de Codex après fiabilisation du runtime.

- Mesurer temps, usage et descendants dans Bridget.
- Définir dans Maicie les limites applicables à l'objectif ou à la délégation.
- Publier séparément pause, blocage, limite d'usage et limite de budget.
- Autoriser la continuation seulement sur preuve d'inactivité et absence de
  travail concurrent incompatible.
- Exiger une décision explicite pour toute clôture métier.

Gate de sortie : chaque limite produit un arrêt ou une attente visible sans
clôture métier implicite ni nouvelle consommation après la borne.

## Stratégie de persistance et migration

1. Ajouter les nouveaux objets sous une version de schéma additive.
2. Ne pas transformer automatiquement les anciennes livraisons en soumissions
   historiques complètes. Les exposer comme héritées avec information inconnue.
3. Maintenir temporairement les projections `busy` et message existantes à
   partir du nouveau modèle, jamais l'inverse.
4. Journaliser chaque migration avec version source, version cible et compte de
   lignes, sans corps de message.
5. Refuser une version future inconnue en écriture et la rendre explicitement
   indisponible en lecture.
6. Tester crash avant commit, après commit, avant événement fournisseur, après
   événement et avant acquittement.

## Stratégie de compatibilité fournisseur

- La release approuvée fixe la liste des versions supportées et les schémas de
  contrat correspondants.
- Le démarrage observe le binaire réellement résolu et sa version.
- Les capacités négociables sont confirmées par le protocole.
- Les capacités non négociables sont issues du contrat de version testé.
- Toute divergence choisit entre refus et repli pré-documenté.
- Les événements inconnus sont conservés bruts mais ne modifient aucun état
  canonique sans mapping explicite.

## Sécurité et permissions

- Séparer l'origine déclarée, l'identité de connexion et l'autorité de contrôle.
- Appliquer le moindre privilège à chaque commande de contrôle.
- Normaliser les demandes d'autorisation et leur résolution sans exposer le
  contenu sensible dans les métriques.
- Borner et signaler les demandes répétées après une décision.
- Refuser par défaut les identités, générations, threads et tours incohérents.
- Conserver le modèle local coopératif actuel comme limite déclarée, sans
  prétendre fournir une isolation hostile multi-utilisateur.

## Observabilité

### Journaux

Chaque transition structurée inclut selon disponibilité :

- `objective_id`, `delegation_id`, `submission_id`, `delivery_id` ;
- `execution_id`, `agent_instance_id`, `generation` ;
- fournisseur, version, session, thread, tour, item et client message ;
- état précédent, nouvel état, raison et source ;
- horodatage monotone local et horodatage fournisseur brut.

Le corps des messages et le raisonnement ne sont pas des attributs de métrique.

### Métriques

- compteurs de transitions et issues par fournisseur et version ;
- histogrammes de latence de remise, visibilité, steering, interruption et
  reprise ;
- jauges de profondeur de file, âge maximal, exécutions actives et attentes ;
- compteurs d'événements tardifs, divergences, boucles d'autorisation et replis ;
- mesures d'usage et de descendants avec cardinalité bornée.

### Traces

Une soumission ouvre le contexte causal. Les livraisons, commandes fournisseur,
événements et transitions d'exécution deviennent des spans ou événements
corrélés sans utiliser le contenu utilisateur comme identifiant.

## Stratégie de test

1. Tests unitaires des transitions pures et invariants de corrélation.
2. Tests de contrat par version et chemin fournisseur, incluant Codex
   app-server, Claude stream-json et Cursor via ACP, à partir des schémas
   officiels ou du contrat ACP supporté.
3. Faux fournisseurs adversariaux : silence, EOF, surcharge, mauvais
   identifiant, sortie tardive, boucle d'autorisation et événement inconnu.
4. Tests d'intégration SQLite avec crash injecté aux frontières atomiques.
5. Tests de reprise après réinscription et changement de génération.
6. Scénarios d'acceptation SPEC-064 lisibles par un opérateur.
7. Mesures réelles contrôlées seulement après intégration et activation admise.
8. Tests négatifs de frontière Maicie : aucune transition métier dérivée du
   seul runtime.

Les mocks ne définissent jamais le protocole. Ils sont validés contre le même
schéma versionné que l'adaptateur et utilisent volontairement des identifiants
distincts.

## Déploiement et réversibilité

- Chaque lot possède sa propre session ou sous-session et sa propre preuve.
- Les nouvelles écritures sont activées avant que les anciennes lectures soient
  retirées.
- Les projections de compatibilité sont retirées seulement après mesure de
  leurs consommateurs.
- Une activation utilise une release matérialisée et admise selon l'ADR 012.
- Aucun worktree, branche non intégrée ou binaire non identifié ne devient une
  dépendance de production.
- Le retour arrière conserve les données nouvelles, mais rétablit la lecture de
  compatibilité si elle reste supportée.

## Constitution Check

| Règle | Verdict | Preuve ou décision |
|---|---|---|
| Documentation française | PASS | Tous les artefacts sont en français. |
| Cycle SpecKit | PASS | Specify, Plan, audit-existing, Tasks et Analyze précèdent le code. |
| ADR structurant | PASS | Une ADR proposée documente les trois plans et le sens des dépendances. |
| Recherche préalable IA | PASS | Baseline locale et validation live de sources primaires dans `research.md`. |
| Isolation worktree | PASS avec avertissement | Worktree dédié créé, parc déjà supérieur à cinq worktrees. |
| Tests avant livraison | PASS | Stratégie de tests par contrat, intégration, reprise et route réelle. |
| Observabilité dès conception | PASS | Logs, métriques, traces et corrélation sont des exigences de chaque lot. |
| Complexité algorithmique | PASS | Files et graphes exigent indexation et parcours bornés documentés. |
| Minimalisme | PASS | Réutilisation des crates et seams existants, aucun moteur générique. |
| Responsabilité future | PASS | Identifiants, états, preuves et suppressions de compatibilité sont explicités. |
| Privacy gate | PASS | Pas de données privées envoyées aux sources externes, recherches publiques uniquement. |

## Article XIX et XX

- Les quatre crates existants sont conservés. Aucun nouveau crate n'est un
  préalable du programme.
- Les concepts nouveaux correspondent à plusieurs usages réels : daemon,
  adaptateurs, Maicie, UI et tests.
- Les états de compatibilité ont une condition et une tâche de suppression.
- Les choix fournisseur restent isolés dans les adaptateurs.
- Chaque lot indique ce qu'un mainteneur peut observer, tester, remplacer ou
  supprimer sans contexte caché.
- Le volume de code n'est jamais un critère de succès. Les critères portent sur
  les issues, preuves, temps de diagnostic et non-régression.

## Risques et réponses

| Risque | Réponse planifiée |
|---|---|
| Layercake de contrats | Étendre les seams existants, gate de trois consommateurs avant extraction. |
| Double source de vérité | Autorité explicite pour chaque entité et projection avec fraîcheur. |
| Migration big bang | Lots additifs, projections de compatibilité et activation par release. |
| Faux tests fournisseur | Schémas versionnés et identifiants volontairement distincts. |
| Cardinalité télémétrique | Labels bornés, identifiants détaillés seulement dans traces et journaux. |
| Autonomie trop précoce | Budgets et continuation dans le dernier lot uniquement. |
| Couplage Codex | Capacité neutre, repli explicite et implémentation par adaptateur. |
| Couplage Maicie | Projection publique et retrait de la dépendance daemon vers Maicie. |

## Livrables de conception

- `spec.md` : besoins, scénarios et critères de succès.
- `research.md` : décisions, alternatives, sources et red flags.
- `data-model.md` : entités, relations et transitions.
- `contracts/execution-control.md` : commandes et événements du plan de contrôle.
- `contracts/provider-capabilities.md` : négociation et preuve des capacités.
- `contracts/mission-execution-boundary.md` : frontière Bridget et Maicie.
- `contracts/observability.md` : attributs, métriques et confidentialité.
- `quickstart.md` : parcours de validation futurs.
- `reuse-audit.md` : audit de l'existant avant tâches.
- `tasks.md` : séquence complète, laissée non cochée dans ce run.

## Gate avant implémentation

- La branche a été réalignée sur `e72a79b28f51d56548ce01a30c6a103005ce169d`
  et `reuse-audit` a été rejoué avec verdict PASS. Avant T001, vérifier que
  `origin/main` n'a pas avancé de nouveau et rejouer Analyze après tout delta.
- La session 063 doit être corrigée et prouvée sur le vrai schéma Codex.
- `reuse-audit.md` doit être `PASS`.
- Analyze ne doit laisser aucun finding CRITICAL.
- Les contrats d'identifiants et d'autorité doivent être approuvés.
- Les tâches doivent être séparées en incréments activables et réversibles.
- Aucune implémentation ne commence dans le présent run.
