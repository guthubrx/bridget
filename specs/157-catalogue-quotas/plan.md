# Plan - Session157 catalogue fiable, quotas, remplacement

**Spec** : `specs/157-catalogue-quotas/spec.md`
**Branche** : `session-157-catalogue-quotas`
**Worktree** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas`
**Date** : 2026-10-11

## Résumé

Trois corrections, dans l'ordre de dépendance :

1. un verdict unique de cible, partagé par le catalogue et l'admission, avec le
   préflight de lancement extrait de `submit_spawn` (R1 à R3) ;
2. une mémoire de limites par fournisseur, montrée au catalogue et respectée à
   l'admission (R4, R5) ;
3. un remplacement ordonné demandé par le parent, signalé et durable (R6) ;
La reprise sur limite d'usage est sortie de la 157 (session future 160, R7 et
`spec.md`). Aucune phase ne dépend de 152.

## Contexte technique

- Langage : Rust 2024, workspace à 3 crates. Travail dans `bridget-daemon` et
  `bridget-transport`.
- Stockage : aucune nouvelle table. Les nouveaux champs de `Task` sont des clés
  JSON optionnelles du payload existant.
- Tests : recette `umask 077` (`quickstart.md`).
- Aucune nouvelle dépendance.

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.fallback`, `FallbackTarget` |
| `crates/bridget-daemon/src/provider_limits.rs` (nouveau, environ 80 lignes) | mémoire des limites, règle d'épuisement, lecture `quota` |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module |
| `crates/bridget-daemon/src/daemon.rs` | champ `provider_limits` de `DaemonState`, écriture dans `handle_rate_limit` |
| `crates/bridget-daemon/src/registry.rs` | réemploi de pin_delegation_model avant préflight pour vérifier le modèle demandé ; visibilité pub(crate), aucun nouveau helper |
| `crates/bridget-daemon/src/lifecycle.rs` | extraction de `launch_preflight` depuis `submit_spawn` ET `prepare_spawn_parts` (erreur d'environnement propagée), exposition du code de `refusal_record` |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | `resolve_target`, catalogue réécrit sur le verdict, quota, boucle de remplacement, extension du `failure` existant (code:JSON) |
| `crates/bridget-daemon/src/delegation.rs` | champ `substitution` |
| `crates/bridget-daemon/src/delegation_mcp.rs` | schéma `fallback`, message de refus avec détail, description du catalogue |
| tests des mêmes modules, `native_delegation_permissions149_tests.rs` | tests `native157_*`, mise à jour du test qui fige les clés du catalogue |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | catalogue, quotas, `fallback`, refus |
| `docs/decisions/157-catalogue-quotas-remplacement.md` | ADR |

Un seul module Rust nouveau, `provider_limits.rs`, avec 3 usages réels
(écriture, catalogue, admission). La séparation évite d'ajouter de la
logique pure à un fichier de 26 000 lignes (`daemon.rs`).

## Décisions architecturales

- **D-A** Un verdict, pas deux (R1, R2). Le catalogue appelle `resolve_target`.
  L'admission aussi. La dérive devient impossible par construction, et un test
  matrice la garde.
- **D-B** Le préflight vient de `submit_spawn` (R3). Il y reste appelé, donc le
  lancement et l'admission ne peuvent pas diverger.
- **D-C** Mémoire de limites volatile par type d'agent (R4). Pas de table SQL :
  les sources republient. Dette notée.
- **D-D** Garder Err(String), avec porteur code:JSON (message/detail).
  Étendre le seul failure existant pour ces préfixes. Codes nus et formats
  brief/workspace conservés ; listes comparées sur le code canonique, pas le
  JSON. Aucun nouveau type, helper ni failure_with_detail.
- **D-E** Le remplacement réécrit la cible dans `record.request` et garde
  l'enveloppe d'origine dans `canonical` (R6). En worktree, Probe choisit une seule cible et sa définition épinglée ;
  Reserve et Commit la consomment. Commit revalide ce même candidat sans
  resélection, avec gardes de chemin et diagnostics154 conservés.
- **D-G** Pas de mesure d'optimisation avant la mesure de référence (Art. XXII,
  R8).
- **D-H** Ordre fixe par candidat : type, modèle, effort, puis préflight,
  politique, quota (R10). Le remplacement ne masque jamais une erreur
  d'appelant.
- **D-I** Garantie de cohérence bornée au contexte déclaré par le catalogue
  (`verdict_context`), état inchangé (R2).
- **D-J** L'épuisement est attesté par fenêtre. Un statut Codex global ne prouve
  aucune fenêtre (R4).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 157 prévue (T024) |
| VIII Forensic | OK | cause du constat lue dans le code, test rouge avant correction (T007), fait réel de l'incident cherché (T002) |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | worktree157 dédié, exécution séquentielle dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Mémoire de limites : écriture et lecture O(log n), n = types d'agent (moins de 20),
  8 fenêtres au plus par type.
- Catalogue : O(p) fournisseurs, avec 3 calculs de politique chacun (avant : 2).
  Mesure avant et après (T001, T012). Une seule passe sur la mémoire de limites.
