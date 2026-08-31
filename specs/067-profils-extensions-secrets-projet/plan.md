# Plan d'implémentation: Profils, extensions et secrets bornés par projet

**Branche**: `session-067-profils-extensions-secrets-projet`
**Spec**: `specs/067-profils-extensions-secrets-projet/spec.md`
**Base vérifiée**: `dda4ec2198b941cc38915a00f847df435d80934d` (`main` après SPEC-066)
**Date**: 2026-08-30
**Statut du plan**: validé pour implémentation dans ce worktree isolé

## Résumé

La troisième tranche rend le backend Docker utilisable par des agents réels
sans monter le home de l'hôte. Maicie compose et fait approuver un profil
projet. Bridget résout les AgentDefinitions, extensions et SecretRefs, produit
une définition canonique et atteste les sources avant de recréer le conteneur.

Le plan réutilise `ProfileConfig`, `profiles.rs`, l'approbation locale,
AgentRegistry, `forbidden_env`, `pass_env` et ProjectRuntimePolicy. Les valeurs
secrètes restent dans des sources hôte privées. Aucun service de secrets n'est
ajouté.

## Contexte technique

| Élément | Choix |
|---|---|
| Autorité profil métier | Maicie |
| Autorité définition runtime | Bridget |
| Agent definitions | AgentRegistry existant |
| Approbation | locale, digest épinglé, flux existant |
| Extensions | bind mounts read-only, sources allowlistées |
| Secrets | références file/dir/process-env, valeurs hors stores |
| Catalogue de sources | document hôte Bridget v1, chemin absolu explicite |
| Fraîcheur secret | révision opérateur + SecretSourceStamp sans contenu |
| Activation | arrêt des agents puis recreate 066 |
| Fournisseurs | Codex app-server, Claude stream-json, Cursor ACP |
| Backend secrets | docker uniquement en v1 |
| Dépendances externes | aucune |

## Architecture cible

```text
HÔTE
  Racines extensions autorisées, read-only
  Racines secrets privées, hors dépôts
             |
Maicie ProjectProfile + approbation locale
             |
  profile refs, aucun secret value
             |
Bridget ResolvedProjectProfile
  AgentRegistry + capabilities + attestations
             |
ProjectRuntimePolicy 066, digest nouveau
             |
recreate explicite du conteneur projet
  /opt/bridget/extensions/*  read-only
  /run/bridget-secrets/*     read-only
  state root projet          read-write
             |
wrappers -> providers Codex/Claude/Cursor
```

## Réutilisation de l'existant

- Étendre `ProfileConfig` et `LoadedProfile`, ne pas créer un second registre
  d'agents.
- Étendre la vue d'approbation et le digest dans `plugins/maicie/src/profiles.rs`.
- Réutiliser propositions, approbations et `SpawnOrder` épinglé de
  `plugins/maicie/src/app.rs` et `store.rs`.
- Résoudre commandes et capabilities via `crates/bridget-daemon/src/registry.rs`.
- Réutiliser `forbidden_env`, `pass_env` et `build_environment`.
- Ajouter les montages au `ProjectRuntimePolicy` et à l'attestation 066.
- Utiliser le wrapper existant pour lire une valeur process-env depuis le
  fichier monté juste avant le provider.
- Réutiliser les audits de greffe comme style de preuve locale, sans transférer
  l'autorité greffe aux profils.
- Réutiliser les incidents runtime délégués SPEC-068 comme canal redacted de
  diagnostic; ne jamais y recopier stdout, stderr, params ou valeur secrète.

## Modèle de profil

ProjectProfile référence:

- un `project_id`;
- une `binding_generation`;
- une version de ProjectRuntimePolicy;
- le `policy_digest` exact;
- une liste d'identifiants AgentProfile existants;
- une liste d'ExtensionRef;
- une liste de SecretRef;
- les besoins réseau et capabilities;
- une génération et un digest canonique.

La proposition Maicie ne porte que `source_ref`. Bridget résout ensuite
`source_revision` depuis le catalogue hôte fermé et la place dans
ResolvedProjectProfile; l'approbation humaine épingle ce résultat exact. Le
catalogue associe la référence opaque à son chemin canonique, son type, son
UID/GID attendu et une liste explicite de projets. Il est chargé une fois par
Bridget depuis le chemin absolu fourni au daemon; une requête ne peut jamais
soumettre un chemin source ou une révision autoritative.

