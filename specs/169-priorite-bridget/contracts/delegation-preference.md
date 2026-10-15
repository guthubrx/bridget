# Contrat169 — Préférence de délégation et frontière d'autorité

Date : 2026-10-15. Statut : retenu pour la conception après contre-revue corrigée ; exécution à prouver. Ce contrat complète les contrats existants ; il n'ajoute aucun RPC ou protocole Bridget/public, ni outil Bridget.

## Configuration utilisateur

Le fichier de réglages T3 existant accepte un seul champ neuf : `preferBridgetDelegation`, de type booléen. L'absence vaut `false`. `true` demande la priorité Bridget. Le champ ne fait pas partie de `ServerSettingsPatch` : ce réglage se modifie par fichier. Une modification d'un autre réglage conserve sa valeur. Les autres valeurs et les désactivations MCP restent applicables.

Le mode dérivé n'est pas écrit dans le JSON. La confiance est conservée avec les réglages dans leur cache interne ; ce n'est pas une sentinelle utilisateur. Une erreur de décodage ou de lecture produit `blocked`, même après un ancien OFF. Les mises à jour depuis cet état refusent par un `ServerSettingsError` nommé avant secrets/écriture ; fichier byte-identique. La réparation valide relue, ou une écriture valide depuis un état fiable, rétablit la confiance. Ne pas injecter le fichier brut, ses secrets ou une erreur sensible dans les consignes.

## Entrée et sortie de composition

La composition reçoit le mode résolu avec la fiabilité de lecture et le montage Bridget réel. Le modèle de données fixe la table : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/data-model.md.

Le transport interne réutilise `ProviderAdapterV2TurnInput.bridgetDelegationPreference?: "off" | "prefer" | "blocked"`. Le manager décore chaque `runtime.startTurn` avec la lecture courante. Le montage réel complète cette préférence lors de la composition : sans montage, `prefer` devient `blocked`. Le champ ne doit pas être perdu par les adaptateurs de compatibilité legacy. Il ne dépend pas de `McpProviderSessionConfig` et ne requiert ni map ni mutation MCP.

| Sortie | Obligation observable |
|---|---|
| `off` | Conserver les consignes T3 habituelles, y compris leur transport de délégation existant. Ne pas ajouter une priorité Bridget. |
| `prefer` | Demander `bridget_auto_delegate` pour une nouvelle sous-tâche. Si le catalogue est paresseux, permettre une seule recherche bornée par la découverte existante avant de conclure à son absence. Conserver les contraintes explicites, laisser les autres axes à Bridget et garder les références de suivi/rejeu. |
| `blocked` | Annoncer le réglage non fiable ou Bridget non monté. Ne pas inventer d'outil et ne pas prescrire de délégation native T3 de secours. |

Les sorties actives demandent de signaler une absence d'outil, une panne ou un refus. Aucun dispatch de secours automatique n'est ajouté à l'hôte. Elles conservent les nouveaux fils T3 explicitement demandés et les outils T3 hors délégation. Corriger les six injonctions contradictoires, y compris revues, délégation depuis le scheduler et repli terminal ACP. Les traces distinguent l'outil T3 `delegate_task` des outils natifs de sous-agent d'un fournisseur.

Les noms de paramètres de l'appel automatique restent ceux du contrat168 existant. Cette session n'en invente aucun. Le modèle courant du parent ne devient pas une contrainte de l'enfant sans choix humain explicite. La présence de l'outil n'atteste pas un droit d'admission ou une authentification future.

## Frontière de sécurité et de continuité

1. La préférence est non autorisante. Aucune permission, identité, révocation ou limite n'en dérive.
2. L'actualisation avant un tour conserve l'objet de session MCP et son credential lorsqu'ils existent. Le mode est non autorisant et doit aussi parvenir aux consignes en leur absence. La preuve compare les références et l'identité avant et après actualisation. Aucun faux credential n'est créé pour transporter le mode.
3. Une vraie rotation suit son cycle existant ; elle n'est ni évitée ni déclenchée par la préférence.
4. L'admission, les contraintes, le rejeu et la non-duplication restent dans Bridget. Une réponse perdue après admission ne crée pas de seconde tâche par l'hôte.
5. Une tâche admise garde son moteur, son suivi et son résultat après tout changement de mode ou coupure T3. Lineage reste une lecture, sans effet de lancement.

Cette frontière évite le piège de /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts:153 : remplacer l'objet de session efface les faits de permissions. La conserver est une obligation de continuité ; ce n'est pas une permission supplémentaire.

## Contrat par raccord

