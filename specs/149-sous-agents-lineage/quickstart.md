# Guide rapide - sous-agents et lignée (session 149)

**Statut au 2026-10-10 : livré sur disque, non activé. Reçu final : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/final.md.** Binaire livré : build 072161262a7d, sha256 6df86239b638dd83e29eb99640606e7e1f3ecb7b62e75cf33e6e781fdbc7a6f8. Les 45 tâches sont validées et cochées par le principal. Les recettes finales réseau r4 et Codex vivant r7 sont validées. Les blocs anciens ci-dessous restent historiques ; le reçu final prévaut. Aucun redémarrage ni activation différée.

Date : 2026-10-10. Document de travail. Mise à jour : après les preuves r9 (tests), r6 (smoke réel avec relance), r3 (réseau et reprise, interface) et la revue des sources r2.

---

## 1. Ce que permet la session 149

Un agent parent confie une mission à un sous-agent. Le sous-agent travaille dans un dossier du projet. Le parent suit l'avancement, lit le journal et peut annuler la mission.

- Une mission s'appelle une **tâche**. Le sous-agent s'appelle un **enfant**. Le **parent** est l'agent qui a créé la tâche.
- Un enfant n'est pas une conversation T3. Dans T3, il apparaît comme un fil virtuel en lecture seule. Il ne reçoit pas de tours de conversation et ne parle pas à l'humain.
- Le parent reste responsable du résultat.
- L'enfant hérite des droits du parent. La session 149 ne crée aucun droit nouveau. Aucun humain ne doit accorder un droit pour qu'une délégation marche.

**Pourquoi :** sans cette couche, un sous-agent disparaît de la vue dès qu'il travaille. On ne sait plus qui a lancé quoi, ni ce qu'il a fait.

---

## 2. État du runtime au 2026-10-10

| Élément | État |
|---|---|
| Application T3 installée | Version 0.0.45-local.148. La session 149 ne l'a pas modifiée. |
| Code chargé dans l'application en marche | Non vérifié pour 149. Aucune relance pendant la session 149. |
| Sources 149 | Dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`. Non commitées, non fusionnées, non poussées, non installées. |
| Binaire candidat privé (r9) | Release immuable `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975`, SHA256 `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8`. Hors installation, non encore utilisé pour les essais réels. Débogage associé : `bridget-0fb06ae920e6`. |
| Binaires des preuves réelles antérieures | Recettes r5 : release `bridget-abfb346e23cc`. Smoke r6 et preuves réseau r3 : release `bridget-0a29ad9b2cdb`. Hors installation. |
| Recettes réelles GLM et Codex (r5, r6) | Preuves disponibles. Modèles réels `glm-5.3-flash` et `gpt-6.1-sol` (effort `high`). Smoke r6 : GLM PASS ; arrêt du daemon pendant un enfant Codex PASS. La relance avec parent Codex vivant a demandé un contournement : pas un PASS propre. Cases T037 et T038 non cochées. |
| Interop réseau et reprise (r2, r3) | Interop 65/65 (r2, pair Codex simulé). En r3, F2 est corrigé (R9.4.a à i). O4.6 (relance rapide) et E2 sont en FAIL sur le binaire r8 : corrections de source approuvées, runtime à prouver en r4. T039 reste partiel. |
| Tests natifs (r9) | 1840 PASS agrégés, 0 FAIL final. Trois échecs transitoires sous charge, rejoués seuls en PASS. SC005 p95 non conclu. Aucun claim global GREEN. |
| Revue des sources (r2) | SOURCE_ONLY_APPROVE, lecture seule. Production de 111 fichiers, empreinte `b2b87458`. |

**Conséquence :** les preuves réelles viennent des recettes r5. Elles n'utilisent pas T3. Les preuves avec T3 utilisent un serveur T3 réel et un faux pair Codex, ou un daemon simulé. Chaque preuve indique son niveau dans `implementation.md`.

---

## 3. Points d'entrée

- **MCP Bridget** (appelé par l'agent) : `bridget_capabilities`, `bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`. Le statut et l'annulation demandent seulement un `task_id`. La délégation MCP ne demande pas de fil T3.
- **CLI Bridget** (terminal) : `bridget lineage inspect`, `bridget lineage watch`, `bridget lineage cancel`. Chaque commande demande `--json`, `--t3-thread` (fil T3 du parent) et `--project-root` (chemin absolu). Le fil T3 sert de lien de lecture à la CLI. Il ne conditionne pas la délégation.

Sources (chemins absolus) :

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/cli_lineage.rs` : grammaire CLI fermée. Une option inconnue est refusée.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_mcp.rs` : schémas MCP.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md` : contrat CLI.

---

