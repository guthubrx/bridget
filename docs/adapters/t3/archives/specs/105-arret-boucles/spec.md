# Archives techniques de l’adaptateur T3

Document d’origine : [specs/105-arret-boucles/spec.md](../../../../../../specs/105-arret-boucles/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Validée le 17 septembre 2026. Correction demandée sur T3 et vérification de tous les adaptateurs. Aucun commit ni déploiement automatique.

<a id="extrait-2"></a>

## Extrait 2

- Un message `reply=false`, notamment une réponse corrélée, ne crée jamais de nouvelle réponse automatique T3.
- Une demande `reply=true` conserve sa réponse utile, même très courte, et sa corrélation. Une nouvelle question explicite reste possible.
- Aucun filtrage lexical de « OK », « reçu » ou « . » : le contrat du message décide, pas le texte.
- Les observations restent sans réponse automatique et gardent leur protection contre les boucles d’observation.
- Après redémarrage, une ancienne attente dont le contrat est inconnu n’est pas relayée sans preuve d’une demande ouverte, non périmée, du bon expéditeur vers ce destinataire. Une liste bornée vide ne justifie aucune suppression.
- Les autres adaptateurs indiquent clairement si une réponse est attendue, sans demander d’accusés de réception. Le garde existant des agents gérés reste effectif.

<a id="extrait-3"></a>

## Extrait 3

Branche `session-105-arret-boucles`, base Git `1738a072`. Le worktree contient une copie de l’état non commité de la session 101, déjà installé, pour ne pas perdre les observations T3 et l’identité attestée. La session 101 et l’arbre principal ne sont pas modifiés. Les changements hérités ne doivent pas être présentés comme produits par la 105.
