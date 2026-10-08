# Specification Quality Checklist — SPEC147

Purpose : vérifier la qualité et la complétude de la spécification avant le plan.

Created : 2026-10-08.

Feature : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/spec.md

## Content Quality

- [x] Aucun choix de langage, bibliothèque, protocole ou stockage imposé. Les besoins portent sur les résultats de consultation.
- [x] Valeur utilisateur explicite : lire les nouveautés sans clic et retrouver le fil choisi.
- [x] Texte lisible sans connaissance du code. Les scénarios distinguent droits, coupure et reprise.
- [x] Sections obligatoires remplies : scénarios, exigences, critères, entités et hypothèses.

## Requirement Completeness

- [x] Aucun marqueur de clarification ne reste. Les hypothèses couvrent redémarrage, fil fermé et délai en panne.
- [x] Exigences testables et non ambiguës. Chaque FR147 est associée à un scénario et à un SC147.
- [x] Résultats mesurables : deux secondes, zéro relecture inchangée, parcours A → B → A et absence de doublons.
- [x] Critères sans dépendance à une technologie : délais visibles, données exactes, droits et nombre d'actions observées.
- [x] Scénarios d'acceptation définis pour les deux besoins et les garanties conservées.
- [x] Cas limites identifiés : fil hors première page, coupure, accès révoqué, suppression, contexte réutilisé, réponse tardive et rafale.
- [x] Périmètre borné : consultation du panneau, références de choix et suivi humain. Pas de mission ni nouvelle composition de message.
- [x] Hypothèses et dépendances identifiées : socle146, conservation de l'ouverture existante et autorisation attestée.

## Feature Readiness

- [x] Chaque exigence dispose de critères d'acceptation. US147-01 couvre FR147-01 à04 ; US147-02 couvre FR147-07 à09 et12 ; US147-03 couvre FR147-05 à06 et10 à11 ; US147-04 couvre FR147-13 à14. FR147-15 à16 sont couvertes par US147-03/04 et SC147-07 à09.
- [x] Parcours principaux couverts indépendamment : actualisation, sélection, reprise et lecture stable.
- [x] Objectifs vérifiables sans accès aux conversations actives. SC147-01 à09 définissent la recette et ne revendiquent pas de résultat déjà exécuté.
- [x] Aucun détail d'implémentation n'est introduit comme solution obligatoire. Le plan doit encore fixer le transport, les bornes et la réutilisation.

## Notes

Validation documentaire du 2026-10-08 : une passe, aucun item ouvert. Ce résultat valide la préparation du plan, pas l'implémentation ni l'installation.

Le template officiel de spécification n'existe pas dans ce worktree. La structure manuelle reprend les sections obligatoires de la compétence speckit-specify. Aucun fichier de template n'a été créé ou modifié.

La SPEC147 ajoute un suivi automatique à la consultation. L'exclusion de lecture périodique de la SPEC146 reste l'historique de ce socle ; elle ne doit pas être réécrite comme si le suivi existait déjà.

## Addendum US5 — 2026-10-09

- [x] Ajout explicitement approuvé à147 ; US1–US4 inchangées. US147-05 P2 comporte parcours indépendant et scénarios none/targets/all/nom absent/multidestinations/copie exacte.
- [x] FR147-17/18 et SC147-10 sont mappés aux T027–T030. Destinataires sollicités ≠ livraison confirmée. Ligne auteur/date existante, targets effectifs/auteur exclu, noms autorisés et fallback honnête. Aucun appel supplémentaire, API, annuaire ou mutation.
- [x] Documents de conception convergés sans qualifier les tests ciblés de recette réelle. T027/T028 ont leurs reçus ; QA/T029 et convergence T030 validées après le gate final.

Les cases de cet addendum portent sur la complétude documentaire, pas sur une activation de production.

## Extension US6 — checklist de conception complète, gate validé

- [x] US147-06 P1 approuvée avec scénarios indépendants et champs UUID déclarés.
- [x] Portée limitée aux commandes et outil de fils partagés ; IDs send, acteur d'autorité, noms/préfixes partiels et contrat humain exclus.
- [x] Réutilisation require_uuid/thread_members et dépendance uuid existante vérifiée, aucun helper/dependency/migration nouveau.
- [x] Normalisation hyphénée36 eq_ignore_ascii_case avant hash/SQL ; invalides refusés et corps inchangés.
- [x] Reçu ACK cohérent avec Post, refus ReceiptInvalid conservé.
- [x] canonical_uuid humain et curseurs fermés stricts inchangés.
- [x] RED comportementaux et CLI avant GREEN, puis non-régressions102/127/145/146/147 dans T031–T034.
- [x] Plan UUID et gate de réutilisation validés par le principal avant implémentation.

