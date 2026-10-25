# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/tasks.md](../../../../../../specs/158-reveils-controle/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Réduisent la charge : T016 (un outil sans portée échoue en CI), T009 (une vue
  pour tous les parents), T004 (une seule décision d'admission).
- Complexité justifiée : T017 (plafond, comble l'écart de sécurité avec T3),
  T003 (reçus bornés, évitent de remettre un arrêt par un rejeu).
- Non faits volontairement : budget de tours et de tokens (compteur non fiable),
  historique d'audit de l'arrêt, enveloppe de compilation des outils.

<a id="extrait-2"></a>

## Extrait 2

Après toutes les preuves et la relecture : Analyze si delta contractuel, Converge code/artefacts dans la borne 3 passages, contre-revue puis audit selon my-specify-all, enfin commit/intégration/push/buildsigné/install/activation autorisée avec zéro mission native non terminale et nettoyage, reçus séparés. ROOT conserve ce pipeline ; aucun cochage technique de ces phases ni commit parcase. T3 ne redémarre pas sans consigne explicite. Le reçu158A reste historique et autonome.