| Raccord | Transport existant de consigne | ON |
|---|---|---|
| Codex | Instructions développeur et contexte additionnel du tour. | `prefer` si Bridget est monté, sinon `blocked`. |
| Claude | Append du preset SDK et clé de réutilisation du contexte. | Même règle, avec append réellement actualisé après reprise. |
| ACP | État comparé et injection actuelle du prompt. | `blocked` sans montage Bridget. Préserver les commandes natives et le prochain contexte utile. |
| OpenCode | Chemins d'injection existants v1 et v2. | `blocked` sans montage Bridget dans les deux variantes. |
| Cursor | Chemin existant du premier contexte, actualisé en cas de changement. | `blocked` sans conserver une ancienne priorité T3. |
| Pi | Source existante du texte d'extension et `before_agent_start`, actualisée par commande privée fermée. | `blocked` fidèle, y compris après changement du mode et ACK. |

Pour Claude, le SDK peut retenir l'append jusqu'à compaction. Utiliser `snapshot: false` avec la clé de contexte cohérente. La clé seule ne suffit pas. Prouver les deux sens OFF/ON après reprise sans compaction. En cas de travail de fond actif, conserver le refus existant de remplacer la query : attente explicite, aucun travail tué ni tour au mode périmé ; le prochain tour après fin prend le nouveau mode. Le compromis de cache/thinking est non chiffré. Les options capturées ne prouvent pas encore l'effet d'un vrai modèle ; les autres paramètres restent identiques.

## Mise à jour privée Pi

L'extension Pi existante expose une seule commande réservée : `/t3-bridget-policy`. L'entrée fermée contient le mode `off/prefer/blocked` et un identifiant court de tentative neuf, borné et validé. Aucun prompt, texte libre, secret ou droit n'est accepté. L'adaptateur vérifie la commande dans `get_commands` de ce processus avant envoi par la requête `prompt` existante. Sinon il refuse : une commande inconnue pourrait partir au modèle. L'extension commence bloquée/inconnue ; ce cycle est obligatoire avant le premier vrai prompt, même OFF.

Le handler synchrone valide tous les arguments avant mutation. Il affecte le mode, puis émet un reçu privé par `ctx.ui.setStatus`, sur une clé réservée, avec mode et identifiant de cette tentative seulement. L'adaptateur intercepte et masque ce reçu. Il n'avance `lastAcknowledgedDelegationPolicy` qu'après reçu fermé du bon mode, corrélé à la tentative courante, ET ACK de requête corrélé `success: true`.

Le SDK peut avaler une erreur de handler et répondre success : l'ACK seul ne prouve donc jamais l'application. Les reçus anciens, étrangers ou invalides ne valident rien. Sur erreur, reçu manquant ou ACK invalide, aucun vrai prompt ni avancement. Les attentes se nettoient après issue ou détachement, sans état durable. La commande reste filtrée du catalogue utilisateur. Son nom réservé est aussi refusé depuis le chemin de prompt/slash utilisateur, avant envoi à Pi, pour empêcher un changement derrière le cache `lastAcknowledgedDelegationPolicy`. Les autres slash commands natives restent inchangées.

Cette commande n'est pas un outil de modèle, un RPC Bridget ou une API publique. Aucun endpoint, service ou fichier supplémentaire n'est créé. Elle remplace l'alternative plus lourde d'un redémarrage Pi pour chaque changement. Le registre existant et le canal de requête Pi portent sa durée de vie.

## Bornes et limites de preuve

La résolution du mode est O(1). Aucun fetch réseau, contrôle de santé périodique, copie du catalogue ou nouveau stockage n'est créé. Les tailles de consignes, journaux et résultats restent dans les limites existantes.

Un test de texte prouve le texte. Un test d'adaptateur prouve sa transmission. Une recette avec un fournisseur existant prouve le parcours observé. Aucun de ces résultats ne promet qu'un modèle suivra toujours une consigne. Les gardes déterministes natives continuent à porter la sécurité.

Pi réel n'est pas attesté : son binaire est hors PATH dans l'environnement constaté. Seuls ses tests de contrat et fixtures sont prévus. Aucune installation de fournisseur non demandée n'est ajoutée pour combler cette limite.

## Livraison activée

Le paquet `0.0.45-local.169` doit être signé, installé et activé selon le plan, avec retour arrière prêt. L'utilisateur a déjà autorisé cette activation. Vérifier le code chargé, ON, les nouveaux contextes et la reprise des conversations. Interdiction de `launchctl submit`, d'activation persistante et de boucle de relance. Les bases et preuves ne sont pas réinitialisées pour valider le lot.