## 4. Parcours : déléguer, suivre, annuler

### Étape 1 - Lire le catalogue

`bridget_capabilities` renvoie, pour chaque type d'agent, ses modèles, ses efforts et ses postures. Il indique si l'héritage est possible (`inherit`) et, sinon, pourquoi (`inherit_refusal`). Ne pas deviner un nom d'agent ni un modèle.

### Étape 2 - Déléguer en un seul appel

Appeler `bridget_delegate` une seule fois par mission.

| Champ | Obligatoire | Rôle |
|---|---|---|
| `request_id` | oui | Identifiant de la demande, de 1 à 128 caractères |
| `agent_type` | oui | Type d'agent, pris dans le catalogue |
| `model` | oui | Modèle exact, pris dans le catalogue |
| `task` | oui | Texte de la mission, 65 536 caractères au plus |
| `cwd` | oui | Dossier de travail, dans le projet |
| `effort` | non | Effort demandé, ou `null` |
| `posture` | non | `discovery` ou `development` |

**Règles de posture :**

- **Absente (cas normal) :** l'enfant hérite de la politique déjà prouvée du parent. Aucun droit nouveau n'est ajouté. Les recettes réelles r5 n'ont pas passé de posture.
- **`discovery`, parent attesté :** réduction en lecture seule de la politique héritée. Cette valeur ne donne jamais un droit de plus.
- **`discovery`, parent non attesté :** chemin de compatibilité 148. Il exige un droit déjà accordé. Ne pas l'utiliser pour une écriture en 149.
- **`development`, parent non attesté :** refusé avec `permission_attestation_unavailable`.

**Règle de rejeu :** même `request_id` et même contenu renvoient la même tâche. Même `request_id` avec un contenu différent donne `envelope_mismatch`.

### Étape 3 - Suivre l'avancement

- Statut court (MCP) : `bridget_task_status` avec `task_id`.
- Journal (CLI) :

```text
bridget lineage inspect --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --action journal --task UUID_TACHE --json
```

- Détail par pages : `--action show --task UUID_TACHE --offset N --limit N --json`. `--limit` vaut 16384 au plus.
- Liste : `--action list --limit N --json`. `--limit` vaut 50 par défaut et 100 au plus. `--cursor` reprend la liste.
- Suivi continu : `bridget lineage watch --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --json`.
- Options du journal : `--after-seq N` reprend après l'événement N. `--follow` suit les nouveaux événements. Le code existe. Aucun test de bout en bout ne le prouve (`implementation.md`, O5).

### Étape 4 - Annuler

- Annulation (MCP) : `bridget_task_cancel` avec `task_id`.
- Annulation (CLI) :

```text
bridget lineage cancel --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --task UUID_TACHE --request-id UUID_DEMANDE --json
```

`--request-id` est un UUID canonique. Rejouer la même annulation renvoie le même reçu. L'annulation arrête la tâche et ses descendants actifs. Elle n'envoie aucun message à l'enfant. Un autre parent ne peut pas annuler la tâche.

**Preuve réelle :** le parent a annulé un enfant Codex après 20 secondes de travail. Son fichier a cessé de grossir et est resté inchangé six secondes après le contrôle. Aucun processus n'est resté (r5, R5). Le délai de traitement de l'annulation n'a pas été mesuré. Le cas GLM n'a pas été joué (limite 3 de `native-real-recipes-sonnet-r5.md`).

---

## 5. Dans l'interface T3 (lecture seule)

- Les sous-agents n'apparaissent pas dans la barre latérale de premier niveau.
- Le panneau **Lineage** affiche l'arbre des tâches du fil parent. Une tâche se lit dans son journal et dans son résultat.
- Il n'y a pas de zone de saisie pour un enfant. T3 n'envoie aucun message à un enfant.
- Le bouton **Arrêter** demande l'annulation de la tâche, et rien d'autre.

**Périmètre de la preuve (T040) :** r5 : interface web du worktree, serveur T3 réel, navigateur de prévisualisation, daemon simulé, données synthétiques. Largeurs 375, 480 et 1280 px : la dernière ligne, la barre et « Arrêter » restent visibles. r1 (`validation/ui-native-recipe149-sonnet-r1.md`) : même interface avec le daemon réel r8 et des enfants Codex fermés (faux serveur app-server) ; APPROVE sur ce périmètre. Pas de modèle réel dans l'interface, pas de test sur l'application de bureau ni sur mobile. Un enfant réel affiché dans l'interface T3 n'est pas prouvé (SC001, `implementation.md`, section 7).

---

## 6. Droits et limites

