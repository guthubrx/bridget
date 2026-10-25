# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/validation/T002-readonly.md](../../../../../../../specs/159-edition-file/validation/T002-readonly.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

3. **Écarts pont et issue indéterminée.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/t3code.rs:1850–1926 traite Saturated, Expired (saga ou demande), RequestClosed et GuidanceUnavailable. `report_discard`:1868 émet DeliveryIndeterminate pour l’idempotent ; il émet aussi DeliveryRejected lorsque reply=true ou guidance indisponible, sauf RequestClosed. Le daemon:19581–19621 appelle `mark_delivery_indeterminate` avec instance/génération attestées ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs:597–618 ne change que dispatching→indeterminate. Cette seule trame ne ferme pas une demande suivie. Wrapper:749–813 renvoie aussi Indeterminate si mauvaise instance, reçu Seen/indéterminé, quota du reçu ou erreur de stockage. Ces causes sont distinctes ; ne pas traduire toute issue indéterminée en non-livraison certaine.

<a id="extrait-2"></a>

## Extrait 2

6. **BatchedDelivery des porteurs gérés.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/wrapper.rs:891–986 contient `deliver_batch_to_interactive` et le type Historic/Idempotent. Son seul appel production trouvé:906 passe un vecteur d’un seul élément. Les appels multiples trouvés sont des tests. Le regroupement effectif de plusieurs livraisons gérées n’est donc pas établi. T3 possède séparément BATCH_BOUND=8 et batch_family:3261 ; ce chemin reste hors159A. Réutiliser le traitement existant lorsque pertinent, sans annoncer un lot géré actuel sur la foi du nom.
