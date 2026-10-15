# Plan d’implémentation — SPEC-167 Coordination CLI autonome

## Résumé

Bridget possède déjà le moteur complet de délégation native. Les outils MCP savent choisir un agent, lancer une mission, attendre, continuer, répondre, annuler et accuser lecture. La ligne de commande expose seulement le parcours technique `auto-delegate`. Elle n’expose pas le cycle de vie complet.

La session ajoute une façade CLI courte. Elle reste un adaptateur vers `delegation_mcp::execute`. Elle ne copie aucune règle métier. Le parent reste un agent Bridget authentifié. Un processus humain sans identité reçoit un refus sûr et une action de rattachement.

## Contexte technique

- Langage : Rust.
- CLI : `crates/bridget-daemon/src/cli.rs`.
- Adaptateur commun : `crates/bridget-daemon/src/delegation_mcp.rs`.
- Moteur natif : `crates/bridget-daemon/src/daemon/native_delegation.rs`.
- Protocole : `crates/bridget-transport/src/protocol.rs`.
- Dépendance nouvelle : aucune.
- Persistance nouvelle : aucune.
- Service nouveau : aucun.

## Constitution Check

| Règle | Décision |
|---|---|
| Processus SpecKit | Session 167 isolée avec spécification, plan, tâches, analyse et reçu. |
| Diagnostic avant modification | Le moteur, l’adaptateur MCP, la CLI et la gestion d’identité ont été lus avant le plan. |
| Réutiliser avant de créer | La CLI appelle `delegation_mcp::execute` et conserve les requêtes natives existantes. |
| Identité | Le parent reste l’identité vivante de l’agent. Aucun principal humain synthétique. |
| Minimalisme | Un groupe de commandes et un formateur. Aucun nouveau moteur ou protocole. |
| Refus sûr | Sans identité, aucune tâche n’est créée. Le message indique le rattachement requis. |
| Tests | Parseurs, sorties, refus d’identité et régressions des commandes historiques. |
| Livraison | Build signé, installation atomique et recette sans T3 si les gardes runtime le permettent. |

Aucune violation constitutionnelle n’est nécessaire.

## Architecture retenue

### 1. Ajouter une entrée simple `bridget delegate`

La forme nominale est :

```text
bridget delegate "mission"
```

Le client ajoute le dossier courant et une clé de demande. Les options avancées reprennent les champs de `bridget_auto_delegate`. Un choix explicite reste explicite. Une option absente reste absente.

### 2. Ajouter le groupe `bridget task`

Le groupe expose :

- `list` ;
- `status` ;
- `wait` ;
- `continue` ;
- `respond` ;
- `cancel` ;
- `ack`.

Les actions qui appartiennent au parent appellent les outils natifs existants. `list` réutilise la projection humaine bornée déjà fournie par `control delegations`.

### 3. Garder une identité d’agent comme frontière

`resolve_current_identity()` reste la source d’identité. La CLI ne déduit rien du dossier, du terminal ou du processus. Une invocation non rattachée refuse avant la connexion métier.

Cette règle permet deux usages sans T3 :

1. un agent lancé par un wrapper Bridget utilise directement les commandes ;
2. un utilisateur demande à cet agent coordinateur de déléguer et de suivre les tâches.

### 4. Partager un seul exécuteur et un seul formateur

Chaque action construit l’objet JSON fermé déjà accepté par `delegation_mcp::execute`. Un helper commun traduit les erreurs et imprime soit une vue humaine, soit le JSON brut.

La sortie humaine montre les identifiants durables, l’état, l’agent choisi et la prochaine action utile. `--json` écrit un seul objet JSON sur la sortie standard.

### 5. Conserver les commandes historiques

`auto-delegate`, `spawn`, `send` et `control delegations` gardent leur syntaxe. La nouvelle façade ne change pas leurs valeurs par défaut.

## Réutilisation de l’existant

- `parse_auto_delegate_args()` définit déjà les options avancées et les limites.
- `delegation_mcp::execute()` construit toutes les requêtes de délégation natives.
- `resolve_current_identity()` impose l’identité du parent.
- `NativeDelegationRequest` porte déjà les opérations de cycle de vie.
- `control delegations` fournit déjà une liste globale bornée et un affichage humain.
- Le magasin natif conserve déjà tâches, résultats, reçus, rejets et clés de rejeu.

## Fichiers prévus

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/cli.rs` | Parser `delegate`, groupe `task`, exécution commune, aide et tests unitaires. |
| `docs/decisions/167-coordination-cli-autonome.md` | Invariant d’identité et architecture de façade. |
| `docs/reference-communication.md` | Parcours nominal sans T3 et commandes de cycle de vie. |
| `README.md` | Exemple court si le point d’entrée utilisateur n’est pas déjà visible. |
| `CHANGELOG.md` | Résumé utilisateur de la nouvelle façade. |
| `specs/167-coordination-cli-autonome/*` | Artefacts, preuves et reçu. |

## Phases d’implémentation

1. Figer le contrat CLI et l’audit de réutilisation.
2. Ajouter les tests RED des parseurs et de la traduction vers les outils natifs.
3. Ajouter `bridget delegate` avec mission positionnelle et options facultatives.
4. Ajouter `bridget task` et son exécuteur commun.
5. Ajouter les vues humaine et JSON.
6. Prouver le refus sans identité et la compatibilité de `auto-delegate`.
7. Exécuter les tests ciblés puis la suite adaptée au risque.
8. Mettre à jour la documentation et la décision d’architecture.
9. Contre-relire, auditer, intégrer, pousser, construire et activer.
10. Faire une recette sans T3 et nettoyer la branche de session.

## Stratégie de test

- Parser nominal : mission seule, dossier courant, clé générée.
- Options : type, modèle, effort, posture, délai, garde, demande enfant, urgence et livrable.
- Refus : option inconnue, doublon, mission vide, borne dépassée et identité absente.
- Cycle : chaque sous-commande produit le nom d’outil et la charge attendus.
- Rejeu : une clé fournie est conservée à l’octet près.
- Sortie : JSON unique sur stdout ; vue humaine avec identifiants utiles.
- Compatibilité : tests existants de `auto-delegate`, délégation MCP et garde CLI.
- Recette : agent Bridget actif, T3 arrêté, délégation puis status/wait/ack.

## Observabilité et preuve

La validation conserve :

- la branche et les commits ;
- les résultats RED puis GREEN ;
- les tests ciblés et la suite complète applicable ;
- le résultat de la contre-revue ;
- le hash du binaire installé ;
- l’état du daemon avant et après activation ;
- une recette sans processus T3 ;
- la preuve que les commandes historiques restent reconnues.

## Risques et protections

| Risque | Protection |
|---|---|
| Fabriquer une identité humaine | `resolve_current_identity()` reste obligatoire. |
| Dupliquer les règles MCP | Un seul appel à `delegation_mcp::execute`. |
| Perdre la clé automatique | L’afficher avant ou avec le reçu ; permettre `--request-id`. |
| Polling dans les scripts | Réutiliser l’attente native bloquante. |
| Sortie JSON polluée | Un seul objet sur stdout ; diagnostics sur stderr. |
| Changer les defaults | Ne pas injecter les options absentes. |
| Casser `auto-delegate` | Parser et commande historiques inchangés, avec régressions. |

## Charge cognitive future

La façade reprend les noms métier déjà présents dans MCP. Elle ajoute deux points d’entrée faciles à mémoriser : `delegate` pour démarrer et `task` pour piloter. Le moteur, le protocole et le stockage restent uniques.
