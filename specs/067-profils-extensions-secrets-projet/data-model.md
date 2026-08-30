# Modèle de données: Profils, extensions et secrets bornés par projet

## ProjectProfile - autorité Maicie

| Champ | Règle |
|---|---|
| `profile_id` | stable dans le projet |
| `project_id` | référence 065 active |
| `binding_generation` | génération 065 approuvée |
| `runtime_policy_version` | référence 066 |
| `policy_digest` | politique 066 exacte approuvée |
| `agent_profile_ids` | références aux profils existants |
| `extension_refs` | liste bornée |
| `secret_refs` | liste bornée |
| `required_capabilities` | ensemble fermé |
| `generation` | incrément explicite |
| `status` | proposed, approved, active, disabled, stale |
| `profile_digest` | métadonnées canoniques, aucune valeur |

## ResolvedProjectProfile - autorité Bridget

Contient le profil exact après résolution AgentRegistry, `binding_generation`,
`runtime_policy_version`, `policy_digest`, les source revisions et stamps, les
digests de définitions agents, l'attestation des extensions et SecretRefs et le
digest image. Il ne contient aucune valeur secrète.

## ExtensionRef

| Champ | Règle |
|---|---|
| `extension_id` | unique dans le profil |
| `kind` | skill, plugin, tool_config |
| `source_ref` | référence hôte opaque côté Maicie |
| `source_revision` | absente de la proposition, résolue par Bridget puis épinglée à l'approbation |
| `destination` | relative à une racine conteneur fermée |
| `version` | opaque et obligatoire |
| `content_digest` | SHA-256 du contenu canonique |
| `read_only` | toujours true en v1 |

## SecretRef

| Champ | Règle |
|---|---|
| `secret_id` | unique dans le projet |
| `kind` | file, directory, process_env |
| `source_ref` | identifiant de source, pas valeur |
| `source_revision` | absente de la proposition, résolue par Bridget puis épinglée à l'approbation |
| `destination_or_env` | cible validée |
| `usage` | agent profiles autorisés, informatif au niveau projet |
| `generation` | incrément de rotation |
| `status` | active, rotation_pending, revoked, unavailable |

## SecretBindingAttestation

Preuve sans valeur: project_id, secret_id, generation, type observé, propriétaire
valide, permissions valides, source sous racine autorisée, destination, instant
et issue. Le chemin source complet reste dans l'audit privé hôte, pas dans les
projections générales.

## ProjectResourceCatalog - autorité de configuration Bridget

Document hôte fermé chargé depuis un chemin absolu fourni au daemon. Chaque
entrée associe `source_ref`, kind, chemin canonique, `source_revision`, UID/GID
attendus et une liste non vide de `project_id` autorisés. Maicie ne voit jamais
le chemin. L'absence ou l'invalidité du catalogue ferme les profils contenant
des ressources, sans affecter les profils historiques sans ressource.

## SecretSourceStamp

Empreinte de fraîcheur sans contenu. Pour un fichier: kind, device, inode,
taille, mtime/ctime en nanosecondes, mode et UID/GID. Pour un répertoire: même
tuple pour la racine plus un manifeste récursif trié des chemins relatifs et de
ces métadonnées pour chaque entrée régulière; les symlinks et fichiers spéciaux
sont refusés. Seul le digest canonique du stamp et la source_revision sont
persistés. Aucun octet ni digest de contenu secret n'est calculé.

## ProjectProfileApproval

Réutilise le modèle d'approbation existant: command_id, principal humain local,
profile_id, project_id, binding_generation, runtime_policy_version,
policy_digest, génération, profile_digest, context_digest, décision et instant.
Une approbation ne peut pas être réutilisée avec un digest, une version runtime
ou une génération de liaison divergents.

## OutputRedactionLease

État éphémère possédé uniquement par le wrapper pour la durée du processus
fournisseur: identifiant de processus, variables concernées, motifs binaires
protégés, état de comparaison par canal et instant de fin. Chaque canal conserve
entre deux lectures le suffixe qui peut encore préfixer une valeur protégée;
aucun octet candidat n'est remis avant décision. La fermeture applique la même
règle avant de vider le tampon. Les octets filtrés seulement traversent vers
JournalWriter, log, métrique, événement ou crash report. La taille des motifs et
du tampon est bornée par la taille maximale admise des process-env. La lease est
détruite après fermeture complète des flux et n'est jamais persistée.

## Transitions

### Profil

```text
proposed -> approved -> active -> stale -> approved après nouvelle approbation
                   |        |
                   +------> disabled
```

### SecretRef

```text
active -> rotation_pending -> active(generation+1)
   |
   +-> revoked
   +-> unavailable -> active après restauration et attestation
```

Invariants:

- profil active seulement si environnement attesté sur le même digest;
- profil active seulement si binding_generation et policy_digest correspondent
  encore à ProjectBinding et ProjectEnvironment;
- runtime_policy_version correspond encore à la politique 066 attestée;
- tout rebind rend le profil stale avant lecture ou montage de ressource;
- les exécutions actives continuent sur l'ancienne génération après rebind,
  mais aucune nouvelle admission n'est possible;
- rotation invalide l'ancien profil pour les nouveaux spawns;
- switch backend, runtime_policy_changed ou SecretSourceStamp divergent rend le
  profil stale avant résolution, lecture ou montage;
- aucun changement avec agents actifs;
- valeur absente de toutes les entités;
- aucune sortie brute process-env n'atteint un sink durable avant redaction;
- la redaction conserve son état aux frontières de fragments et de lignes pour
  chaque canal;
- destination unique dans le profil.

## Index et bornes

- Index par project_id/profile_id/generation.
- Index par secret_id/generation et extension_id/version.
- Listes bornées et validées avec sets O(n), pas de recherche imbriquée.
- Le digest canonique est O(taille du profil), elle-même bornée.
