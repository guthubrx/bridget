# Contrat169 — Préférence de délégation et frontière d'autorité

Date : 2026-10-15. Statut : retenu pour la conception après contre-revue corrigée ; exécution à prouver. Ce contrat complète les contrats existants ; il n'ajoute aucun RPC ou protocole Bridget/public, ni outil Bridget.

## Configuration utilisateur

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-1).

Le mode dérivé n'est pas écrit dans le JSON. La confiance est conservée avec les réglages dans leur cache interne ; ce n'est pas une sentinelle utilisateur. Une erreur de décodage ou de lecture produit `blocked`, même après un ancien OFF. Les mises à jour depuis cet état refusent par un `ServerSettingsError` nommé avant secrets/écriture ; fichier byte-identique. La réparation valide relue, ou une écriture valide depuis un état fiable, rétablit la confiance. Ne pas injecter le fichier brut, ses secrets ou une erreur sensible dans les consignes.

## Entrée et sortie de composition

La composition reçoit le mode résolu avec la fiabilité de lecture et le montage Bridget réel. Le modèle de données fixe la table : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/data-model.md.

Le transport interne réutilise `ProviderAdapterV2TurnInput.bridgetDelegationPreference?: "off" | "prefer" | "blocked"`. Le manager décore chaque `runtime.startTurn` avec la lecture courante. Le montage réel complète cette préférence lors de la composition : sans montage, `prefer` devient `blocked`. Le champ ne doit pas être perdu par les adaptateurs de compatibilité legacy. Il ne dépend pas de `McpProviderSessionConfig` et ne requiert ni map ni mutation MCP.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-3).

Les noms de paramètres de l'appel automatique restent ceux du contrat168 existant. Cette session n'en invente aucun. Le modèle courant du parent ne devient pas une contrainte de l'enfant sans choix humain explicite. La présence de l'outil n'atteste pas un droit d'admission ou une authentification future.

## Frontière de sécurité et de continuité

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-5).

## Contrat par raccord

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-6).

Pour Claude, le SDK peut retenir l'append jusqu'à compaction. Utiliser `snapshot: false` avec la clé de contexte cohérente. La clé seule ne suffit pas. Prouver les deux sens OFF/ON après reprise sans compaction. En cas de travail de fond actif, conserver le refus existant de remplacer la query : attente explicite, aucun travail tué ni tour au mode périmé ; le prochain tour après fin prend le nouveau mode. Le compromis de cache/thinking est non chiffré. Les options capturées ne prouvent pas encore l'effet d'un vrai modèle ; les autres paramètres restent identiques.

## Mise à jour privée Pi

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/169-priorite-bridget/contracts/delegation-preference.md#extrait-7).

Le handler synchrone valide tous les arguments avant mutation. Il affecte le mode, puis émet un reçu privé par `ctx.ui.setStatus`, sur une clé réservée, avec mode et identifiant de cette tentative seulement. L'adaptateur intercepte et masque ce reçu. Il n'avance `lastAcknowledgedDelegationPolicy` qu'après reçu fermé du bon mode, corrélé à la tentative courante, ET ACK de requête corrélé `success: true`.

Le SDK peut avaler une erreur de handler et répondre success : l'ACK seul ne prouve donc jamais l'application. Les reçus anciens, étrangers ou invalides ne valident rien. Sur erreur, reçu manquant ou ACK invalide, aucun vrai prompt ni avancement. Les attentes se nettoient après issue ou détachement, sans état durable. La commande reste filtrée du catalogue utilisateur. Son nom réservé est aussi refusé depuis le chemin de prompt/slash utilisateur, avant envoi à Pi, pour empêcher un changement derrière le cache `lastAcknowledgedDelegationPolicy`. Les autres slash commands natives restent inchangées.

Cette commande n'est pas un outil de modèle, un RPC Bridget ou une API publique. Aucun endpoint, service ou fichier supplémentaire n'est créé. Elle remplace l'alternative plus lourde d'un redémarrage Pi pour chaque changement. Le registre existant et le canal de requête Pi portent sa durée de vie.

## Bornes et limites de preuve

La résolution du mode est O(1). Aucun fetch réseau, contrôle de santé périodique, copie du catalogue ou nouveau stockage n'est créé. Les tailles de consignes, journaux et résultats restent dans les limites existantes.

Un test de texte prouve le texte. Un test d'adaptateur prouve sa transmission. Une recette avec un fournisseur existant prouve le parcours observé. Aucun de ces résultats ne promet qu'un modèle suivra toujours une consigne. Les gardes déterministes natives continuent à porter la sécurité.

Pi réel n'est pas attesté : son binaire est hors PATH dans l'environnement constaté. Seuls ses tests de contrat et fixtures sont prévus. Aucune installation de fournisseur non demandée n'est ajoutée pour combler cette limite.

## Livraison activée

Le paquet `0.0.45-local.169` doit être signé, installé et activé selon le plan, avec retour arrière prêt. L'utilisateur a déjà autorisé cette activation. Vérifier le code chargé, ON, les nouveaux contextes et la reprise des conversations. Interdiction de `launchctl submit`, d'activation persistante et de boucle de relance. Les bases et preuves ne sont pas réinitialisées pour valider le lot.