Le digest couvre les métadonnées, `runtime_policy_version` et versions, pas les
valeurs secrètes. La rotation d'un secret incrémente explicitement sa
génération et la `source_revision`. Une modification sans rotation est détectée
par une SecretSourceStamp composée des métadonnées stat du fichier ou du
manifeste récursif trié d'un répertoire. Aucun octet ni hash du contenu secret
n'entre dans le stamp. Une divergence produit `secret_generation_stale` avant
lecture ou montage. La résistance à un administrateur root capable de falsifier
toutes les métadonnées reste hors du modèle de menace local coopératif.

## SecretRef v1

Types admis:

- `file`: monté read-only sous `/run/bridget-secrets/files/`;
- `directory`: monté read-only sous `/run/bridget-secrets/dirs/`;
- `process_env`: valeur dans un fichier privé monté, lue par le wrapper puis
  injectée au processus fournisseur si la variable est allowlistée.

Une source doit être sous une racine autorisée, hors dépôt, sans symlink et
avec permissions privées. Aucun secret n'est passé dans `docker exec --env` ou
dans un argument de commande.

## Extensions v1

Types admis: `skill`, `plugin`, `tool-config`. Chaque référence possède source,
destination relative dans un ensemble fermé, version et digest. Les sources
sont montées read-only. Aucun téléchargement réseau ni mise à jour automatique.

Les catalogues globaux peuvent être des sources communes, mais chaque projet
les référence explicitement. Les écritures restent dans le state root projet.

## Découpage technique

### Lot 1 - Contrats et vue d'approbation

- Étendre ProjectProfile et les contrats publics.
- Produire la vue exhaustive sans valeurs.
- Comparer AgentProfiles à AgentRegistry et capabilities avant approbation.
- Épingler binding_generation et policy_digest dans résolution et approbation;
  épingler aussi runtime_policy_version; un rebind ou tout changement de
  backend/politique/image/UID/GID invalide le profil avant toute ressource.

Gate: toute divergence de définition ou champ caché refuse l'approbation.

### Lot 2 - Résolution et attestation d'extensions

- Valider sources, destinations, owner, permissions, version et digest.
- Résoudre chaque source uniquement par le catalogue hôte et vérifier sa liste
  fermée de projets autorisés.
- Étendre la politique et l'inspection des montages.
- Détecter les modifications entre approbation et spawn.

Gate: deux projets n'exposent que leurs extensions approuvées.

### Lot 3 - Résolution et attestation des secrets

- Valider les SecretRefs sans lire prématurément les valeurs.
- Calculer et épingler SecretSourceStamp sans contenu, puis la revérifier avant
  create et immédiatement avant spawn.
- Monter file/dir read-only.
- Faire lire process-env par le wrapper après admission capabilities.
- Créer une OutputRedactionLease avant spawn et filtrer stdout, stderr et
  canaux structurés avant tout sink durable, jusqu'à fermeture complète.

Gate: scanners négatifs sur stores, args, labels, logs et projections.

### Lot 4 - Rotation, fournisseurs et rollback

- Incrémenter une génération par rotation approuvée.
- Marquer stale sur rebind et exiger nouvelle approbation puis recreate.
- Marquer également stale sur `runtime_policy_changed`, changement de backend,
  policy version/digest, image ou UID/GID.
- Refuser les anciens profils, arrêter les agents puis recreate.
- Exécuter les contrats communs Codex, Claude et Cursor ACP.
- Désactiver le profil et recréer sans ressources.

Gate: ancienne génération absente et trois fournisseurs conformes.

## Frontière de confiance

Le conteneur est partagé. Un agent du projet qui peut lire les fichiers du
projet peut également tenter de lire les secrets montés au projet. Cette version
ne fournit aucune isolation par agent. La vue d'approbation doit l'énoncer.

Un secret individuel exige soit un conteneur distinct, soit un broker de
capabilities qui ne remet pas la valeur. Ces solutions ajoutent un modèle
d'autorisation et sont différées jusqu'à un besoin prouvé.