Conception US6 validée. Reçus RED puis GREEN26PASS/1ignored et régressions reçus ; T031–T034 cochées sur GO du principal. L'autorité MCP OS n'est pas déduite de la fixture UUID.

## Extension US7 — gate de conception MCP validé

- [x] US147-07 P1 et exigences de catalogue/identité/opt-out/permissions approuvées.
- [x] Cible signalée identifiée : regional-wrkr-1 / claude_glm / home .claude-glm.
- [x] Configuration MCP effective et chemin du cas exact corroborés : regional-wrkr-1/claude_glm, home .claude-glm, mcp get bridget RC1 et injection T3 t3-code seulement.
- [x] Plan MCP concret et réutilisations validés par le principal avant tâches/code US7.
- [x] Tests catalogue/options SDK/ vraie façade MCP privée sans modèle définis sur les chemins effectifs, T035–T041.
- [x] Barrière avant sessions.set/ready/prompt ; timeout ferme candidate/consumer et interdit montage tardif.
- [x] Métadonnées MCP fail-closed : NotFound distinct d'I/O ; Skills conserve son comportement fail-open.

- [x] Complément T040 approuvé avant code : tests spec101_identity existants, IdentityBindings.refresh réel, DBT3 provider_session_runtime privée, PID/birth/lineage natifs et tuple live/HTTP synthétique explicite ; pas nouveau helper/module.

Cause locale précise vérifiée dans diagnostic-mcp.md. Reçus SDK294PASS/1SKIP finaux, façade OS3PASS et qualification refresh1PASS ; régressions identité22/14. Revues finales lifetime/flags APPROVE et GO principal autorisent T049/T050 cochées, preuves292/293 antérieures conservées. Les preuves restent distinctes, pas chaîne T3+SDK+modèle complète. Aucun modèle/restart/production. Clôture globale US8/US9 encore ouverte.

## Extension US8 — gate de conception, lecture finale principale

- [x] Besoin approuvé : ajout seul par créateur initial, fil ouvert, maximum16 après UUID normalisation/dédoublonnage.
- [x] Audience union complète validée avant DB puis recheck transactionnel contre divergence.
- [x] Migration opérationsv2→v3 ciblée, reçus/index préservés ; NoChange sans reçu ni clé engagée.
- [x] Hello auxiliaire combiné projects+capacité dédiée et garde attestée couverts par tests CLI/MCP réels.
- [x] Historique complet autorisé, curseur0, zéro alerte/réveil/replaymission ; ancien wakepending non dispatché.
- [x] Futur notify=all inclut nouveau membre, anciens targets/kinds inchangés ; watch postcommit sans changement activité.
- [x] T042–T048 séparent RED, implémentation, interop/régression et docs locales ; pas nouveau composeur UI.

Plan et gate5/5 acceptés par le principal, deux passes Analyze sans finding CRITICAL. Capacité ThreadMembersV1 / thread_members_v1 figée ; tests sérialisation et migrationsv1/v2→v3, seconde ouverture/rollback/reçus exacts requis. Les résultats US8 finaux sont attendus.

## US9 — ajout visuel et plan natif approuvés

- [x] US147-09 P2 approuvée dans147, pas nouvelle session148.
- [x] Ligne choisie mise en valeur native T3, bloc inférieur répété retiré, détails accessibles.
- [x] Hors première page/recherche et retour A→B→A couverts par les critères, sans sélection automatique.
- [x] Pas nouveau contrat/API/store, rebrand ou modification des communications/permissions.
- [x] Plan Web court et gate5/5 de réutilisation approuvés avant tâches/code US9 : item27, wrappers/styles natifs relus.
- [x] Cinq groupes RED comportementaux et aperçu isolé définis sur les composants natifs existants.

Besoin complet31FR/20SC/9US. Ce gate de conception était ouvert avant les preuves ; clôture courante ci-dessous.

## Clôture globale — GO principal après Converge

- [x]31FR/20SC/9US mappées après deux passes code↔artefacts, aucun orphelin ni finding actif.
- [x] US8 RED/GREEN, interop CLI/MCP privée et non-régressions reçus ; limites de qualification conservées.
- [x] US9 correction FR31 RED/GREEN122PASS, build/statiqueRC0 et recette native finale validés ; titre/membres complets et sélection accessible.
- [x] Audits frais Rust/T3 validés RC0/0erreur/0warning, grades limités aux deltas et constats corrigés historiques conservés.
- [x] SHA tasks identique pendant Converge,54/54 cochées seulement sur GO final.
- [x] NON installé/NON activé ; preuves SDK, autorité, publication et DOM synthétique non assemblées en fausse preuve E2E.
