# Spécification 109 — Sortir le nom Maicie du noyau de communication

## Fiche synthèse

Spec: 109-sortir-maicie
Statut: Implemented (2026-09-18)
Priorité: P1
Tâches: 7/7
Date: 2026-09-18
Branche: session-109-sortir-maicie
Dépendances : contrat de service 015, boîte humaine 087, registre 065. Aucune dépendance nouvelle.

## Problème

La séparation entre Bridget et son orchestrateur compagnon a été faite pour le code : aucun module,
aucune logique métier, et la commande de migration refuse explicitement l'option correspondante.
Le nom est pourtant resté gravé dans le contrat public : un service tiers doit s'appeler `maicie`
pour être accepté en rôle Service, une capacité négociée s'appelle `maicie_guichet`, un producteur
de la boîte humaine s'appelle `maicie`, et quatre outils MCP portent ce préfixe.

Vérifié le 2026-09-18 : la suite de tests passe sans binaire ni configuration de cet orchestrateur,
le daemon tourne sans qu'il soit connecté, et aucune ligne persistée ne porte ces valeurs.
C'est donc un reliquat de nommage, pas une dépendance.

## Objectif

Le noyau de communication n'impose plus le nom d'un produit compagnon. N'importe quel service
externe peut se brancher sur le guichet en négociant ses capacités, sans changer de nom.

## Scénarios utilisateur

- **US1 (P1) — Brancher son propre service.** Un service tiers se présente sous le nom réservé
  neutre et négocie la capacité de guichet ; il est accepté comme avant.
- **US2 (P1) — Déposer une mission.** Un agent dépose une délégation, clôt un objectif, ajoute un
  constat au registre ou demande un statut, avec des outils dont le nom ne cite aucun produit tiers.
- **US3 (P2) — Lire une documentation neutre.** La documentation publiée décrit un guichet et un
  service compagnon générique, sans nommer un projet privé.

## Exigences fonctionnelles

- **FR-001** : le nom de service réservé accepté en rôle Service est `guichet`, pas un nom de produit.
- **FR-002** : la capacité négociée se nomme `guichet_v1` dans le contrat sérialisé.
- **FR-003** : le producteur d'un item de boîte humaine se nomme `guichet`.
- **FR-004** : les quatre outils MCP sont préfixés `guichet_`, le catalogue reste fermé à vingt outils.
- **FR-005** : aucune autre capacité, aucun autre champ, aucun autre comportement n'est modifié.
- **FR-006** : la documentation publiée et la skill ne nomment plus le produit compagnon.
- **FR-007** : l'outillage local d'exploitation qui pilote ce produit reste dans le dépôt de travail,
  mais sort du périmètre publié.

## Critères de succès

- **SC-001** : recherche insensible à la casse du nom du produit dans les crates, la documentation
  publiée et la skill : zéro occurrence.
- **SC-002** : recette complète verte, sans régression du contrat de service et de la boîte humaine.
- **SC-003** : le catalogue MCP expose toujours vingt outils, dont quatre préfixés `guichet_`.

## Hors périmètre

Suppression du guichet lui-même, changement de sémantique des capacités, migration de données,
renommage des tables. Le produit compagnon n'est pas modifié : il devra s'adapter au nom neutre.
