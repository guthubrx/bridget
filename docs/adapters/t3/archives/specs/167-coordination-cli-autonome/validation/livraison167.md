# Archives techniques de l’adaptateur T3

Document d’origine : [specs/167-coordination-cli-autonome/validation/livraison167.md](../../../../../../../specs/167-coordination-cli-autonome/validation/livraison167.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le parcours réutilise le moteur natif `delegation_mcp::execute`. Il n’ajoute ni protocole, ni base, ni service. T3 reste compatible, mais il n’est pas requis.

<a id="extrait-2"></a>

## Extrait 2

- Le client 0.3.0 est actif sur disque pour toute nouvelle commande.
- Le daemon PID 68982 et le pont PID 68977 n’ont pas été relancés. Ils servent encore le Build ID `3b4f101e0732`.
- Cette conservation est volontaire. La version 167 ajoute une façade client et réutilise le protocole natif déjà actif. Une relance aurait interrompu 58 agents sans bénéfice pour cette livraison.
- Le pont reste `running` avec 32 fils.
- Le pont conserve 173 fichiers ouverts. Le daemon en conserve 126.
- T3 reste sur les PID 65544 et 65608. Il n’a pas été relancé.
- La tâche parasite `com.bridget.t3-activate166` est absente.
- Aucun `auth_failed`, HTTP 401, `EMFILE` ou `Too many open files` récent n’apparaît dans le journal du pont.
