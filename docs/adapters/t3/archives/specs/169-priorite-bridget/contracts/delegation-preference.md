# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/contracts/delegation-preference.md](../../../../../../../specs/169-priorite-bridget/contracts/delegation-preference.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le fichier de réglages T3 existant accepte un seul champ neuf : `preferBridgetDelegation`, de type booléen. L'absence vaut `false`. `true` demande la priorité Bridget. Le champ ne fait pas partie de `ServerSettingsPatch` : ce réglage se modifie par fichier. Une modification d'un autre réglage conserve sa valeur. Les autres valeurs et les désactivations MCP restent applicables.

<a id="extrait-2"></a>

## Extrait 2

| Sortie | Obligation observable |
|---|---|
| `off` | Conserver les consignes T3 habituelles, y compris leur transport de délégation existant. Ne pas ajouter une priorité Bridget. |
| `prefer` | Demander `bridget_auto_delegate` pour une nouvelle sous-tâche. Si le catalogue est paresseux, permettre une seule recherche bornée par la découverte existante avant de conclure à son absence. Conserver les contraintes explicites, laisser les autres axes à Bridget et garder les références de suivi/rejeu. |
| `blocked` | Annoncer le réglage non fiable ou Bridget non monté. Ne pas inventer d'outil et ne pas prescrire de délégation native T3 de secours. |

<a id="extrait-3"></a>

## Extrait 3

Les sorties actives demandent de signaler une absence d'outil, une panne ou un refus. Aucun dispatch de secours automatique n'est ajouté à l'hôte. Elles conservent les nouveaux fils T3 explicitement demandés et les outils T3 hors délégation. Corriger les six injonctions contradictoires, y compris revues, délégation depuis le scheduler et repli terminal ACP. Les traces distinguent l'outil T3 `delegate_task` des outils natifs de sous-agent d'un fournisseur.

<a id="extrait-4"></a>

## Extrait 4

1. La préférence est non autorisante. Aucune permission, identité, révocation ou limite n'en dérive.
2. L'actualisation avant un tour conserve l'objet de session MCP et son credential lorsqu'ils existent. Le mode est non autorisant et doit aussi parvenir aux consignes en leur absence. La preuve compare les références et l'identité avant et après actualisation. Aucun faux credential n'est créé pour transporter le mode.
3. Une vraie rotation suit son cycle existant ; elle n'est ni évitée ni déclenchée par la préférence.
4. L'admission, les contraintes, le rejeu et la non-duplication restent dans Bridget. Une réponse perdue après admission ne crée pas de seconde tâche par l'hôte.
5. Une tâche admise garde son moteur, son suivi et son résultat après tout changement de mode ou coupure T3. Lineage reste une lecture, sans effet de lancement.

<a id="extrait-5"></a>

## Extrait 5

Cette frontière évite le piège de /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts:153 : remplacer l'objet de session efface les faits de permissions. La conserver est une obligation de continuité ; ce n'est pas une permission supplémentaire.

<a id="extrait-6"></a>

## Extrait 6

| Raccord | Transport existant de consigne | ON |
|---|---|---|
| Codex | Instructions développeur et contexte additionnel du tour. | `prefer` si Bridget est monté, sinon `blocked`. |
| Claude | Append du preset SDK et clé de réutilisation du contexte. | Même règle, avec append réellement actualisé après reprise. |
| ACP | État comparé et injection actuelle du prompt. | `blocked` sans montage Bridget. Préserver les commandes natives et le prochain contexte utile. |
| OpenCode | Chemins d'injection existants v1 et v2. | `blocked` sans montage Bridget dans les deux variantes. |
| Cursor | Chemin existant du premier contexte, actualisé en cas de changement. | `blocked` sans conserver une ancienne priorité T3. |
| Pi | Source existante du texte d'extension et `before_agent_start`, actualisée par commande privée fermée. | `blocked` fidèle, y compris après changement du mode et ACK. |

<a id="extrait-7"></a>

## Extrait 7

L'extension Pi existante expose une seule commande réservée : `/t3-bridget-policy`. L'entrée fermée contient le mode `off/prefer/blocked` et un identifiant court de tentative neuf, borné et validé. Aucun prompt, texte libre, secret ou droit n'est accepté. L'adaptateur vérifie la commande dans `get_commands` de ce processus avant envoi par la requête `prompt` existante. Sinon il refuse : une commande inconnue pourrait partir au modèle. L'extension commence bloquée/inconnue ; ce cycle est obligatoire avant le premier vrai prompt, même OFF.
