# Audit de reutilisation de l'existant — conversation complète

## Decision

Statut: PASS
Date: 2026-11-10
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/258-conversation-complete/specs/258-conversation-complete

Conclusion courte: étendre les composants et puits existants. Les nouveaux
contrats relient des autorités différentes ; aucun second service ni store.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 9 |
| Items audites | 9 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Rendu | ThreadTimeline/MessagesTimeline | T3 ChatView.tsx:3336 | Étendre le raccord existant |
| Suivi | useTimelineFollow | T3 ChatView.tsx:2126 | Recevoir les faits du journal |
| Boîte | ChatComposer | T3 ChatComposer.tsx:353 | Même éditeur, transport distinct |
| Sélecteurs | ComposerModelEffort | T3 ComposerModelEffort.tsx:95 | Choix validés par la cible |
| Brouillon | composerDraftStore | T3 useComposerProviderModel.ts:46 | Garder le store existant |
| Lecture et commandes | BridgetReader/BridgetLineage | T3 BridgetLineage.ts:22 | RPC additive, aucun nouveau service |
| Autorité humaine | HumanLineage | lineage_protocol.rs:359, native_lineage.rs:94 | Capacité dédiée pour les effets |
| Modèle/effort | SelectRuntime | daemon.rs:14439, wrapper.rs:5441 | Factoriser la destination, garder la confirmation |
| Remise et interruption | Send/ExecutionControlCommand | daemon.rs:23839, :12987 | Réutiliser reçus et gardes |

## Existant potentiellement pertinent non mentionne

Aucun : les trois puits de mutation sont explicitement repris.

## Duplications evidentes

Aucune. Une seconde boîte, un autre renderer et un catalogue parent répliqué
sont explicitement interdits par le plan.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS Bridget | build.py et livraison native | Cache entretenu, actif/retour préservés |
| AGENTS T3 | base active protégée | Test isolé et aucun restart T3 |
| Constitution XIX/XX | Réutiliser et expliquer | Aucun wrapper sans responsabilité |
| my-specify-all | Pipeline complet | Analyze, Converge et livraison requis |
| preuves-correction | Circuit visible réel | Clics et diaporama dans le commit |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 149 | Lignée froide humaine | Conserver les gardes de racine |
| 154 | Workspace | Aucun déplacement/rejeu |
| 155 | Questions | Ne pas confondre saisie et approbation |
| 241 | Reprise et autorité | Revalider après E/S |
| 251 | Rendu commun | Reprendre le commit T3 75c7dafe51 |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg HumanLineage / SelectRuntime | crates | Puits et matrice de capacités trouvés |
| rg bridgetTaskRef / canOperateThread | T3 server/web | Gardes ordinaires à conserver |
| rg useTimelineFollow / ChatComposer | T3 web | Raccords UI existants |
| rg lineageCancel / stdin | T3 BridgetReader/processRunner | CLI stdin et service existants |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Contrat de boîte | Étendre HumanLineage | Autorité humaine différente du parent-agent | 2026-11-10 |
| RPC de boîte | Étendre ports existants | Aucun équivalent pour ces mutations humaines | 2026-11-10 |
| Runtime distant | Étendre le chemin de contrôle | Aucune substitution par une sélection locale | 2026-11-10 |
| Interface | Réutiliser | Exigence utilisateur explicite | 2026-11-10 |
| Adapter les faits natifs aux contrôles T3 | Créer BridgetComposer.logic.ts | ChatComposer/useComposerProviderModel attendent ServerProvider ; aucun adaptateur de catalogue de tâche humaine n'existe. Traduction seulement, pas de catalogue parallèle | 2026-11-10 |
| État de la commande native | Créer useBridgetComposer.ts, étendre composerDraftStore | Trois effets réels : envoi, sélection, interruption. useComposerSend est propriétaire des tours T3 et ne peut prendre cette autorité ; les reçus incertains vont dans le store de brouillon existant | 2026-11-10 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Arbitrage de réalisation natif (10 novembre, root)

La recherche des reçus existants a couvert `native_delegation_cancel_receipts`,
`idempotency` (Send/Spawn, PublicResult sans charge de réponse) et
`execution_control_commands` (accusés de contrôle, sans réponse modèle/capacités).
Aucun ne peut conserver la réponse fermée complète du compositeur et son canon
(tâche, racine, ronde, opération) à travers une perte du client.
Décision : petite table `native_delegation_compose_receipts` dans le même
DelegationStore, sans nouveau store ni service. Elle est réservée avant effet,
rejouée sans remise et supprimée avec l'arbre conservé. La remise et les contrôles
gardent leurs puits existants et permettent de résoudre leurs accusés tardifs.
Le catalogue worker220 a été examiné : il décrit la connexion, et ne prouve pas
à lui seul le compte encore chargé par un enfant existant. Il faut une lecture
du pilote ciblé avant de déclarer le catalogue propre à l'enfant. Le raccord
ci-dessous fournit cette preuve sans utiliser le catalogue de connexion.

Raccord complété : ReadRuntimeCatalogue utilise ManagedSession et le wrapper
ciblé. Codex partage maintenant runtime_catalogue_until et le parseur existant
model_page entre lecture et sélection. La voie worker220 et son helper de
connexion envisagés ont été retirés du code : ils ne prouvaient pas la session
chargée. Une attente corrélée bornée dans DaemonState reçoit seulement le
wrapper ciblé ; aucun service ni collecteur séparé.

Les contrôles communs acceptent deux capacités optionnelles : runtimeControlsAvailable et modelSelectionAvailable. Elles limitent les surfaces du compositeur aux opérations réellement transportables, sans second menu. Les valeurs par défaut conservent les pages normales. Le hook refuse également toute opération absente des capacités et relit la cible lors d’un changement de génération de lignée.

IncidentOAuth : parser/classifier ClaudeStreamJson, task_view/current_task_view et deux notifications d’échec existants relus. Étendre ces puits, pas de service d’authentification ou table ajoutée. provider_error_recovery partage le guidage fermé entre les trois sorties (statut et deux notifications) ; aucun texte d’erreur libre ni identifiant de compte copié. La reconnexion du compte reste une interaction utilisateur distincte, jamais un repli automatique.
