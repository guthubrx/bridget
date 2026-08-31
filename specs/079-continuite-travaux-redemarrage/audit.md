# Audit adversarial - SPEC-079

Date: 2026-08-31
Verdict: PASS pour revue, non livrée

## Vérifications hostiles

- Une ronde désactivée ne peut pas arrêter un agent: elle ne filtre que les
  futurs réveils.
- Un projet ne peut pas recevoir la politique d'un autre: projet et génération
  sont exigés sur lecture, mutation et dispatch.
- Un rebind ne conserve pas l'ancienne autorisation: la nouvelle génération est
  désactivée jusqu'à une activation explicite.
- Deux ticks identiques gardent les mêmes identifiants dérivés du projet, de la
  génération et de l'occurrence.
- Un redémarrage répété ne fabrique pas une chaîne: une remise liée en vol est
  retrouvée puis réaffectée.
- Une reconnexion avec tour vivant ne peut pas reconstruire.
- Une reprise ne change pas de projet et ne branche pas sur un provider.
- Une enveloppe perdue n'est jamais remplacée par une synthèse.

## Surfaces contrôlées

Protocole, compatibilité historique, transaction remise-exécution, monotonie,
sélection bornée, payload exact, rebind, idempotence opérateur, scheduler sans
provider et absence de mutation productive.

Aucun finding critique, élevé ou moyen attribuable au diff.
