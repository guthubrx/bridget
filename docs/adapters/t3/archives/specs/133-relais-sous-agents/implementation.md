# Archives techniques de l’adaptateur T3

Document d’origine : [specs/133-relais-sous-agents/implementation.md](../../../../../../specs/133-relais-sous-agents/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

État privé : `BRIDGET_HOME=/tmp/bg133-baseline-home`,
`BRIDGET_SOCKET=/tmp/bg133-baseline-home/b.sock`. Aucun daemon installé, état
T3 réel ou fournisseur payant n'a été utilisé.

<a id="extrait-2"></a>

## Extrait 2

- compilation `bridget-core` ciblée : PASS ;
- compilation des tests `bridget-daemon --lib` : PASS ;
- `mcp_identity::tests::` : 10 PASS ;
- `t3code_identity::spec101_identity::` : 19 PASS ;
- `message::tests::` : 12 PASS ;
- `communication::tests::` : 4 PASS ;
- enveloppe SPEC-131 T3 : 1 PASS.

<a id="extrait-3"></a>

## Extrait 3

Le pont T3 publie une preuve privée séparée pour chaque fournisseur interne
rattaché sans ambiguïté à un parent vivant. La résolution MCP rend l'identité du
parent avec une provenance enfant fermée. `bridget_who` et `bridget_send` sont
les deux seules actions admises. Le routage, l'idempotence et les réponses
restent dans la portée du parent. Les rendus ACP, Codex et T3 affichent la
mention « via sous-agent » sans identifiant natif complet.

<a id="extrait-4"></a>

## Extrait 4

L'observation de production a aussi trouvé la cause du blocage global T3. Le
processus `lsof` pouvait produire plus de données que le tube de sortie. Le pont
attendait sa fin avant de vider ce tube. `lsof` restait alors bloqué en écriture,
le délai expirait et le pont publiait un inventaire indisponible pour tous les
fils T3. La correction vide maintenant la sortie dans un lecteur dédié pendant
l'exécution, conserve la borne de taille et termine proprement le processus en
cas d'erreur.

<a id="extrait-5"></a>

## Extrait 5

- Nécessité : l'incident réel empêchait toute identité T3, puis le sous-agent
  restait sans autorité bornée même avec une identité parent valide.
- Solution la plus simple : réutiliser la filiation et les marqueurs existants,
  ajouter une preuve enfant séparée, puis vider le tube `lsof` en parallèle.
- Hypothèses : le compte local et le pont T3 restent dans le modèle coopératif
  défini par la spécification.
- Vérifications : tests RED/GREEN ciblés, reproduction du volume `lsof` et
  comparaison du premier échec workspace avec `main`.
- Non vérifié à ce stade : redémarrage des services installés et essai réel du
  fil « Véhicules », réservés à la livraison.
- Code évité : aucune identité enfant durable, table, dépendance ou daemon ajouté.
- Complexité ajoutée : un marqueur éphémère, un contexte MCP fermé et un lecteur
  borné pour le tube de l'inventaire.

<a id="extrait-6"></a>

## Extrait 6

Deux tests historiques de contrôle CLI restent sensibles à l'identité réelle du
processus qui lance le banc. Ils attendent une connexion CLI nue, mais utilisent
l'identité T3 réelle lorsqu'elle est publiée. Le même échec
`auxiliary_credential_required` est reproduit sur `main` sans la SPEC-133. Cette
limite du harnais n'est donc pas attribuée au changement. Elle ne touche ni le
chemin MCP délégué ni le pont T3 de production.
