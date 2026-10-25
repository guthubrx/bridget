# Guide rapide - sous-agents et lignée (session 149)

**Statut au 2026-10-10 : livré sur disque, non activé. Reçu final : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/final.md.** Binaire livré : build 072161262a7d, sha256 6df86239b638dd83e29eb99640606e7e1f3ecb7b62e75cf33e6e781fdbc7a6f8. Les 45 tâches sont validées et cochées par le principal. Les recettes finales réseau r4 et Codex vivant r7 sont validées. Les blocs anciens ci-dessous restent historiques ; le reçu final prévaut. Aucun redémarrage ni activation différée.

Date : 2026-10-10. Document de travail. Mise à jour : après les preuves r9 (tests), r6 (smoke réel avec relance), r3 (réseau et reprise, interface) et la revue des sources r2.

---

## 1. Ce que permet la session 149

Un agent parent confie une mission à un sous-agent. Le sous-agent travaille dans un dossier du projet. Le parent suit l'avancement, lit le journal et peut annuler la mission.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-1).

**Pourquoi :** sans cette couche, un sous-agent disparaît de la vue dès qu'il travaille. On ne sait plus qui a lancé quoi, ni ce qu'il a fait.

---

## 2. État du runtime au 2026-10-10

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-3).

---

## 3. Points d'entrée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-4).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-6).

### Étape 4 - Annuler

- Annulation (MCP) : `bridget_task_cancel` avec `task_id`.
- Annulation (CLI) :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-7).

`--request-id` est un UUID canonique. Rejouer la même annulation renvoie le même reçu. L'annulation arrête la tâche et ses descendants actifs. Elle n'envoie aucun message à l'enfant. Un autre parent ne peut pas annuler la tâche.

**Preuve réelle :** le parent a annulé un enfant Codex après 20 secondes de travail. Son fichier a cessé de grossir et est resté inchangé six secondes après le contrôle. Aucun processus n'est resté (r5, R5). Le délai de traitement de l'annulation n'a pas été mesuré. Le cas GLM n'a pas été joué (limite 3 de `native-real-recipes-sonnet-r5.md`).

---

## 5. Dans l'interface adaptateur de bureau (lecture seule)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-9).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-10).

---

## 8. Six scénarios de panne

Chaque scénario a une preuve. Le tableau sépare la preuve sur fixture ou unitaire, la preuve réelle et ce qui reste ouvert. Détail : `implementation.md`, section 6.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-12).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/quickstart.md#extrait-13).

```bash
./node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts
```

**Recettes réelles :** l'ordre scripté se trouve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recipes/README.md`. Ce guide affiche encore « préparé, non exécuté ». Les exécutions r5 ont suivi cet ordre avec des amendements (`--type-prompt`, `--extra-argv`, aide au grant retirée, touches différées). Ces amendements sont listés dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md`, section « Outils modifiés ».

Règles d'arrêt : arrêter un processus par son PID seul, avec SIGTERM. Ne jamais utiliser `kill -9`, `pkill` ni un groupe de processus.

Pour l'état détaillé des preuves, voir `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/implementation.md`.

---

**Ce document ne prouve pas que la session est livrée, ni que les agents sont autonomes.** Il décrit les commandes lues dans les sources et les preuves r2 à r9 au 2026-10-10. La validation globale et le cochage restent au principal, après les essais finaux r4 et r7.