## Sécurité

- Frappe locale pour approve, rotate et revoke.
- Aucune route distante de mutation.
- Sources hors dépôts, paths canoniques, owners et modes vérifiés.
- `source_ref` résolue exclusivement par le catalogue Bridget; liste de projets
  explicite et aucun wildcard en v1.
- Destinations fermées, read-only, sans collision.
- Capabilities vérifiées avant lecture/montage.
- Valeurs absentes des arguments Docker et stores.
- Redaction de stderr/stdout avant persistance de contenu sensible connu.
- Recreate obligatoire, aucun hot update.

## Observabilité

- Audit: principal local, project_id, profile digest, refs et générations, sans
  valeurs ni chemins secrets complets dans les métriques.
- Métriques: activation/rotation/refus par issue, profils par état, divergences
  d'extension et secret, redactions totales.
- Statut: profil, refs, génération, recreate_required et prochaine action.
- Alerte: source manquante, permission élargie, digest extension divergent,
  ancienne génération encore active.

## Stratégie de test

1. Tests unitaires des contrats et digests.
2. Tests de permissions, symlinks, collisions et destinations.
3. Tests de vue d'approbation exhaustive et absence de valeurs.
4. Tests Docker inspect des montages.
5. Tests process-env sans valeur dans ps, inspect, args ou stores.
6. Tests de fuite par stdout/stderr provider fixture.
7. Tests de rotation/recreate et probes d'ancienne génération.
8. Contrat commun Codex, Claude et Cursor ACP.
9. Test host refusant un profil Docker.
10. Gherkin `tests/features/067-profils-extensions-secrets-projet.feature`.
11. Mutations hors rotation d'un fichier et d'un membre de répertoire,
    détection par SecretSourceStamp et refus avant lecture/montage.
12. Invalidation sur switch backend et changement de runtime_policy_version.
13. Scanners négatifs sur les trames et stores SPEC-068, avec seulement code,
    référence pseudonymisée et ProjectReference autorisés.

## Déploiement et réversibilité

- Livrer les lecteurs et projections avant toute activation.
- Tester avec fausses valeurs et extensions fixtures.
- Activer un seul profil projet pilote.
- Rotation et révocation exigent agents arrêtés.
- Rollback: désactiver profil, recréer l'environnement, puis revenir host si
  nécessaire. Les références historiques restent auditées, pas les valeurs.

## Constitution Check

| Règle | Verdict | Décision |
|---|---|---|
| Moindre privilège | PASS | Ressources par référence approuvée et allowlists. |
| Privacy gate | PASS | Valeurs non envoyées aux LLM de revue ni stockées. |
| Minimalisme | PASS | Aucun broker, Vault, mémoire ou manager dynamique. |
| Réutilisation | PASS | Profils, approvals, registry, env et runtime existants. |
| Responsabilité future | PASS | Frontière projet et limite intra-projet explicites. |
| Observabilité | PASS | Audit et redaction conçus avant activation. |
| Réversibilité | PASS | Recreate sans profil retire les expositions. |
| Portabilité | PASS | Contrat neutre validé sur trois transports. |

## Gate avant implémentation

- Créer un nouveau worktree depuis `dda4ec2`, tête propre contenant la livraison
  de SPEC-066; ne pas réutiliser le
  worktree documentaire courant.
- L'absence de `.specify` ou de l'outil `specify` ne bloque pas T001 et ne doit
  déclencher ni installation ni mise à jour. Les documents versionnés sous
  `specs/067-profils-extensions-secrets-projet/` sont la source de vérité.
- SPEC-066 passe ses preuves de non-destruction, ingress et rollback.
- Une racine secrets fixture et une racine extensions fixture sont préparées.
- Un catalogue fixture fermé associe références, révisions, UID/GID et projets
  autorisés; aucune source réelle n'est nécessaire.
- La politique de confiance intra-projet est approuvée par l'utilisateur.
- `reuse-audit.md` reste PASS contre la tête d'implémentation.
- Analyze ne laisse aucun finding CRITICAL.
- Aucune donnée réelle ni credential de production n'est utilisé dans les tests.
- L'implémentation suit `tasks.md` dans l'ordre, test-first, sans déploiement.
