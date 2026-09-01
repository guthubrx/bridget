# Contrat - Publication d'artefact Bridget V1

## But

Définir l'unique publication structurée exposée aux fournisseurs capables et les
réponses Bridget qui peuvent être rendues dans le fil.

## Outil annoncé

Nom : bridget_publish_artifact

Le contrat est annoncé par le registre d'outils Bridget. Il n'est jamais
interprété depuis Markdown.

## Entrée conceptuelle

| Champ | Obligatoire | Règle |
|---|---|---|
| idempotency_key | oui | clé non vide réutilisable seulement pour le même contenu |
| kind | oui | chart, kpi, table, timeline, image, file ou html |
| title | oui | titre humain borné |
| payload | oui | données structurées ou contenu déclaré propre au type |
| sources | oui | provenance, éventuellement user_supplied ou agent_computed |
| quality_notices | non | lacunes, estimations ou hypothèses |
| parent_artifact_ref | non | requis pour refresh, restore_changed ou save_interaction |
| publication_reason | oui | initial, refresh, restore_changed ou save_interaction |

Les identités de projet, conversation et tour viennent du contexte attesté du
fournisseur. Une entrée ne peut pas les redéfinir.

### Publication HTML sandboxée

Pour `kind: html`, `payload.html` contient le document complet autonome,
`payload.data` les seules données injectées et `payload.inline_height_hint` une
hauteur demandée entre 0 et 1 200 px. Le document ne référence ni CDN, ni URL
distante, ni fichier local. Bridget l'enregistre comme blob canonique puis le
rend uniquement dans la sandbox HTML. Après un reçu de succès, l'agent répond
par une confirmation brève et ne recopie jamais le document dans Markdown.

## Réponse de succès

| Champ | Signification |
|---|---|
| artifact_ref | référence stable de l'artefact |
| version_ref | référence exacte de la version publiée |
| state | published ou partial |
| content_digest | empreinte Bridget |
| warnings | avis de qualité non bloquants |
| conversation_reference | lien au tour de publication |
| storage_state | canonique écrit et état de cache |

## Refus de validation

| Code | Signification | Réaction attendue de l'agent |
|---|---|---|
| invalid_schema | type, champ ou valeur non admis | corriger le payload |
| payload_too_large | limite dépassée | réduire, découper ou expliquer |
| missing_provenance | origine insuffisante | compléter sources ou déclarer origine calculée |
| policy_blocked | quota, droit ou état de projet | expliquer et attendre action opérateur |
| idempotency_conflict | clé réutilisée avec contenu différent | créer une nouvelle publication explicite |
| unavailable | Bridget ou stockage indisponible | reprendre plus tard sans simuler une publication |

## Routes relay de lecture

Les routes sont lecture seule, tokenisées et filtrées par projet. Les noms et
paths exacts seront figés durant implémentation, mais le contrat fonctionnel est :

| Opération | Résultat |
|---|---|
| liste projet | cartes de métadonnées sans blob lourd |
| lecture version | manifeste, état, contenu autorisé ou indisponibilité explicite |
| données et source | provenance, données, transformations et empreintes |
| export | fichier dérivé de la version demandée |
| restauration | commande Bridget explicite, reçue comme nouvel état ou version |
| partage | référence explicite, jamais extension implicite de visibilité |

Aucune route ne prend un chemin de fichier arbitraire, une URL non validée ou un
identifiant de projet non autorisé.
