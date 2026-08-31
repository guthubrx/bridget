# Analyse inter-artefacts - SPEC-076

**Date**: 2026-08-31
**Base analysee**: branche session-076-interface-projets-coordinateur sur 4ad487e.

## Resultat

| Axe | Verdict | Preuve |
|---|---|---|
| Autorite projet | PASS | Maicie conserve identite, Bridget conserve liaison et audit, interface sans store metier. |
| Racines et concurrence | PASS | Politique versionnee, ecriture atomique et course de generation testee. |
| Creation et import | PASS | Previsualisation pure, Git explicite, reprise sans fusion ni ecrasement. |
| Reactivation et retrait | PASS | Activate typee, audit conserve, disable non destructif et reprise explicite. |
| Projection UI | PASS | Identite, liaison, dernier audit et agents ProjectReference dans le snapshot existant. |
| Coordinateur | PASS | Instantane durable, digest, upstream explicite et aucune substitution. |
| Decouverte | PASS | Definitions derivees restreintes, duree bornee, confirmation et empreinte avant apres. |
| Frontiere locale | PASS | Relais loopback existant, jeton UI et contrats UID pair consommes. |
| Secrets et profils | PASS | Aucune route UI ne modifie profil, extension ou secret. |

## Tests retenus

- SPEC-076 : 20 passes.
- Daemon unitaire complet : 743 passes, 7 ignores.
- Maicie project : 12 passes. Transport runtime : 2 passes. Interface JavaScript : 93 passes.
- Format et Clippy workspace : PASS.
- Workspace complet : ECHEC uniquement dans managed_parity_test, 6 passes et 4 echecs reproductibles.

Les echecs concernent une empreinte de corpus FR-008, deux attentes de prompt
MCP Codex et le nettoyage de six groupes managed-wrapper. Ils etaient connus
avant les derniers changements 076 et leurs sources directes ne sont pas dans
le diff de la SPEC. Ils empechent neanmoins le verdict global CONVERGED.

## Findings ouverts

1. T062 : tests workspace non verts.
2. T063 : validation manuelle utilisateur non recueillie.
3. T066 : contre-revue adverse indisponible depuis ce canal.
4. T067 : Converge depend des trois points precedents.

## Verdict

**PASS_IMPLEMENTATION_TARGETED_VALIDATION**. Aucun finding CRITICAL dans le
perimetre SPEC-076. La livraison reste suspendue jusqu a la resolution ou
acceptation explicite des findings ouverts.
