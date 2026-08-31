# Modèle de données - SPEC-082

## Principes

- Toute identité est générée côté Bridget et n'est jamais choisie librement par
  l'agent.
- Toute version est immuable.
- Toute référence de contenu binaire est une empreinte SHA-256 calculée par
  Bridget après écriture atomique.
- La provenance est un composant obligatoire de chaque version, même lorsque
  son origine est une donnée fournie par l'opérateur ou un calcul agent.

## Artifact

| Champ | Règle |
|---|---|
| artifact_id | identifiant stable Bridget |
| project_id | projet attesté, requis |
| conversation_id | conversation de publication, requise |
| turn_id | tour de publication, requis |
| kind | chart, kpi, table, timeline, image, file ou html réservé à SPEC-083 |
| title | texte affichable, borné |
| created_at | horodatage daemon |
| current_version | version visible par défaut |
| pinned | uniquement modifiable par opérateur |
| deleted_at | suppression logique explicite |
| origin_instance | instance Bridget qui porte le contenu canonique |

## ArtifactVersion

| Champ | Règle |
|---|---|
| artifact_id et version | clé unique, version positive strictement croissante |
| parent_version | version parent explicite ou null pour création |
| payload_digest | empreinte du contenu structuré canonique |
| render_recipe | configuration déclarative propre au type |
| provenance_digest | empreinte du manifeste de provenance |
| state | published, partial, unavailable, failed, deleted |
| quality_notices | lacune, estimation, hypothèse, limite ou erreur attestée |
| created_by | opérateur ou agent identifié |
| created_at | horodatage daemon |
| supersedes_reason | refresh, restore_changed, save_interaction ou publication initiale |

Transition autorisée : un version published ou partial ne change jamais. Une
nouvelle action crée une ligne enfant. Un état failed ne devient pas published :
une nouvelle tentative crée une nouvelle version ou un reçu de même idempotence.

## ArtifactManifest

| Section | Contenu obligatoire |
|---|---|
| identity | type, titre, version de contrat et référence de version |
| provenance | origine, sources, date, unités, transformations, hypothèses |
| content | données structurées, blob ou référence à un blob canonique |
| rendering | recette déclarative et options locales admissibles |
| quality | complétude, avertissements et limites connues |
| integrity | empreintes des données, manifeste et blobs |
| retention | épinglage, poids, dernière utilisation cache et politique appliquée |

## ArtifactSource

| Champ | Règle |
|---|---|
| source_kind | remote, local_project, user_supplied, agent_computed, restored |
| locator | URL HTTPS validée ou identifiant non secret |
| fetched_at | requis pour une collecte externe |
| content_digest | requis si octets observés |
| citation | étiquette affichable et non trompeuse |
| units | obligatoire lorsque les valeurs ont une unité |
| transformations | suite ordonnée de traitements déclarés |
| access_status | available, expired, unavailable, blocked ou unknown |

## ArtifactBlob

| Champ | Règle |
|---|---|
| digest | SHA-256, clé primaire |
| media_type | liste autorisée et bornée |
| byte_length | contrôlé avant publication |
| storage_class | canonical ou cache |
| canonical_path | jamais exposé au renderer ni à un agent |
| reference_count | transactionnel, jamais négatif |
| written_at | horodatage Bridget |

## ArtifactReference

| Champ | Règle |
|---|---|
| reference_id | identifiant Bridget |
| artifact_id et version | référence exacte et immuable |
| scope | conversation, turn, explicit_agent_share ou operator_bookmark |
| target | cible attestée par Bridget |
| created_by | opérateur ou service Bridget |
| created_at | horodatage daemon |

Une référence inter-agent exige scope explicit_agent_share et une cible. La
visibilité de projet ne constitue jamais une référence partageable.

## ArtifactPolicy

| Champ | Valeur initiale |
|---|---|
| cache_max_bytes | 1 Gio |
| cache_max_age_days | 30 |
| published_warning_bytes | 8 Gio |
| published_block_bytes | 10 Gio |
| manifest_max_bytes | 512 Kio |
| structured_payload_max_bytes | 16 Mio |
| binary_blob_max_bytes | 128 Mio |
| max_sources | 100 |
| default_remote_fetch | désactivé jusqu'au réglage local explicite |
| eviction | cache non épinglé, ancienneté puis taille |

## Invariants à tester

1. Une version ne peut pas appartenir à un autre projet que son artefact.
2. Deux appels d'idempotence identiques produisent un même reçu.
3. Le hash réel du blob doit correspondre à son manifeste.
4. Une version partielle possède au moins un avis de qualité.
5. Un artefact externe possède une date de collecte et au moins une source.
6. Un artefact agent_computed décrit ses entrées et sa transformation.
7. Une suppression ne collecte un blob que lorsque reference_count est nul.
8. Un cache ne devient jamais la source d'autorité d'une restauration.
9. Les requêtes de liste et recherche ne franchissent pas project_id sans filtre
   global explicite d'opérateur.

