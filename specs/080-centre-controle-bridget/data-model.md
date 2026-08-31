# Modèle de données - SPEC-080 Centre de contrôle Bridget

## Préférences locales

### DesktopPreferencesV1

| Champ | Type | Règle |
|---|---|---|
| version | entier | exactement 1 |
| operator_display_name | chaîne optionnelle | maximum 96 caractères, affichage local seulement |
| color_scheme | enum | `system`, `light`, `dark` |
| timezone | enum ou IANA | `system` ou identifiant IANA valide |
| interface_scale | enum | `small`, `medium`, `large` |
| reduce_motion | booléen | valeur locale, défaut système |
| updated_at | entier Unix | métadonnée de résolution locale |

Relation: un document par installation Bridget Desktop. Il ne contient ni hôte, ni secret, ni jeton du relais.

## Catalogue de contrôle serveur

### ServerControlSnapshotV1

| Champ | Règle |
|---|---|
| version | 1 |
| server | identité bornée: version daemon, instance, état relay, horloge et dernière observation |
| generation | entier strictement positif de la configuration contrôlable |
| categories | catégories annoncées, y compris lecture seule |
| settings | descripteurs du catalogue fermé |

### ControlSetting

| Champ | Règle |
|---|---|
| key | enum fermé, jamais une clé fournie librement par le client |
| category | `projects`, `providers`, `execution`, `observability`, `maintenance`, `security` |
| scope | `server`, `project`, `read_only` |
| mutability | `mutable`, `read_only`, `project_controlled`, `forbidden` |
| value | valeur typée, non secrète |
| schema | type, limite, unité et aide de rendu |
| reason | obligatoire lorsque non modifiable |

État initial mutable: `project_roots.allowed_roots`. Il réutilise la génération de `ProjectRootPolicy`. Les autres descripteurs sont lecture seule jusqu'à ce qu'une autorité de persistance sûre soit ajoutée.

### ControlChangePreview

| Champ | Règle |
|---|---|
| command_id | UUID ou identifiant idempotent validé |
| expected_generation | génération lue et prévisualisée |
| changes | ancienne et nouvelle valeurs normalisées |
| warnings | liste bornée de conséquences non sensibles |
| refusal | absence ou code fermé |

Transition: `read -> preview -> local_confirmed -> applied` ou `refused`. Un conflit de génération retourne vers `read`; aucune transition ne modifie une valeur pendant `preview`.

### ControlChangeReceipt

| Champ | Règle |
|---|---|
| receipt_id | identifiant unique |
| command_id | unique par émetteur local |
| server_instance_id | instance attestée |
| previous_generation / resulting_generation | monotones |
| changes | valeurs autorisées uniquement |
| applied_at | horodatage serveur |
| outcome | `applied` ou `refused`, sans faux succès |

## Usage et tarifs

### UsageSampleDimension

Les échantillons existants gardent leurs compteurs. La projection ajoute seulement les dimensions attestées au moment de la collecte: serveur, agent, fournisseur, modèle optionnel, projet optionnel et source. Une absence est stockée et rendue comme inconnue.

### UsageRate

| Champ | Règle |
|---|---|
| rate_id / version | identifiant et version monotones |
| provider / model | correspondance exacte, pas de préfixe implicite |
| effective_from | instant Unix; aucune rétroactivité silencieuse |
| currency | ISO 4217 |
| input/output/cache rates | prix par million, décimal borné, optionnel si non facturable |
| source_label | provenance humaine ou documentée, sans secret |

Le montant est une fonction déterministe des compteurs et du tarif applicable au moment de l'observation. Sans tarif ou dimension modèle, `estimated_cost` est absent avec une raison.