- Admission avec remplacement : au plus 4 candidats, donc O(1) calculs de
  politique.

### Minimalisme (Article XIX)

- Un seul module nouveau, 3 usages.
- Pas de table, pas de dépendance, pas de CLI nouvelle.
- Réutilisés : `submit_spawn` (préflight), `refusal_record` (codes),
  `child_policy` (droits), `canonical` (enveloppe), `Presence.rate_limits`
  (sources).
- Rejetés : vérification d'authentification par appel fournisseur, bascule
  automatique, bascule en cours de mission, liste de remplacement dans le
  attente d'un quota futur à l'admission.

Potentiel minimalisme : le verdict partagé supprime la logique de test dupliquée
du catalogue (environ 10 lignes). Le reste est du code neuf justifié par les
critères de succès.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Non vérifié dans ce dépôt : l'incident réel (R1) et le nom enregistré par les
  présences des trois sources (R4). Suivis dans `analyze-report.md`.
- Le catalogue dit ce qu'il ne vérifie pas (`checks.auth: false`).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | incident du 2026-10-11 (catalogue vrai, admission refuse), binaire absent crée une tâche, quota épuisé admis |
| Unitaire | règle d'épuisement (table), mémoire (clé, péremption, bornes), préflight, extension du `failure` existant (code:JSON), sérialisation sans nouveau champ |
| Matrice | parent (Claude sans `launch_context`, Claude avec, Claude avec source illisible, Codex, aucun fait) × cible (claude, codex, glm, codex-pro) × posture (3) : catalogue égal admission à contexte égal ; cas hors garantie documentés (fait limité à une demande, sous-répertoire, binaire changé) |
| Remplacement | ordre, motifs permis et interdits, `model_unavailable` jamais remplacé, rejeu, doublons, 3 candidats au plus, `fallback_exhausted` |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` |
| Mesure | latence du catalogue avant et après |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Cause R1 fausse ou double (`launch_context` présent mais source illisible) | deux tests rouges T007 ; fait réel expurgé en T002 ; si un test passe déjà, arrêter et observer (Art. VIII) |
| Extraction du préflight incomplète | extraction depuis `submit_spawn` et `prepare_spawn_parts`, définition épinglée, cas facturation et environnement (T005) |
| Remplacement qui masque une erreur d'appelant | modèle et effort avant les motifs remplaçables (R10, T017) |
| Épuisement lu sur un statut Codex global | épuisement attesté par fenêtre seulement (R4, T004) |
| Catalogue plus lent (3e politique) | mesure T001 et T012, seuil 2 fois, plan de repli en R8 |
| Clé `agent_type` des présences différente du registre | test T004 sur les trois sources |
| Refus plus précoce visible (`command_missing`) | documenté, test, annoncé dans le contrat |
| Collision d'édition dans `native_delegation.rs` | exécution séquentielle, une tâche à la fois |

## Livraison

Livraison Git, binaire signé installé, activation Bridget et nettoyage autorisés
à chaque spec. T3 reste ouvert. Diff cumulé enregistré dans `validation/diff-157.patch`.

## Précisions retenues de la revue157-r1 — appliquées au GO157

GO157 le2026-10-12 après livraison et activation155/156. Base source156
cf498755dc871b51eb8c0add5f8a882d67651d2a, base Git avec clôture documentaire
8b5b313694cf05eabfe6a86a970b96f03b015f49. Les références150/1f2c661d
restent des preuves historiques ; les fonctions sont revalidées sur cette base.

Le verdict conserve l’ordre type/modèle/effort, préflight, politique, gardes
liées aux options de l’appel, quota. Workspace et, après livraison155,
child_requests sont non remplaçables pour la cible demandée ; un candidat
explicitement listé refusé est noté dans tried. Aucun gain de droits ni
contournement des gardes du parent. En worktree, sélection unique en Probe,
candidat et définition épinglés transmis à Reserve puis Commit. Commit
revalide ce même candidat dans le dossier enfant réel ; si une garde échoue,
refuser sans nouvelle sélection et conserver les diagnostics de chemin154.
Les options immuables et canonical original restent distincts de request effectif.

Quota : huit fenêtres ; purger les passées avant l’écriture. Une neuvième clé
évince le plus petit observed_at, égalité départagée par clé lexicale.
unknown du hook n’est pas une preuve seul ; pourcentage>=100 avec retour
futur épuise toute source. provider_limit_reached_window_unknown est l’unique
contrainte informative possible si can_run, hors famille verdict et jamais
rendue comme refus d’admission. Aucun nouveau champ public.

Refus enrichi : Err(String) sous forme code:JSON contenant message/detail.
Étendre le seul failure existant ; les comparaisons de listes portent sur
le code canonique. JSON non conforme : refus fermé, aucune valeur inventée.
Les anciens codes nus et préfixes restent compatibles. Aucun nouveau type,
helper, framework, table, outil, ID ou scénario autonome. Les tests prévus
T004/T016/T017/T018 couvrent ces précisions. Les26 tâches restent ouvertes.
