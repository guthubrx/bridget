# Archives techniques de l’adaptateur T3

Document d’origine : [specs/135-controle-missions-silencieux/spec.md](../../../../../../specs/135-controle-missions-silencieux/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Branche** : session-135-controle-missions-silencieux
**Date** : 2026-10-05
**Statut** : Implemented
**Priorité** : P1
**Demande** : réduire les messages répétés des boucles et rendre le suivi des
missions plus contraignant.
**Dépendances** : 098 (pont T3), 114 (lots), 133 (relais sous-agents), agent-loop.

<a id="extrait-2"></a>

## Extrait 2

- Forcer un agent à produire un raisonnement.
- Évaluer automatiquement la qualité métier d'un résultat.
- Réaffecter automatiquement une mission sans politique explicite.
- Modifier Bridget, T3 ou les fournisseurs de modèles.
