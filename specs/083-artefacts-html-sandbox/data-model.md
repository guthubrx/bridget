# Modèle de données - SPEC-083

Les identifiants, versions, manifeste et blobs proviennent de SPEC-082. Ce
document ne crée pas un second registre. Il définit les métadonnées spécifiques
au runtime isolé et au Browser Desktop.

## SandboxArtifactVersionV1

| Champ | Type | Règle |
|---|---|---|
| `artifact_id` | UUID | Référence obligatoire à SPEC-082 |
| `version_id` | UUID | Version exacte immuable |
| `html_blob_sha256` | SHA-256 | Octets HTML canoniques, jamais URL arbitraire |
| `data_blob_sha256` | SHA-256 optionnel | Données explicitement déclarées pour cette version |
| `runtime_policy` | enum | Valeur v1 fixe `sandbox-v1`, pas de niveau fourni par agent |
| `inline_height_hint` | entier | 0 à 1200, revalidé par l'hôte |
| `source_refs` | liste bornée | Références de provenance de SPEC-082 |
| `parent_version_id` | UUID optionnel | Obligatoire pour une version issue d'un état sauvegardé |

## SandboxRuntimeStateV1

État temporaire, détenu par le cadre isolé tant que la vue est ouverte.

| Champ | Type | Règle |
|---|---|---|
| `frame_instance_id` | UUID éphémère | Change à chaque ouverture, non persistant |
| `artifact_id` / `version_id` | UUID | Doivent correspondre au ticket local |
| `ui_state` | JSON borné | Sérialisable, 128 KiB maximum, sans URL ou capacité ajoutée |
| `requested_height` | entier | 0 à 1200 inline, plafonné par l'hôte |
| `status` | enum | `ready`, `degraded`, `blocked`, `closed` |

Un clic ou un filtre ne modifie pas `SandboxArtifactVersionV1`. La commande
explicite Enregistrer comme nouvelle version soumet `ui_state` à Bridget, qui
crée une version enfant ou refuse avec une cause structurée.

## BrowserPanelStateV1

Métadonnées de l'interface locale dans `DesktopPreferences`, jamais envoyées au
daemon ni à un agent.

| Champ | Type | Règle |
|---|---|---|
| `right_panel_visible` | booléen | Valeur par défaut définie par l'ergonomie Desktop |
| `right_panel_maximized` | booléen | N'efface ni conversation ni navigation |
| `active_tab` | enum | `browser`, `artifacts`, `files`, `links`, `activity` |
| `browser_session_mode` | enum | `persistent` ou `ephemeral`, choix opérateur local |
| `browser_profile_generation` | entier | Augmenté après effacement de données |
| `last_project_scope` | UUID optionnel | Portée locale de présentation, pas autorisation agent |

## BrowserNavigationRequestV1

| Champ | Type | Règle |
|---|---|---|
| `request_id` | UUID | Corrélation journalisée |
| `initiator` | enum | `operator`, `conversation-link`, `artifact-source` |
| `target_kind` | enum | `https_url`, `published_local`, `artifact_version` |
| `target` | valeur validée | HTTPS ou référence canonique, jamais `file:` libre |
| `gesture_at` | instant | Requis pour les initiateurs non Browser |
| `decision` | enum | `opened`, `blocked`, `cancelled`, `failed` |
| `reason` | texte borné | Cause précise si refus ou échec |

## Invariants

1. Aucun champ de Browser ne contient cookie, identifiant, mot de passe ou
   octets de session.
2. Aucun message sandbox ne porte un chemin local, un jeton Tauri ou une
   permission dynamiquement accordée.
3. Toute référence de source est une référence de provenance SPEC-082 ou une
   URL HTTPS revalidée par Bridget.
4. La suppression du profil Browser n'a aucun effet sur les blobs et versions
   canoniques d'artefact.
5. La destruction d'un cadre sandbox perd son état temporaire non sauvegardé,
   mais ne modifie jamais l'historique.
