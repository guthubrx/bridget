# Recherche et décisions: Profils, extensions et secrets bornés par projet

## Décision 1 - Étendre les profils existants

**Décision**: ProjectProfile compose les AgentProfiles Maicie existants et les
définitions AgentRegistry Bridget.

**Rationale**: modèle, effort, outils, personality, SpawnOrder, approbation et
digest existent déjà. Un nouveau registre doublerait l'autorité.

**Alternative considérée**: fichier manifeste autonome par conteneur.

**Rejet**: risque de divergence avec l'approbation et les capabilities réelles.

## Décision 2 - SecretRef et valeur séparés

**Décision**: seuls référence, type, destination, usage et génération sont
durables. La valeur reste dans une source hôte privée.

**Alternative considérée**: chiffrer les valeurs dans SQLite.

**Rejet**: il faudrait gérer une clé maître, la rotation et l'accès, créant un
service de secrets maison sans besoin.

**Alternative considérée**: variables d'environnement du daemon.

**Rejet**: elles seraient globales, héritées et difficiles à attribuer au projet.

## Décision 3 - Recreate plutôt que hot reload

**Décision**: toute modification de montage ou secret exige zéro agent actif et
une recréation.

**Rationale**: Docker fixe les montages à la création. Un hot reload créerait
plusieurs générations concurrentes dans le même conteneur.

## Décision 4 - Projet comme frontière de secret v1

**Décision**: tous les agents du projet sont considérés capables de lire les
secrets projet.

**Rationale**: ils partagent le même conteneur, user et filesystem. Dire le
contraire serait techniquement faux.

**Alternative considérée**: permissions Unix par agent dans le même conteneur.

**Rejet**: les outils, worktrees et processus actuels ne portent pas une
identité Unix distincte; l'isolation serait fragile et contournable.

## Décision 5 - Extensions read-only et épinglées

**Décision**: aucun téléchargement ou mise à jour automatique. Le code approuvé
est monté read-only par digest.

**Rationale**: skills et plugins peuvent influencer ou exécuter du code. Leur
provenance doit être aussi stable que celle de l'image.

## Décision 6 - Mémoire globale différée

**Décision**: seuls des catalogues globaux read-only peuvent être référencés.
Toute mémoire writable reste par projet.

**Rationale**: une mémoire globale introduit confidentialité, autorité,
fraîcheur, suppression et contamination croisée. Aucun besoin ni métrique ne
justifie encore ce système.

## Sources primaires

- Claude Code documente les plugins comme des bundles de commandes, agents,
  hooks et serveurs MCP, ce qui confirme qu'ils doivent être traités comme du
  code versionné:
  https://code.claude.com/docs/en/plugins
- Claude Code documente aussi le sandboxing et ses limites:
  https://code.claude.com/docs/en/sandboxing
- OpenAI recommande de borner explicitement filesystem et réseau pour Codex:
  https://openai.com/index/running-codex-safely/
- Cursor décrit le stockage chiffré des secrets de background agents et
  l'environnement isolé, confirmant que les secrets sont une frontière
  distincte du dépôt:
  https://docs.cursor.com/background-agent
- Docker recommande d'utiliser des secrets plutôt que des variables ou
  arguments de build pour les données sensibles:
  https://docs.docker.com/build/building/secrets/

## Red flags retenus

- Un secret dans `docker inspect`, la CLI, un label ou un store est une fuite.
- Un plugin read-write peut s'auto-modifier et invalider l'approbation.
- Un hash de valeur faible peut permettre une attaque par dictionnaire; la
  génération est explicite sans digest de valeur durable.
- Une redaction parfaite de toute sortie LLM n'est pas prouvable; les valeurs
  réelles ne doivent pas être incluses dans les preuves ou revues externes.
- Une mémoire globale writable crée une frontière de données nouvelle et est
  exclue.

## Complément de décision RC8 - 2026-08-30

- `source_ref` est résolue uniquement par un catalogue hôte Bridget fermé. Le
  catalogue porte le chemin, la révision, l'UID/GID et les projets autorisés;
  Maicie ne reçoit jamais le chemin réel.
- Une rotation planifiée change génération et source_revision. Une mutation
  hors rotation est détectée par SecretSourceStamp, composé uniquement de
  métadonnées filesystem et d'un manifeste trié pour les répertoires.
- Aucun hash de contenu secret n'est produit. La protection contre un root
  hostile capable de falsifier les métadonnées reste hors du modèle v1.
- runtime_policy_version rejoint policy_digest dans toutes les décisions, et
  tout changement backend/politique/image/UID/GID rend le profil stale.

## Complément après intégration de SPEC-068 - 2026-08-30

- SPEC-068 ajoute une nouvelle famille de sinks durables: trame d'incident,
  store, notification, rejeu et acquittement. La frontière de redaction doit
  précéder leur construction, pas seulement l'écriture du journal.
- Le contrat autorise seulement un code fermé, une référence pseudonymisée, le
  `ProjectReference` et les métadonnées déjà admises. Les octets fournisseur,
  arguments, variables d'environnement et chemins de secrets restent interdits.
