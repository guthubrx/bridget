# Modèle de présentation — SPEC143

Aucune donnée persistante, API ou migration nouvelle.

| Élément | Données autorisées | Invariant |
|---|---|---|
| Entrée Bridget reconnue | Projection existante, texte brut, groupes141 | Seul style du conteneur change ; contenu/copie/actions restent exacts |
| Sortie MCP confirmée | Identité outil et payload réellement attestés | Une ligne factuelle au repos ; contenu complet à l'ouverture |
| Destinataire | Nom explicitement attesté ou ID présent | Aucun nom déduit ni enrichissement réseau |
| État technique | État actuel de l'appel outil | Aucune conversion en preuve de livraison |
| Sortie non confirmée | Données originales de work entry | Rendu natif et actions inchangés |

État d'ouverture : réutiliser la clé fil/message ou work entry et le gestionnaire de toggle existants. Ne pas stocker un nouvel état métier. Les interactions des groupes141 restent indépendantes et stables. Les sorties attestées viennent de l'appel MCP `bridget_send` Codex/Claude ; paramètres et résultat appartiennent à la même work entry. Aucun schéma runtime nouveau.

## Projection US4 — Sans persistance nouvelle

| Élément | Données autorisées | Invariant |
|---|---|---|
| Réponse textuelle interagent | Texte assistant terminé avec préfixe exact en début, UUID complet et ligne blanche | Streaming et formats ambigus restent natifs ; aucun verdict de livraison |
| Relais repliable | Corps original avant une éventuelle frontière utilisateur explicite | Ligne à gauche sans cadre ; corps complet au dépliage |
| Note utilisateur | `---` hors bloc de code, ligne blanche, puis `Résumé pour toi :` ou `Pour toi :`, simple ou en gras | Toujours visible ; variantes incomplètes, plusieurs frontières ou fences ambigus restent natifs |
| Association UUID/nom | En-têtes directs Bridget antérieurs dans la même conversation | Pas de corps libre, futur, autre fil, annuaire ou I/O ; UUID court sinon, complet accessible |
| Copie et citation | Message assistant original et demande de citation existante | Copie originale ; citation ciblée force le natif pour conserver les offsets |
| Métadonnées et fichiers modifiés | Données assistant existantes | Restent hors du repli |

Les associations de noms sont préparées par un parcours mémoïsé O(n) des `timelineEntries` existantes. La projection ne modifie ni ces entrées ni leurs textes. Le toggle réutilise un état d'ouverture local associé au message ; aucun état métier ou API nouveau.

Contenu mixte : le corps agent demeure monté sous CSS `display:none` au repli, sans `hidden` ni `aria-hidden`. Son texte participe toujours au flux canonique des citations, ce qui empêche une sélection dans la note de devenir ambiguë au passage natif. Cette compatibilité conserve le coût Markdown du rendu complet antérieur143 ; aucune baisse de coût n'est revendiquée.

Références Markdown, notes de bas de page et HTML hors code susceptibles de traverser la frontière : message complet natif. État lié au fil/message ; réédition du texte A → B → A : état replié, pas restauration de son ancienne ouverture. Le changement d'identité fil/message protège séparément l'état de rendu. La présentation ne stocke pas l'ouverture historique d'un autre fil.

La garde finale conserve le natif pour toute réponse mixte contenant `[` hors fence, et pour une note ou frontière non canonique. Elle préserve les liens/actions par conservation du rendu complet, pas par réécriture Markdown. Le toggle utilise l'état de focus existant via `onToggleWorkEntry(row.id,false)` ; Enter/Space ne rendent pas le focus au composeur. Pas de remount à chaque fragment de streaming.
