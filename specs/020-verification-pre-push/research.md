# Recherche technique — Spec 020

## Contrat officiel du hook

**Décision** : traiter l'entrée standard comme une transaction complète.

**Justification** : la documentation Git définit une ligne par mise à jour,
avec référence et objet locaux puis référence et objet distants. Un code retour
non nul interrompt l'envoi avant toute mise à jour.

**Source primaire** :
https://git-scm.com/docs/githooks#_pre_push

## Définition des commits introduits

**Décision** : soustraire l'ensemble des commits déjà atteignables depuis
toutes les références distantes observées à l'ensemble atteignable depuis les
nouvelles têtes de la transaction.

**Justification** : une plage par tête rescannerait l'héritage déjà accepté et
une seule tête manquerait les autres références de la transaction.

**Alternative rejetée** : utiliser seulement les références de suivi locales.
Elles peuvent être périmées et une branche neuve n'a pas d'amont.

## Politique d'indétermination

**Décision** : refuser lorsque l'observation ou le calcul ne peuvent être
démontrés.

**Justification** : un succès ambigu donnerait une assurance fausse. Le coût
opérationnel accepté est une actualisation du dépôt avant de retenter.
