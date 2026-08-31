# Contre-revue adverse RC7 - itération 2

**Périmètre relu** : SPEC-065, SPEC-066, SPEC-067 et ADR-016 dans l'arbre
`session-065-programme-environnements-projet`.

## VERDICT: APPROVE_WITH_CHANGES

Les corrections ferment C1, C2, H1 et H2. Aucun finding **CRITICAL** ne
bloque le programme documentaire. M1 rend maintenant la frontière de
redaction causale, mais son oracle ne couvre pas encore le découpage réel d'un
flux : cette précision doit être ajoutée avant d'implémenter les secrets
`process_env`.

Le passage à l'implémentation reste, dans tous les cas, conditionné aux gates
des SPEC-063 et SPEC-064, aux gates propres aux trois specs, et à une
validation humaine explicite.

## État des findings précédents

| Finding | État | Vérification |
|---|---|---|
| C1 - convergence concurrente | **CLOSED** | `ProjectRegistrationCommand` sépare désormais `proposed_project_id` de `resolved_project_id`; une identité pending ou perdante n'est pas référençable. La contrainte atomique Bridget sur la racine canonique élit une liaison; l'issue `root_already_bound` rend l'identité gagnante et sa génération. Les deux alias/symlinks concurrents sont un oracle nommé. |
| C2 - ingress privé et attestation | **CLOSED** | L'endpoint est privé par `project_id` et `binding_generation`, monté dans le seul conteneur concerné. Le premier handshake compare projet, génération de liaison, conteneur, epoch et génération Bridget avant tout effet. Les tests exigent le refus A-vers-B et d'une identité/génération forgée. |
| H1 - exclusion spawn/lifecycle | **CLOSED** | Une même autorité transactionnelle porte réservation et passage à `stopping`/`switching`; l'epoch est capturé puis revérifié juste avant `docker exec`. Le test d'interleaving déterministe est une tâche indépendante, non une simple promesse de synchronisation. |
| H2 - invalidation au rebind | **CLOSED** | Profil résolu, approbation et réservation épinglent `binding_generation` et `policy_digest`. Une divergence rend le profil `stale` avant lecture ou montage; ré-approbation locale et recréation sont requises. L'oracle de rebind vérifie le refus avant ressource. |
| M1 - redaction causale | **PARTIALLY CLOSED** | `OutputRedactionLease` est désormais créée avant le spawn, vit jusqu'à fermeture complète, et place stdout, stderr, canaux structurés, logs, métriques, événements et crash reports derrière la comparaison avant `JournalWriter`. Les tests demandent une sentinelle tardive et l'absence d'octets bruts dans `JournalWriter`. La propriété reste incomplète pour les frontières de fragments. |

## Findings

### HIGH - M1 : la redaction n'est pas encore spécifiée contre une sentinelle coupée entre fragments

**Fichiers** :

- `specs/067-profils-extensions-secrets-projet/data-model.md`, section
  `OutputRedactionLease` (lignes 64-71)
- `specs/067-profils-extensions-secrets-projet/contracts/project-profile-v1.md`,
  section `Secret process-env` (lignes 76-86)
- `specs/067-profils-extensions-secrets-projet/tasks.md`, T023 et T029
  (lignes 48 et 54)

Le contrat promet que les octets bruts sont comparés avant tout sink durable,
mais ni le modèle ni les tâches n'imposent une comparaison qui conserve l'état
d'un flux entre deux lectures. Un fournisseur peut écrire une même valeur en
plusieurs fragments; un filtre par fragment laisserait alors passer chaque
partie au `JournalWriter` sans jamais voir la sentinelle entière. Le test
actuel d'une sentinelle émise après démarrage ne discrimine pas cette
implémentation fautive.

**Correction minimale** :

1. Spécifier une redaction binaire par canal, à état conservé entre fragments,
   qui ne remet aucun octet candidat à un sink avant d'avoir décidé qu'il ne
   termine pas une valeur protégée; la fermeture du flux doit vider ce tampon
   selon la même règle.
2. Étendre T023/T029 avec une sentinelle synthétique découpée au milieu de la
   valeur, à une frontière de retour ligne et à travers plusieurs canaux
   structurés. L'assertion doit inspecter le `JournalWriter` réel et ne jamais
   y trouver la sentinelle brute.
3. Ajouter le mutant qui réinitialise l'état du filtre à chaque fragment (ou
   transfère le fragment avant comparaison) : il doit mourir sur l'assertion
   métier du `JournalWriter`, pas au montage ni au spawn.

### LOW - H2 : clarifier le sort des agents déjà actifs au moment d'un rebind

**Fichiers** :

- `specs/065-registre-identite-projets/spec.md`, rebind et agents actifs
  (lignes 185-190)
- `specs/067-profils-extensions-secrets-projet/tasks.md`, T030 (ligne 55)

Le programme protège correctement les nouvelles lectures, montages et spawns,
mais ne dit pas explicitement si un rebind est refusé lorsqu'un ancien agent
porte encore un profil/exposition actifs, ou si cet agent finit naturellement
sur l'ancienne génération pendant que tout nouveau spawn est refusé.

**Correction minimale** : choisir et tester explicitement l'une des deux
politiques. La seconde est compatible avec la non-destruction de SPEC-065 :
conserver l'exécution existante jusqu'à terminalité, classer l'environnement
`recreate_required`, refuser toute nouvelle admission, puis exiger
ré-approbation et recréation. Cette clarification évite qu'une implémentation
interprète `stale` comme un arrêt implicite.

## Contrôles transversaux

- **Progressivité** : conservée. SPEC-065 reste utile seule sur `host`; SPEC-066
  reste opt-in, sans secret; SPEC-067 arrive seulement après le runtime attesté.
- **Frontières** : Maicie reste l'autorité métier et d'approbation; Bridget
  porte liaison, runtime, admission et attestation; les fournisseurs restent
  derrière un contrat de capabilities commun à Codex, Claude et Cursor ACP.
- **Conteneur, host/docker et worktrees** : un conteneur partage les agents d'un
  projet, sans migration implicite des projets host; les worktrees externes ne
  sont admis que par le même `common dir` attesté.
- **Absence de doublon et rollback** : le registre, la flotte et les profils
  existants sont étendus plutôt que dupliqués; rollback host, suppression du
  conteneur et retrait de profil conservent dépôt, worktrees et états durables.

Aucun autre finding bloquant n'a été trouvé dans les corrections relues.
