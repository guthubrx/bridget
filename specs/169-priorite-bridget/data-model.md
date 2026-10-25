# Modèle de données SPEC169 — Un réglage, aucun stockage supplémentaire

Date : 2026-10-15.

## Réglage persistant existant étendu

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/data-model.md#extrait-1).

Le fichier absent dans une installation sans réglages conserve le défaut normal OFF. Une erreur de lecture autre que cette absence normale ne prouve pas OFF. Aucun schéma de base, migration, index ni fichier de politique n'est ajouté.

Le cache existant conserve ensemble les réglages et leur confiance de lecture. S'il est non fiable, `updateAndPersistSettings` refuse avant secrets/écriture et garde le fichier byte-identique. La confiance revient après réparation valide relue, ou écriture valide depuis un état fiable. Un ancien OFF ne prouve pas la valeur actuelle d'un fichier invalide.

## Faits internes du tour

| Fait | Origine et durée | Autorité |
|---|---|---|
| Préférence lue | Service de réglages existant, actualisée avant chaque tour. | Influence seulement le texte de délégation. |
| `settingsFileTrusted` | Résultat du chargeur existant ; état cohérent avec le réglage lu. | Distingue une lecture valide des defaults après erreur. Jamais sérialisé comme clé utilisateur. |
| Montage Bridget | Chemin de montage réel du driver, avec ses désactivations existantes. | Prouve seulement le montage, pas la santé ni l'authentification future du daemon. |
| Outil exposé au tour | Catalogue d'outils effectivement présenté à l'agent. | La présence de `bridget_auto_delegate` permet de tenter son appel ; les gardes natives tranchent l'admission. |
| Mode de consigne | Calcul dérivé `off`, `prefer` ou `blocked`. | N'autorise aucune opération. Pas de stockage durable séparé. |

Le contexte par tour existant `ProviderAdapterV2TurnInput` reçoit le champ optionnel interne `bridgetDelegationPreference?: "off" | "prefer" | "blocked"`. Le manager le renseigne à chaque `runtime.startTurn`, indépendamment de la présence de MCP. Les compositeurs y associent le montage réel ; une préférence `prefer` sans montage devient `blocked`. La compatibilité vers les drivers legacy doit conserver ce champ.

## Résolution et transitions

| Lecture fiable | Préférence | Montage Bridget | Mode |
|---|---|---|---|
| Non | Toute valeur ou défaut | Tout état | `blocked` — réglage indisponible. |
| Oui | Absente ou `false` | Tout état | `off`. |
| Oui | `true` | Présent et non désactivé | `prefer`. |
| Oui | `true` | Absent ou désactivé | `blocked` — Bridget non monté. |

Tous les changements prennent effet dans les nouvelles consignes du prochain tour. Ils ne réécrivent pas le tour en cours. La disparition de l'outil au moment de son usage produit un blocage explicite selon les consignes ; elle ne déclenche pas de dispatch de secours de l'hôte.

## Relation avec l'identité MCP

La configuration MCP, si elle existe, et les faits de permissions conservent leurs références et leur cycle existants. La simple préférence ne change ni endpoint, ni credential, ni identité du fil, ni fait de permissions. Son portage utilise le contexte par tour, pas cet objet de configuration. Aucune map, mutation MCP ou faux credential n'est nécessaire.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/data-model.md#extrait-2).

## État d'injection des fournisseurs

Les caches et comparaisons existants de consignes doivent inclure le mode effectif et la disponibilité de montage utile. Ils ne constituent pas une seconde politique. Une injection manquée, notamment pour une commande native, ne doit pas être enregistrée comme réussie.

Claude transmet son append par le SDK avec `snapshot: false`, sans remplacer une query au travail de fond actif ni accepter un mode périmé. ACP compare son état. Cursor tient compte du changement après le premier tour. OpenCode couvre ses deux adaptateurs.

Pi commence bloqué/inconnu. Avant le premier vrai prompt, même OFF, la commande privée découverte par `get_commands` reçoit mode et identifiant de tentative neuf validés. Après affectation, elle émet un reçu fermé par `ctx.ui.setStatus`. Ce reçu est masqué par l'adaptateur. `lastAcknowledgedDelegationPolicy` avance seulement après reçu du bon mode corrélé à la tentative ET ACK de requête corrélé `success: true`. Les reçus anciens/étrangers sont ignorés. L'ACK seul, y compris après erreur avalée, n'avance rien. Sur erreur ou reçu manquant, aucun vrai prompt n'est envoyé. La commande reste filtrée du catalogue utilisateur.

Les tâches existantes gardent leurs données et leur moteur. Aucun champ169 n'est ajouté aux tâches Bridget, aux résultats ou aux événements Lineage.