- **Héritage, sans nouveau grant humain.** La session 149 ne crée aucune ligne de droit. La table `native_delegation_grants` reste à 0 ligne pendant les recettes réelles r5 et le smoke r6. L'enfant reçoit les règles du parent, lues depuis le fait attesté du parent.
- **Le parent doit avoir un droit fournisseur observé**, lié à sa session. Sinon, Bridget refuse avec un code nommé. Il n'y a pas de repli silencieux.
- **Un enfant ne reçoit jamais plus que son parent.** La grammaire CLI 149 ne contient aucun drapeau qui approuve tout. Une politique « tout autorisé » du parent peut toutefois être héritée. Elle reste visible dans la définition de la mission.
- **Règles d'un parent Claude (GLM) :** l'enfant applique les règles `allow` et `deny` du settings projet et de l'utilisateur. Le snapshot de la tâche garde 17 sources révisionnées (r5, R3). Un `--settings` utilisateur explicite fait refuser la délégation avant tout effet (`permission_source_unavailable`).
- **Mapping de Codex vers GLM (figé) :**

| Droit du parent (Codex) | Effet sur l'enfant GLM | Preuve réelle (r5) | Refus possible |
|---|---|---|---|
| `dangerFullAccess` | Bypass dans la définition de la mission seulement | R2a : écriture réussie | Aucune réduction silencieuse du réseau |
| `readOnly` | Discovery en mode plan restreint | Non joué en réel (plan non exécuté) | `permission_mapping_unavailable` si le réseau demandé ne se traduit pas |
| `workspaceWrite` | Sans confinement OS équivalent | R2b : `provider_confinement_unavailable`, 0 ligne | `provider_confinement_unavailable` |

- **Codex vers Codex :** l'enfant garde `approval never` et `workspaceWrite` du parent (R2c). Il écrit dans le workspace. Il est refusé hors workspace par le bac à sable de Codex.
- Bridget ne promet pas de confinement OS entre fournisseurs.
- Bridget ne modifie jamais la configuration globale d'un fournisseur pour élargir un droit.
- **Limite de l'outil GLM :** le refus de l'enfant GLM vient d'une règle du CLI Claude, pas d'un confinement du système. L'enfant GLM n'a pas `Bash` dans ce profil.
- **Secrets :** un jeton, un mot de passe ou une clé ne va jamais dans `task`. Le wrapper injecte les identifiants privés.
- **Résultat :** un enfant qui répond « OK » n'est pas un succès. Le résultat doit être relié à la tâche par son identifiant.

---

## 7. Codes d'erreur

| Code | Sens | Action | Preuve |
|---|---|---|---|
| `invalid_request` | Commande mal formée : option inconnue, UUID non canonique, chemin non absolu | Corriger l'appel | S6.11 (r2) |
| `binding_unavailable` | Le lien avec le fil T3 est indisponible | Vérifier le fil T3 | R4.1, V12 (r2) |
| `project_mismatch` | Le projet ne correspond pas à la tâche | Vérifier `--project-root` | S6.3 (r2) |
| `task_unavailable` | La tâche n'existe pas, ou n'appartient pas à ce parent | Vérifier le fil et le projet | R2.1, R3.1 (r2) |
| `snapshot_changed` | La liste a changé entre deux pages | Relire depuis la page 1 | S6.10 (r2) |
| `journal_unavailable` | Le journal de la tâche n'est pas lisible | Réessayer. Vérifier le magasin. | Test unitaire |
| `result_offset_invalid` | Décalage hors du résultat | Corriger `--offset` | Test unitaire |
| `envelope_mismatch` | Même `request_id`, contenu différent | Renvoyer le contenu d'origine, ou un autre `request_id` | S6.8, R1.2 (r2) |
| `resource_limit` | Limite de taille ou de nombre atteinte | Réduire la demande | Test unitaire |
| `store_unavailable` | Magasin verrouillé. Réessai possible. | Réessayer plus tard | Test unitaire |
| `invalid_output` | Sortie de la CLI non décodable (lecteur T3) | Vérifier la version de la CLI | Test T3 |
| `delegation_grant_required` | Révocation explicite par un humain, ou `discovery` sans droit existant | Demander à l'humain. Bridget ne recrée pas le droit seul. | Test unitaire |
| `permission_not_inherited` | Le parent n'a pas le droit demandé, ou sa racine est révoquée | Réduire la mission. Ne pas chercher de droit. | S3.9, R8.2 (r2) |
| `permission_attestation_unavailable` | Aucune preuve de droits du parent | Relancer le parent avec une source de droits attestable, en options privées | R4b (r5, daemon absent), S2.1 (r2) |
| `permission_source_unavailable` | Une source de règles est opaque | Utiliser une source inline ou connue | R4a (r5) |
| `permission_mapping_unavailable` | Le réseau demandé ne se traduit pas | Réduire la demande ou changer de cible | Test unitaire |
| `provider_confinement_unavailable` | Pas de confinement équivalent | Changer de cible ou réduire la mission | R2b (r5) |
| `settings_revision_changed` | Une source de droits a changé après l'admission | Refus avant tout effet. Vérifier les droits, puis relancer. | Test unitaire |
| `provider_permission_denied` | Le fournisseur a refusé un outil | Lire le journal. La tâche passe en `failed`. | R1 (r5) : tâche `failed`, une seule ligne |
| `unreachable` | Le daemon a été arrêté pendant la mission | Réessayer le parent. Rejeu : même tâche, aucun relancement. | R9.2.a à e (r2) |

