# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/code-review.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/code-review.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La revue a suivi les retraits de liens, les fermetures du worker et du lecteur,
les obligations durables, les reprises bornées, les remises pendant le drain et
les contrats T3 V1/V2. Elle ne relève aucune fuite de descripteur ni de tâche.

<a id="extrait-2"></a>

## Extrait 2

- Un snapshot ancien peut, dans une fenêtre étroite, lever l'attente d'un
  changement après dispatch. Une remise avec réponse attendue reste protégée
  par `pending`. T3 ne fournit pas de séquence de snapshot permettant un ordre
  plus fiable.
- Un sous-agent arrêté refuse les messages Bridget directs jusqu'à sa reprise
  par T3. L'ADR et le quickstart décrivent maintenant cette limite.
- Une relation V2 inconnue arrête le pont. C'est le comportement fermé voulu.
  Son coût opérationnel est maintenant explicite dans l'ADR.
- La petite fenêtre héritée entre la fin du worker et la fermeture du lecteur
  n'est pas aggravée. Le correctif rapporte les remises connues avant la
  désinscription et réduit cette fenêtre.
