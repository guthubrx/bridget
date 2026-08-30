# Contrat public v1: profil de projet

## ProjectProfileProposal

```json
{
  "contract_version": 1,
  "command_id": "opaque-command-id",
  "project_id": "opaque-project-id",
  "binding_generation": 4,
  "runtime_policy_version": 1,
  "profile_id": "dev-default",
  "generation": 1,
  "agent_profiles": ["codex-code", "claude-review", "cursor-acp"],
  "extensions": [
    {
      "extension_id": "speckit",
      "kind": "skill",
      "source_ref": "catalog:speckit",
      "destination": "skills/speckit",
      "version": "2026.08",
      "content_digest": "sha256:..."
    }
  ],
  "secrets": [
    {
      "secret_id": "codex-session",
      "kind": "directory",
      "source_ref": "secret:codex-session",
      "destination_or_env": "provider/codex",
      "generation": 3
    }
  ],
  "required_capabilities": ["codex_app_server", "claude_stream_json", "acp"],
  "policy_digest": "sha256:...",
  "profile_digest": "sha256:..."
}
```

La forme publique ne contient jamais la valeur ni le chemin source réel du
secret.

`source_ref` est résolue exclusivement par le catalogue hôte Bridget v1. La
proposition ne peut fournir ou remplacer aucun chemin ni `source_revision`.
Bridget ajoute la révision du catalogue à ResolvedProjectProfile; la vue et
l'approbation locale l'épinglent ensuite dans le digest résolu.

## ResolvedProjectProfileView

La vue locale d'approbation affiche:

- identité projet et profil;
- binding generation, runtime policy version, image et policy digest;
- chaque agent avec commande, args, protocole, modèle, effort, tools,
  forbidden_env, pass_env et capabilities;
- chaque extension avec source ref, source revision, destination, version et digest;
- chaque secret avec id, kind, source revision, destination, génération, stamp
  sans contenu et portée projet;
- avertissement explicite: tous les processus du conteneur projet peuvent lire
  les secrets projet;
- réseau et limites;
- digest final.

## Ensemble fermé de raisons

- `project_inactive`
- `runtime_backend_incompatible`
- `profile_digest_mismatch`
- `agent_definition_mismatch`
- `capability_missing`
- `extension_source_invalid`
- `extension_digest_mismatch`
- `extension_destination_collision`
- `secret_source_invalid`
- `secret_permissions_invalid`
- `secret_destination_collision`
- `secret_generation_stale`
- `resource_catalog_unavailable`
- `resource_project_not_allowed`
- `secret_source_stamp_mismatch`
- `runtime_policy_version_mismatch`
- `runtime_policy_changed`
- `forbidden_environment_variable`
- `active_agents_present`
- `recreate_required`
- `local_approval_required`

## Secret process-env

Le contrat ne transporte que le nom de variable. Bridget vérifie qu'il figure
dans l'allowlist de la définition résolue et pas dans `forbidden_env`. Le
wrapper lit la valeur depuis le fichier monté et la place dans l'environnement
du processus provider. Avant le spawn, il crée une OutputRedactionLease
éphémère qui reste active jusqu'à fermeture complète de stdout, stderr et des
canaux structurés. Chaque canal utilise une comparaison binaire streaming qui
conserve entre fragments tout suffixe pouvant préfixer une valeur protégée. Il
ne libère aucun octet candidat avant décision, y compris à une frontière de
retour ligne, et applique la même règle à la fermeture du flux. Le wrapper ne
transmet que les octets filtrés à JournalWriter, log, métrique, événement ou
crash report. Aucun buffer brut n'est persisté; la lease est détruite seulement
après fermeture des flux. La valeur n'est jamais passée à Docker CLI.

## Invalidation par rebind

La proposition, la résolution, l'approbation et la réservation de spawn portent
la même `binding_generation`, le même `runtime_policy_version` et le même
`policy_digest`. Un rebind, switch backend, `runtime_policy_changed`, changement
d'image/UID/GID ou divergence rend le profil `stale` avant résolution de
SecretRef ou montage. Une nouvelle approbation locale et une recréation sont
obligatoires. Les processus déjà actifs terminent sur leur ancienne génération;
aucune nouvelle admission n'est acceptée et aucun arrêt implicite n'est
déclenché.

## Fraîcheur d'une source secrète

À l'approbation, Bridget épingle `source_revision` et le digest d'une
SecretSourceStamp sans contenu. Il revérifie ces deux valeurs avant create et
immédiatement avant spawn. Une divergence produit `secret_generation_stale` ou
`secret_source_stamp_mismatch` avant lecture, montage ou création de la
OutputRedactionLease. Le stamp ne contient ni octet ni digest du secret.

## Incidents runtime délégués

Avant création d'un `DelegatedRuntimeEventFrame`, toute donnée fournisseur passe
par la même redaction binaire avec état que stdout, stderr et les autres sorties
durables. L'incident durable ne contient qu'un code fermé, une référence
pseudonymisée, le `ProjectReference` de l'exécution et les métadonnées déjà
autorisées par SPEC-068.

Les valeurs secrètes, fragments bruts, arguments, variables d'environnement et
chemins de sources secrètes sont interdits dans la trame, le store, la
notification, le rejeu et l'acquittement. Les scanners de non-fuite couvrent
explicitement toutes ces surfaces.

## Compatibilité

- Les AgentProfiles historiques restent valides hors ProjectProfile.
- Un profil sans extension ni secret peut rester utilisable sur host.
- Un profil avec secret/montage Docker est refusé sur host.
- Cursor reste un AgentProfile ACP, sans type de contrat séparé.