---

## 8. Six scénarios de panne

Chaque scénario a une preuve. Le tableau sépare la preuve sur fixture ou unitaire, la preuve réelle et ce qui reste ouvert. Détail : `implementation.md`, section 6.

| # | Scénario | Preuve sur fixture ou unitaire | Preuve réelle | Reste ouvert |
|---|---|---|---|---|
| 1 | Rejeu : même `request_id`, même contenu | Tests natifs et T3 (D10) | R1.1 (r2) : 20 rejeux, 1 lancement, 1 tour, 1 remise | Rejeu avec modèle réel |
| 2 | Annulation : le parent arrête la tâche et ses enfants | Tests natifs et T3 (D13) | R5 (r5) : enfant Codex annulé après 20 s de travail, 0 processus restant. R2.2 (r2) : deux PID réellement terminés | Annulation d'un enfant GLM. Délai de traitement, nombre exact d'appels et de PID non mesurés (r5). |
| 3 | Reprise après coupure du daemon | Tests T3 (D7, D9) | R9.1 et R9.2.a à e (r2) : F1 corrigé. Aucune relance, un seul tour. R9.4.a à i (r3) : F2 corrigé, une seule remise. Arrêt SIGTERM du daemon avec enfant Codex (r6) : processus morts en 1,32 s au plus. | O4.6 (relance rapide, SIGKILL après 0,5 s) et E2 : à prouver en r4. Relance avec parent Codex vivant : contournée en r6, pas un PASS propre. |
| 4 | Parent extérieur : un autre fil ne voit ni ne pilote la tâche | Tests natifs (S149-13) | R3.1, S6.1, S6.2 (r2) : `task_unavailable`, aucune fuite | Aucun |
| 5 | Identité inconnue : preuve invalide | Tests natifs et T3 (S149-32) | R4.1, S4.5 (r2) : refus nommé, aucune identité voisine | Identité réelle d'un parent GLM hors fixture (T036) |
| 6 | Refus fournisseur : l'outil est refusé | Test sur faux CLI `/bin/sh` | R1 (r5) : écriture interdite refusée, tâche `failed`. R2b : confinement refusé. | Refus de Codex réel par `can_use_tool`. Passage à `failed` côté daemon (G4). |

**Couche T3 :** 13 contrôles PASS et 1 SKIP. Le groupe natif T3 n'est pas joué (`degradation149.md`).

---

## 9. Vérifier et rejouer

Ces commandes servent à relire les tests. Les lancer seulement dans un environnement isolé, avec `umask 077` et un `TMPDIR` court. Sans cela, des tests de socket échouent à tort.

```bash
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage

# Grammaire CLI et bout en bout (daemon fixture)
cargo test -p bridget-daemon --test lineage_cli_test

# Permissions et délégation, module 149 du daemon
cargo test -p bridget-daemon --lib native_delegation_permissions149

# Transport : permissions, refus corrélés, contrat de lignée
cargo test -p bridget-transport --test native_permissions149_test
cargo test -p bridget-transport --test native_permissions149_e2e
cargo test -p bridget-transport --test lineage_contract_test
```

Tests T3, depuis `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` :

```bash
./node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts
```

**Recettes réelles :** l'ordre scripté se trouve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recipes/README.md`. Ce guide affiche encore « préparé, non exécuté ». Les exécutions r5 ont suivi cet ordre avec des amendements (`--type-prompt`, `--extra-argv`, aide au grant retirée, touches différées). Ces amendements sont listés dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md`, section « Outils modifiés ».

Règles d'arrêt : arrêter un processus par son PID seul, avec SIGTERM. Ne jamais utiliser `kill -9`, `pkill` ni un groupe de processus.

Pour l'état détaillé des preuves, voir `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/implementation.md`.

---

**Ce document ne prouve pas que la session est livrée, ni que les agents sont autonomes.** Il décrit les commandes lues dans les sources et les preuves r2 à r9 au 2026-10-10. La validation globale et le cochage restent au principal, après les essais finaux r4 et r7.
