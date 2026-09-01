# Programme de livraison - SPEC-084 à SPEC-086

## Ordre obligatoire

```text
SPEC-084 Projets multi-serveurs et emplacements
    -> SPEC-085 Runtime Docker de production
        -> SPEC-086 Projet système Bridget et dogfooding expert
```

## Pourquoi cet ordre

1. SPEC-084 fixe d'abord où vit un projet et quel serveur en porte l'autorité. Elle empêche le checkout Bridget de servir de parent de création.
2. SPEC-085 rend ensuite l'environnement Docker utilisable pour un projet standard, avec installation, image, état et rollback Host.
3. SPEC-086 n'ajoute qu'après ces preuves l'exception experte permettant à un projet système unique de modifier Bridget dans un worktree borné.

## Gates de progression

### Gate A après SPEC-084

- serveur et emplacement visibles avant confirmation;
- migration v1 prévisualisée;
- checkout Bridget classable comme projet exact, jamais comme workspace implicite;
- aucun changement Docker.

### Gate B après SPEC-085

- installation existante toujours Host par défaut;
- projet standard activable en Docker sans fallback silencieux;
- image et politiques attestées;
- worktrees utilisables aux mêmes chemins;
- retour Host prouvé.

### Gate C après SPEC-086

- un seul projet système attesté;
- disabled réellement read-only;
- enabled limité à un worktree non-main attribué;
- coexistence avec agents externes prouvée;
- aucune action automatique de livraison système.

## Arrêts obligatoires

- Ne pas commencer SPEC-085 si la migration 084 peut encore créer sous une racine v1 ambiguë.
- Ne pas commencer SPEC-086 si l'activation Docker, le rollback ou la topologie Git de 085 ne sont pas prouvés.
- Ne pas activer le dogfooding en production tant que la validation opérateur de 086 n'est pas consignée.

## Stratégie Git

Chaque SPEC conserve sa branche et son worktree. Les branches suivantes doivent intégrer la précédente seulement après son merge validé. Aucun travail ne doit être transplanté par copie manuelle de source.

