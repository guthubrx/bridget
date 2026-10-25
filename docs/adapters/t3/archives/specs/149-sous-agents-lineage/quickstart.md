# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/quickstart.md](../../../../../../specs/149-sous-agents-lineage/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Une mission s'appelle une **tâche**. Le sous-agent s'appelle un **enfant**. Le **parent** est l'agent qui a créé la tâche.
- Un enfant n'est pas une conversation T3. Dans T3, il apparaît comme un fil virtuel en lecture seule. Il ne reçoit pas de tours de conversation et ne parle pas à l'humain.
- Le parent reste responsable du résultat.
- L'enfant hérite des droits du parent. La session 149 ne crée aucun droit nouveau. Aucun humain ne doit accorder un droit pour qu'une délégation marche.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

**Conséquence :** les preuves réelles viennent des recettes r5. Elles n'utilisent pas T3. Les preuves avec T3 utilisent un serveur T3 réel et un faux pair Codex, ou un daemon simulé. Chaque preuve indique son niveau dans `implementation.md`.

<a id="extrait-4"></a>

## Extrait 4

- **MCP Bridget** (appelé par l'agent) : `bridget_capabilities`, `bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`. Le statut et l'annulation demandent seulement un `task_id`. La délégation MCP ne demande pas de fil T3.
- **CLI Bridget** (terminal) : `bridget lineage inspect`, `bridget lineage watch`, `bridget lineage cancel`. Chaque commande demande `--json`, `--t3-thread` (fil T3 du parent) et `--project-root` (chemin absolu). Le fil T3 sert de lien de lecture à la CLI. Il ne conditionne pas la délégation.

<a id="extrait-5"></a>

## Extrait 5

```text
bridget lineage inspect --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --action journal --task UUID_TACHE --json
```

<a id="extrait-6"></a>

## Extrait 6

- Détail par pages : `--action show --task UUID_TACHE --offset N --limit N --json`. `--limit` vaut 16384 au plus.
- Liste : `--action list --limit N --json`. `--limit` vaut 50 par défaut et 100 au plus. `--cursor` reprend la liste.
- Suivi continu : `bridget lineage watch --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --json`.
- Options du journal : `--after-seq N` reprend après l'événement N. `--follow` suit les nouveaux événements. Le code existe. Aucun test de bout en bout ne le prouve (`implementation.md`, O5).

<a id="extrait-7"></a>

## Extrait 7

```text
bridget lineage cancel --t3-thread ID_FIL --project-root /chemin/absolu/du/projet --task UUID_TACHE --request-id UUID_DEMANDE --json
```

<a id="extrait-8"></a>

## Extrait 8

- Les sous-agents n'apparaissent pas dans la barre latérale de premier niveau.
- Le panneau **Lineage** affiche l'arbre des tâches du fil parent. Une tâche se lit dans son journal et dans son résultat.
- Il n'y a pas de zone de saisie pour un enfant. T3 n'envoie aucun message à un enfant.
- Le bouton **Arrêter** demande l'annulation de la tâche, et rien d'autre.

<a id="extrait-9"></a>

## Extrait 9

**Périmètre de la preuve (T040) :** r5 : interface web du worktree, serveur T3 réel, navigateur de prévisualisation, daemon simulé, données synthétiques. Largeurs 375, 480 et 1280 px : la dernière ligne, la barre et « Arrêter » restent visibles. r1 (`validation/ui-native-recipe149-sonnet-r1.md`) : même interface avec le daemon réel r8 et des enfants Codex fermés (faux serveur app-server) ; APPROVE sur ce périmètre. Pas de modèle réel dans l'interface, pas de test sur l'application de bureau ni sur mobile. Un enfant réel affiché dans l'interface T3 n'est pas prouvé (SC001, `implementation.md`, section 7).

<a id="extrait-10"></a>

## Extrait 10

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

<a id="extrait-11"></a>

## Extrait 11

| # | Scénario | Preuve sur fixture ou unitaire | Preuve réelle | Reste ouvert |
|---|---|---|---|---|
| 1 | Rejeu : même `request_id`, même contenu | Tests natifs et T3 (D10) | R1.1 (r2) : 20 rejeux, 1 lancement, 1 tour, 1 remise | Rejeu avec modèle réel |
| 2 | Annulation : le parent arrête la tâche et ses enfants | Tests natifs et T3 (D13) | R5 (r5) : enfant Codex annulé après 20 s de travail, 0 processus restant. R2.2 (r2) : deux PID réellement terminés | Annulation d'un enfant GLM. Délai de traitement, nombre exact d'appels et de PID non mesurés (r5). |
| 3 | Reprise après coupure du daemon | Tests T3 (D7, D9) | R9.1 et R9.2.a à e (r2) : F1 corrigé. Aucune relance, un seul tour. R9.4.a à i (r3) : F2 corrigé, une seule remise. Arrêt SIGTERM du daemon avec enfant Codex (r6) : processus morts en 1,32 s au plus. | O4.6 (relance rapide, SIGKILL après 0,5 s) et E2 : à prouver en r4. Relance avec parent Codex vivant : contournée en r6, pas un PASS propre. |
| 4 | Parent extérieur : un autre fil ne voit ni ne pilote la tâche | Tests natifs (S149-13) | R3.1, S6.1, S6.2 (r2) : `task_unavailable`, aucune fuite | Aucun |
| 5 | Identité inconnue : preuve invalide | Tests natifs et T3 (S149-32) | R4.1, S4.5 (r2) : refus nommé, aucune identité voisine | Identité réelle d'un parent GLM hors fixture (T036) |
| 6 | Refus fournisseur : l'outil est refusé | Test sur faux CLI `/bin/sh` | R1 (r5) : écriture interdite refusée, tâche `failed`. R2b : confinement refusé. | Refus de Codex réel par `can_use_tool`. Passage à `failed` côté daemon (G4). |

<a id="extrait-12"></a>

## Extrait 12

**Couche T3 :** 13 contrôles PASS et 1 SKIP. Le groupe natif T3 n'est pas joué (`degradation149.md`).

<a id="extrait-13"></a>

## Extrait 13

Tests T3, depuis `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` :
