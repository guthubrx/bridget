# Programme progressif 065-067

**Statut**: conception terminée, trois specs `Draft`, 104 tâches non implémentées

**Contre-revue**: RC8 documentaire du 2026-08-30, corrections intégrées;
implémentation toujours interdite avant reprise sur une future tête `main` propre

## Ordre obligatoire

```text
SPEC-063 interruption/pilotage prouvée
                 |
SPEC-064 plan de contrôle stabilisé
                 |
SPEC-068 incidents runtime délégués stabilisés
                 |
SPEC-065 registre et identité, backend host
                 |
SPEC-066 environnement Docker partagé, sans secrets projet
                 |
SPEC-067 profils, extensions et secrets bornés
```

## Incréments et rollback

| Spec | Valeur autonome | Activation | Rollback |
|---|---|---|---|
| 065 | identité et diagnostic projet | enregistrement volontaire | ignorer project_id, backend host |
| 066 | isolation filesystem/process par projet | backend docker opt-in | repasser explicitement à host |
| 067 | profils, skills/plugins et secrets projet | approbation locale | désactiver profil et recréer l'environnement |

## Décisions gelées

- Maicie et Bridget restent sur l'hôte.
- Maicie et Bridget restent déterministes; seuls les agents sont agentiques.
- Un conteneur est partagé par tous les agents d'un même projet.
- Les worktrees séparent les travaux concurrents, pas les agents par principe.
- Le projet est la frontière de confiance v1.
- Les racines projet, politiques runtime et sources de ressources proviennent
  de trois configurations hôte fermées possédées par Bridget; leur absence
  ferme uniquement la fonctionnalité concernée.
- Les conteneurs sont jetables; code, worktrees et état durable restent sur
  l'hôte.
- `backend=host` reste disponible et aucun projet existant n'est migré seul.
- Kubernetes, Vault, gVisor, broker dynamique et mémoire globale sont différés.

## Gates de progression

- 066 ne commence pas avant une 065 prouvée, réversible et sans régression.
- 067 ne commence pas avant une 066 capable de recréer un environnement sans
  perte et sans secret.
- Une spec peut être abandonnée sans rendre la précédente inutilisable.
- Chaque activation est explicite par projet; aucun basculement global.
- Chaque spec doit être implémentée dans un nouveau worktree créé depuis la
  tête `main` propre du moment, au minimum `d589b24` qui contient SPEC-068, et
  refaire son audit de réutilisation; si `main` avance encore, l'audit doit être
  rejoué contre cette nouvelle tête. Le worktree documentaire courant n'est pas
  une base de code.
- L'absence de `.specify` ou de l'outil `specify` ne bloque pas ce programme et
  ne justifie ni installation, ni mise à jour de SpecKit. Les artefacts présents
  dans `specs/` sont la source de vérité documentaire de cette tranche.
