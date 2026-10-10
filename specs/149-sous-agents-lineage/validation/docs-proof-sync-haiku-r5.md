# Synchronisation documentaire après preuves r5, r2 et r7 - session 149 (Haiku 5.5)

Date : 2026-10-10. Auteur : agent documentaire Haiku 5.5. Périmètre : documentation seule.

## Verdict

Documents mis à jour : `quickstart.md` et `implementation.md`. Ce rapport est nouveau. Statut global inchangé : **PARTIEL, non livré**. Aucune validation globale n'est prononcée. Aucune case de `tasks.md` n'est cochée.

## Fichiers touchés (et seulement ceux-là)

| Fichier | Action |
|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/quickstart.md` | Réécrit : statut, runtime, droits sans grant humain, codes d'erreur, six scénarios avec preuves réelles, commandes. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/implementation.md` | Réécrit : statut par preuve, phases, six scénarios, mapping SC, compteurs, gaps et observations. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/docs-proof-sync-haiku-r5.md` | Créé : ce rapport. |

Non touchés : code, Git, tests, build, services, configuration, base de données, `tasks.md`, les rapports de preuve, les recettes et les anciens rapports Haiku r3 et r4.

## Preuves utilisées (chemins absolus)

- Recettes réelles r5 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md`, `provider-write149.md`, `standalone149.md`.
- Interop et reprise r2 : `.../validation/native-network-proofs-r2.md`, `interop149.md`, `recovery149.md`.
- Fixture SQLite r7 : `.../validation/native-fixture-sqlite-sonnet-r7.md`, `native149-validation-receipt-r7.json`.
- Latence du hachage : `.../validation/hash-latency149.md`.
- T3 : `.../validation/t3-final-boundary-tests-sonnet-r2.md`, `t3-runtime-hardening-sonnet-r5.md`, `t3-lint-format-haiku-r2.md`.

## Ce qui a changé dans la documentation

1. Les modèles réels sont désormais documentés : `glm-5.3-flash` et `gpt-6.1-sol` (effort `high`), sans T3, avec les résultats r5. Les anciennes phrases « aucun modèle réel prouvé » sont remplacées.
2. Droits : héritage sans nouveau grant humain. La table `native_delegation_grants` reste à 0 ligne pendant les recettes r5.
3. Les six scénarios ont une colonne « preuve réelle » séparée de la preuve fixture ou unitaire.
4. F1 est marqué corrigé (r2, R9.2.a à e). F2 et O4 sont marqués ouverts. Les correctifs de production de Sol sont dits « libérés, non validés ».
5. G3 et G5 référencent `t3-final-boundary-tests-sonnet-r2.md`, pas r1.
6. Compteurs : daemon 1478 PASS (r7), transport 328 PASS (r6), total 1806 indiqué comme agrégat de deux exécutions.
7. Commandes et architecture : vérifiées. Les chemins de `cli_lineage.rs`, `delegation_mcp.rs`, des tests cargo cités et des recettes existent. Les drapeaux `--after-seq`, `--follow`, `--cursor`, `--request-id`, `--action`, `--offset`, `--limit`, `--task`, `--project-root`, `--t3-thread` et `--json` figurent dans `cli_lineage.rs`. Aucune commande n'a été relancée.

## Limites et incohérences relevées (à trancher par le principal)

1. **`validation/recipes/README.md` est périmé.** Il indique encore « préparé, non exécuté ». Il ne doit pas servir de preuve d'exécution. Le guide le signale. Je ne l'ai pas modifié, car il ne fait pas partie du périmètre.
2. **Brief : « retire unavailable never v1 fallback » pour G3.** Le texte de `t3-final-boundary-tests-sonnet-r2.md` donne 262 PASS, 21 tests G3 et 48 tests G5. Il ne contient pas la formulation sur le retrait de `unavailable`. Cette formulation vient de `t3-final-boundary-tests-sonnet-r1.md`, cité par l'ancien journal. Je n'ai retenu que les compteurs de r2.
3. **Code TypeScript.** Le brief cite « TS3770 ». Le rapport r5 indique `TS377030` pour `CodexMcp.ts`. J'ai gardé `TS377030`.
4. **Empreinte de production r7.** L'empreinte de production de r7 (`6cc9a2be…`) est identique à celle de r6. Les correctifs F2 et O4 de Sol ne figurent donc pas dans r7. Ils doivent être revalidés. Le principal doit décider du binaire à rejouer.
5. **Ronde r8.** Le brief parle d'une ronde Sonnet r8 (tests et revue de sources) en cours. Aucun fichier r8 n'était présent dans `validation/` au moment de la lecture. Le statut F2 et O4 est donc « ouvert, en cours », sans preuve écrite.
6. **Rapports Haiku r3 et r4.** Le brief dit qu'ils sont faux quand ils affirment qu'aucun modèle n'est prouvé. Je ne les ai pas lus ni modifiés. Ils restent à corriger ou à marquer historiques par le principal.
7. **Revue finale et matrice de preuves.** `review-final149.md` et `proof-map149.md` sont antérieurs aux rondes r2 à r7. Je ne les ai pas relus. Le journal demande leur relecture après T039.
8. **GLM en mode plan et annulation GLM.** Non joués (limites 3 et 4 de r5). La documentation le dit. Aucune généralisation n'est faite.
9. **Passage à `failed` (G4).** La recette réelle R1 (r5) montre `failed` + `provider_permission_denied` avec GLM. Le test unitaire de G4 n'est pas cité comme fermé. Je n'ai pas clos G4.
10. **Hachage.** Le gain de 4,6 fois est une comparaison release contre release, pas un gain sur le même binaire. La variante `opt-level 3` n'a pas été livrée et n'est pas documentée comme livrée.

## Vérifications faites

- Lecture complète des deux documents avant réécriture.
- Lecture des preuves listées ci-dessus.
- Vérification des chemins et drapeaux CLI cités (commande `grep` et `ls`).
- Contrôle des tirets cadratins dans les trois fichiers : voir la section suivante.

## Contrôle de forme

Les trois fichiers emploient le tiret simple « - ». Aucun tiret cadratin n'a été écrit. Les codes d'erreur et identifiants restent sous leur forme d'origine.
