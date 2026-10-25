# Audit de réutilisation — SPEC140

Statut : PASS de conception, contrôle final validé par le principal avant tasks.
Date : 2026-10-07. Ce PASS ne représente aucune implémentation ni test exécuté.

## Inventaire et décisions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent

Les publications de fil action/blocker/decision/history existent dans le contrat de transport, mais une sollicitation ne transporte pas leur nature. Ne pas les déduire du corps. ThreadNotice refuse les champs inconnus : l'étendre casserait la compatibilité. BridgetMessage accepte le champ racine additif. Les titres des anciennes enveloppes ne sont pas retrouvés par une interrogation du navigateur.

## Duplications évidentes

Aucune après sélection de la branche existante de CollapsibleUserMessageBody, de la logique existante et de Store.thread_show. Un composant local n'est acceptable que s'il rend cette branche lisible sans créer de seconde responsabilité métier. Aucun store global de cartes ou registre de noms.

## Règles et specs applicables

AGENTS140 et constitution : isolation, zéro commit automatique, anti-doublon, interfaces simples, tests observables. SPEC134 pour les noms ; SPEC136 pour le silence et le canon historique ; SPEC138 pour l'autorité des destinataires ; SPEC139 pour la reconnaissance d'enveloppe. Les standards Cartae sur Next/i18n/backend Python ne remplacent pas la structure React/Vite et Rust de ces dépôts.

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/reuse-audit.md#extrait-2).

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/reuse-audit.md#extrait-3).

## Gate avant tasks

- [x] Items du plan extraits et équivalents recherchés.
- [x] Réutilisations privilégiées et créations limitées justifiées.
- [x] Aucune duplication évidente non arbitrée.
- [x] Extension de titre autorisée et compatibilité traitée.
- [x] Aucun nouveau service, API, stockage, migration ou dépendance.

Le principal a lu spec, plan, rapport et checklist puis validé le gate PASS avant génération de tasks.md. Ce rapport ne permet pas à lui seul de déclarer les tests ou la livraison terminés.

## Arbitrages effectivement appliqués pendant l'implémentation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/reuse-audit.md#extrait-4).

Correction audit QUAL-001 : reconnaissance limitée au suffixe fermé via sous-agent codex/claude/cursor avec référence16hex dans la projection existante. Les formats inconnus restent verbatim.12 tests RED/GREEN et une copie UI exacte vérifient le bénéfice ; aucun identifiant de routage n'est supprimé du brut.
