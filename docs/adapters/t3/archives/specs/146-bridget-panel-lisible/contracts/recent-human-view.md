# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/contracts/recent-human-view.md](../../../../../../../specs/146-bridget-panel-lisible/contracts/recent-human-view.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-08. Statut : contrat réalisé, interopRust/T3 vérifiée ; clôture en cours, aucune installation146.

<a id="extrait-2"></a>

## Extrait 2

T3 emploie sa RPC `bridget.read`, ses schémas fermés et son lecteur CLI existants. Le serveur résout conversation et projet ; le navigateur ne fournit pas une identité libre autorisante. La commande est exécutée par arguments séparés, jamais par shell.

<a id="extrait-3"></a>

## Extrait 3

Entrée : contexte T3 attesté, `list_recent`, `limit` facultatif de1 à100 et `after` facultatif sur128 caractères au plus.

<a id="extrait-4"></a>

## Extrait 4

Le curseur canonique vaut `<last_activity_at>:<uuidlowercase>`. Le timestamp est décimal de0 à MAX_SAFE, sans zéro initial sauf `0`. L'UUID est canonique en minuscules. T3 garde ce curseur opaque. Les schémas refusent doublons, ordre incohérent ou curseur qui ne désigne pas le dernier résumé reçu. Le curseur ne crée aucune autorisation. Sans changement de données, les pages sont sans trou ni doublon. Une publication concurrente peut déplacer un fil : T3 déduplique les fils chargés et le rafraîchissement repart du début. Aucun instantané complet de liste n'est annoncé.

<a id="extrait-5"></a>

## Extrait 5

Entrée : contexte T3 attesté, `history_recent`, identifiant de fil, `limit` facultatif de1 à200, `before_seq` et `to_seq` facultatifs, entiers sûrs non négatifs. Les champs T3 correspondants sont `sharedThreadId`, `beforeSeq` et `toSeq` ; le lecteur traduit les arguments CLI séparément.

<a id="extrait-6"></a>

## Extrait 6

`thread inspect --t3-thread CONVERSATION --project-root RACINE --action list_recent --after CURSEUR --limit N --json`.

<a id="extrait-7"></a>

## Extrait 7

`thread inspect --t3-thread CONVERSATION --project-root RACINE --action history_recent --thread FIL --before-seq S --to-seq SNAPSHOT --limit N --json`.

<a id="extrait-8"></a>

## Extrait 8

Le contrat rend seulement les données utiles déjà autorisées. Il n'expose ni secrets, transport, credentials ni curseur agent. Les corps restent exacts, sans résumé ni transformation. T3 ne traite pas ce texte comme HTML exécutable.
